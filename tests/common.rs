//! # Shared test helpers
//!
//! The account configuration every suite writes, its token stored in a
//! file beside it, the built binary run against it, the provider
//! sections of config.sample.toml, and the environment the live suites
//! read their credentials from.

#![allow(dead_code)]

use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use serde_json::Value;

/// Deterministic 2048-bit test key, the RustCrypto pkcs8 crate's
/// rsa2048-priv.pem example (mirrors the io-oauth test fixture).
pub const KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC2xCxRXxCmqvKC
xj7b4kJDoXDz+iYzvUgzY39Hyk9vNuA6XSnvwxkayA85DYdLOeMPQU/Owfyg7YHl
R+3CzTgsdvYckBiXPbn6U3lyp8cB9rd+CYLfwV/AGSfuXnzZS09Zn/BwE6fIKBvf
Ity8mtfKu3xDEcmC9Y7bchOtRVizMiZtdDrtgZLRiEytuLFHOaja2mbclwgG2ces
RQyxPQ18V1+xmFNPxhvEG8DwV04OATDHu7+9/cn2puLj4q/xy+rIm6V4hFKNVc+w
gyeh6MifTgA88oiOkzJB2daVvLus3JC0Tj4JX6NwWOolsT9eKVy+rG3oOKuMUK9h
4piXW4cvAgMBAAECggEAfsyDYsDtsHQRZCFeIvdKudkboGkAcAz2NpDlEU2O5r3P
uy4/lhRpKmd6CD8Wil5S5ZaOZAe52XxuDkBk+C2gt1ihTxe5t9QfX0jijWVRcE9W
5p56qfpjD8dkKMBtJeRV3PxVt6wrT3ZkP97T/hX/eKuyfmWsxKrQvfbbJ+9gppEM
XEoIXtQydasZwdmXoyxu/8598tGTX25gHu3hYaErXMJ8oh+B0smcPR6gjpDjBTqw
m++nJN7w0MOjwel0DA2fdhJqFJ7Aqn2AeCBUhCVNlR2wfEz5H7ZFTAlliP1ZJNur
6zWcogJSaNAE+dZus9b3rcETm61A8W3eY54RZHN2wQKBgQDcwGEkLU6Sr67nKsUT
ymW593A2+b1+Dm5hRhp+92VCJewVPH5cMaYVem5aE/9uF46HWMHLM9nWu+MXnvGJ
mOQi7Ny+149Oz9vl9PzYrsLJ0NyGRzypvRbZ0jjSH7Xd776xQ8ph0L1qqNkfM6CX
eQ6WQNvJEIXcXyY0O6MTj2stZwKBgQDT8xR1fkDpVINvkr4kI2ry8NoEo0ZTwYCv
Z+lgCG2T/eZcsj79nQk3R2L1mB42GEmvaM3XU5T/ak4G62myCeQijbLfpw5A9/l1
ClKBdmR7eI0OV3eiy4si480mf/cLTzsC06r7DhjFkKVksDGIsKpfxIFWsHYiIUJD
vRIn76fy+QKBgQDOaLesGw0QDWNuVUiHU8XAmEP9s5DicF33aJRXyb2Nl2XjCXhh
fi78gEj0wyQgbbhgh7ZU6Xuz1GTn7j+M2D/hBDb33xjpqWPE5kkR1n7eNAQvLibj
06GtNGra1rm39ncIywlOYt7p/01dZmmvmIryJV0c6O0xfGp9hpHaNU0S2wKBgCX2
5ZRCIChrTfu/QjXA7lhD0hmAkYlRINbKeyALgm0+znOOLgBJj6wKKmypacfww8oa
sLxAKXEyvnU4177fTLDvxrmO99ulT1aqmaq85TTEnCeUfUZ4xRxjx4x84WhyMbTI
61h65u8EgMuvT8AXPP1Yen5nr1FfubnedREYOXIpAoGAMZlUBtQGIHyt6uo1s40E
DF+Kmhrggn6e0GsVPYO2ghk1tLNqgr6dVseRtYwnJxpXk9U6HWV8CJl5YLFDPlFx
mH9FLxRKfHIwbWPh0//Atxt1qwjy5FpILpiEUcvkeOEusijQdFbJJLZvbO0EjYU/
Uz4xpoYU8cPObY7JmDznKvc=
-----END PRIVATE KEY-----
";

/// Writes a configuration holding `name` as the default account, made
/// of `lines` and storing its token in token.json beside it through
/// `cat` and `tee`, the file seeded with `token`. Returns the
/// configuration and token paths.
pub fn write_config(dir: &Path, name: &str, lines: &str, token: &str) -> (PathBuf, PathBuf) {
    let token_path = dir.join("token.json");
    fs::write(&token_path, token).unwrap();

    let config = dir.join("config.toml");
    let t = token_path.display();
    let storage = format!(
        "storage.read.command = [\"cat\", \"{t}\"]\nstorage.write.command = [\"tee\", \"{t}\"]"
    );
    let account = format!("[accounts.{name}]\ndefault = true\n{lines}\n{storage}\n");
    fs::write(&config, account).unwrap();

    (config, token_path)
}

/// Runs the built binary against `config`, its stdin closed.
pub fn ortie(config: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ortie"))
        .arg("-c")
        .arg(config)
        .args(args)
        .output()
        .unwrap()
}

/// Runs the built binary against `config`, `input` written to its stdin.
pub fn ortie_with_input(config: &Path, args: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ortie"))
        .arg("-c")
        .arg(config)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

/// The failure object a `--json` run printed, failing unless it exited
/// with 1.
pub fn json_failure(out: &Output) -> Value {
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{stdout}");
    serde_json::from_slice(&out.stdout).unwrap()
}

/// The token JSON the storage file holds.
pub fn stored(token: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(token).unwrap()).unwrap()
}

/// The commented section `name` of config.sample.toml, uncommented,
/// each line setting a key of `overrides` set to its value instead.
pub fn sample_section(name: &str, overrides: &[(&str, &str)]) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.sample.toml");
    let sample = fs::read_to_string(path).unwrap();
    let heading = format!("# {name}");

    let mut lines = sample.lines().skip_while(|line| *line != heading).skip(2);
    let mut section = String::new();

    for line in lines.by_ref().take_while(|line| !line.is_empty()) {
        let line = line.strip_prefix('#').unwrap_or(line);
        let key = line.split(" = ").next().unwrap_or_default();

        match overrides.iter().find(|(k, _)| *k == key) {
            Some((key, value)) => section.push_str(&format!("{key} = {value}\n")),
            None => section.push_str(&format!("{line}\n")),
        }
    }

    assert!(
        !section.is_empty(),
        "no {name} section in config.sample.toml"
    );
    section
}

/// The environment variable `name`, an empty value counting as unset,
/// since a workflow maps a missing secret to an empty one.
pub fn env_var(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.is_empty())
}

/// [`env_var`], failing the test with the variable to set when unset.
pub fn require_env(name: &str) -> String {
    env_var(name).unwrap_or_else(|| panic!("set {name} to run this test"))
}

/// The file the `{name}_FILE` variable names, or else the content of
/// `name` written to a file of `dir` only the owner reads.
pub fn env_file(dir: &Path, name: &str) -> Option<PathBuf> {
    if let Some(path) = env_var(&format!("{name}_FILE")) {
        return Some(PathBuf::from(path));
    }

    let content = env_var(name)?;
    let path = dir.join(name.to_lowercase());
    fs::write(&path, content).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    }

    Some(path)
}
