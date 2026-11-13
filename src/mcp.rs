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
//! feed it events. pimalaya-cli's [`mcp`] module answers the protocol and
//! checks each tool call, this one runs the tools and their flows.
//!
//! [`mcp`]: pimalaya_cli::mcp

use std::{
    io::{self, Read, Write},
    mem,
    net::{SocketAddr, TcpStream},
    sync::mpsc::{self, RecvTimeoutError, Sender},
    thread::{self, JoinHandle},
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
use pimalaya_cli::{
    clap::commands::json_schema::output_schema,
    error::{ErrorReport, NoErrorCode},
    mcp::{
        Input, ServerInfo, ToolCall, ToolResult, answer, error_object, send, spawn_stdin_reader,
        tool_output_schema,
    },
    printer::CapturePrinter,
};
use pimalaya_config::secret::Secret;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Map, Value, json};
use url::{Position, Url};

use crate::{
    account::{Account, DEFAULT_REDIRECTION, bind_loopback},
    auth::{
        get::{
            AuthGetCommand, AuthorizationRequest, authorization_request, is_loopback_redirect,
            report_token_issued, request_device_authorization,
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
        open_world: false,
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
        open_world: true,
    },
    McpTool {
        name: "auth_status",
        description: "Report the flow auth_get started: none, pending, issued with the \
                      receipt, or failed with the error.",
        arg: None,
        output: McpToolOutput::AuthStatus,
        read_only: true,
        destructive: false,
        open_world: false,
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
        open_world: true,
    },
    McpTool {
        name: "token_refresh",
        description: "Renew the stored token with its refresh token, or re-run the grant on \
                      a client credentials account. Returns the receipt.",
        arg: None,
        output: McpToolOutput::Registry("ortie-token-refresh"),
        read_only: false,
        destructive: false,
        open_world: true,
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
        open_world: true,
    },
];

/// The tool only `--allow-revoke` lists.
const REVOKE_TOOL: &str = "token_revoke";

/// How long a stopped listener is given to answer.
const LISTENER_STOP_TIMEOUT: Duration = Duration::from_secs(2);

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
        spawn_stdin_reader(events.clone(), McpEvent::Stdin);

        let mut server = McpServer {
            account,
            allow_revoke: self.allow_revoke,
            events,
            flow: McpFlow::None,
            flows: 0,
        };

        let info = ServerInfo {
            name: "ortie",
            version: env!("CARGO_PKG_VERSION"),
            instructions: Some(INSTRUCTIONS),
        };

        let tools = server.definitions();
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
                McpEvent::Stdin(Input::Eof) => break,
                McpEvent::Stdin(Input::Line(line)) => {
                    let response = answer(&line, &info, &tools, |call| server.call(call));

                    if let Some(response) = response {
                        send(&mut stdout, &response)?;
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
    /// A line read off stdin, one JSON-RPC message, or its end.
    Stdin(Input),
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
        /// The listener catching the redirect, `None` when the
        /// redirection is not loopback.
        listening: Option<McpListener>,
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

/// The loopback listener of a code flow, waiting on its own thread.
struct McpListener {
    addr: SocketAddr,
    thread: JoinHandle<()>,
}

impl McpListener {
    /// Ends the listener with the one request it takes, a refusal
    /// echoing `state`, then waits for its port to be free.
    ///
    /// Its flow no longer being the pending one, nobody reads the
    /// refusal.
    fn stop(self, redirect_uri: &Url, state: &Oauth20State) {
        let Ok(mut stream) = TcpStream::connect_timeout(&self.addr, LISTENER_STOP_TIMEOUT) else {
            return;
        };

        let mut target = redirect_uri.clone();
        target.set_fragment(None);
        target
            .query_pairs_mut()
            .clear()
            .append_pair("error", "access_denied")
            .append_pair("state", &String::from_utf8_lossy(state.expose()));

        let request = format!("GET {} HTTP/1.0\r\n\r\n", &target[Position::BeforePath..]);
        let mut answer = Vec::new();

        // NOTE: a listener busy with another connection answers late, and
        // still ends on this request: it is then left to end by itself.
        let answered = stream
            .set_read_timeout(Some(LISTENER_STOP_TIMEOUT))
            .and_then(|()| stream.write_all(request.as_bytes()))
            .and_then(|()| stream.read_to_end(&mut answer))
            .is_ok();

        if answered && answer.starts_with(b"HTTP/1.0 200") {
            let _ = self.thread.join();
        }
    }
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
    /// The tools the gate lists.
    fn listed(&self) -> impl Iterator<Item = &'static McpTool> {
        let allow_revoke = self.allow_revoke;
        TOOLS
            .iter()
            .filter(move |tool| allow_revoke || tool.name != REVOKE_TOOL)
    }

    /// The definitions `tools/list` answers, each output schema the
    /// printed shape of what the tool answers.
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
                    McpToolOutput::AuthGet => Some(output_schema::<McpAuthGetOutput>()),
                    McpToolOutput::AuthStatus => Some(output_schema::<McpAuthStatusOutput>()),
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
                        "openWorldHint": tool.open_world,
                    },
                });

                if let Some(output) = output.as_ref().and_then(tool_output_schema) {
                    definition["outputSchema"] = output.clone();
                }

                definition
            })
            .collect()
    }

    /// Runs a tool call, listed and holding only the arguments its tool
    /// declares, a failure being a result with `isError` rather than a
    /// protocol error, so the caller reads it.
    fn call(&mut self, call: &ToolCall) -> ToolResult {
        let Some(tool) = self.listed().find(|tool| tool.name == call.name) else {
            return tool_result(Err(anyhow!("Unknown tool: {}", call.name)));
        };

        let arg = match tool.argument(&call.arguments) {
            Ok(arg) => arg,
            Err(err) => return tool_result(Err(err)),
        };

        match tool.name {
            "token_inspect" => self.run(|p, a| TokenInspectCommand.execute(p, a)),
            "token_refresh" => self.run(|p, a| TokenRefreshCommand.execute(p, a)),
            "token_revoke" => self.run(|p, a| TokenRevokeCommand.execute(p, a)),
            "auth_get" => tool_result(self.auth_get()),
            "auth_status" => ToolResult::success(self.auth_status()),
            "auth_resume" => self.auth_resume(arg.unwrap_or_default()),
            name => tool_result(Err(anyhow!("Unknown tool: {name}"))),
        }
    }

    /// Runs one command over the account, answering what it printed.
    fn run(
        &mut self,
        cmd: impl FnOnce(&mut CapturePrinter, &mut Account) -> Result<()>,
    ) -> ToolResult {
        let mut printer = CapturePrinter::default();
        let result = cmd(&mut printer, &mut self.account);
        ToolResult::printed::<NoErrorCode>(printer, result)
    }

    /// Runs one command over the account, answering the single value it
    /// printed.
    fn capture(
        &mut self,
        cmd: impl FnOnce(&mut CapturePrinter, &mut Account) -> Result<()>,
    ) -> Result<Value> {
        let mut printer = CapturePrinter::default();
        cmd(&mut printer, &mut self.account)?;
        printer.into_single()
    }

    /// Starts the account's grant, replacing the pending flow.
    fn auth_get(&mut self) -> Result<Value> {
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
            let interval = device.poll_interval();
            let deadline = now + device.lifetime();
            self.flow = McpFlow::Device {
                token_endpoint,
                next_poll: deadline.min(now + interval),
                interval,
                deadline,
                device,
            };

            return to_value(output);
        }

        if self.account.grant.is_client_credentials() {
            let receipt = self.capture(|p, a| AuthGetCommand.execute(p, a, ""))?;
            self.flow = McpFlow::Issued(receipt.clone());

            let output = McpAuthGetOutput {
                kind: McpAuthGetKind::Issued,
                receipt: Some(receipt),
                ..McpAuthGetOutput::default()
            };

            return to_value(output);
        }

        self.auth_get_code()
    }

    /// Starts an authorization code flow, binding the loopback listener
    /// before the authorization URI goes out so no other process can
    /// take its port.
    fn auth_get_code(&mut self) -> Result<Value> {
        let (redirect_uri, listener) = match self.account.redirection_endpoint.clone() {
            Some(uri) if is_loopback_redirect(&uri) => {
                let (uri, listener) = bind_loopback(&uri)?;
                (uri, Some(listener))
            }
            Some(uri) => (uri, None),
            None => {
                let uri: Url = DEFAULT_REDIRECTION.parse()?;
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
                let addr = listener.local_addr()?;
                let events = self.events.clone();
                let redirect_uri = redirect_uri.clone();
                let state = state.clone();

                let thread = thread::spawn(move || {
                    let result = await_redirect_on(&listener, &redirect_uri, Some(&state))
                        .map(|uri| uri.to_string())
                        .map_err(|err| err.to_string());
                    let _ = events.send(McpEvent::Redirect { flow: id, result });
                });

                Some(McpListener { addr, thread })
            }
        };

        let redirect_captured = listening.is_some();

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
            redirect_captured: Some(redirect_captured),
            ..McpAuthGetOutput::default()
        };

        to_value(output)
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
    fn auth_resume(&mut self, uri: String) -> ToolResult {
        let McpFlow::Code { .. } = self.flow else {
            let err = anyhow!("No authorization code flow is pending, call auth_get first");
            return tool_result(Err(err));
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

        if let Some(listener) = listening {
            listener.stop(&redirect_uri, &state);
        }

        match self.exchange(uri, state, pkce, redirect_uri) {
            Ok(receipt) => {
                self.flow = McpFlow::Issued(receipt.clone());
                ToolResult::success(receipt)
            }
            Err(err) => {
                let report = error_object::<NoErrorCode>(err);
                self.flow = McpFlow::Failed(report.clone());
                ToolResult::failure(report)
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
            Err(err) => McpFlow::Failed(error_object::<NoErrorCode>(err)),
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

        self.capture(|p, a| cmd.execute(p, a))
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
            McpDevicePoll::Done(Err(Oauth20ClientStdError::DeviceCodeExpired.into()))
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
                        *interval = interval.saturating_add(Duration::from_secs(5));
                    }
                    *next_poll = deadline.min(Instant::now() + *interval);
                }
            }
            McpDevicePoll::Done(Ok(receipt)) => self.flow = McpFlow::Issued(receipt),
            McpDevicePoll::Done(Err(err)) => {
                self.flow = McpFlow::Failed(error_object::<NoErrorCode>(err))
            }
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
            state,
            redirect_uri,
            listening: Some(listener),
            ..
        } = mem::replace(&mut self.flow, McpFlow::None)
        {
            listener.stop(&redirect_uri, &state);
        }
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
    /// Whether the tool reaches the authorization server, an entity
    /// outside the server and its account.
    open_world: bool,
}

impl McpTool {
    /// The value of the tool's argument in a call whose keys are checked
    /// already, failing when it is missing or not a string.
    fn argument(&self, arguments: &Map<String, Value>) -> Result<Option<String>> {
        let Some((name, _)) = self.arg else {
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

/// Serializes `data`, an output type of this module.
fn to_value(data: impl Serialize) -> Result<Value> {
    Ok(serde_json::to_value(data)?)
}

/// The tool result of `outcome`, a failure carrying the `ortie-error`
/// object.
fn tool_result(outcome: Result<Value>) -> ToolResult {
    match outcome {
        Ok(value) => ToolResult::success(value),
        Err(err) => ToolResult::error::<NoErrorCode>(err),
    }
}
