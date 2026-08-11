//! Scope step of the wizard: what the token is allowed to reach.
//!
//! It runs after the application step, because what may be requested
//! is a property of the application requesting it. A well-known public
//! application is verified for a fixed set of scopes and its
//! authorization request is refused on any other, so that set is the
//! whole list. Dynamic registration is bound only by what the
//! authorization server advertises, and sends the scopes inside the
//! registration request, so it prompts from within the application
//! step rather than after it. A custom application prompts for
//! nothing: the discovered scopes stay in the fragment, for the user
//! to adjust to what their own registration was granted.

use anyhow::Result;
use io_pim_discovery::rfc8414::DiscoveryOauthServerMetadata;
use pimalaya_cli::prompt;

use crate::wizard::{OauthConfig, advertised_scopes};

/// The scope options the application step leaves for the scope step.
pub enum Source {
    /// The set a well-known public application is registered for: the
    /// only scopes its client id can be granted.
    Registered(Vec<String>),
    /// The scopes the authorization server and the provider quirks
    /// advertise, widened by the discovered ones: what a client
    /// registered for this account may ask for.
    Advertised(Vec<String>),
    /// Nothing left to prompt: the application step sent the scopes in
    /// a registration request, or left the account without an
    /// application, which is the user's to fill in along with the
    /// scopes their own registration was granted.
    Taken,
}

/// Prompts for the scopes the token will carry, among the options the
/// application allows, with the discovered ones selected. Skipped when
/// there is nothing to choose from.
pub fn prompt(config: &mut OauthConfig, source: Source) -> Result<()> {
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

/// The options for a client bound to no registered set: the discovered
/// scopes first, since they are the narrow per-service ones the wizard
/// resolved, then everything the authorization server metadata and the
/// provider quirks advertise.
pub fn advertised(config: &OauthConfig, metadata: Option<&DiscoveryOauthServerMetadata>) -> Source {
    let mut options = config.scopes.clone();

    let supported = metadata
        .map(|metadata| metadata.scopes_supported.clone())
        .unwrap_or_default();

    let quirks = advertised_scopes(&config.endpoints)
        .into_iter()
        .map(ToString::to_string);

    for scope in supported.into_iter().chain(quirks) {
        if !options.contains(&scope) {
            options.push(scope);
        }
    }

    Source::Advertised(options)
}

#[cfg(test)]
mod tests {
    use crate::wizard::Endpoints;

    use super::*;

    #[test]
    fn advertised_options_lead_with_the_discovered_scopes() {
        let config = OauthConfig {
            endpoints: Endpoints {
                token: Some("https://api.fastmail.com/oauth/refresh".to_string()),
                ..Default::default()
            },
            scopes: vec!["urn:ietf:params:oauth:scope:mail".to_string()],
            ..OauthConfig::empty()
        };

        let Source::Advertised(options) = advertised(&config, None) else {
            panic!("expected advertised options");
        };

        // The discovered scope leads and is not repeated by the quirk
        // list it also belongs to.
        assert_eq!(options[0], "urn:ietf:params:oauth:scope:mail");
        assert_eq!(
            options.iter().filter(|scope| *scope == &options[0]).count(),
            1
        );
        assert!(options.contains(&"offline_access".to_string()));
    }

    #[test]
    fn a_provider_without_quirks_offers_only_what_was_discovered() {
        let config = OauthConfig {
            endpoints: Endpoints {
                token: Some("https://as.example.test/token".to_string()),
                ..Default::default()
            },
            scopes: vec!["mail".to_string()],
            ..OauthConfig::empty()
        };

        let Source::Advertised(options) = advertised(&config, None) else {
            panic!("expected advertised options");
        };

        assert_eq!(options, ["mail"]);
    }
}
