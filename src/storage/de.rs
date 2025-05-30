#[allow(unused)]
use anyhow::{bail, Error};
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
pub enum Storage {
    #[cfg_attr(not(feature = "command"), serde(deserialize_with = "command"))]
    Command(Command),
    #[cfg_attr(not(feature = "keyring"), serde(deserialize_with = "keyring"))]
    Keyring(Entry),
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
