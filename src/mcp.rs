//! # MCP command
//!
//! The `mcp` subcommand, a Model Context Protocol server over stdio
//! holding one account open, so an agent drives the OAuth flows of that
//! account without any secret crossing into its context.
//!
//! Tools answer receipts and metadata only. The state, PKCE verifier,
//! redirect URI and device code of a flow stay in the server: the
//! loopback listener runs on a background thread, the device code is
//! polled at the server's interval, and the token is exchanged and
//! stored as soon as the user consents. Applications keep reading the
//! token out of band, through `ortie token show` as their secret command.
//!
//! The main thread owns the account and runs everything touching it,
//! requests, exchanges and polls alike, so the account needs no lock and
//! two storage writes never race. The stdin reader and the listener only
//! feed it events.

use std::{
    fmt,
    io::{self, BufRead, Write},
    mem,
    net::{SocketAddr, TcpStream},
    sync::mpsc::{self, RecvTimeoutError, Sender},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow, bail};
use clap::Parser;
use io_oauth::{
    client::{Oauth20ClientStd, Oauth20ClientStdError, await_redirect_on},
    rfc6749::{
        issue_access_token::{Oauth20AccessTokenErrorCode, Oauth20AccessTokenResponse},
        state::Oauth20State,
    },
    rfc7636::pkce::Oauth20PkceCodeVerifier,
    rfc8628::{auth::Oauth20DeviceAuthSuccessParams, token::Oauth20DeviceAccessTokenRequestParams},
};
use log::debug;
use pimalaya_cli::{error::ErrorReport, output::Bytes, printer::Printer};
use pimalaya_config::secret::Secret;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Map, Value, json};
use url::Url;

use crate::{
    account::{Account, DEFAULT_REDIRECTION, bind_loopback},
    auth::{
        get::{
            AuthGetCommand, AuthorizationRequest, authorization_request,
            device_poll_client_error_hook_params, is_loopback_redirect, report_token_issued,
            request_device_authorization,
        },
        resume::AuthResumeCommand,
    },
    config::GrantConfig,
    json_schema,
    token::{
        inspect::TokenInspectCommand, issued::TokenIssuedOutput, refresh::TokenRefreshCommand,
        revoke::TokenRevokeCommand,
    },
};

/// The MCP revision the server speaks, the first carrying
/// `structuredContent` and `outputSchema`.
const PROTOCOL_VERSION: &str = "2025-06-18";

/// What the server tells the agent at initialization.
const INSTRUCTIONS: &str = "Obtains and renews the OAuth 2.0 token of one \
account. Call auth_get, then show the user the authorizationUri to open, or \
the userCode to enter at the verificationUri, and call auth_status until the \
flow is issued or failed. When redirectCaptured is false, ask the user for \
the URL their browser was redirected to and pass it to auth_resume. No tool \
returns the token itself.";

/// The tools, in the order they are listed.
const TOOLS: &[McpTool] = &[
    McpTool {
        name: "token_inspect",
        description: "Show the metadata of the stored token: its type, when it was issued, \
                      what remains of its lifetime, whether a refresh token is held, and \
                      the granted scope. The token itself is never shown.",
        arg: None,
        output: McpToolOutput::Registry("ortie-token-inspect"),
        read_only: true,
        destructive: false,
    },
    McpTool {
        name: "auth_get",
        description: "Start the account's OAuth grant, replacing any pending flow. Returns \
                      the authorization URI for the user to open, or the user code and the \
                      verification URI for the user to enter it at; a client credentials \
                      account is issued at once. The server stores the token as soon as \
                      the user consents: follow with auth_status.",
        arg: None,
        output: McpToolOutput::AuthGet,
        read_only: false,
        destructive: false,
    },
    McpTool {
        name: "auth_status",
        description: "Report the flow auth_get started: none, pending, issued with the \
                      receipt, or failed with the error.",
        arg: None,
        output: McpToolOutput::AuthStatus,
        read_only: true,
        destructive: false,
    },
    McpTool {
        name: "auth_resume",
        description: "Complete the pending authorization code flow from the URL the user's \
                      browser was redirected to, when the server could not capture the \
                      redirection itself (redirectCaptured false).",
        arg: Some((
            "uri",
            "The URL the browser was redirected to after consent, carrying the code.",
        )),
        output: McpToolOutput::Registry("ortie-auth-resume"),
        read_only: false,
        destructive: false,
    },
    McpTool {
        name: "token_refresh",
        description: "Renew the stored token with its refresh token, or re-run the grant on \
                      a client credentials account. Returns the receipt.",
        arg: None,
        output: McpToolOutput::Registry("ortie-token-refresh"),
        read_only: false,
        destructive: false,
    },
    McpTool {
        name: "token_revoke",
        description: "Revoke the stored refresh token, or the access token when none is \
                      held, at the authorization server. The account needs a new auth_get \
                      afterwards.",
        arg: None,
        output: McpToolOutput::Registry("ortie-token-revoke"),
        read_only: false,
        destructive: true,
    },
];

/// The tool only `--allow-revoke` lists.
const REVOKE_TOOL: &str = "token_revoke";

/// Start an MCP server over stdio for one account.
///
/// An agent drives the account's OAuth flows through it: it starts a
/// grant, shows the user what to open or enter, and follows the flow to
/// its end, while the server keeps the state, the PKCE verifier and the
/// device code to itself and stores the token on consent. No tool
/// returns the token: applications keep reading it through `ortie token
/// show`.
#[derive(Debug, Parser)]
pub struct McpCommand {
    /// List the revocation tool too.
    ///
    /// Revoking leaves the account without a working token until the
    /// next grant, and an agent can be told to by the content it reads,
    /// so it is never on by default.
    #[arg(long)]
    pub allow_revoke: bool,
}

impl McpCommand {
    /// Serves JSON-RPC requests off stdin until EOF, answering each on
    /// stdout.
    pub fn execute(self, account: Account) -> Result<()> {
        let (events, inbox) = mpsc::channel();

        let lines = events.clone();
        thread::spawn(move || {
            for line in io::stdin().lock().lines() {
                let Ok(line) = line else {
                    break;
                };

                if lines.send(McpEvent::Line(line)).is_err() {
                    return;
                }
            }

            let _ = lines.send(McpEvent::Eof);
        });

        let mut server = McpServer {
            account,
            allow_revoke: self.allow_revoke,
            events,
            flow: McpFlow::None,
            flows: 0,
        };

        let mut stdout = io::stdout();

        loop {
            // NOTE: a pending device flow wakes the loop at its next
            // poll, which runs here, on the thread owning the account.
            let event = match server.next_poll() {
                Some(at) => {
                    match inbox.recv_timeout(at.saturating_duration_since(Instant::now())) {
                        Ok(event) => event,
                        Err(RecvTimeoutError::Timeout) => {
                            server.poll_device();
                            continue;
                        }
                        Err(RecvTimeoutError::Disconnected) => break,
                    }
                }
                None => match inbox.recv() {
                    Ok(event) => event,
                    Err(_) => break,
                },
            };

            match event {
                McpEvent::Eof => break,
                McpEvent::Line(line) if line.trim().is_empty() => continue,
                McpEvent::Line(line) => {
                    if let Some(response) = server.answer(&line) {
                        serde_json::to_writer(&mut stdout, &response)?;
                        writeln!(stdout)?;
                        stdout.flush()?;
                    }
                }
                McpEvent::Redirect { flow, result } => server.redirected(flow, result),
            }
        }

        server.cancel_flow();

        Ok(())
    }
}

/// What the main loop is woken by.
enum McpEvent {
    /// A line read off stdin, one JSON-RPC message.
    Line(String),
    /// stdin closed.
    Eof,
    /// The listener of flow `flow` caught a request, or failed.
    Redirect {
        flow: u64,
        result: Result<String, String>,
    },
}

/// The OAuth flow `auth_get` started.
enum McpFlow {
    /// No flow was started.
    None,
    /// An authorization code flow waiting for its redirect.
    Code {
        id: u64,
        state: Oauth20State,
        pkce: Option<Oauth20PkceCodeVerifier>,
        redirect_uri: Url,
        /// The address of the listener catching the redirect, `None`
        /// when the redirection is not loopback.
        listening: Option<SocketAddr>,
    },
    /// A device flow waiting for the user's approval.
    Device {
        token_endpoint: Url,
        device: Oauth20DeviceAuthSuccessParams,
        next_poll: Instant,
        interval: Duration,
        deadline: Instant,
    },
    /// The flow ended with a stored token, holding its receipt.
    Issued(Value),
    /// The flow ended in failure, holding the `ortie-error` object.
    Failed(Value),
}

/// The outcome of one device poll.
enum McpDevicePoll {
    Pending,
    SlowDown,
    Done(Result<Value>),
}

/// The server: the account, the gates and the pending flow.
struct McpServer {
    account: Account,
    allow_revoke: bool,
    events: Sender<McpEvent>,
    flow: McpFlow,
    flows: u64,
}

impl McpServer {
    /// Answers one JSON-RPC message, `None` for a notification or a
    /// response.
    fn answer(&mut self, line: &str) -> Option<Value> {
        let message: Value = match serde_json::from_str(line) {
            Ok(message) => message,
            Err(err) => return Some(failure(Value::Null, -32700, format!("Parse error: {err}"))),
        };

        let id = message.get("id").cloned();
        let params = message.get("params").cloned().unwrap_or(Value::Null);

        let Some(method) = message.get("method").and_then(Value::as_str) else {
            // NOTE: a response to a request the server never sends.
            return id.map(|id| failure(id, -32600, String::from("Invalid request")));
        };

        let id = id?;

        let result = match method {
            "initialize" => Ok(json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": { "name": "ortie", "version": env!("CARGO_PKG_VERSION") },
                "instructions": INSTRUCTIONS,
            })),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(json!({ "tools": self.definitions() })),
            "tools/call" => self.call(&params),
            method => Err((-32601, format!("Method not found: {method}"))),
        };

        Some(match result {
            Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            Err((code, message)) => failure(id, code, message),
        })
    }

    /// The tools the gate lists.
    fn listed(&self) -> impl Iterator<Item = &'static McpTool> {
        let allow_revoke = self.allow_revoke;
        TOOLS
            .iter()
            .filter(move |tool| allow_revoke || tool.name != REVOKE_TOOL)
    }

    /// The definitions `tools/list` answers.
    fn definitions(&self) -> Vec<Value> {
        let schemas = json_schema::registry().schemas;

        self.listed()
            .map(|tool| {
                let mut properties = Map::new();
                let mut required = Vec::new();

                if let Some((name, description)) = tool.arg {
                    properties.insert(
                        name.into(),
                        json!({ "type": "string", "description": description }),
                    );
                    required.push(name);
                }

                let output = match tool.output {
                    McpToolOutput::Registry(key) => schemas.get(key).cloned(),
                    McpToolOutput::AuthGet => {
                        serde_json::to_value(schemars::schema_for!(McpAuthGetOutput)).ok()
                    }
                    McpToolOutput::AuthStatus => {
                        serde_json::to_value(schemars::schema_for!(McpAuthStatusOutput)).ok()
                    }
                };

                let mut definition = json!({
                    "name": tool.name,
                    "description": tool.description,
                    "inputSchema": {
                        "type": "object",
                        "properties": properties,
                        "required": required,
                        "additionalProperties": false,
                    },
                    "annotations": {
                        "readOnlyHint": tool.read_only,
                        "destructiveHint": tool.destructive,
                        "openWorldHint": !tool.read_only,
                    },
                });

                if let Some(output) = output {
                    definition["outputSchema"] = output;
                }

                definition
            })
            .collect()
    }

    /// Runs a tool call, a failure being a result with `isError` rather
    /// than a protocol error, so the caller reads it.
    fn call(&mut self, params: &Value) -> Result<Value, (i64, String)> {
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .ok_or((-32602, String::from("Missing tool name")))?;

        let tool = self
            .listed()
            .find(|tool| tool.name == name)
            .ok_or_else(|| (-32602, format!("Unknown tool: {name}")))?;

        let empty = Map::new();
        let arguments = match params.get("arguments") {
            None | Some(Value::Null) => &empty,
            Some(Value::Object(arguments)) => arguments,
            Some(_) => return Err((-32602, String::from("Arguments must be an object"))),
        };

        let outcome = tool
            .arguments(arguments)
            .map_err(report)
            .and_then(|arg| self.run_tool(tool.name, arg));

        Ok(match outcome {
            Ok(value) => json!({
                "content": [{ "type": "text", "text": value.to_string() }],
                "structuredContent": value,
            }),
            Err(report) => json!({
                "content": [{ "type": "text", "text": report.to_string() }],
                "isError": true,
            }),
        })
    }

    /// Runs the tool `name`, answering its value or the `ortie-error`
    /// object.
    fn run_tool(&mut self, name: &str, arg: Option<String>) -> Result<Value, Value> {
        match name {
            "token_inspect" => self.run(|p, a| TokenInspectCommand.execute(p, a)),
            "token_refresh" => self.run(|p, a| TokenRefreshCommand.execute(p, a)),
            "token_revoke" => self.run(|p, a| TokenRevokeCommand.execute(p, a)),
            "auth_get" => self.auth_get(),
            "auth_status" => Ok(self.auth_status()),
            "auth_resume" => self.auth_resume(arg.unwrap_or_default()),
            name => Err(anyhow!("Unknown tool: {name}").into()),
        }
        .map_err(|err| match err {
            McpRunError::Report(report) => report,
            McpRunError::Error(err) => report(err),
        })
    }

    /// Runs one command over the account, answering the single value it
    /// printed.
    fn run(
        &mut self,
        cmd: impl FnOnce(&mut McpPrinter, &mut Account) -> Result<()>,
    ) -> Result<Value, McpRunError> {
        let mut printer = McpPrinter::default();
        cmd(&mut printer, &mut self.account)?;

        let mut values = printer.values;
        match values.pop().filter(|_| values.is_empty()) {
            Some(value) => Ok(value),
            None => {
                Err(anyhow!("The command printed {} values, one expected", values.len()).into())
            }
        }
    }

    /// Starts the account's grant, replacing the pending flow.
    fn auth_get(&mut self) -> Result<Value, McpRunError> {
        self.cancel_flow();

        if self.account.grant == GrantConfig::Device {
            let (token_endpoint, device) = request_device_authorization(&mut self.account)?;
            let output = McpAuthGetOutput {
                kind: McpAuthGetKind::Device,
                user_code: Some(device.user_code.clone()),
                verification_uri: Some(device.verification_uri.clone()),
                verification_uri_complete: device.verification_uri_complete.clone(),
                ..McpAuthGetOutput::default()
            };

            let now = Instant::now();
            let interval = Duration::from_secs(device.interval as u64);
            self.flow = McpFlow::Device {
                token_endpoint,
                next_poll: now + interval,
                interval,
                deadline: now + Duration::from_secs(device.expires_in as u64),
                device,
            };

            return Ok(to_value(output)?);
        }

        if self.account.grant.is_client_credentials() {
            let receipt = self.run(|p, a| AuthGetCommand.execute(p, a, ""))?;
            self.flow = McpFlow::Issued(receipt.clone());

            let output = McpAuthGetOutput {
                kind: McpAuthGetKind::Issued,
                receipt: Some(receipt),
                ..McpAuthGetOutput::default()
            };

            return Ok(to_value(output)?);
        }

        self.auth_get_code()
    }

    /// Starts an authorization code flow, binding the loopback listener
    /// before the authorization URI goes out so no other process can
    /// take its port.
    fn auth_get_code(&mut self) -> Result<Value, McpRunError> {
        let (redirect_uri, listener) = match self.account.redirection_endpoint.clone() {
            Some(uri) if is_loopback_redirect(&uri) => {
                let (uri, listener) = bind_loopback(&uri)?;
                (uri, Some(listener))
            }
            Some(uri) => (uri, None),
            None => {
                let uri = DEFAULT_REDIRECTION.parse().map_err(anyhow::Error::from)?;
                let (uri, listener) = bind_loopback(&uri)?;
                (uri, Some(listener))
            }
        };

        let AuthorizationRequest { uri, state, pkce } =
            authorization_request(&self.account, &redirect_uri)?;

        self.flows += 1;
        let id = self.flows;

        let listening = match listener {
            None => None,
            Some(listener) => {
                let addr = listener.local_addr().map_err(anyhow::Error::from)?;
                let events = self.events.clone();
                let redirect_uri = redirect_uri.clone();

                thread::spawn(move || {
                    let result = await_redirect_on(&listener, &redirect_uri)
                        .map(|uri| uri.to_string())
                        .map_err(|err| err.to_string());
                    let _ = events.send(McpEvent::Redirect { flow: id, result });
                });

                Some(addr)
            }
        };

        self.flow = McpFlow::Code {
            id,
            state,
            pkce: pkce.map(|challenge| challenge.verifier),
            redirect_uri,
            listening,
        };

        let output = McpAuthGetOutput {
            kind: McpAuthGetKind::Authorization,
            authorization_uri: Some(uri.to_string()),
            redirect_captured: Some(listening.is_some()),
            ..McpAuthGetOutput::default()
        };

        Ok(to_value(output)?)
    }

    /// Reports the flow `auth_get` started.
    fn auth_status(&self) -> Value {
        let output = match &self.flow {
            McpFlow::None => McpAuthStatusOutput::new(McpAuthStatusKind::None),
            McpFlow::Code { .. } | McpFlow::Device { .. } => {
                McpAuthStatusOutput::new(McpAuthStatusKind::Pending)
            }
            McpFlow::Issued(receipt) => McpAuthStatusOutput {
                receipt: Some(receipt.clone()),
                ..McpAuthStatusOutput::new(McpAuthStatusKind::Issued)
            },
            McpFlow::Failed(error) => McpAuthStatusOutput {
                error: Some(error.clone()),
                ..McpAuthStatusOutput::new(McpAuthStatusKind::Failed)
            },
        };

        to_value(output).unwrap_or(Value::Null)
    }

    /// Completes the pending code flow from a redirected URI the
    /// listener did not catch.
    fn auth_resume(&mut self, uri: String) -> Result<Value, McpRunError> {
        let McpFlow::Code { .. } = self.flow else {
            return Err(
                anyhow!("No authorization code flow is pending, call auth_get first").into(),
            );
        };

        let McpFlow::Code {
            state,
            pkce,
            redirect_uri,
            listening,
            ..
        } = mem::replace(&mut self.flow, McpFlow::None)
        else {
            unreachable!();
        };

        if let Some(addr) = listening {
            wake(addr);
        }

        match self.exchange(uri, state, pkce, redirect_uri) {
            Ok(receipt) => {
                self.flow = McpFlow::Issued(receipt.clone());
                Ok(receipt)
            }
            Err(err) => {
                let report = report(err);
                self.flow = McpFlow::Failed(report.clone());
                Err(McpRunError::Report(report))
            }
        }
    }

    /// Takes the redirect the listener of flow `id` caught, exchanging
    /// it when that flow is still the pending one.
    fn redirected(&mut self, id: u64, result: Result<String, String>) {
        let McpFlow::Code { id: pending, .. } = &self.flow else {
            return;
        };

        if *pending != id {
            return;
        }

        let McpFlow::Code {
            state,
            pkce,
            redirect_uri,
            ..
        } = mem::replace(&mut self.flow, McpFlow::None)
        else {
            return;
        };

        let outcome = match result {
            Ok(uri) => self.exchange(uri, state, pkce, redirect_uri),
            Err(err) => Err(anyhow!("{err}").context("Capture the redirection error")),
        };

        self.flow = match outcome {
            Ok(receipt) => McpFlow::Issued(receipt),
            Err(err) => McpFlow::Failed(report(err)),
        };
    }

    /// Exchanges the code a redirected URI carries, as `auth resume`
    /// does, answering the receipt.
    fn exchange(
        &mut self,
        uri: String,
        state: Oauth20State,
        pkce: Option<Oauth20PkceCodeVerifier>,
        redirect_uri: Url,
    ) -> Result<Value> {
        let cmd = AuthResumeCommand {
            input: uri,
            state: Some(state),
            pkce,
            redirect_uri: Some(redirect_uri),
        };

        self.run(|p, a| cmd.execute(p, a)).map_err(|err| match err {
            McpRunError::Error(err) => err,
            McpRunError::Report(report) => anyhow!("{report}"),
        })
    }

    /// When the pending device flow polls next.
    fn next_poll(&self) -> Option<Instant> {
        match &self.flow {
            McpFlow::Device { next_poll, .. } => Some(*next_poll),
            _ => None,
        }
    }

    /// Polls the token endpoint once for the pending device flow.
    fn poll_device(&mut self) {
        let (token_endpoint, device, deadline) = match &self.flow {
            McpFlow::Device {
                token_endpoint,
                device,
                deadline,
                next_poll,
                ..
            } if Instant::now() >= *next_poll => {
                (token_endpoint.clone(), device.clone(), *deadline)
            }
            _ => return,
        };

        let poll = if Instant::now() >= deadline {
            let err = Oauth20ClientStdError::DeviceCodeExpired;
            if let Some(params) = device_poll_client_error_hook_params(&err) {
                debug!("execute issue access token error hook");
                self.account.execute_on_issue_error_hook(&params);
            }
            McpDevicePoll::Done(Err(err.into()))
        } else {
            self.poll_device_once(&token_endpoint, &device)
        };

        let slow_down = matches!(poll, McpDevicePoll::SlowDown);

        match poll {
            McpDevicePoll::Pending | McpDevicePoll::SlowDown => {
                if let McpFlow::Device {
                    next_poll,
                    interval,
                    ..
                } = &mut self.flow
                {
                    // NOTE: RFC 8628 section 3.5 adds 5 seconds to the
                    // interval at every slow_down.
                    if slow_down {
                        *interval += Duration::from_secs(5);
                    }
                    *next_poll = Instant::now() + *interval;
                }
            }
            McpDevicePoll::Done(Ok(receipt)) => self.flow = McpFlow::Issued(receipt),
            McpDevicePoll::Done(Err(err)) => self.flow = McpFlow::Failed(report(err)),
        }
    }

    /// Sends one device access token request and sorts its answer.
    fn poll_device_once(
        &mut self,
        token_endpoint: &Url,
        device: &Oauth20DeviceAuthSuccessParams,
    ) -> McpDevicePoll {
        let res = match request_device_token(&self.account, token_endpoint, device) {
            Ok(res) => res,
            Err(err) => return McpDevicePoll::Done(Err(err)),
        };

        match res {
            Ok(res) => {
                McpDevicePoll::Done(report_token_issued(&mut self.account, &res).and_then(to_value))
            }
            Err(res) if res.error == Oauth20AccessTokenErrorCode::AuthorizationPending => {
                McpDevicePoll::Pending
            }
            Err(res) if res.error == Oauth20AccessTokenErrorCode::SlowDown => {
                McpDevicePoll::SlowDown
            }
            Err(res) => {
                debug!("execute issue access token error hook");
                self.account.execute_on_issue_error_hook(&res);

                let err = anyhow!("Issue access token error (code {:?})", res.error);
                McpDevicePoll::Done(Err(match (res.error_description, res.error_uri) {
                    (None, None) => err,
                    (Some(desc), None) => anyhow!("{desc}").context(err),
                    (None, Some(uri)) => anyhow!("{uri}").context(err),
                    (Some(desc), Some(uri)) => anyhow!("{desc}: {uri}").context(err),
                }))
            }
        }
    }

    /// Drops the pending flow, stopping its listener.
    fn cancel_flow(&mut self) {
        if let McpFlow::Code {
            listening: Some(addr),
            ..
        } = &self.flow
        {
            wake(*addr);
        }

        self.flow = McpFlow::None;
    }
}

/// A tool failure: an error to report, or a report already built.
enum McpRunError {
    Error(anyhow::Error),
    Report(Value),
}

impl From<anyhow::Error> for McpRunError {
    fn from(err: anyhow::Error) -> Self {
        Self::Error(err)
    }
}

/// One tool: what it is called, what it takes and what it answers.
struct McpTool {
    name: &'static str,
    description: &'static str,
    /// The one string argument the tool takes, with its description.
    arg: Option<(&'static str, &'static str)>,
    output: McpToolOutput,
    read_only: bool,
    destructive: bool,
}

impl McpTool {
    /// Checks the call arguments against the tool's, answering the
    /// value of its argument.
    ///
    /// An unknown key is refused naming the known ones rather than
    /// ignored, so a misspelt argument never passes for a valid call.
    fn arguments(&self, arguments: &Map<String, Value>) -> Result<Option<String>> {
        let known = self.arg.map(|(name, _)| name);

        if let Some(key) = arguments.keys().find(|key| Some(key.as_str()) != known) {
            match known {
                Some(name) => bail!("Unknown argument `{key}`, {} takes `{name}`", self.name),
                None => bail!("Unknown argument `{key}`, {} takes none", self.name),
            }
        }

        let Some(name) = known else {
            return Ok(None);
        };

        match arguments.get(name) {
            Some(Value::String(value)) => Ok(Some(value.clone())),
            Some(_) => bail!("Argument `{name}` must be a string"),
            None => bail!("Missing argument `{name}`"),
        }
    }
}

/// Where a tool's output schema comes from.
enum McpToolOutput {
    /// The json-schema registry entry of the command it runs.
    Registry(&'static str),
    /// The `auth_get` answer.
    AuthGet,
    /// The `auth_status` answer.
    AuthStatus,
}

/// What `auth_get` answers: what the user must do next, never a secret.
#[derive(Debug, Default, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAuthGetOutput {
    /// The kind of flow started.
    pub kind: McpAuthGetKind,
    /// The URI the user opens to consent, on an authorization code flow.
    pub authorization_uri: Option<String>,
    /// Whether the server catches the redirection itself; when `false`,
    /// the redirected URL goes to `auth_resume`.
    pub redirect_captured: Option<bool>,
    /// The code the user enters, on a device flow.
    pub user_code: Option<String>,
    /// The page the user enters the code at, on a device flow.
    pub verification_uri: Option<String>,
    /// The verification page with the code already in it, when the
    /// server offers one.
    pub verification_uri_complete: Option<String>,
    /// The receipt of a token issued at once, on a client credentials
    /// account.
    #[schemars(with = "Option<TokenIssuedOutput>")]
    pub receipt: Option<Value>,
}

/// The kind of flow `auth_get` started.
#[derive(Debug, Default, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum McpAuthGetKind {
    /// An authorization code flow: the user opens the authorization URI.
    #[default]
    Authorization,
    /// A device flow: the user enters the code at the verification URI.
    Device,
    /// A client credentials grant, issued at once.
    Issued,
}

/// What `auth_status` answers.
#[derive(Debug, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAuthStatusOutput {
    /// Where the flow stands.
    pub status: McpAuthStatusKind,
    /// The receipt of the stored token, once issued.
    #[schemars(with = "Option<TokenIssuedOutput>")]
    pub receipt: Option<Value>,
    /// The `ortie-error` object, once failed.
    #[schemars(with = "Option<ErrorReport>")]
    pub error: Option<Value>,
}

impl McpAuthStatusOutput {
    fn new(status: McpAuthStatusKind) -> Self {
        Self {
            status,
            receipt: None,
            error: None,
        }
    }
}

/// Where the flow `auth_get` started stands.
#[derive(Debug, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum McpAuthStatusKind {
    /// No flow was started.
    None,
    /// The flow waits for the user.
    Pending,
    /// The token was issued and stored.
    Issued,
    /// The flow failed.
    Failed,
}

/// A [`Printer`] keeping each value and event a command prints as JSON.
#[derive(Default)]
struct McpPrinter {
    values: Vec<Value>,
}

impl Printer for McpPrinter {
    fn out<T: fmt::Display + Serialize>(&mut self, data: T) -> Result<()> {
        self.values.push(serde_json::to_value(&data)?);
        Ok(())
    }

    fn event<T: fmt::Display + Serialize>(&mut self, _event: T) -> Result<()> {
        Ok(())
    }

    fn out_bytes(&mut self, bytes: Bytes) -> Result<()> {
        self.out(bytes)
    }

    fn is_json(&self) -> bool {
        true
    }
}

/// Sends one device access token request for `device`.
fn request_device_token(
    account: &Account,
    token_endpoint: &Url,
    device: &Oauth20DeviceAuthSuccessParams,
) -> Result<Oauth20AccessTokenResponse> {
    let client_secret = account.client_secret.clone().map(Secret::get).transpose()?;
    let mut client = Oauth20ClientStd::connect(
        token_endpoint.clone(),
        &account.tls,
        account.client_id.clone(),
    )?;
    client.client_secret = client_secret;

    let res = client.request_device_access_token(Oauth20DeviceAccessTokenRequestParams {
        client_id: account.client_id.as_str().into(),
        device_code: device.device_code.clone(),
    })?;

    Ok(res)
}

/// Wakes the listener at `addr` so its thread ends: the empty request it
/// reads fails, and its flow no longer being pending, nobody reads it.
fn wake(addr: SocketAddr) {
    let _ = TcpStream::connect_timeout(&addr, Duration::from_secs(1));
}

/// Serializes `data`, an output type of this module.
fn to_value(data: impl Serialize) -> Result<Value> {
    Ok(serde_json::to_value(data)?)
}

/// The `ortie-error` object of `err`.
fn report(err: anyhow::Error) -> Value {
    let report: ErrorReport = err.into();
    serde_json::to_value(report).unwrap_or_else(|err| Value::from(err.to_string()))
}

/// A JSON-RPC error response.
fn failure(id: Value, code: i64, message: String) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}
