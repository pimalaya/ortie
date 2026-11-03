---
cairn: log
change: release-polish
date: 2026-11-03
---

# Release polish

Audited the refresh rotation window: `TokenRefreshCommand::refresh` writes the new token right after the server answers, keeping the old refresh token when none rotated in, with nothing fallible in between. The gap left was a failing storage write, which left the in-memory token on the old refresh token a rotating server had already invalidated: `Account::write_to_storage` now caches the token before running the write command, and the refresh path names the risk in its error. The README gained an OAuth 2.1 section (grants, PKCE, redirect matching, rotation) and the RFC 7009, 8707 and 9728 coverage rows. The obsolete tasks (changelog notes already in 2.0.0 and 2.1.0, the minor released as 2.1.0) were closed as such.

Capabilities moved: `token` (Refresh rotation ordering modified).

Verified: `a_failed_refresh_write_keeps_the_issued_token_in_session` in tests/repl.rs fails against the previous code (the session printed `at-stale`) and passes now; `cargo test --all-features` and clippy clean.
