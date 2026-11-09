---
cairn: log
change: readme-trimmed
landed: 2026-11-09
---

# README trimmed

The README Features list is down to eight one-line bullets (ready-made accounts, every common grant, token lifecycle, your own storage, persistent session, MCP server, JSON output, full client). The OAuth 2.1 list keeps its four points in one short line each. Configuration follows readme-010: the three paths, `-c` and `ORTIE_CONFIG` with `:`-merging, links to config.sample.toml and config.samples/, the samples table, the scope notes and the registration notes gone; the owner dropped the Fastmail registration sentence too, provider-quirks keeping Fastmail's RFC 7591 registration. The description is "CLI to manage OAuth tokens" in the README, Cargo.toml and flake.nix (owner). Usage points at `--help` and shows a flow from `curl`-appending config.samples/google.toml to `auth get`, `token show`, `token inspect` and `token revoke`, then one line on the `auth listen` and `auth resume` hand-offs `auth get` prints; REPL and MCP server subsections follow, each a sentence and one block.

The Google Desktop client's 7-day refresh tokens in testing and Microsoft's "Mobile and desktop applications" platform for `http://localhost` moved from the README to provider-quirks. The config spec's samples requirement no longer names own-application lines, which the owner had removed from google.toml and microsoft.toml, and its renamed-field scenario takes config.sample.toml's commented `client-secret.raw`.

Capabilities moved: config (cairn/spec/config.md: "Samples, one per provider" modified), provider-quirks (cairn/spec/provider-quirks.md: "Current Google endpoints" and "Outlook scope resource" modified).

Open: readme-011 wants per-provider recipes in the README, which config.samples/ replaces here.
