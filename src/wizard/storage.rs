//! Storage step of the wizard: where the issued token is persisted.
//!
//! Ortie persists nothing itself, so an account is only usable once it
//! names a pair of shell commands: one printing the stored token JSON
//! on stdout, one receiving it on stdin. This is the Ortie twin of the
//! Himalaya wizard's credential prompt, and follows the same two steps
//! as the shared picker it mirrors: a strategy (a credential provider
//! CLI known for the running OS, or custom commands), then the entry
//! the secret lives under, seeded with the account name.
//!
//! The strategies are ordered by what the running system can actually
//! do: a provider whose CLI is on the `PATH` leads, one that is only
//! relevant for the OS follows and says so. Nothing is hidden, since a
//! provider missing today is one package install away and the config
//! the wizard writes for it is correct either way.
//!
//! The entry is used verbatim, exactly as in Himalaya, so a keyring
//! already holding a token for this account is named as it is rather
//! than under a namespace Ortie picked. Unlike Himalaya, which only
//! ever *reads* a secret the user stored, Ortie owns the value, so both
//! directions are recorded: the write command stores what a later
//! `ortie auth get` issues.

use std::{env, fmt};

use anyhow::Result;
use pimalaya_cli::{prompt, wizard::keyring::KeyringProvider};

use crate::wizard::{OauthConfig, Storage, StorageCommand, StorageEntry};

/// Runs the storage step against `config`, filling in its read and
/// write commands.
///
/// The pick list holds the credential provider CLIs relevant on the
/// running OS, the ones actually installed first, and ends with a
/// custom entry, so a platform with no known provider (Windows) is
/// offered the custom entry alone.
pub fn configure(config: &mut OauthConfig) -> Result<()> {
    let mut known: Vec<(KeyringProvider, bool)> = KeyringProvider::available()
        .into_iter()
        .map(|provider| (provider, installed(provider)))
        .collect();

    // NOTE: a stable sort, so the OS-native order the picker resolved
    // survives inside the installed and the missing group alike.
    known.sort_by_key(|(_, installed)| !installed);

    let mut choices: Vec<Choice> = known
        .into_iter()
        .map(|(provider, installed)| Choice::Known {
            provider,
            installed,
        })
        .collect();
    choices.push(Choice::Custom);

    let Choice::Known { provider, .. } = prompt::item("Token storage strategy:", choices, None)?
    else {
        return custom(config);
    };

    let entry = prompt::text("Token storage keyring entry:", Some(config.name.as_str()))?;
    config.storage = Some(keyring_storage(provider, entry.trim()));

    Ok(())
}

/// The read and write pair for a secret living at `entry` in
/// `provider`. The entry is used verbatim, so a pre-existing one is
/// read and written exactly as named.
///
/// The two halves land in different shapes because the picker hands
/// them over that way: reads are an argv, so no shell reinterprets an
/// entry name, while writes stay a shell line since some rely on shell
/// features (`$(cat)` on macOS).
fn keyring_storage(provider: KeyringProvider, entry: &str) -> Storage {
    Storage {
        read: StorageEntry {
            command: StorageCommand::Argv(provider.read_command(None, entry)),
        },
        write: StorageEntry {
            command: StorageCommand::Shell(provider.write_command(None, entry)),
        },
    }
}

/// Prompts for the custom storage commands, run through the platform
/// shell. The write prompt is skipped when the read command is left
/// empty for later, and the fragment keeps empty placeholders.
fn custom(config: &mut OauthConfig) -> Result<()> {
    let read = prompt::some_text::<&str>("Read command (leave empty for now):", None)?
        .filter(|command| !command.is_empty());

    let Some(read) = read else {
        return Ok(());
    };

    let write = prompt::text::<&str>("Write command (receives the token on stdin):", None)?;

    config.storage = Some(Storage {
        read: StorageEntry {
            command: StorageCommand::Shell(read),
        },
        write: StorageEntry {
            command: StorageCommand::Shell(write),
        },
    });

    Ok(())
}

/// Whether the provider's CLI can be found on the `PATH`, which is
/// what leads the pick list: a provider that is installed is one the
/// user can store a token in today, while the rest are a package
/// install away.
///
/// The provider names its own program, being the first element of its
/// read command, so no table of binary names is duplicated here.
fn installed(provider: KeyringProvider) -> bool {
    let argv = provider.read_command(None, "");

    let Some(program) = argv.first() else {
        return false;
    };

    let Some(paths) = env::var_os("PATH") else {
        return false;
    };

    env::split_paths(&paths).any(|dir| dir.join(program).is_file())
}

/// One entry in the token storage pick list: a well-known credential
/// provider CLI, or the trailing custom entry.
#[derive(Debug, Eq, PartialEq)]
enum Choice {
    Known {
        provider: KeyringProvider,
        installed: bool,
    },
    Custom,
}

impl fmt::Display for Choice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Known {
                provider,
                installed: true,
            } => write!(f, "{}", provider.name()),
            // Offered anyway: a provider can be installed right after,
            // and the config it yields is written all the same.
            Self::Known {
                provider,
                installed: false,
            } => write!(f, "{}, not found on PATH", provider.name()),
            Self::Custom => write!(f, "Custom shell commands"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_keyring_entry_is_used_verbatim() {
        let storage = keyring_storage(KeyringProvider::Pass, "ortie/posteo");

        assert_eq!(
            storage.read.command,
            StorageCommand::Argv(vec![
                "pass".to_string(),
                "show".to_string(),
                "ortie/posteo".to_string(),
            ])
        );
        assert_eq!(
            storage.write.command,
            StorageCommand::Shell("pass insert -m -f ortie/posteo".to_string())
        );
    }

    #[test]
    fn both_directions_target_the_same_entry() {
        let storage = keyring_storage(KeyringProvider::SecretTool, "posteo");

        assert_eq!(
            storage.read.command,
            StorageCommand::Argv(vec![
                "secret-tool".to_string(),
                "lookup".to_string(),
                "account".to_string(),
                "posteo".to_string(),
            ])
        );
        assert_eq!(
            storage.write.command,
            StorageCommand::Shell("secret-tool store --label posteo account posteo".to_string())
        );
    }
}
