---
cairn: tasks
change: the-handoff-leaves-the-command-line
---

- [x] owner ruling (2026-11-19): handoff off the command line, flags kept, handoff file parked
- [x] owner ruling (2026-11-19, second): explicit `--handoff <PATH|->`, never an implicit stdin read, a conflict with `--state` and `--pkce`
- [x] `--handoff` on `auth listen` and `auth resume`, refused on other grants and as `-` in a REPL session
- [x] document `--state` and `--pkce` as visible to other local users
- [x] tests failing on the old code, an open stdin left alone among them
- [x] fold into the auth capability, log, CHANGELOG
