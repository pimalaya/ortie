//! # Scope step
//!
//! The wizard step deciding what the token is allowed to reach, run
//! after the application step since what may be requested is a property
//! of the application requesting it.
//!
//! A well-known public application is verified for a fixed set of
//! scopes and refused on any other, so that set is the whole list.
//!
//! Dynamic registration is bound only by what the server advertises,
//! and prompts from within the application step, its scopes travelling
//! inside the registration request.
//!
//! A custom application prompts for nothing: the discovered scopes stay
//! in the fragment, for the user to adjust to what their own
//! registration was granted.

use anyhow::Result;
use io_pim_discovery::rfc8414::DiscoveryOauthServerMetadata;
use pimalaya_cli::prompt;

use crate::wizard::AccountDraft;

/// The scope options the application step leaves for the scope step.
pub enum Source {
    /// The set a well-known public application is registered for, the
    /// only scopes its client id can be granted.
    Registered(Vec<String>),
    /// What the authorization server and the provider quirks advertise,
    /// widened by the discovered scopes.
    Advertised(Vec<String>),
    /// Nothing left to prompt, the application step having sent the
    /// scopes already or left the account without an application.
    Taken,
}

/// Prompts for the scopes the token will carry, among the options the
/// application allows and with the discovered ones selected.
///
/// Skipped when there is nothing to choose from.
pub fn prompt(config: &mut AccountDraft, source: Source) -> Result<()> {
    let options = match source {
        Source::Registered(options) | Source::Advertised(options) => options,
        Source::Taken => return Ok(()),
    };

    if options.is_empty() {
        return Ok(());
    }

    let selected: Vec<usize> = options
        .iter()
        .enumerate()
        .filter_map(|(index, option)| config.scopes.contains(option).then_some(index))
        .collect();

    config.scopes = prompt::items("Scopes:", options, selected)?;

    Ok(())
}

/// The options for a client bound to no registered set.
///
/// The discovered scopes lead, being the narrow per-service ones the
/// wizard resolved, then comes everything the server metadata
/// advertises.
pub fn advertised(
    config: &AccountDraft,
    metadata: Option<&DiscoveryOauthServerMetadata>,
) -> Source {
    let mut options = config.scopes.clone();

    let supported = metadata
        .map(|metadata| metadata.scopes_supported.clone())
        .unwrap_or_default();

    for scope in supported {
        if !options.contains(&scope) {
            options.push(scope);
        }
    }

    Source::Advertised(options)
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value, json};

    use super::*;

    /// The discovered scopes lead, and the metadata list widens them
    /// without repeating one.
    #[test]
    fn advertised_options_lead_with_the_discovered_scopes() {
        let config = AccountDraft {
            scopes: vec!["mail".to_string()],
            ..AccountDraft::empty()
        };
        let metadata: DiscoveryOauthServerMetadata = from_value(json!({
            "issuer": "https://as.example.test/",
            "scopes_supported": ["openid", "mail", "offline_access"],
        }))
        .unwrap();

        let Source::Advertised(options) = advertised(&config, Some(&metadata)) else {
            panic!("expected advertised options");
        };

        assert_eq!(options, ["mail", "openid", "offline_access"]);
    }

    #[test]
    fn without_metadata_only_the_discovered_scopes_are_offered() {
        let config = AccountDraft {
            scopes: vec!["mail".to_string()],
            ..AccountDraft::empty()
        };

        let Source::Advertised(options) = advertised(&config, None) else {
            panic!("expected advertised options");
        };

        assert_eq!(options, ["mail"]);
    }
}
