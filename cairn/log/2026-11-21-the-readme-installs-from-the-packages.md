---
cairn: log
change: the-readme-installs-from-the-packages
landed: 2026-11-21
---

# The README installs from the packages

Ortie 3.0.0 ships to [pkg.pimalaya.org](https://pkg.pimalaya.org) (APT, DNF) and the pimalaya/homebrew-tap formula, so the README's installation section now leads with them.

## What landed

- README.md: a sentence naming pkg.pimalaya.org, then three sections ahead of the others: Debian and Ubuntu (the deb822 repository setup and `apt install ortie`), Fedora and RHEL (the .repo download and `dnf install ortie`), Homebrew (`brew install pimalaya/tap/ortie`), the commands copied from the live install page. The installer section now reads as the way for other systems; Cargo, Nix and Sources are unchanged.

## Spec

No capability moved: the README is documentation.
