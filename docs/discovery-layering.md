# Discovery layering: domain-scoped discovery and the resource trap

Design note on where OAuth and PIM discovery code should live, prompted by a
concrete trap: Fastmail refused to work in Ortie for the exact reason it once
refused in cardamum-android, because both consumers had to re-derive the same
provider knowledge by hand. This note records the boundary rule that prevents
the repeat, and the refactor that makes it real. It spans io-oauth and
io-pim-discovery; Ortie is only the consumer that surfaced it.

## The trap

Fastmail's dynamic registration accepts only a reverse-DNS private-use redirect
scheme (not http, not loopback), and its authorization endpoint hard-requires an
RFC 8707 `resource` parameter: omit it and the request is rejected with
`invalid_target` before any consent screen, all undocumented. cardamum-android
learned this the hard way and injects `resource` (from the discovered RFC 9728
protected-resource metadata) into every authorize request. Ortie, written later,
had no typed reminder that `resource` exists, never sent it, and Fastmail bounced
silently.

Tellingly, io-oauth already models the *failure*: both `Oauth20AuthResponse` and
the token error enum carry `InvalidTarget`, with a doc that even names Fastmail.
But no request type carries a `resource` field. `Oauth20AuthRequestParams` routes
it through the generic `extras` bag (documented as the home of provider-specific
knobs like Google's `access_type`), and `Oauth20AccessTokenRequestParams` has no
channel for it at all. io-oauth models the exception but not the remedy, and
mis-files the remedy as a provider quirk. That asymmetry is why every consumer
re-learns Fastmail.

## The boundary rule: spec provenance

A discovery mechanism belongs to whichever library owns the RFC that defines it.

- Spec-defined discovery goes in the spec's domain library.
- Heuristic or cross-domain discovery stays in io-pim-discovery.

| mechanism | defining spec | home |
| --- | --- | --- |
| AS metadata / protected-resource metadata | RFC 8414 / 9728 (OAuth) | io-oauth |
| JMAP session bootstrap (SRV + well-known) | RFC 8620 §2.2 (JMAP) | io-jmap |
| CalDAV / CardDAV well-knowns | RFC 6764 (DAV) | io-webdav |
| `WWW-Authenticate` challenge parse | RFC 9110 (HTTP) | io-http (already moved) |
| autoconfig / ISPDB, provider rules | none (heuristic) | io-pim-discovery |

io-pim-discovery does not vanish under this rule; it keeps what no protocol owns:
the non-RFC autoconfig/ISPDB formats, the hardcoded provider rules (Google,
Microsoft, Fastmail endpoints and scopes), and the composition itself (run every
resolver in parallel, dedupe, rank, and the refine pass that turns a discovered
issuer into a concrete grant). Owning 8414/9728/8620/6764 was accidental; owning
the orchestration is its real identity.

## The enabling investment: a shared effect vocabulary

For io-pim-discovery to drive a discovery coroutine from any domain library
uniformly, they must all yield "wants" it knows how to satisfy. HTTP is already
shared (everyone is on io-http). The open gap is DNS/SRV: OAuth metadata is a
plain HTTPS GET (trivial, no new surface), but JMAP session bootstrap (RFC 8620
§2.2) and DAV discovery (RFC 6764) need SRV lookups and redirect chasing, which
would give io-jmap and io-webdav a DNS want they lack today. Because these
coroutines stay I/O-free (they yield a DNS want, they do not resolve), this does
not compromise `#![no_std]` or drag a resolver into the protocol library; it only
asks for one shared DNS/SRV want in the same house style as
`WantsFileExists`/`WantsTime`. That vocabulary is the actual work; the module
moves are easy once it exists.

## The fix, in two typed halves

- io-oauth gains a first-class `resource: Vec<Url>` on `Oauth20AuthRequestParams`
  and `Oauth20AccessTokenRequestParams` (RFC 8707 allows several; empty omits
  it), serialized as repeated `resource=` pairs and documented next to the
  `InvalidTarget` it prevents. The standard param stops being a provider quirk
  hidden in `extras`.
- io-pim-discovery carries the value it already fetches: `resource: Option<Url>`
  on the OAuth grant descriptors, filled from the 9728 metadata during the refine
  pass. Fastmail resolves to `https://api.fastmail.com/jmap/session`.

Consumers then do trivial typed wiring (`auth_params.resource = grant.resource`),
and no one re-derives Fastmail. The same treatment applies, less urgently, to
scope negotiation: carry the advertised `scopes_supported` on the grant so
consumers stop hand-rolling intersections like cardamum's `contacts_scope()`.

## Merging, one day

The clean version of this moves the 8414/9728 types and their pure fetch
coroutines back into io-oauth (where they were born, as I/O-free coroutines over
`Http11Send`, before the 2026-07-12 relocation), leaving only the
resolution/refine orchestration in io-pim-discovery. That is the same instinct
that retired io-email/io-addressbook/io-calendar: keep domains self-owned, and
make the composition layer thin and explicit rather than an aggregator that
absorbs everyone's protocol logic.

Sequencing matters more than speed:

- Do OAuth first: it is the trivial pure-HTTP case, and the one that unblocks the
  resource trap (the 9728 type lands next to the request field it feeds). Prove
  the shared-wants pattern here.
- Then move the SRV-heavy JMAP (8620) and DAV (6764) discovery, once the shared
  DNS/SRV want exists.
- Autoconfig, ISPDB and provider rules never move.

The cost is real: this reverses a five-day-old decision, and cardamum-android
migrated to the current 8414/9728 location on 2026-07-15. Doing it while these
crates are still unreleased is far cheaper than after they settle, but it is a
deliberate refactor across io-oauth, io-pim-discovery and every consumer, not a
drive-by.

## Until then: the config stopgap

None of the above is needed to use Fastmail today. Ortie already forwards
`extras` verbatim onto the authorize request, so a Fastmail account works the
moment its config carries the resource by hand:

```toml
[accounts.fastmail.extras]
resource = "https://api.fastmail.com/jmap/session"
```

See the Fastmail recipe in `config.sample.toml` for the full block (endpoints,
scopes, redirect). On a desktop the private-use redirect still cannot be captured
by the local listener, so `auth get` prints the manual `auth resume` command to
finish the flow by hand; that is a property of Fastmail's mobile-oriented
redirect policy, independent of the resource fix.
