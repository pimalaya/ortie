---
cairn: tasks
change: sample-shipped-in-share
---

- [x] package.nix installs share/doc/ortie/config.sample.toml; install.sh copies it under its prefix
- [x] `shipped_sample` and `find_sample` (src/cli.rs), the welcome copying the shipped sample or downloading it
- [x] `shell_quote` double quotes on Windows; `curl.exe`, `mkdir` and `copy` in the Windows welcome
- [x] Tests: the shipped sample found installed and extracted, the copy commands on Unix, the Windows commands (compiled on Windows only)
- [x] Seen under a pseudo-terminal: a symlinked binary in a prefix with a read-only sample, an extracted archive, a bare build; the printed commands run, leaving a 644 copy
- [x] Nix build: the package holds share/doc/ortie/config.sample.toml
- [x] README, CHANGELOG, src/main.rs header, config and auth specs
- [x] fmt, clippy `-D warnings`, `cargo test --all-features`
- [x] Delta folded, log written
