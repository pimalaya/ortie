---
cairn: tasks
change: a-refresh-lost-to-a-concurrent-one
---

- [x] re-read the storage once on `invalid_grant`, take a changed refresh token
- [x] regression tests: `token show` and `token refresh` win, an unchanged store still refused
- [x] fold into the token and mcp capabilities, log
- [x] step 2, a lock per account: ruled out by the owner on 2026-11-19 unless a sample provider is shown to revoke the whole token family on reuse
