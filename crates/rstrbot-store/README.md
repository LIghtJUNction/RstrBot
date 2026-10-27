# RstrBot storage and authorization

This crate owns durable users, workspace membership, private/shared resources,
ordered events, request claims, and budget reservations. SQLite and PostgreSQL
use the same schema and business methods. It does not start model calls or run
plugin code inside a database transaction.

## Behavior

- SQLite uses one pooled connection, WAL, `synchronous=FULL`, and a five-second
  busy timeout. PostgreSQL uses at most eight pooled connections. Pool acquisition
  times out after ten seconds.
- Passwords use Argon2 with individual random salts. Random 256-bit session
  tokens are returned once and only their SHA-256 digests are stored. Sessions
  expire after seven days. `Debug` redacts tokens.
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

## Trusted internal methods

`internal_list_page`, `internal_list`, `user_by_id`, `internal_update_resource`,
`internal_append_event`, `settle`, `finish_operation`, `export_backup_local`,
and `recover_password` are trusted engine/maintenance APIs. Never expose them as
unscoped HTTP endpoints or plugin methods. The internal update/event methods
record already-in-flight outcomes after revocation; they do not authorize new
work. Version and existence checks still prevent a late result from resurrecting
a deleted or reset resource.

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

Snapshots contain password hashes and may contain encrypted credential records.
Treat them as sensitive. They intentionally omit login sessions, external plugin
state, attachments, and the separately protected encryption key. The serialized
`includes_external_state: false` flag means this is a database backup, not a claim
of complete application disaster recovery.

`import_backup` only accepts an empty destination. It validates the version,
manifest, checksum, column types, foreign keys, ownership and reservation totals,
then imports within one transaction. Failure leaves the empty destination
unchanged and never modifies the source. The restored instance enters recovery
mode. The server must keep external effects and scheduled work paused until an
instance administrator invokes `finish_recovery` after reconciliation. Unfinished
request claims and unknown reservations remain intact.

The same format supports controlled SQLite↔PostgreSQL database transfer. Files,
plugin state, master keys and production cutover remain separate operations.

## Verification

```sh
cargo test -p rstrbot-store
cargo clippy -p rstrbot-store --all-targets -- -D warnings
```

For PostgreSQL, point the same tests at a **disposable** database whose test user
can create schemas:

```sh
RSTRBOT_TEST_DATABASE_URL=postgresql://test@127.0.0.1/rstrbot_test \
  cargo test -p rstrbot-store --test behavior
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

PostgreSQL RLS policies, separate migration/runtime accounts, service identities,
scoped personal API keys, OIDC/Passkey and full external-state disaster recovery
are not supplied by this crate. They must not be marked complete from these tests.
