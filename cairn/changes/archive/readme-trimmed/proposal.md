---
cairn: change
id: readme-trimmed
status: landed
created: 2026-11-09
---

# README trimmed

## Why

The README had grown past what a reader needs: fourteen feature bullets, a long OAuth 2.1 list, a Configuration section explaining every sample and three registrations, and a Usage section with an MCP paragraph. The owner asked on 2026-11-09 for the most important features on one line each, a shorter OAuth 2.1 list, a Configuration section saying how and where to configure and nothing more, and a Usage section showing one complete flow, with concise REPL and MCP subsections. The owner had also removed the commented own-application client lines from the Google and Microsoft samples, leaving the README and the config spec describing them.

## What

- Features: eight bullets of one line each (readme-008), the manual completion, PKCE and extra parameters bullets folded into the coverage and OAuth 2.1 sections or dropped.
- OAuth 2.1: one short line per point.
- Configuration: the readme-010 paths and overrides, config.sample.toml and config.samples/ linked. The samples table, the scope notes and the registration notes go, the samples' comments carrying the scopes and provider-quirks the registrations (owner, 2026-11-09).
- Usage: `--help`, then a flow from a sample to a token (`auth get`, `token show`, `token inspect`, `token revoke`) and the `auth listen` or `auth resume` hand-off in one line; REPL and MCP server subsections.
- The Google and Microsoft own-application notes move to provider-quirks: the Desktop client's 7-day refresh tokens in testing, and `http://localhost` under the "Mobile and desktop applications" platform.
- config spec: a sample's commented code is the scopes it leaves out, no own-application lines; the renamed-field scenario takes config.sample.toml's commented `client-secret.raw`.

## Not done

- readme-011 asks for per-provider recipes as Configuration subsections of the README; Ortie keeps them in config.samples/, tested by the suite, so that rule needs an Ortie exception or an amendment in pimalaya/.github.
