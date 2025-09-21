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
use anyhow::{bail, Error};
use serde::Deserialize;

#[cfg(feature = "keyring")]
use io_keyring::entry::KeyringEntry;
#[cfg(feature = "command")]
use io_process::command::Command;

#[cfg(not(feature = "keyring"))]
pub type KeyringEntry = ();
#[cfg(not(feature = "command"))]
pub type Command = ();

#[allow(unused)]
use pimalaya_toolbox::feat;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Storage {
    #[cfg_attr(not(feature = "command"), serde(deserialize_with = "command"))]
    Command(Command),
    #[cfg_attr(not(feature = "keyring"), serde(deserialize_with = "keyring"))]
    Keyring(KeyringEntry),
}

impl TryFrom<Storage> for super::Storage {
    type Error = Error;

    fn try_from(storage: Storage) -> Result<Self, Self::Error> {
        match storage {
            #[cfg(feature = "command")]
            Storage::Command(cmd) => Ok(Self::Command(cmd)),
            #[cfg(not(feature = "command"))]
            Storage::Command(_) => bail!(feat!("command")),

            #[cfg(feature = "keyring")]
            Storage::Keyring(entry) => Ok(Self::Keyring(entry)),
            #[cfg(not(feature = "keyring"))]
            Storage::Keyring(_) => bail!(feat!("keyring")),
        }
    }
}

#[cfg(not(feature = "command"))]
pub fn command<'de, T, D: serde::Deserializer<'de>>(_: D) -> Result<T, D::Error> {
    Err(serde::de::Error::custom(feat!("command")))
}

#[cfg(not(feature = "keyring"))]
pub fn keyring<'de, T, D: serde::Deserializer<'de>>(_: D) -> Result<T, D::Error> {
    Err(serde::de::Error::custom(feat!("keyring")))
}
