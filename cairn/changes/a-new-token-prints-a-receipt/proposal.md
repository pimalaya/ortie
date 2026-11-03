---
cairn: change
id: a-new-token-prints-a-receipt
status: landed
created: 2026-11-03
---

# A new token prints a receipt

## Why

pimalaya-cli 0.3 (change every-output-is-data) removes `Message`: every output is a typed value, a sentence being only its text form. Ortie printed two: `token refresh` and the grants storing a token (`auth resume`, and `auth get` when it issues the token itself) answered "Access token successfully issued (expires in 59m 59s)", which a caller could not read the expiry from. The 0.3 registry also builds schemas from the serialize contract and checks the command tree itself.

## What

- `TokenIssuedOutput` (src/token/issued.rs), `{expiresIn, withRefreshToken}`, the token's lifetime in seconds as the server stated it (`null` when it stated none) and whether a refresh token came with it. Its `Display` is the former sentence. The token itself never appears.
- `token refresh` and `auth resume` print it and carry the keys `ortie-token-refresh` and `ortie-auth-resume`.
- `auth get` gains a third alternative, `Issued`, printed when its grant issues the token in-process (client credentials, or an interactive grant once its redirection or polling completed); `withRefreshToken` tells it from the device handoff.
- `report_token_issued` persists and fires the hooks, then answers the receipt; each command prints it.
- The registry is built with `JsonSchemaRegistry::new` and `insert`, which registers `completion`, `manual` and `json-schema` too, and a test asserts pimalaya-cli's `check` over the command tree is empty, bare `ortie` and `ortie repl` declared silent. It replaces the hand-written walk.

## Compatibility

Breaking under `--json`: the two commands printed `{"message": …}`, and schemas now mark a key always printed as required, `null` or not. Ortie's next major (3.0.0, owed for the camelCase keys).

## Left

The rest of Ortie's ledger line in pimalaya-cli every-output-is-data: epoch times in `token inspect`, and the REPL help under `--json`.
