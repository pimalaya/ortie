use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoints {
    pub authorization: Url,
    pub token: Url,
    pub redirection: Option<Url>,
}

impl Endpoints {
    const DEFAULT_REDIRECTION_SCHEME: &'static str = "http";
    const DEFAULT_REDIRECTION_HOST: &'static str = "127.0.0.1";

    pub fn redirection_scheme(&self) -> &str {
        let Some(uri) = &self.redirection else {
            return Self::DEFAULT_REDIRECTION_SCHEME;
        };

        uri.scheme()
    }

    pub fn redirection_host(&self) -> &str {
        let Some(uri) = &self.redirection else {
            return Self::DEFAULT_REDIRECTION_HOST;
        };

        let Some(host) = uri.host_str() else {
            return Self::DEFAULT_REDIRECTION_HOST;
        };

        host
    }

    pub fn redirection_port(&self) -> u16 {
        let Some(uri) = &self.redirection else {
            return 80;
        };

        let Some(port) = uri.port_or_known_default() else {
            return if uri.scheme().eq_ignore_ascii_case("https") {
                443
            } else {
                80
            };
        };

        port
    }
}
