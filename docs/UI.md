# RstrBot Web UI

The 0.1.2 web client is a React and TypeScript application built by Vite. It uses
TanStack Query for keyed request caching, cancellation and revalidation. It is
served from the same origin as the Rust HTTP API; model credentials are not
persisted in browser storage. Authentication is an HttpOnly server cookie. Mutations include the
server-required `X-RstrBot-Request: 1` header.

## Build and development

```sh
cd web
npm ci
npm test
npm run build
```

`web/dist` is the production output. `npm run dev` starts a local development
server and proxies `/api` to `http://127.0.0.1:8787`. Set the backend to this port
when using the development server. Production does not require Node.js.

The checked-in npm lock file pins the dependency resolution. Development and CI
use Node.js 24.22.1. The frontend tests run TypeScript's erasable syntax directly
and report each assertion group with the Node test runner.

## Implemented workflows

- One-time initialization with a terminal-issued token and always-visible
  SQLite/PostgreSQL choices. Testing and applying a connection are separate
  operations; an unapplied engine change blocks setup completion. Local account
  login and logout remain separate from database administration.
- Workspace creation and switching, private or workspace-visible conversations,
  model and agent selection, durable event polling, cancellation, preserved
  history, context reset, deletion and edit-as-branch.
- Real model connection tests, write-only API key fields and optional token
  prices entered in USD per million. Prices convert to integer microdollars
  without binary floating-point rounding. Retargeting a saved credential requires
  a replacement or an explicit clear; model and channel editors provide the clear
  controls and explain this requirement.
- Installed plugin package selection, separate workspace instances, enable,
  disable and reload. A saved configuration is not described as a running
  process.
- UTF-8 text/Markdown knowledge import, user-maintained memory, archive/restore,
  one-time/interval/calendar schedules, agent instructions and channel forms.
- Administrator-approved Webhook sending, stable request keys, private receipt history,
  and separate accepted/rejected/unknown status. Uncertain attempts are not resent.
- Workspace members and roles; instance-administrator account creation;
  budgets, visible-data export, administrator backup and metadata audit views.
- Chinese/English text, dark/light themes, native keyboard-accessible dialogs,
  mobile navigation and 320px layout rules. IME composition does not send a
  message when Enter confirms composition.

## Privacy and stale-response controls

Every resource request includes the workspace path. The server remains the
authority for permissions. Query keys and component lifetimes distinguish
workspaces; switching cancels old queries and unmounts their event subscriber.
Draft keys include user, workspace and conversation IDs. Drafts remain in that
browser profile; signing in as another account does not display them. A 401 from
any business page returns to authentication even if the old user is cached.
Successful login cancels outstanding queries, clears the cache and reloads the
application before the new account uses the workspace.

Event polling uses monotonically increasing IDs from the durable event store.
The in-memory cursor and stream buffers survive navigation in the authenticated
session; a full reload can replay the stored events. Duplicate and out-of-order
events are handled by ID. `run.finished` releases generated text, and later
deltas cannot revive a recently finished run. Missing-terminal buffers are also
bounded to 16 streams of at most 200,000 characters, with 256 recent terminal IDs.
Authentication/access failures discard the workspace cache and refresh membership.
A delayed send acknowledgement clears only its submitted draft revision, preserving
new text typed while waiting or in another conversation. Retried submissions keep
their request key; the server rejects reusing a key with a different payload.

The active run is fetched from the conversation's `active_run` reference, so an
older active run remains visible independently of workspace list pagination.
Selecting an agent applies its configured default model; the user can override
that model afterward. Shared conversations only offer shared agents, and editing,
enabling or manually triggering a schedule requires its owner. These UI controls
complement the server's independent authorization checks.

Model output is rendered as React text. It is never inserted as raw HTML.
Malformed history entries are skipped individually; valid duplicate message IDs
are rendered once. Unknown model cost remains unknown.

### 0.1.2 role changes and reading position

`web/src/permissions.ts` mirrors the store's role thresholds and ownership checks
for UI controls. Instance administration alone does not make a user a workspace
administrator. Members can read another user's shared conversation but cannot
send, reset, delete or branch it. Operators receive the shared-resource controls
the server allows; private resources still require their owner. Run cancellation
and memory archive controls use the same permission helper. These are display
rules in addition to independent server authorization, not a security boundary.

When refreshed membership changes the active workspace role, the shell resets
workspace queries and remounts the page under the new role. An open management
page disappears when its navigation permission is lost. Membership normally
refreshes periodically and when the browser visibility/focus refresh occurs;
the server does not wait for that UI refresh to enforce revocation.

While the chat page remains mounted, switching conversations stores and restores
its reading position. Loading earlier messages preserves the visible anchor;
loading historical records alone does not raise the new-message indicator.
Reading positions are in-memory UI state, not a promise of cross-device or full
browser-restart synchronization. Draft storage retains its user/workspace/
conversation scope and revision-based acknowledgement handling.

Resource editors convert Unix timestamps only for datetime fields. Other
numeric fields are not passed through date rendering, avoiding an unrelated
invalid-date exception for large numeric values.

Implementation references: [UI permission rules](../web/src/permissions.ts),
[membership refresh and navigation](../web/src/App.tsx), and
[chat state and reading position](../web/src/Chat.tsx).

## Visual design

The original visual direction uses charcoal, warm paper and restrained champagne
accents. The application uses ordinary document elements and lists. Only the
welcome screen lazily loads a decorative folded-ribbon particle sculpture; it
does not display fictional telemetry. The sculpture uses 850 deterministic
points, caps device pixel ratio at 1.5, observes reduced motion/data saving,
pauses offscreen and in a background tab, supports manual pause, and removes its
animation callbacks and observers on unmount. Core controls do not depend on the
canvas.

No remote font, image or script is needed at runtime. The initial JavaScript
bundle in the historical 0.1.1 local production build measured approximately
109.7 kB gzip and CSS 6.6 kB gzip. The separately loaded sculpture module measured 1.5 kB gzip.
These are bundle sizes, not an LCP, accessibility or device-performance result.

## Verification and current limits

### Current 0.1.2 verification scope

The current frontend test suite contains **14 focused tests**: the previous ten
history/event/draft/pagination checks plus three role/ownership checks for
instance administrators, shared-resource operators and fail-closed private or
unknown-role handling. `npm test` runs all matching test files. The production
build includes strict TypeScript checking.

The browser harness now defines **14 acceptance groups**. It retains the previous
eleven listed below and adds 320px first-run database choices, a member reading
another owner's shared conversation with write controls unavailable, and a live
administrator downgrade that removes management navigation and the open page.
These additions use a real temporary core, independent authentication sessions
and awaited API/DOM conditions, not mocked permission results. New screenshots
are versioned under `docs/screenshots/v0.1.2/` when collected.

[0.1.2 evidence](evidence/v0.1.2/) is the release-specific source for final pass,
failure and not-executed status. The current script count and the earlier
0.1.1 pass record below do not establish acceptance of the final 0.1.2 binary.
Reading-position code, full keyboard/screen-reader coverage and all browser/
device combinations are not implied by the 14 groups.

### Historical 0.1.1 results

The 0.1.1 production build and strict TypeScript check passed. **Ten focused tests** cover
history validation, duplicate IDs, untrusted text, monotonic/replayed/out-of-order
events, terminal stream cleanup and resource limits, delayed draft acknowledgement,
and paginated resource overlap. This is the earlier ten-test record; the current
`cd web && npm test` command runs the thirteen tests described above. The independent
[test/build receipt](evidence/v0.1.1/web-tests-build.json) and
[complete command log](evidence/v0.1.1/web-tests-build.log) record both commands,
their exit codes and unchanged source hashes.

**Eleven browser acceptance groups passed** against the final optimized Linux
0.1.1 binary and the exact built WebUI. The run used Playwright 1.62.1, Chromium
156.0.8078.47, Node.js 24.22.1, SQLite and Valkey disabled. The
[versioned browser receipt](evidence/v0.1.1/browser.json) binds the core, workers,
WebUI files and harness by SHA-256 and verifies that the input files did not change
during the run. It records the individual groups and screenshot hashes:

1. Actual first-run initialization and HttpOnly authentication.
2. Model and plugin forms, including a real diagnostic worker connection test.
3. A model-tool-model conversation and its tool-result/trace display.
4. A delayed actual HTTP response while the user types the next draft.
5. Knowledge import, searchable chunks, and safe display of HTML-looking text.
6. Visible conversation pagination past 200 resources.
7. Agent default-model selection and a subsequent explicit override.
8. Credential-destination rejection followed by explicit clearing in the editor.
9. Desktop light/dark and Chinese 320px navigation, no horizontal overflow,
   scroll-reachable dialog actions and keyboard dismissal.
10. Session expiry outside Chat followed by a different member signing into the
    same workspace without seeing the previous account's private resources/draft.
11. No unexpected JavaScript errors or nonlocal page requests, plus stable build hashes.

The six historical 0.1.1 screenshots were inspected for readable layout and
clipping. The harness wrote the originals to `artifacts/screenshots/`; the
following preserved repository copies belong to that earlier run and are not
new 0.1.2 screenshots:

| View | Screenshot |
| --- | --- |
| Dark setup, 1440px | [Workspace initialization](screenshots/01-setup-dark-desktop.png) |
| Dark chat, 1440px | [Model and tool round trip](screenshots/02-chat-dark-desktop.png) |
| Light desktop, 1440px | [Conversation workspace](screenshots/03-chat-light-desktop.png) |
| Chinese dark chat, 320px | [Mobile conversation](screenshots/04-chat-dark-zh-320.png) |
| Chinese knowledge editor, 320px | [Form fields](screenshots/05-knowledge-zh-320.png) |
| Chinese knowledge editor, scrolled | [Reachable form actions](screenshots/06-knowledge-actions-zh-320.png) |

The reproduced
pre-fix expiry failure is retained separately in
[browser-session-expiry-before.json](evidence/browser-session-expiry-before.json)
and is not counted as a passing run.

The [browser harness instructions](../tests/browser/README.md) describe a clean
local reproduction. `.github/workflows/browser.yml` runs the same isolated flow
with the locked Playwright dependency and publishes receipts/screenshots as CI
artifacts. This workflow definition is not evidence of a successful GitHub run.

This is Chromium coverage with a diagnostic process fixture, not a commercial
model or external messaging account. Native screen-reader testing, 200% zoom,
other browsers, the full OS matrix and Tauri installation remain unverified.
The full backend/database/fault-injection matrix remains a separate release gate.

Conversation metadata and message history are fetched separately. History is
retrieved in 50-message pages with stable backward cursors; duplicate IDs are
removed. Resource lists use 100-item pages with visible load-more controls and
deduplicate overlap by ID/version. Search labels specify that they search loaded
entries. The existing offset API is not a snapshot: concurrent deletions can move
rows between pages, and refreshing reconciles the list. This is pagination, not
virtual scrolling or full-collection server search.
Attachment upload/media preview, rich rendered Markdown, plugin-specific schema
configuration, dynamic third-party navigation surfaces and Tauri installers are
not implemented in the web client. Unsupported backend capabilities remain
visible in the capability report rather than being reported as successful.

Text/Markdown imports are limited to 512 KiB by the client. A one-time schedule's
datetime is explicitly interpreted in the device timezone and sent as an
absolute timestamp; calendar schedules use the selected IANA timezone. The
backend performs final validation and schedule calculation.
