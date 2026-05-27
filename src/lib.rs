#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![doc = include_str!("../README.md")]

pub mod authorization_code_grant;
#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "client")]
pub mod client;
pub mod issue_access_token;
pub mod refresh_access_token;
