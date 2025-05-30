#[cfg(feature = "command")]
use std::process::Output;
#[cfg(feature = "notify")]
use std::{borrow::Cow, time::Duration};

#[allow(unused)]
use anyhow::Result;
#[cfg(feature = "notify")]
use humantime::format_duration;
use io_oauth::v2_0::{IssueAccessTokenErrorParams, IssueAccessTokenSuccessParams};
#[cfg(feature = "command")]
use io_process::{
    coroutines::SpawnThenWaitWithOutput, runtimes::std::handle as handle_process, Command,
};
use log::trace;
#[cfg(feature = "notify")]
use notify_rust::Notification;
#[cfg(feature = "command")]
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

#[cfg(feature = "notify")]
use crate::notify::NotifyHook;

use super::de;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Hooks {
    #[serde(default)]
    pub success: Hook,
    #[serde(default)]
    pub error: Hook,
}

impl Hooks {
    pub fn execute_success(&self, res: &IssueAccessTokenSuccessParams) {
        trace!("execute success hook: {res:?}");

        #[cfg(feature = "command")]
        if let Some(cmd) = &self.success.command {
            let mut cmd = cmd.clone();

            cmd.expand = true;
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

            if let Err(err) = self.execute_command(cmd) {
                log::debug!("execute command hook error: {err}");
            }
        }

        #[cfg(feature = "notify")]
        if let Some(config) = &self.success.notify {
            let home_dir = || dirs::home_dir().map(|p| p.to_string_lossy().to_string());
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

            let summary = match shellexpand::full_with_context(&config.summary, home_dir, get_env) {
                Ok(summary) => summary,
                Err(_) => (&config.summary).into(),
            };

            let body = match shellexpand::full_with_context(&config.body, home_dir, get_env) {
                Ok(body) => body,
                Err(_) => (&config.body).into(),
            };

            let notif = Notification::new().summary(&summary).body(&body).show();

            if let Err(err) = notif {
                log::debug!("execute notify hook error: {err}");
            }
        }
    }

    pub fn execute_error(&self, res: &IssueAccessTokenErrorParams) {
        trace!("execute error hook: {res:?}");

        #[cfg(feature = "command")]
        if let Some(cmd) = &self.error.command {
            let mut cmd = cmd.clone();

            cmd.expand = true;
            cmd.env("ERROR", &format!("{:?}", res.error));

            if let Some(desc) = &res.error_description {
                cmd.env("ERROR_DESCRIPTION", desc);
            }

            if let Some(uri) = &res.error_uri {
                cmd.env("ERROR_URI", uri);
            }

            if let Err(err) = self.execute_command(cmd) {
                log::debug!("execute command hook error: {err}");
            }
        }

        #[cfg(feature = "notify")]
        if let Some(config) = &self.error.notify {
            let home_dir = || dirs::home_dir().map(|p| p.to_string_lossy().to_string());
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

            let summary = match shellexpand::full_with_context(&config.summary, home_dir, get_env) {
                Ok(summary) => summary,
                Err(_) => (&config.summary).into(),
            };

            let body = match shellexpand::full_with_context(&config.body, home_dir, get_env) {
                Ok(body) => body,
                Err(_) => (&config.body).into(),
            };

            let notif = Notification::new().summary(&summary).body(&body).show();

            if let Err(err) = notif {
                log::debug!("execute notify hook error: {err}");
            }
        }
    }

    #[cfg(feature = "command")]
    pub fn execute_command(&self, cmd: Command) -> Result<()> {
        let mut spawn = SpawnThenWaitWithOutput::new(cmd);
        let mut arg = None;

        let Output {
            status,
            stdout,
            stderr,
        } = loop {
            match spawn.resume(arg.take()) {
                Ok(output) => break output,
                Err(io) => arg = Some(handle_process(io)?),
            }
        };

        log::debug!("successfully executed command hook");

        if log::log_enabled!(log::Level::Trace) {
            if status.success() {
                let out = String::from_utf8_lossy(&stdout);
                log::trace!("command hook stdout: {out}");
            } else {
                let bytes = if stdout.is_empty() { stderr } else { stdout };
                let err = anyhow::anyhow!("{}", String::from_utf8_lossy(&bytes));
                log::trace!("command hook stderr: {err}");
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(from = "de::Hook")]
pub struct Hook {
    #[cfg(feature = "command")]
    pub command: Option<Command>,
    #[cfg(feature = "notify")]
    pub notify: Option<NotifyHook>,
}
