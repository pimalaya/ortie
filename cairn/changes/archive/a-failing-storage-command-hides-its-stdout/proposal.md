---
cairn: change
id: a-failing-storage-command-hides-its-stdout
status: landed
created: 2026-11-19
---

# A failing storage command never reports its stdout

## Why

Finding ORT-1 of the CLI family analysis of 2026-11-15 (MEDIUM, security, re-checked open on 2026-11-16 and again on 2026-11-19 at 58de3a9). src/account.rs reported a failing storage command by its stdout when it wrote any, by its stderr otherwise. A read command printing the token JSON then exiting non-zero put the access and refresh tokens in the `sources` of `token inspect --json`, in the `storage` row of `account check`, and in the answer of the ungated MCP tool `token_inspect`. A write command echoing its input (`tee` to a full disk, a wrapper printing what it was handed) did the same with the token it was asked to store.

The owner's ruling on COH-1 (2026-11-18, pimalaya-config cairn/log/2026-11-18-secret-command-named.md) keeps the family rule: a caller never reads a failed helper's stdout, it reports the program, the exit status and the stderr, and hints to run the command.

## What

- Both storage commands are reported by their trimmed stderr, or "<program> failed with exit status: <n>" when they wrote none, pimalaya-config's wording.
- A failing read command's report ends with ", run `<command>` yourself to see why", the command as configured (pimalaya-config's `Display` for `CommandConfig`). The write command gets no hint: run by hand it would wait for a token on its stdin.

### Breaking

None: the code stays `null`, only the wording of a failure changes.
