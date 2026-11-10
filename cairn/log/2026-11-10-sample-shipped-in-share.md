---
cairn: log
change: sample-shipped-in-share
landed: 2026-11-10
---

# Sample shipped in share

package.nix installs config.sample.toml as share/doc/ortie/config.sample.toml, beside the man pages and completions, so the Nix package and the release archives, which carry the share/ of that build next to the binary, ship it. install.sh copies it to `$PREFIX/share/doc/ortie/` when the downloaded archive holds it, the current releases not yet.

`shipped_sample` (src/cli.rs) takes the running binary, canonicalized on Unix so a Nix profile or Homebrew symlink leads to its own prefix, and `find_sample` looks for share/doc/ortie/config.sample.toml under the parent of its directory (an installed prefix), then under its directory (an extracted archive). `main` hands the result to `welcome`, which prints `mkdir -p` for a missing configuration directory and `install -m 644` from the shipped sample, since a `cp` from the read-only Nix store would leave a read-only configuration; without a shipped sample it prints the `curl` download as before.

On Windows, `shell_quote` (src/auth/get.rs, formerly `shell_single_quote`) double quotes, single quotes being literal to cmd, and the welcome prints `curl.exe`, Windows PowerShell 5.1 aliasing `curl` to Invoke-WebRequest, then `mkdir` and `copy`, which cmd and PowerShell both run. The `auth listen` and `auth resume` handoffs use the same quoting. A path holding `%` or `$` would still expand in cmd or PowerShell; Windows paths hold no double quote.

`the_shipped_sample_is_found_installed_or_extracted` covers both layouts; `the_welcome_commands_run_as_printed` covers the copy commands, a path holding a space and a quote; `the_welcome_commands_run_in_cmd_and_powershell` compiles on Windows only, so it was not run. Seen under `script`: a symlink to a binary in a prefix holding a read-only sample printed the `mkdir -p` and `install -m 644` commands, which ran and left a 644 copy identical to the sample; the archive layout found the sample beside the binary; a bare build printed the download. `nix build .#default` put share/doc/ortie/config.sample.toml in the package with mode 444, and the packaged binary printed the copy from the store.

Capabilities moved: config (cairn/spec/config.md: "A missing configuration meets a person with the welcome" and "One sample configuration, with a section per provider" modified), auth (cairn/spec/auth.md: "Manual resume fallback" quotes for the platform).
