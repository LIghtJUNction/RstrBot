# Local browser acceptance

This harness starts a disposable SQLite RstrBot instance and drives its actual
WebUI using Chromium. No personal browser, account, external model or external
website is used. All users, documents and conversations are synthetic fixtures.

Build the Rust binaries and `web/dist`, then run:

```sh
cd tests/browser
npm ci
npx playwright install chromium
npm test -- --binary ../../target/debug/rstrbot
```

To use an already-installed Chromium executable, pass `--chromium /absolute/path`.
Use `--evidence-dir /path/to/results` to select a separate receipt/log directory.
The harness can also load the exact pinned Playwright version from
`CODEX_PRIMARY_RUNTIME_NODE_MODULES` when that runtime is available.

The run checks setup, model and plugin forms, the actual diagnostic worker/tool
round trip, delayed **real** chat HTTP responses, draft preservation after page
unmount/remount, workspace changes while conversation creation is pending,
removal of deleted conversation content from the visible cache, denied caches
remaining empty across 503 errors and remounts until a successful read, knowledge editing,
resource pagination beyond 200 entries, an agent's default model,
account switching after session expiry, explicit credential clearing when a model
destination changes, mobile navigation, locale and theme.
Snapshots and hashes are written to `artifacts/screenshots/` and
`docs/evidence/browser.json`. The binary and web hashes are checked again at exit;
a changed build invalidates the result.

The diagnostic worker is a process fixture, not an AI model. This is a bounded
Chromium acceptance check, not complete browser, screen-reader, native-desktop,
visual-design or production-provider certification. Resource list search is over
loaded pages; the harness exercises the visible pagination controls.
