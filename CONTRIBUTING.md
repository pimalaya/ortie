# Contributing guide

Thank you for investing your time in contributing to Ortie.

Whether you are a human or an AI agent, read these in order before touching the code:

1. the [Pimalaya README](https://github.com/pimalaya) for what the project is and how its repositories stack;
2. the [Pimalaya CONTRIBUTING](https://github.com/pimalaya/.github/blob/master/CONTRIBUTING.md) guide, which chains to the shared architecture and guidelines;
3. the inline header documentation, starting with src/main.rs: it is the architecture document of this crate;
4. the docs/ folder for the development history and living plans.

Everything below documents only what differs from the Pimalaya standards.

## Where changes belong

Ortie is a pure CLI binary: the glue between a user's TOML configuration and two engines it drives. Fixes usually land in one of three places, so triage before patching:

- OAuth wire semantics (grants, PKCE, token and registration requests) belong in [io-oauth](https://github.com/pimalaya/io-oauth);
- service discovery consumed by the wizard belongs in [io-pim-discovery](https://github.com/pimalaya/io-pim-discovery);
- configuration shape, token storage, hooks and command UX live here.

The shared clap, printer and prompt primitives come from [pimalaya/cli](https://github.com/pimalaya/cli), the TOML loader and secret resolution from [pimalaya/config](https://github.com/pimalaya/config), and the TCP and TLS plumbing from [pimalaya/stream](https://github.com/pimalaya/stream).

## Feature matrix

The binary always ships the full CLI; the only cargo features left are the TLS providers (rustls-ring default, rustls-aws, native-tls), vendored and notify. Build against a non-default provider to check it still compiles:

```sh
cargo build --no-default-features --features native-tls
```
