//! # Google data disclosure
//!
//! The disclosure of the Google data Pimalaya's own Google application
//! reaches, which [`crate::auth::get`] and the MCP `auth_get` tool obtain
//! the user's agreement to before asking Google for the user's consent.

use std::{
    fmt,
    io::{self, BufRead, IsTerminal, Write},
};

use anyhow::{Error, Result};
use pimalaya_cli::{fields::Fields, session::in_session};

use crate::{
    config::GrantConfig,
    error::{self, ErrorCode},
};

/// The client id of Pimalaya's Google Desktop application.
pub const PIMALAYA_GOOGLE_CLIENT_ID: &str =
    "803561575535-41415ghqi04uibrl6t5umg3p27stdhqr.apps.googleusercontent.com";

/// The privacy policy the disclosure summarises.
pub const PRIVACY_POLICY: &str = "https://pimalaya.org/privacy/";

/// The question the interactive disclosure ends with.
const PROMPT: &str = "Agree and continue? [y/N] ";

/// The failure of a disclosure declined at the prompt.
const DECLINED: &str = "Sign-in cancelled, the Google data disclosure of \
                        Pimalaya's application was not agreed to";

/// The Google data disclosure of an account on Pimalaya's application.
#[derive(Debug, Eq, PartialEq)]
pub struct AuthDisclosure {
    /// The Google services the account's scopes reach, in display order.
    pub services: Vec<AuthDisclosureService>,
}

impl AuthDisclosure {
    /// The disclosure an account of `client_id` running `grant` over
    /// `scopes` owes before the consent, `None` when it owes none.
    ///
    /// Only Pimalaya's Google application on a grant asking the user's
    /// consent owes one; scopes reaching no known service name them all.
    pub fn of(client_id: &str, grant: GrantConfig, scopes: &[String]) -> Option<Self> {
        if client_id != PIMALAYA_GOOGLE_CLIENT_ID || grant.is_client_credentials() {
            return None;
        }

        let mut services: Vec<_> = scopes
            .iter()
            .filter_map(|scope| AuthDisclosureService::of(scope))
            .collect();

        services.sort();
        services.dedup();

        if services.is_empty() {
            services = AuthDisclosureService::ALL.to_vec();
        }

        Some(Self { services })
    }

    /// Obtains the user's agreement before `auth get` sends anything.
    ///
    /// `agreed` is `--agree-data-use`, the disclosure then printed unless
    /// `json`. Otherwise a terminal outside a session is asked, and any
    /// other run refused coded `agreement-missing`.
    pub fn obtain(&self, agreed: bool, json: bool) -> Result<()> {
        if agreed {
            if !json {
                println!("{self}");
            }

            return Ok(());
        }

        let stdin = io::stdin();
        let terminal = stdin.is_terminal() && io::stdout().is_terminal();

        if json || !terminal || in_session() {
            return Err(self.refusal("run auth get again with --agree-data-use to agree to it"));
        }

        let agreed = self.ask(&mut stdin.lock(), &mut io::stdout())?;
        println!();

        if !agreed {
            return Err(error::coded(ErrorCode::AgreementMissing, DECLINED));
        }

        Ok(())
    }

    /// Prints the disclosure and its question on `output`, answering
    /// whether the line read from `input` agrees.
    pub fn ask(&self, input: &mut impl BufRead, output: &mut impl Write) -> Result<bool> {
        writeln!(output, "{self}")?;
        write!(output, "{PROMPT}")?;
        output.flush()?;

        let mut answer = String::new();
        input.read_line(&mut answer)?;

        let answer = answer.trim();
        Ok(answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes"))
    }

    /// The disclosure in one sentence, for a failure or an agent to show.
    pub fn sentence(&self) -> String {
        let phrases: Vec<_> = self.services.iter().map(|s| s.phrase()).collect();

        let reached = match phrases.split_last() {
            Some((last, [])) => last.to_string(),
            Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
            None => String::new(),
        };

        format!(
            "Pimalaya's Google application gives this token access to {reached}, \
             used only to carry out your own actions on this device: Pimalaya \
             runs no server, your data goes between Google and this device only \
             and is never sold or used for ads (privacy policy: {PRIVACY_POLICY})"
        )
    }

    /// The failure of a run that cannot ask, coded `agreement-missing`,
    /// the sentence followed by `how` to agree.
    pub fn refusal(&self, how: &str) -> Error {
        let message = format!("{}; {how}", self.sentence());
        error::coded(ErrorCode::AgreementMissing, message)
    }
}

impl fmt::Display for AuthDisclosure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fields = Fields::default();

        for service in &self.services {
            fields.push(service.name(), service.data());
        }

        writeln!(f, "Pimalaya and your Google data")?;
        writeln!(f)?;
        writeln!(
            f,
            "Signing in with Pimalaya's Google application gives this token access to:"
        )?;
        writeln!(f)?;
        writeln!(f, "{fields}")?;
        writeln!(
            f,
            "It is used only to carry out your own actions on this device. Pimalaya \
             runs no server: your data goes between Google and this device only, and \
             is never sold or used for ads. Read the privacy policy:"
        )?;
        writeln!(f)?;
        writeln!(f, "  {PRIVACY_POLICY}")
    }
}

/// A Google service the disclosure names, with the data it reaches.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AuthDisclosureService {
    /// Gmail, over its API, IMAP or SMTP.
    Gmail,
    /// Google Calendar, over its API or CalDAV.
    Calendar,
    /// Google Contacts, over the People API or CardDAV.
    Contacts,
    /// Google Tasks.
    Tasks,
}

impl AuthDisclosureService {
    /// Every service, in display order.
    const ALL: [Self; 4] = [Self::Gmail, Self::Calendar, Self::Contacts, Self::Tasks];

    /// The service a Google scope reaches, `None` for another scope.
    fn of(scope: &str) -> Option<Self> {
        if scope == "https://mail.google.com/" {
            return Some(Self::Gmail);
        }

        let api = scope.strip_prefix("https://www.googleapis.com/auth/")?;

        match api.split('.').next()? {
            "gmail" => Some(Self::Gmail),
            "calendar" => Some(Self::Calendar),
            "contacts" | "carddav" => Some(Self::Contacts),
            "tasks" => Some(Self::Tasks),
            _ => None,
        }
    }

    /// The service's name, the label of its row.
    fn name(self) -> &'static str {
        match self {
            Self::Gmail => "Gmail",
            Self::Calendar => "Google Calendar",
            Self::Contacts => "Google Contacts",
            Self::Tasks => "Google Tasks",
        }
    }

    /// The data the service reaches, the value of its row.
    fn data(self) -> &'static str {
        match self {
            Self::Gmail => "your messages, labels and sending addresses",
            Self::Calendar => "your calendars and events",
            Self::Contacts => "your contacts and labels",
            Self::Tasks => "your task lists and tasks",
        }
    }

    /// The data the service reaches, as the sentence names it.
    fn phrase(self) -> &'static str {
        match self {
            Self::Gmail => "your Gmail messages, labels and sending addresses",
            Self::Calendar => "your Google calendars and events",
            Self::Contacts => "your Google contacts and labels",
            Self::Tasks => "your Google task lists and tasks",
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use pimalaya_cli::error::code_of;

    use crate::{
        auth::disclosure::{AuthDisclosure, AuthDisclosureService, PIMALAYA_GOOGLE_CLIENT_ID},
        config::GrantConfig,
        error::ErrorCode,
    };

    /// The README's Google scopes.
    fn scopes() -> Vec<String> {
        [
            "https://mail.google.com/",
            "https://www.googleapis.com/auth/calendar",
            "https://www.googleapis.com/auth/contacts",
            "https://www.googleapis.com/auth/tasks",
        ]
        .map(String::from)
        .to_vec()
    }

    #[test]
    fn only_pimalayas_google_client_on_a_consent_grant_owes_the_disclosure() {
        let scopes = scopes();
        let of = |client_id, grant| AuthDisclosure::of(client_id, grant, &scopes).is_some();

        assert!(of(
            PIMALAYA_GOOGLE_CLIENT_ID,
            GrantConfig::AuthorizationCode
        ));
        assert!(of(PIMALAYA_GOOGLE_CLIENT_ID, GrantConfig::Device));
        assert!(!of(
            PIMALAYA_GOOGLE_CLIENT_ID,
            GrantConfig::ClientCredentials
        ));
        assert!(!of(
            PIMALAYA_GOOGLE_CLIENT_ID,
            GrantConfig::ClientCredentialsJwt
        ));
        assert!(!of(
            "000000000000-own.apps.googleusercontent.com",
            GrantConfig::AuthorizationCode
        ));
    }

    #[test]
    fn the_disclosure_names_the_services_the_scopes_reach() {
        use AuthDisclosureService::*;

        let services = |scopes: &[&str]| {
            let scopes: Vec<_> = scopes.iter().map(|s| s.to_string()).collect();
            let grant = GrantConfig::AuthorizationCode;
            AuthDisclosure::of(PIMALAYA_GOOGLE_CLIENT_ID, grant, &scopes)
                .unwrap()
                .services
        };

        let tasks_and_mail = [
            "https://www.googleapis.com/auth/tasks",
            "https://mail.google.com/",
            "https://www.googleapis.com/auth/gmail.send",
        ];
        assert_eq!(services(&tasks_and_mail), [Gmail, Tasks]);

        let narrow = [
            "https://www.googleapis.com/auth/calendar.events",
            "https://www.googleapis.com/auth/carddav",
        ];
        assert_eq!(services(&narrow), [Calendar, Contacts]);

        assert_eq!(services(&["openid"]), AuthDisclosureService::ALL);
        assert_eq!(services(&[]), AuthDisclosureService::ALL);
    }

    #[test]
    fn only_a_yes_agrees_at_the_prompt() {
        let grant = GrantConfig::AuthorizationCode;
        let disclosure = AuthDisclosure::of(PIMALAYA_GOOGLE_CLIENT_ID, grant, &scopes()).unwrap();

        let ask = |answer: &str| {
            let mut output = Vec::new();
            let agreed = disclosure
                .ask(&mut Cursor::new(answer.as_bytes()), &mut output)
                .unwrap();
            (agreed, String::from_utf8(output).unwrap())
        };

        for answer in ["y\n", "yes\n", "Y\n", " YES \n", "y"] {
            assert!(ask(answer).0, "{answer:?}");
        }

        for answer in ["\n", "", "n\n", "no\n", "yes please\n", "o\n"] {
            assert!(!ask(answer).0, "{answer:?}");
        }

        let (_, printed) = ask("\n");
        assert!(
            printed.starts_with("Pimalaya and your Google data\n"),
            "{printed}"
        );
        assert!(
            printed.contains("\nGmail:            your messages"),
            "{printed}"
        );
        assert!(
            printed.contains("\n  https://pimalaya.org/privacy/\n"),
            "{printed}"
        );
        assert!(
            printed.ends_with("\nAgree and continue? [y/N] "),
            "{printed}"
        );
    }

    #[test]
    fn the_refusal_is_coded_and_holds_the_disclosure() {
        let scopes = ["https://mail.google.com/".to_owned()];
        let grant = GrantConfig::AuthorizationCode;
        let disclosure = AuthDisclosure::of(PIMALAYA_GOOGLE_CLIENT_ID, grant, &scopes).unwrap();
        let err = disclosure.refusal("pass --agree-data-use");

        assert_eq!(code_of(&err), Some(ErrorCode::AgreementMissing));
        assert_eq!(
            err.to_string(),
            "Pimalaya's Google application gives this token access to your Gmail \
             messages, labels and sending addresses, used only to carry out your own \
             actions on this device: Pimalaya runs no server, your data goes between \
             Google and this device only and is never sold or used for ads (privacy \
             policy: https://pimalaya.org/privacy/); pass --agree-data-use"
        );
    }
}
