---
cairn: tasks
change: the-google-application-ships
---

# Tasks

## The disclosure

- [x] src/auth/disclosure.rs: the trigger (Pimalaya's Google client id, a consent grant), the services the scopes reach, the text and one-sentence forms, the y/N prompt, the `agreement-missing` refusal
- [x] `agreement-missing` in the code vocabulary (src/error.rs)
- [x] `auth get --agree-data-use`, the agreement obtained before anything is sent, the disclosure printed before the request in text
- [x] `ortie mcp --agree-data-use`, `auth_get` refused without it before the pending flow is replaced, `disclosure` in its answer, the instructions naming it
- [x] Tests: the trigger, the services, the prompt answers, the refusal under `--json`, piped and in a REPL, the MCP refusal and answer

## The shipped client

- [x] README Google section: Pimalaya's client id and secret (from ortie's history, never printed), the four scopes, in production pending verification, the disclosure
- [x] config.sample.toml checked: one generic account pointing at the README, nothing to change; no config.samples/

## Landing

- [x] `nix develop --command cargo build`, `cargo test`, `cargo fmt`
- [x] Fold the delta into cairn/spec/ (auth, mcp, output, config, provider-quirks), log, CHANGELOG under Unreleased
- [x] Mark landed, move to cairn/changes/archive/
