---
cairn: change
id: wizard-storage-installed-first
status: landed
created: 2026-08-11
---

# Order the storage strategies by what the system has installed

## Why

The storage pick list is built from the credential provider CLIs the shared picker knows for the running OS, which is a compile-time answer: on Linux it always leads with `secret-tool`, then `kwallet-query`, then `pass`. A GNOME machine without `secret-tool`, or a `pass` user on KDE, is offered a strategy that cannot store anything until a package is installed, ahead of the one already sitting on their `PATH`. The wizard knows how to find out, and asking the system is cheap.

## What

Order the strategies by whether their CLI resolves on the `PATH`, installed ones first, and mark the ones that do not resolve rather than hide them: a provider missing today is one package install away, and the read and write commands the wizard writes for it are correct either way. The order inside each group stays the picker's own, most native first, since the sort is stable.

The `PATH` lookup lives in Ortie, not in the shared picker: `pimalaya-cli` is consumed as a released crate here, so upstreaming it would need a release cycle for a check this small. The program name is read from the provider's own read command rather than duplicated, so a provider added upstream needs no table kept in step here.

## Non-goals

- Filtering out what is not installed. An empty list is worse than an honest one, and the check is a heuristic: a `PATH` that differs between shells would silently drop a working provider.
- Probing whether the provider actually works (a running Secret Service, an initialised password store). Resolving the binary is the cheap, non-invasive half; running the CLI to find out is not something a wizard should do behind the user's back.
