#[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
use std::sync::Arc;
use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

use anyhow::{bail, Result};
#[cfg(feature = "native-tls")]
use native_tls::TlsConnector;
#[cfg(feature = "rustls-aws")]
use rustls::crypto::aws_lc_rs;
#[cfg(feature = "rustls-ring")]
use rustls::crypto::ring;
#[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
use rustls::{ClientConfig, ClientConnection, StreamOwned};
#[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
use rustls_platform_verifier::ConfigVerifierExt;
use serde::{Deserialize, Serialize};
use url::Url;

use super::de;

#[derive(Debug)]
pub enum Stream {
    Plain(TcpStream),
    #[cfg(feature = "native-tls")]
    NativeTls(native_tls::TlsStream<TcpStream>),
    #[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
    Rustls(StreamOwned<ClientConnection, TcpStream>),
}

impl Stream {
    pub fn connect(uri: &Url, tls: &Tls) -> Result<(String, Self)> {
        let Some(host) = uri.host_str() else {
            bail!("missing host in token endpoint: {uri}");
        };

        let Some(port) = uri.port_or_known_default() else {
            bail!("missing port in token endpoint: {uri}");
        };

        let stream = if uri.scheme().eq_ignore_ascii_case("https") {
            tls.connect(host)?
        } else {
            let tcp = TcpStream::connect((host, 80))?;
            Stream::Plain(tcp)
        };

        Ok((format!("{host}:{port}"), stream))
    }
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.read(buf),
            #[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
            Self::Rustls(stream) => stream.read(buf),
            #[cfg(feature = "native-tls")]
            Self::NativeTls(stream) => stream.read(buf),
        }
    }
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.write(buf),
            #[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
            Self::Rustls(stream) => stream.write(buf),
            #[cfg(feature = "native-tls")]
            Self::NativeTls(stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Plain(stream) => stream.flush(),
            #[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
            Self::Rustls(stream) => stream.flush(),
            #[cfg(feature = "native-tls")]
            Self::NativeTls(stream) => stream.flush(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "de::Tls")]
pub enum Tls {
    None,
    #[cfg(feature = "native-tls")]
    NativeTls,
    #[cfg(feature = "rustls-aws")]
    RustlsAws,
    #[cfg(feature = "rustls-ring")]
    RustlsRing,
}

impl Tls {
    #[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
    fn connect_rustls(host: &str) -> Result<StreamOwned<ClientConnection, TcpStream>> {
        let config = Arc::new(ClientConfig::with_platform_verifier());
        let server_name = host.to_owned().try_into()?;
        let conn = ClientConnection::new(config, server_name)?;
        let tcp = TcpStream::connect((host, 443))?;
        Ok(StreamOwned::new(conn, tcp))
    }

    pub fn connect(&self, host: &str) -> Result<Stream> {
        match self {
            Self::None => {
                bail!("missing TLS configuration to connet to {host}");
            }
            #[cfg(feature = "rustls-aws")]
            Self::RustlsAws => {
                let _ = aws_lc_rs::default_provider().install_default();
                let tls = Self::connect_rustls(host)?;
                Ok(Stream::Rustls(tls))
            }
            #[cfg(feature = "rustls-ring")]
            Self::RustlsRing => {
                let _ = ring::default_provider().install_default();
                let tls = Self::connect_rustls(host)?;
                Ok(Stream::Rustls(tls))
            }
            #[cfg(feature = "native-tls")]
            Self::NativeTls => {
                let connector = TlsConnector::new()?;
                let tcp = TcpStream::connect((host, 443))?;
                let tls = connector.connect(host, tcp)?;
                Ok(Stream::NativeTls(tls))
            }
        }
    }
}

#[cfg(not(feature = "native-tls"))]
#[cfg(not(feature = "rustls-aws"))]
#[cfg(not(feature = "rustls-ring"))]
impl Default for Tls {
    fn default() -> Self {
        Self::None
    }
}

#[cfg(feature = "native-tls")]
#[cfg(not(feature = "rustls-aws"))]
#[cfg(not(feature = "rustls-ring"))]
impl Default for Tls {
    fn default() -> Self {
        Self::NativeTls
    }
}

#[cfg(feature = "rustls-aws")]
impl Default for Tls {
    fn default() -> Self {
        Self::RustlsAws
    }
}

#[cfg(not(feature = "rustls-aws"))]
#[cfg(feature = "rustls-ring")]
impl Default for Tls {
    fn default() -> Self {
        Self::RustlsRing
    }
}
