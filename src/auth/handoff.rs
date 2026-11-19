//! # Auth handoff
//!
//! The flow a non-interactive or `--json` [`crate::auth::get`] hands off,
//! read back by `--handoff` of [`crate::auth::listen`] and
//! [`crate::auth::resume`], from a file or stdin, so the state and the
//! PKCE verifier stay off their command line, which other local users can
//! read.

use std::{
    fs::File,
    io::{self, Read},
};

use anyhow::{Context, Result, anyhow, bail};
use io_oauth::{rfc6749::state::Oauth20State, rfc7636::pkce::Oauth20PkceCodeVerifier};
use serde::Deserialize;
use url::Url;

/// The most bytes of stdin read as a handoff.
const HANDOFF_MAX: u64 = 64 * 1024;

/// The object `auth get --json` prints, its other keys ignored.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthHandoff {
    /// The CSRF state of the flow.
    pub state: Option<String>,
    /// The PKCE code verifier, absent when PKCE is off.
    pub pkce_code_verifier: Option<String>,
    /// The redirect URI the authorization request carried.
    pub redirect_uri: Option<Url>,
}

impl AuthHandoff {
    /// Reads the handoff from `source`, a path or `-` for stdin, which
    /// only `stdin_allowed` lets it read.
    ///
    /// A refusal names where the object breaks, never a value of it.
    pub fn read(source: &str, stdin_allowed: bool) -> Result<Self> {
        let mut input = Vec::new();

        if source == "-" {
            if !stdin_allowed {
                bail!("A REPL session reads its lines from stdin, give --handoff a file");
            }

            io::stdin()
                .lock()
                .take(HANDOFF_MAX)
                .read_to_end(&mut input)
                .context("Read the handoff on stdin")?;
        } else {
            File::open(source)
                .and_then(|file| file.take(HANDOFF_MAX).read_to_end(&mut input))
                .with_context(|| format!("Read the handoff file {source}"))?;
        }

        serde_json::from_slice(&input).map_err(|err| {
            let (line, column) = (err.line(), err.column());
            anyhow!(
                "Invalid handoff at line {line} column {column}, expected the \
                 object `auth get --json` prints"
            )
        })
    }

    /// Fills the flags a command left out from the handoff.
    pub fn fill(
        self,
        state: &mut Option<Oauth20State>,
        pkce: &mut Option<Oauth20PkceCodeVerifier>,
        redirect_uri: &mut Option<Url>,
    ) -> Result<()> {
        if let Some(handoff) = self.state {
            let handoff = Oauth20State::try_from(handoff.as_bytes())
                .map_err(|err| anyhow!("Invalid state in the handoff: {err}"))?;
            state.get_or_insert(handoff);
        }

        if let Some(handoff) = self.pkce_code_verifier {
            let handoff = Oauth20PkceCodeVerifier::try_from(handoff.as_bytes())
                .map_err(|err| anyhow!("Invalid PKCE verifier in the handoff: {err}"))?;
            pkce.get_or_insert(handoff);
        }

        if let Some(handoff) = self.redirect_uri {
            redirect_uri.get_or_insert(handoff);
        }

        Ok(())
    }
}
