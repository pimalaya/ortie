---
cairn: change
id: the-live-suites-run-weekly
status: landed
created: 2026-11-15
---

# The live suites run weekly

> Line N0 of the family's CLI alignment round, X10 (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15, tasks.md).

## Why

Every push to master ran the `live` job against Google, Microsoft and Fastmail. Across the family a push storm burns Google's quota, hits Graph throttling and collides with the owner's live testing, so the family moves every live suite off push onto one weekly slot per repository, and onto a run by hand.

## What

- .github/workflows/tests.yml: `on:` gains `workflow_dispatch:` and `schedule:` at `0 2 * * 1` (Mondays 02:00 UTC, ortie's slot), and the `live` job runs only on those two events. Its matrix, secrets and steps stay; the `tests` and `features` jobs stay on push.
- What the live job touches: `anonymous::` sends made-up clients and tokens to the Google, Microsoft and Fastmail OAuth endpoints (a device authorization, a refused client-credentials grant, a refused resume and revoke); `credentialed::` mints client-credentials tokens on the Microsoft test tenant, by secret and by signed JWT. No test creates a provider resource, so nothing long-lived is needed.
- CONTRIBUTING.md and cairn/spec/packaging.md ("Live provider suites") say the suites run weekly and by hand (`gh workflow run tests.yml`), and that a release runs them by hand on the commit it tags.
