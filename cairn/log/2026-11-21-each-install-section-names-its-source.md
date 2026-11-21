---
cairn: log
change: each-install-section-names-its-source
landed: 2026-11-21
---

# Each install section names its source

The opening sentence of the README's installation section, naming pkg.pimalaya.org for every package channel at once, moved into each section it concerns, so a section can later show the official package beside the Pimalaya one.

## What landed

- README.md: the Debian and Ubuntu and the Fedora and RHEL sections say Ortie is not in the official repositories yet and point to the APT or DNF repository of pkg.pimalaya.org; the Homebrew section says it is not in homebrew-core yet and points to the pimalaya/tap formula. Repology listed no Debian, Ubuntu or Fedora package and formulae.brew.sh no core formula on 2026-11-21.

## Spec

No capability moved: the README is documentation.
