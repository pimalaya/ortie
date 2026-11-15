---
cairn: log
change: the-live-suites-run-weekly
date: 2026-11-15
---

# The live suites run weekly

Line N0 of the CLI alignment round. Capability **packaging** moved.

- **.github/workflows/tests.yml**: the `live` job runs on `workflow_dispatch` and on a weekly `schedule` (`0 2 * * 1`), never on push; `tests` and `features` stay on push. Matrix, secrets and steps unchanged.
- The live job touches the OAuth endpoints of Google, Microsoft and Fastmail with made-up clients, and mints client-credentials tokens on the Microsoft test tenant; no test creates a provider resource.
- CONTRIBUTING.md ("Live provider suites") and packaging.md say the suites run weekly and by hand (`gh workflow run tests.yml`), a release running them by hand on the commit it tags.
