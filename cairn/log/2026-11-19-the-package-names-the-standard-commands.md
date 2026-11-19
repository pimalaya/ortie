---
cairn: log
change: the-package-names-the-standard-commands
landed: 2026-11-19
---

# The Nix package names the standard commands

Finding PKG-6 of the CLI family analysis of 2026-11-15, ortie's part. package.nix runs `manual`, `completion` and `json-schema` (the schemas new, into share/schemas) and links `releases/tag/<tag>` as `meta.changelog`. Checked by running the three commands of the built binary into a scratch directory and by `nix eval` of the flake package's `postInstall`; no package build was run. `meta.changelog` of the flake package does not evaluate, its `src` being the local tree with no `tag`, as before this change: the line serves the nixpkgs derivation this file mirrors.

PKG-2 does not concern ortie: its install.sh already fails on an HTTP error (2026-11-18), and checksums wait on pimalaya/nix publishing them.

Capabilities moved: packaging ("The Nix package installs what the binary generates", added).
