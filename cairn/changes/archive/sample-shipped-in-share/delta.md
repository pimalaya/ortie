---
cairn: delta
change: sample-shipped-in-share
---

## MODIFIED Requirements

### Requirement: A missing configuration meets a person with the welcome
When no configuration exists at the path `-c` gave or the default one, and stderr is a terminal with `--json` off, a bare `ortie` and every account-bound command SHALL print the welcome on stderr and exit 1, printing nothing more. The welcome SHALL frame Ortie in one sentence, name the missing path, give the commands creating it from config.sample.toml, invite the reader to edit it and uncomment the options they need, and give the `auth get` command carrying the `-c` paths. When share/doc/ortie/config.sample.toml exists under the prefix holding the binary's directory or under that directory itself, symlinks resolved, the commands SHALL copy it there with a writable mode, after creating the missing directory; otherwise they SHALL download it with `curl --create-dirs`. Every path SHALL be quoted for the platform's shells (single quotes on Unix, double quotes and `curl.exe` on Windows, which cmd and PowerShell both read), so each command runs as printed. Otherwise the missing-configuration error SHALL stand, as JSON under `--json`.

#### Scenario: A first run
- GIVEN no file at $XDG_CONFIG_HOME/ortie/config.toml, $HOME/.config/ortie/config.toml or $HOME/.ortierc
- WHEN `ortie` runs in a terminal
- THEN the welcome names $XDG_CONFIG_HOME/ortie/config.toml, the commands creating it from config.sample.toml and `ortie auth get`, and the exit status is 1

#### Scenario: A packaged binary
- GIVEN ortie installed as /usr/bin/ortie with /usr/share/doc/ortie/config.sample.toml, and no configuration
- WHEN `ortie` runs in a terminal
- THEN the welcome creates the configuration directory and copies /usr/share/doc/ortie/config.sample.toml there with mode 644, downloading nothing

### Requirement: One sample configuration, with a section per provider
The repository SHALL ship config.sample.toml, and the release archives and the Nix package SHALL install it as share/doc/ortie/config.sample.toml, which install.sh copies under its prefix. It holds one `[accounts.example]` table with dotted keys and no nested table. It SHALL list the fields every provider shares and each of their values, the default active and the others commented as `#key = value`, every line ending in one tab-aligned sentence, and every command a failing `exit 127` placeholder. It SHALL end with one section per provider (Google, Microsoft, Fastmail) and a custom one for an application of one's own at any other provider, each a paragraph of commented code only, headed by its name, holding the client, the endpoints, the redirection, the resources and the scopes the provider needs, Pimalaya's application where one exists and example values in the custom one. The file SHALL hold a working account once one section is uncommented. The test suite SHALL parse config.sample.toml through the loader `-c` uses with each section uncommented whole, then on top of it each commented element inside it and each line of commented code outside the sections, uncommented in place of the active lines of its field, so a configuration change that breaks one fails the build.

#### Scenario: A field is renamed
- GIVEN config.sample.toml, whose commented code sets `client-secret.raw`
- WHEN the field is renamed in the configuration types and the sample is not updated
- THEN the test suite fails naming the sample

### Requirement: Manual resume fallback
When the account's redirection uses a non-loopback scheme the local listener cannot capture (for example a reverse-DNS private-use scheme), `auth get` SHALL skip the listener and print the manual `auth resume` command (state, PKCE and redirect URI included) after opening the browser. A non-interactive or `--json` `auth get` SHALL hand off with the authorization URI, the state, the PKCE verifier and the redirect URI (`redirectUri`), and its human output SHALL print the `auth listen` command, when the redirect URI is loopback, before the `auth resume` one. `auth resume` interprets its positional input per the account's grant: the redirected URI on an authorization-code account.

The printed commands SHALL be runnable as printed, whatever the generated values look like: each value SHALL be attached to its flag with `=` and quoted for the platform's shells, single quoted on Unix and double quoted on Windows, which cmd and PowerShell both read. A PKCE verifier is drawn from the RFC 7636 unreserved set and a state from URL-safe base64, so either can begin with `-` and be taken for a flag, which the `=` form settles before the parser sees it, or with `~`, which the quotes stop the shell expanding.

The printed commands, and the one the device grant prints when it hands off, SHALL carry one `-c` per configuration path and the `-a` name `auth get` ran with, each quoted the same way, and omit either flag when it was not given. Resumed against another account, the code reaches a token endpoint that did not issue it, or the token lands in another account's storage.
