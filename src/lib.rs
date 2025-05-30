#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![doc = include_str!("../README.md")]

pub mod account;
pub mod auth;
pub mod cli;
pub mod command;
pub mod completion;
pub mod config;
pub mod endpoint;
pub mod hook;
pub mod manual;
pub mod notify;
pub mod secret;
pub mod storage;
pub mod stream;
pub mod token;

#[macro_export]
macro_rules! feat {
    ($feat:literal) => {
        format!("missing `{}` cargo feature", $feat)
    };
}
