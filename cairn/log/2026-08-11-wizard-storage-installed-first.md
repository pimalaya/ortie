---
cairn: log
id: wizard-storage-installed-first
date: 2026-08-11
---

# Order the storage strategies by what the system has installed

The storage pick list now leads with the credential provider CLIs actually found on the `PATH`. What the shared picker returns is a compile-time answer, so a `pass` user on a machine without `secret-tool` was offered `secret-tool` first; now the provider they can use today comes first.

Providers that do not resolve are still offered, labelled `, not found on PATH`. Hiding them would be worse: one is a package install away, the commands the wizard writes for it are correct either way, and a `PATH` that differs between shells would silently drop a working provider. The sort is stable, so the picker's own most-native-first order survives inside each group.

The lookup lives in Ortie rather than in `pimalaya-cli`, which is consumed here as a released crate: upstreaming a check this small would cost a release cycle. The program name is read from the provider's own read command (its first argv element), so no table of binary names is duplicated and a provider added upstream needs nothing kept in step.

Capabilities moved: discovery.
