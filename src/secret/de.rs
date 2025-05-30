#[allow(unused)]
use anyhow::{bail, Error};
use secrecy::SecretString;
use serde::Deserialize;

#[cfg(feature = "keyring")]
use io_keyring::Entry;
#[cfg(feature = "command")]
use io_process::Command;

#[cfg(not(feature = "keyring"))]
pub type Entry = ();
#[cfg(not(feature = "command"))]
pub type Command = ();

#[allow(unused)]
use crate::feat;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Secret {
    Raw(SecretString),
    #[cfg_attr(not(feature = "command"), serde(deserialize_with = "command"))]
    Command(Command),
    #[cfg_attr(not(feature = "keyring"), serde(deserialize_with = "keyring"))]
    Keyring(Entry),
}

impl TryFrom<Secret> for super::Secret {
    type Error = Error;

    fn try_from(secret: Secret) -> Result<Self, Self::Error> {
        match secret {
            Secret::Raw(secret) => Ok(Self::Raw(secret)),
            #[cfg(feature = "command")]
            Secret::Command(cmd) => Ok(Self::Command(cmd)),
            #[cfg(not(feature = "command"))]
            Secret::Command(_) => bail!(feat!("command")),
            #[cfg(feature = "keyring")]
            Secret::Keyring(entry) => Ok(Self::Keyring(entry)),
            #[cfg(not(feature = "keyring"))]
            Secret::Keyring(_) => bail!(feat!("keyring")),
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
