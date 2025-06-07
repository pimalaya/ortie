#[allow(unused)]
use anyhow::Result;
#[allow(unused)]
use pimalaya_toolbox::feat;
use serde::Deserialize;

#[cfg(feature = "notify")]
use crate::notify::NotifyHook;
#[cfg(feature = "command")]
use io_process::Command;

#[cfg(not(feature = "notify"))]
pub type NotifyHook = ();
#[cfg(not(feature = "command"))]
pub type Command = ();

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hook {
    #[cfg_attr(not(feature = "command"), serde(default, deserialize_with = "command"))]
    pub command: Option<Command>,
    #[cfg_attr(not(feature = "notify"), serde(default, deserialize_with = "notify"))]
    pub notify: Option<NotifyHook>,
}

impl From<Hook> for super::Hook {
    fn from(#[allow(unused)] hook: Hook) -> Self {
        Self {
            #[cfg(feature = "command")]
            command: hook.command,
            #[cfg(feature = "notify")]
            notify: hook.notify,
        }
    }
}

#[cfg(not(feature = "command"))]
pub fn command<'de, T, D: serde::Deserializer<'de>>(_: D) -> Result<T, D::Error> {
    Err(serde::de::Error::custom(feat!("command")))
}

#[cfg(not(feature = "notify"))]
pub fn notify<'de, T, D: serde::Deserializer<'de>>(_: D) -> Result<T, D::Error> {
    Err(serde::de::Error::custom(feat!("notify")))
}
