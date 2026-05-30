//! Flat runtime account.
//!
//! Built from [`crate::cli::config::AccountConfig`] (the nested
//! TOML DTO) by flattening every `storage.*.command` and
//! `hooks.*.*.{command,notify}` into a direct field on this type.
//! Commands consume `Account` and call the driver methods
//! (`read_from_storage`, `write_to_storage`,
//! `execute_on_{issue,refresh}_{success,error}_hook`,
//! `redirection`) instead of walking the original config tree.

use alloc::{
    borrow::Cow,
    format,
    string::{String, ToString},
    vec::Vec,
};
#[cfg(feature = "notify")]
use std::time::Duration;
use std::{
    io::Write,
    net::TcpListener,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, anyhow, bail};
#[cfg(feature = "notify")]
use humantime::format_duration;
use log::trace;
#[cfg(feature = "notify")]
use notify_rust::Notification;
use pimalaya_config::secret::Secret;
use pimalaya_stream::tls::Tls;
use secrecy::ExposeSecret;
use url::Url;

use crate::{
    cli::config::{
        AccountConfig, EndpointsConfig, HookConfig, HookStatusConfig, HooksConfig, NotifyConfig,
        StorageConfig, StoragesConfig,
    },
    issue_access_token::{IssueAccessTokenErrorParams, IssueAccessTokenSuccessParams},
};

/// Flat, command-ready view of one OAuth 2.0 account.
#[derive(Debug)]
pub struct Account {
    pub default: bool,
    pub client_id: String,
    pub client_secret: Option<Secret>,
    pub tls: Tls,
    pub scopes: Vec<String>,
    pub pkce: bool,
    pub auto_refresh: bool,

    pub authorization_endpoint: Url,
    pub token_endpoint: Url,
    pub redirection_endpoint: Option<Url>,

    pub read_storage_command: Command,
    pub write_storage_command: Command,

    pub on_issue_success_hook_command: Option<Command>,
    pub on_issue_error_hook_command: Option<Command>,
    pub on_refresh_success_hook_command: Option<Command>,
    pub on_refresh_error_hook_command: Option<Command>,

    #[cfg(feature = "notify")]
    pub on_issue_success_hook_notify: Option<NotifyConfig>,
    #[cfg(feature = "notify")]
    pub on_issue_error_hook_notify: Option<NotifyConfig>,
    #[cfg(feature = "notify")]
    pub on_refresh_success_hook_notify: Option<NotifyConfig>,
    #[cfg(feature = "notify")]
    pub on_refresh_error_hook_notify: Option<NotifyConfig>,
}

impl From<AccountConfig> for Account {
    fn from(cfg: AccountConfig) -> Self {
        let AccountConfig {
            default,
            client_id,
            client_secret,
            endpoints,
            tls,
            scopes,
            pkce,
            auto_refresh,
            storage,
            hooks,
        } = cfg;

        let EndpointsConfig {
            authorization,
            token,
            redirection,
        } = endpoints;

        let StoragesConfig {
            read: StorageConfig { command: read_cmd },
            write: StorageConfig { command: write_cmd },
        } = storage;

        let HooksConfig {
            on_issue,
            on_refresh,
        } = hooks;

        let HookStatusConfig {
            success: issue_success,
            error: issue_error,
        } = on_issue;
        let HookStatusConfig {
            success: refresh_success,
            error: refresh_error,
        } = on_refresh;

        let HookConfig {
            command: on_issue_success_hook_command,
            #[cfg(feature = "notify")]
                notify: on_issue_success_hook_notify,
        } = issue_success;
        let HookConfig {
            command: on_issue_error_hook_command,
            #[cfg(feature = "notify")]
                notify: on_issue_error_hook_notify,
        } = issue_error;
        let HookConfig {
            command: on_refresh_success_hook_command,
            #[cfg(feature = "notify")]
                notify: on_refresh_success_hook_notify,
        } = refresh_success;
        let HookConfig {
            command: on_refresh_error_hook_command,
            #[cfg(feature = "notify")]
                notify: on_refresh_error_hook_notify,
        } = refresh_error;

        Self {
            default,
            client_id,
            client_secret,
            tls,
            scopes,
            pkce,
            auto_refresh,
            authorization_endpoint: authorization,
            token_endpoint: token,
            redirection_endpoint: redirection,
            read_storage_command: read_cmd,
            write_storage_command: write_cmd,
            on_issue_success_hook_command,
            on_issue_error_hook_command,
            on_refresh_success_hook_command,
            on_refresh_error_hook_command,
            #[cfg(feature = "notify")]
            on_issue_success_hook_notify,
            #[cfg(feature = "notify")]
            on_issue_error_hook_notify,
            #[cfg(feature = "notify")]
            on_refresh_success_hook_notify,
            #[cfg(feature = "notify")]
            on_refresh_error_hook_notify,
        }
    }
}

impl Account {
    /// Resolve the redirection URI: returns the configured one when
    /// set, otherwise binds to `127.0.0.1:0` and returns the
    /// resulting `http://127.0.0.1:<port>` URL.
    pub fn redirection(&self) -> Result<Cow<'_, Url>> {
        if let Some(url) = self.redirection_endpoint.as_ref() {
            return Ok(Cow::Borrowed(url));
        }

        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        let url: Url = format!("http://127.0.0.1:{port}").parse()?;

        Ok(Cow::Owned(url))
    }

    pub fn read_from_storage(&mut self) -> Result<IssueAccessTokenSuccessParams> {
        let cmd = &mut self.read_storage_command;

        let output = cmd
            .output()
            .context("Spawn command to read OAuth 2.0 access token")?;

        if !output.status.success() {
            let bytes = if output.stdout.is_empty() {
                output.stderr
            } else {
                output.stdout
            };
            let err = anyhow!("{}", String::from_utf8_lossy(&bytes));
            return Err(err.context("Read access token via command error"));
        }

        let res = IssueAccessTokenSuccessParams::try_from(output.stdout.as_slice())
            .context("Parse access token from command error")?;

        Ok(res)
    }

    pub fn write_to_storage(&mut self, res: &IssueAccessTokenSuccessParams) -> Result<()> {
        let cmd = &mut self.write_storage_command;
        let json = String::try_from(res)?.into_bytes();

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Spawn command to save OAuth 2.0 access token")?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(&json)
                .context("Write access token to command stdin")?;
        }

        let output = child
            .wait_with_output()
            .context("Wait for save command to finish")?;

        if !output.status.success() {
            let err = "Write access token via command error";

            let data = if output.stdout.is_empty() {
                output.stderr
            } else {
                output.stdout
            };

            if data.is_empty() {
                bail!(err);
            }

            let err2 = anyhow!("{}", String::from_utf8_lossy(&data));
            return Err(err2.context(err));
        }

        Ok(())
    }

    pub fn execute_on_issue_success_hook(&mut self, res: &IssueAccessTokenSuccessParams) {
        #[cfg(feature = "notify")]
        let notify = self.on_issue_success_hook_notify.as_ref();
        #[cfg(not(feature = "notify"))]
        let notify = None;
        execute_success_hook(self.on_issue_success_hook_command.as_mut(), notify, res);
    }

    pub fn execute_on_issue_error_hook(&mut self, res: &IssueAccessTokenErrorParams) {
        #[cfg(feature = "notify")]
        let notify = self.on_issue_error_hook_notify.as_ref();
        #[cfg(not(feature = "notify"))]
        let notify = None;
        execute_error_hook(self.on_issue_error_hook_command.as_mut(), notify, res);
    }

    pub fn execute_on_refresh_success_hook(&mut self, res: &IssueAccessTokenSuccessParams) {
        #[cfg(feature = "notify")]
        let notify = self.on_refresh_success_hook_notify.as_ref();
        #[cfg(not(feature = "notify"))]
        let notify = None;
        execute_success_hook(self.on_refresh_success_hook_command.as_mut(), notify, res);
    }

    pub fn execute_on_refresh_error_hook(&mut self, res: &IssueAccessTokenErrorParams) {
        #[cfg(feature = "notify")]
        let notify = self.on_refresh_error_hook_notify.as_ref();
        #[cfg(not(feature = "notify"))]
        let notify = None;
        execute_error_hook(self.on_refresh_error_hook_command.as_mut(), notify, res);
    }
}

fn execute_success_hook(
    cmd: Option<&mut Command>,
    #[cfg_attr(not(feature = "notify"), allow(unused))] notify: Option<&NotifyConfig>,
    res: &IssueAccessTokenSuccessParams,
) {
    trace!("execute success hook: {res:?}");

    if let Some(cmd) = cmd {
        cmd.env("ACCESS_TOKEN", res.access_token.expose_secret());
        cmd.env("TOKEN_TYPE", &res.token_type);

        if let Some(exp) = res.expires_in {
            cmd.env("EXPIRES_IN", exp.to_string());
        }

        if let Some(token) = &res.refresh_token {
            cmd.env("REFRESH_TOKEN", token.expose_secret());
        }

        if let Some(scope) = &res.scope {
            cmd.env("SCOPE", scope);
        }

        if let Err(err) = execute_command_hook(cmd) {
            log::debug!("execute command hook error: {err}");
        }
    }

    #[cfg(feature = "notify")]
    if let Some(config) = notify {
        let get_env = |key: &str| -> Result<Option<Cow<str>>, ()> {
            if key == "EXPIRES_IN" {
                return match res.expires_in {
                    None => Ok(Some("unknown".into())),
                    Some(exp) => {
                        let exp = Duration::from_secs(exp as u64 + 1);
                        Ok(Some(format_duration(exp).to_string().into()))
                    }
                };
            }

            if key == "TOKEN_TYPE" {
                let t = (&res.token_type).into();
                return Ok(Some(t));
            }

            match std::env::var(key) {
                Ok(val) => Ok(Some(val.into())),
                Err(_) => Ok(None),
            }
        };

        notify_with(config, get_env);
    }
}

fn execute_error_hook(
    cmd: Option<&mut Command>,
    #[cfg_attr(not(feature = "notify"), allow(unused))] notify: Option<&NotifyConfig>,
    res: &IssueAccessTokenErrorParams,
) {
    trace!("execute error hook: {res:?}");

    if let Some(cmd) = cmd {
        cmd.env("ERROR", format!("{:?}", res.error));

        if let Some(desc) = &res.error_description {
            cmd.env("ERROR_DESCRIPTION", desc);
        }

        if let Some(uri) = &res.error_uri {
            cmd.env("ERROR_URI", uri);
        }

        if let Err(err) = execute_command_hook(cmd) {
            log::debug!("execute command hook error: {err}");
        }
    }

    #[cfg(feature = "notify")]
    if let Some(config) = notify {
        let get_env = |key: &str| -> Result<Option<Cow<str>>, ()> {
            if key == "ERROR" {
                return Ok(Some(format!("{:?}", res.error).into()));
            }

            if key == "ERROR_DESCRIPTION" {
                return Ok(res.error_description.as_ref().map(Into::into));
            }

            if key == "ERROR_URI" {
                return Ok(res.error_uri.as_ref().map(Into::into));
            }

            match std::env::var(key) {
                Ok(val) => Ok(Some(val.into())),
                Err(_) => Ok(None),
            }
        };

        notify_with(config, get_env);
    }
}

fn execute_command_hook(cmd: &mut Command) -> Result<()> {
    let output = cmd
        .output()
        .map_err(|err| anyhow!("Spawn command hook error: {err}"))?;

    log::debug!("successfully executed command hook");

    if log::log_enabled!(log::Level::Trace) {
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout);
            log::trace!("command hook stdout: {out}");
        } else {
            let bytes = if output.stdout.is_empty() {
                &output.stderr
            } else {
                &output.stdout
            };
            let err = anyhow!("{}", String::from_utf8_lossy(bytes));
            log::trace!("command hook stderr: {err}");
        }
    }

    Ok(())
}

#[cfg(feature = "notify")]
fn notify_with<'a, F>(config: &'a NotifyConfig, get_env: F)
where
    F: Fn(&str) -> Result<Option<Cow<'a, str>>, ()> + Copy,
{
    let home_dir = || dirs::home_dir().map(|p| p.to_string_lossy().to_string());

    let summary = shellexpand::full_with_context(&config.summary, home_dir, get_env)
        .unwrap_or_else(|_| (&config.summary).into());

    let body = shellexpand::full_with_context(&config.body, home_dir, get_env)
        .unwrap_or_else(|_| (&config.body).into());

    let notif = Notification::new().summary(&summary).body(&body).show();

    if let Err(err) = notif {
        log::debug!("execute notify hook error: {err}");
    }
}
