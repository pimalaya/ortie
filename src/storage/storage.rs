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

#[cfg(feature = "command")]
use std::{
    io::{pipe, Write},
    process::Output,
};

#[allow(unused)]
use anyhow::{anyhow, bail, Context, Result};
#[cfg(feature = "keyring")]
use io_keyring::{
    coroutines::{
        read::{ReadSecret, ReadSecretResult},
        write::{WriteSecret, WriteSecretResult},
    },
    entry::KeyringEntry,
    runtimes::std::handle as handle_keyring,
};
use io_oauth::v2_0::issue_access_token::IssueAccessTokenSuccessParams;
#[cfg(feature = "command")]
use io_process::{
    command::Command,
    coroutines::spawn_then_wait_with_output::{
        SpawnThenWaitWithOutput, SpawnThenWaitWithOutputResult,
    },
    runtimes::std::handle as handle_process,
};
#[cfg(feature = "keyring")]
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

use super::de;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Storages {
    pub read: Storage,
    pub write: Storage,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "de::Storage")]
pub enum Storage {
    None,
    #[cfg(feature = "command")]
    Command(Command),
    #[cfg(feature = "keyring")]
    Keyring(KeyringEntry),
}

impl Storages {
    pub fn read(&self) -> Result<IssueAccessTokenSuccessParams> {
        match &self.read {
            Storage::None => bail!("Cannot read token: storage not defined"),
            #[cfg(feature = "command")]
            Storage::Command(cmd) => {
                let mut spawn = SpawnThenWaitWithOutput::new(cmd.clone());
                let mut arg = None;

                let Output {
                    status,
                    stdout,
                    stderr,
                } = loop {
                    match spawn.resume(arg.take()) {
                        SpawnThenWaitWithOutputResult::Ok(output) => break output,
                        SpawnThenWaitWithOutputResult::Io(io) => arg = Some(handle_process(io)?),
                        SpawnThenWaitWithOutputResult::Err(err2) => {
                            let err = "Spawn command to read OAuth 2.0 access token error";
                            return Err(anyhow!("{err2}").context(err));
                        }
                    }
                };

                if !status.success() {
                    let bytes = if stdout.is_empty() { stderr } else { stdout };
                    let err = anyhow!("{}", String::from_utf8_lossy(&bytes));
                    return Err(err.context("Read access token via command error"));
                };

                let mut res = IssueAccessTokenSuccessParams::try_from(stdout.as_slice())
                    .context("Parse access token from command error")?;

                res.sync_expires_in();

                Ok(res)
            }

            #[cfg(feature = "keyring")]
            Storage::Keyring(entry) => {
                let mut read = ReadSecret::new(entry.clone());
                let mut arg = None;

                let secret = loop {
                    match read.resume(arg.take()) {
                        ReadSecretResult::Ok(output) => break output,
                        ReadSecretResult::Io(io) => arg = Some(handle_keyring(io)?),
                        ReadSecretResult::Err(err2) => {
                            let err = "Read keyring to get OAuth 2.0 access token error";
                            return Err(anyhow!("{err2}").context(err));
                        }
                    }
                };

                let secret_bytes = secret.expose_secret().as_bytes();

                let mut res = IssueAccessTokenSuccessParams::try_from(secret_bytes)
                    .context("Parse access token from keyring error")?;

                res.sync_expires_in();

                Ok(res)
            }
        }
    }

    pub fn write(&self, #[allow(unused)] res: &IssueAccessTokenSuccessParams) -> Result<()> {
        match &self.write {
            Storage::None => bail!("Cannot write token: storage not defined"),
            #[cfg(feature = "command")]
            Storage::Command(cmd) => {
                let mut cmd = cmd.clone();
                let json = String::try_from(res)?.into_bytes();
                let (stdout, mut stdin) = pipe()?;

                cmd.stdin(stdout);
                stdin.write_all(&json)?;
                drop(stdin);

                let mut spawn = SpawnThenWaitWithOutput::new(cmd);
                let mut arg = None;

                let Output {
                    status,
                    stdout,
                    stderr,
                } = loop {
                    match spawn.resume(arg.take()) {
                        SpawnThenWaitWithOutputResult::Ok(output) => break output,
                        SpawnThenWaitWithOutputResult::Io(io) => arg = Some(handle_process(io)?),
                        SpawnThenWaitWithOutputResult::Err(err2) => {
                            let err = "Spawn command to save OAuth 2.0 access token error";
                            return Err(anyhow!("{err2}").context(err));
                        }
                    }
                };

                if !status.success() {
                    let err = "Write access token via command error";

                    let data = if stdout.is_empty() { stderr } else { stdout };
                    if data.is_empty() {
                        bail!(err);
                    }

                    let err2 = anyhow!("{}", String::from_utf8_lossy(&data));
                    return Err(err2.context(err));
                };

                Ok(())
            }
            #[cfg(feature = "keyring")]
            Storage::Keyring(entry) => {
                let json = String::try_from(res)?;
                let mut write = WriteSecret::new(entry.clone(), json);
                let mut arg = None;

                loop {
                    match write.resume(arg.take()) {
                        WriteSecretResult::Ok(()) => break,
                        WriteSecretResult::Io(io) => arg = Some(handle_keyring(io)?),
                        WriteSecretResult::Err(err2) => {
                            let err = "Read keyring to get OAuth 2.0 access token error";
                            return Err(anyhow!("{err2}").context(err));
                        }
                    }
                }

                Ok(())
            }
        }
    }
}
