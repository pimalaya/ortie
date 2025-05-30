#[cfg(feature = "command")]
use std::{
    io::{pipe, Write},
    process::Output,
};

#[allow(unused)]
use anyhow::{anyhow, bail, Context, Result};
#[cfg(feature = "keyring")]
use io_keyring::{
    coroutines::{Read as ReadEntry, Write as WriteEntry},
    runtimes::std::handle as handle_keyring,
    Entry,
};
use io_oauth::v2_0::IssueAccessTokenSuccessParams;
#[cfg(feature = "command")]
use io_process::{
    coroutines::SpawnThenWaitWithOutput, runtimes::std::handle as handle_process, Command,
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
    Keyring(Entry),
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
                        Ok(output) => break output,
                        Err(io) => arg = Some(handle_process(io)?),
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
                let mut read = ReadEntry::new(entry.clone());
                let mut arg = None;

                let secret = loop {
                    match read.resume(arg.take()) {
                        Ok(secret) => break secret,
                        Err(io) => arg = Some(handle_keyring(io)?),
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
                        Ok(output) => break output,
                        Err(io) => arg = Some(handle_process(io)?),
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
                let mut write = WriteEntry::new(entry.clone(), json);
                let mut arg = None;

                while let Err(io) = write.resume(arg.take()) {
                    arg = Some(handle_keyring(io)?)
                }

                Ok(())
            }
        }
    }
}
