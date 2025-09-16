// This file is part of Ortie, a CLI to manage OAuth 2.0 access
// tokens.
//
// Copyright (C) 2025 soywod <clement.douin@posteo.net>
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU Affero General Public License
// as published by the Free Software Foundation, either version 3 of
// the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public
// License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

#[allow(unused)]
use anyhow::Result;
#[allow(unused)]
use pimalaya_toolbox::feat;
use serde::Deserialize;

#[cfg(feature = "notify")]
use crate::notify::NotifyHook;
#[cfg(feature = "command")]
use io_process::command::Command;

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
