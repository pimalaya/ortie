---
cairn: change
id: the-package-names-the-standard-commands
status: landed
created: 2026-11-19
---

# The Nix package names the standard commands

## Why

Finding PKG-6 of the CLI family analysis of 2026-11-15, ortie's part (re-checked open on 2026-11-16 and on 2026-11-19): package.nix called the hidden aliases `manuals` and `completions`, shipped no JSON Schemas, and its `meta.changelog` linked `releases/<tag>`, a 404 on GitHub.

## What

package.nix runs `manual`, `completion` and `json-schema` into share/man, share/completions and share/schemas, as himalaya, carillon and neverest do, and links `releases/tag/<tag>`.

### Breaking

None.
