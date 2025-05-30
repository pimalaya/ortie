#[cfg(feature = "keyring")]
use io_keyring::Entry;
#[cfg(feature = "command")]
use io_process::Command;
use secrecy::ExposeSecret;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Secret {
    Raw(String),
    #[cfg(feature = "command")]
    Command(Command),
    #[cfg(feature = "keyring")]
    Keyring(Entry),
}

impl Into<Secret> for super::Secret {
    fn into(self) -> Secret {
        match self {
            Self::Raw(secret) => Secret::Raw(secret.expose_secret().to_owned()),
            #[cfg(feature = "command")]
            Self::Command(cmd) => Secret::Command(cmd),
            #[cfg(feature = "keyring")]
            Self::Keyring(entry) => Secret::Keyring(entry),
        }
    }
}
