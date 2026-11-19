---
cairn: log
change: failures-named-and-defused
landed: 2026-11-19
---

# Failures named, server text defused

Findings ORT-8 and ORT-9 of the CLI family analysis of 2026-11-15.

- src/main.rs runs `Logger::try_init` as the first step of the result, so an unopenable `--log-file` prints the failure object ("Cannot open log file <path>") and exits 1 instead of panicking with 101. The same `expect` sits in io-pimdir's `pimdir` binary (PIM-C2), outside this repository.
- src/token/refresh.rs gives the refresh's connection the context "Refresh access token error" (the request itself keeps its own error, so a bare 403 or 429 still names its status at the top) and logs the refresh and the re-acquisition; src/account.rs logs the storage reads and writes.
- src/token/inspect.rs and src/auth/get.rs defuse the token type, the scopes, the user code and the verification URIs with `pimalaya_cli::sanitize::text`. The failure report's own sanitizing belongs to the kit (KIT-2).

Evidence, failing on 58de3a9: tests/exit.rs `a_log_file_that_cannot_be_opened_is_a_failure_object`, tests/token_refresh.rs `an_unreachable_refresh_is_named_a_refresh`, the unit test `the_text_defuses_what_the_server_wrote`.

Capabilities moved: output ("A log file that cannot be opened fails the command", added; "A text record prints as aligned fields"), token ("Refresh rotation ordering").
