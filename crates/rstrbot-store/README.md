# RstrBot storage and authorization

This crate owns durable users, verified email and external identity bindings,
workspace membership, private/shared resources, ordered events, request claims,
and budget reservations. SQLite and PostgreSQL
use the same schema and business methods. It does not start model calls or run
plugin code inside a database transaction.

## Behavior

- SQLite uses one pooled connection, WAL, `synchronous=FULL`, and a five-second
  busy timeout. PostgreSQL uses at most eight pooled connections. Pool acquisition
  times out after ten seconds.
- Passwords use Argon2 with individual random salts. Random 256-bit session
  tokens are returned once and only their SHA-256 digests are stored. Sessions
  expire after seven days. `Debug` redacts tokens.
- Argon2 verification and hashing run outside transactions. Session issuance and
  password replacement then recheck the verified credential and active account
  under the user row's write lock. A password change/recovery cannot be undone
  by a login or password change that verified the old password earlier. Password
  changes revoke all sessions in the same transaction without issuing a
  temporary verification session.
- A verified email can be used as the password-login identifier. Both the exact
  password hash and the still-current username/email binding are rechecked at
  session issuance. A Google profile email never creates a login alias or a
  recovery address.
- HTTP/CLI must verify the short-lived setup proof before calling `bootstrap`.
  A transactional unique initialization marker prevents a second initialization.
- Current membership is checked on every user operation. Reads query both
  workspace membership and resource visibility. Workspace owners and instance
  administrators do not gain implicit access to another user's private resource.
- A workspace row lock serializes short mutations with membership changes.
  Owners cannot remove or demote the last owner. Admins cannot grant ownership,
  change another admin, or change an owner.
- Resource edits require their last observed version. Concurrent edits cannot
  silently overwrite one another.
- A shared schedule still executes as its owner, so only that owner can change
  its configuration or re-enable it. Operators/admins may delete a shared
  schedule for moderation; visibility is not delegation of execution authority.
  The owner's current membership and role are checked in the update transaction.
- Shared conversations require workspace-visible agent configurations, because
  their run records contain an agent snapshot. A rejected private-agent choice
  leaves no run, message, event or idempotency claim. Private conversations may
  still use their owner's private agents.
- Resource CRUD events contain identifiers, type and version, never resource
  payloads. Explicit run events keep the resource's private/shared boundary.
  Deleting a resource removes its replayable event bodies and adds a tombstone.
- Request claims survive restart. A claimed but unfinished request is a conflict,
  not permission to repeat a potentially successful external effect. Completed
  requests return their original result only for the same fingerprint.
- Reservations atomically consume available budget before dispatch. Unknown cost
  retains the full reservation. A verified later cost settles it exactly once.
  Costs above the reservation are recorded as debt, rather than discarded.

All amounts use integer micro-US dollars. Currency conversion and provider pricing
belong to the caller; a reservation does not prove that an unbounded provider
operation has a known maximum price.

## Account security

Each account retains its local password and can bind a verified email, multiple
Google subjects, and up to 20 passkeys. Google subjects and WebAuthn credential
IDs are globally unique. Binding a method requires the current authenticated
session and authentication within the preceding five minutes. Password and
passkey reauthentication refresh only that session's recent-auth
timestamp; they do not issue another session or extend its seven-day expiry.
Logout, password replacement, and credential changes serialize on the user row.
An expired recent-auth proof returns `RecentAuthRequired`, while an expired,
revoked, or disabled-account session returns `Unauthorized`.

Google sign-in does not establish a recent-auth proof: SSO can complete without
fresh entry of an account credential. Its session can perform ordinary signed-in
work immediately, but sensitive account changes require password or passkey
verification. Google is a login/link method, not a reauthentication method.

Passkey registration and authentication store server-owned verifier JSON. Its
version is an opaque, globally increasing generation: deleting and re-enrolling
the same credential ID cannot revive an older verified snapshot. Authentication
checks this version and updates the verifier state in the transaction that
issues the session. Removing a Google binding or passkey revokes sessions
created through that method. Atomic passkey replacement inserts the verified
new credential before removing the old one and its sessions; any failure rolls
back the entire change. Removing the last remaining sign-in method is rejected.
Credential names are display metadata and do not replace counter state.

Email changes issue a random 256-bit token, store only its SHA-256 digest, and
expire after 15 minutes. The initiating session must still be active when the
token is consumed. A pending replacement leaves the existing verified address
usable until confirmation. Confirmation invalidates older email/reset intents;
it checks both account ownership and the previous email generation. Aliases
cannot collide with another account's existing username or verified email.

Password reset is available only for an explicitly verified email. The token
captures the current password fingerprint and email generation, so a delayed
reset cannot overwrite a subsequent password or email change. Completion
revokes all sessions and pending user-bound authentication challenges. Delivery
is limited per account and purpose to one request per minute, three per hour,
and five per day. Unknown addresses, throttled requests, disabled accounts, and
exhausted challenge capacity all return `None` to the HTTP adapter, which must
give the same generic response; storage failures still propagate.

Other login/registration ceremonies have a server-side one-time challenge,
bound to purpose, optional user, and optional originating session. Lifetimes are
at most ten minutes and payloads at most 64 KiB. Each user has at most 20 pending
challenges, anonymous ceremonies at most 1,024, and the database at most 4,096.
Consumption is atomic and precedes cryptographic proof validation. Expired
records are pruned during creation; excess capacity returns `RateLimited`.

The `authentication` identity setting uses a revision compare-and-swap under the
same global identity lock as external credential commits. A Google or WebAuthn
result must carry the configuration revision under which its ceremony began.
The public origin and RP ID cannot change while any passkey is stored. Secrets
inside the setting must already be encrypted by the core before writing it.

## Trusted internal methods

`internal_list_page`, `internal_list`, `user_by_id`, `internal_update_resource`,
`internal_append_event`, `settle`, `finish_operation`, `export_backup_local`,
and `recover_password` are trusted engine/maintenance APIs. Never expose them as
unscoped HTTP endpoints or plugin methods. The internal update/event methods
record already-in-flight outcomes after revocation; they do not authorize new
work. Version and existence checks still prevent a late result from resurrecting
a deleted or reset resource.

The identity methods ending in `_verified` are also trusted core APIs. The core
must first verify the Google ID token or complete WebAuthn cryptographic
verification with the server-held challenge. Passing a subject, an email,
credential ID, or verifier JSON is not proof of identity by itself. HTTP account
summaries must omit `PasskeyCredential.credential`; public capability responses
must omit the stored identity setting and its encrypted secrets. This crate does
not send mail, call Google, or contact authenticators while holding transactions.

Recovery and recurring schedulers use `internal_list_page(kind, cursor, limit)`.
Pass `None` for the first cursor, process `page.items`, then follow `page.next`
until it is `None`. Each page has at most 500 records and 16 MiB of stored JSON;
a short page can still have a continuation when the byte limit was reached.
The database connection is released before the caller processes the page.

The cursor orders records by immutable `(created_at, id)` keys and fixes the
initial upper position. Deleting already-read rows cannot skip unread rows;
appending newer rows cannot extend the current scan forever. This is a bounded
traversal, not a transactional snapshot: concurrent inserts behind the cursor
wait for the next pass. Startup recovery runs before accepting work, and each
scheduler pass starts a fresh scan. Cursors are opaque, bound to a resource kind,
and never grant user or plugin authorization. A dedicated index supports the
cross-workspace trusted scan; ordinary resource reads retain their current
membership and private/shared visibility checks.

The legacy `internal_list` still rejects more than 10,000 records and is not
suitable for recovery or recurring scheduling. Public resource listing is
bounded to 500 rows; event replay is bounded to 500 events. Resource JSON is
limited to 8 MiB and explicit event JSON to 128 KiB.

## Snapshot and migration

`export_backup` requires a current instance administrator. `export_backup_local`
is for an already-authorized local operator. PostgreSQL uses a repeatable-read
snapshot; SQLite uses a read transaction. The snapshot is streamed with row/size
limits and contains a SHA-256 checksum over the table manifest.

Version 2 snapshots contain password hashes, verified email bindings, Google
subjects, passkey public credentials/verifier state, and encrypted configuration.
Treat them as sensitive. They intentionally omit login sessions, recent-auth
proofs, pending authentication/email challenges, delivery limits, external plugin
state, attachments, and the separately protected encryption key. The serialized
`includes_external_state: false` flag means this is a database backup, not a claim
of complete application disaster recovery.

`import_backup` only accepts an empty destination. It validates the version,
manifest, checksum, column types, foreign keys, ownership, reservation totals,
identity generations, credential bounds, and email alias consistency,
then imports within one transaction. Failure leaves the empty destination
unchanged and never modifies the source. The restored instance enters recovery
mode. The server must keep external effects and scheduled work paused until an
instance administrator invokes `finish_recovery` after reconciliation. Unfinished
request claims and unknown reservations remain intact.

The same format supports controlled SQLite↔PostgreSQL database transfer. Files,
plugin state, master keys and production cutover remain separate operations.

Schema version 2 adds identity tables without rewriting version 1 user or session
rows. An existing password session remains valid and can reauthenticate after
upgrade. Database probing recognizes both versions and rejects unknown future
versions without migration. Version 1 backup manifests remain importable into
the current schema; they create no external identity or verified-email bindings.
All restored instances use schema version 2, start in recovery mode, and discard
pre-existing anonymous challenges from the empty destination.

## Verification

```sh
cargo test -p rstrbot-store
cargo clippy -p rstrbot-store --all-targets -- -D warnings
```

For PostgreSQL, point the same tests at a **disposable** database whose test user
can create schemas:

```sh
RSTRBOT_TEST_DATABASE_URL=postgresql://test@127.0.0.1/rstrbot_test \
  cargo test -p rstrbot-store --all-targets
```

Each test creates a unique schema. Schemas remain available for failure diagnosis;
drop the disposable database after the run. An unset variable runs real SQLite
files in temporary directories, rather than an in-memory database substitute.

The suite covers concurrent bootstrap, private-resource isolation from owners,
revocation, role escalation, last-owner protection, concurrent edits, deleted
event payloads, racing reservations, unknown funds, overrun debt, request claims,
restore quarantine, late revoked-actor receipts, scans beyond 10,000 historical
records, stable continuation after deletion, bounded concurrent growth and large
payload page limits.

Authentication regressions deterministically pause between password verification
and commit to cover password change/recovery, late replacement attempts, account
disablement, and an administrator role changed while login was in flight. They
run against the same selected SQLite or disposable PostgreSQL database.

Identity regressions additionally cover stale email login snapshots, verified
email ownership and collision checks, reset throttling and challenge-capacity
privacy, recent-auth expiration, single-use ceremonies, Google subject isolation,
passkey version/re-enrollment races, atomic replacement, method-session revocation,
last-method protection, configuration CAS, schema upgrades, and both backup
versions. Directly seeded test state provides deterministic expiry and race
orderings without timing sleeps. Synthetic passkey JSON at this layer tests
storage invariants; cryptographic WebAuthn/Google adapter tests belong to the core.

PostgreSQL RLS policies, separate migration/runtime accounts, service identities,
scoped personal API keys, general-purpose OIDC and full external-state disaster recovery
are not supplied by this crate. They must not be marked complete from these tests.
