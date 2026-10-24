# HTML Colors — Native V2

Standalone Rust library and compiled-in Tezbar adapter for the enhanced HTML
Colors source at `ccd3bcb56fa43e993381dec63d3b3e7e9403bac0`. The single command is
`html-color`; canonical ID is `com.tezbar.native.html-colors`, replacing
`raycast.html-colors`. macOS and Windows application verification is **not-run**.

The port retains all 16 basic and 140 extended records, source order and empty
query duplicates. Nonempty queries use the pinned source's **locked Fuse 7.5.0**
algorithm, weighted fields, field normalization, strict score cutoff, stable
ranking and name/category merging. The source declares `^7.1.0` but its lockfile
selects 7.5.0; this distinction matters for Bitap match masks and licensing.
Whitespace is preserved: `""`, `" "`, and surrounding spaces are distinct queries.
Unicode lowercasing, UTF-16 offsets, wrapping 32-bit Bitap operations and long
pattern chunks are retained. Unpaired UTF-16 surrogates cannot enter the Rust
String API: the pinned beta's serde_json boundary rejects such JSON. Valid
Unicode queries, including surrogate pairs split across internal Bitap chunks,
are covered by the executable oracle.

All Colors, Basic Colors, Extended Colors and Group by Shade are supported.
Grouping follows the source's match score, name-match count and static shade
order. HEX is shown initially and details are initially hidden. Per row actions,
in source order, copy HEX, RGB and the named-color ID, then toggle format and
details. Copy effects resolve from the current canonical server result only.
There are no preferences, storage, saved searches, JS runtime or runtime network
requests. The icon and original provenance remain credited separately from the
enhanced source. See [provenance.json](provenance.json) and [notices](licenses/NOTICE.txt).

## Compiled-in integration

1. Vendor this directory at **`src-tauri/native-extensions/html-colors`**.
   Add to the host's `src-tauri/Cargo.toml`:

   ```toml
   html-colors-native-v2 = { path = "native-extensions/html-colors" }
   ```

2. Register the concrete adapter in
   `src-tauri/src/native_core/extensions/mod.rs`:

   ```rust
   #[path = "../../../native-extensions/html-colors/src/tezbar_adapter.rs"]
   mod v2_html_colors;
   ```

   In `v2_runtime.rs`, add `"html-colors"` to the `slug` allowlist, route
   `commands("html-colors")` to `super::v2_html_colors::commands()`, and route its
   async `handle` arm to:

   ```rust
   super::v2_html_colors::handle(ctx, command, operation, payload, state).await
   ```

3. Add the object in [manifest.json](manifest.json) to the bundled
   `v2_catalog.json` array. Copy **`assets/html-colors.png`** unchanged to
   **`src/renderer/public/extensions/icons/html-colors.png`**. Its public URL is
   `/extensions/icons/html-colors.png`. Retain this package's license notices.

4. Rebuild Tezbar. Follow the host's normal canonical-ID installation flow.
   A manifest or `native-v2.json` marker alone cannot register executable Rust;
   beta has no external Rust extension loader. Platform verification remains
   `not-run` until actual UI and clipboard checks are recorded.

The adapter is compiled unchanged in the integration tests against byte-exact
beta `common.rs` and `v2_sdk.rs` from
`fd300e276356587726566026f072563eb9c34cb1`. Only the unused Tauri app handle is
stubbed. Copy uses **`v2_sdk::copy` → `clipboard:write`**, requiring an object with
`ok: true`, no non-null `error`, and no invalid/false `success`. Transport errors,
malformed envelopes and negative acknowledgements leave the current view and
actions available for retry. Feedback is returned as native response
`message`/`error`; the adapter does not issue HUD/toast RPCs or promise an OS notice.

Every emitted action is tied to one session and result generation. Search,
view changes and display toggles invalidate prior action IDs. Dispose invalidates
the session; relaunch creates new IDs. Extra text, colors, URLs and arbitrary
form values are rejected. Refresh returns `unchanged` without creating IDs;
search mutations stay in beta's staged state until `native.rs` commits them.
This avoids publishing action IDs from an uncommitted refresh or dropped search.

## Pinned beta presentation limits

The actual `List.Dropdown` wire is supported: its `props.actionId` receives
`formValues: { value: "all" | "basic" | "extended" | "grouped" }`. Beta paints
these options as buttons, flattens the Grouping section, and keeps its active
option locally. It ignores the supplied controlled `value` on remount; a fresh
command starts at All Colors, matching the initial local selection.

The source's 16px circle SVG data URI and circle-mask descriptor are supplied.
Beta paints the SVG in a 20px image slot and ignores the mask; the SVG itself is
a circle. The source's 200×150 rounded rectangle preview and exact Categories,
Name, Named color, HEX, RGB labels and separators are supplied and rendered by
the actual beta detail/Markdown components.

Circle/CircleEllipsis category accessories and tooltips retain source data and
basic-first order. Beta's accessory parser drops tooltips and its renderer
returns null for icon-only accessories. Thus those category glyphs are not
visible in this beta; category metadata in details remains visible. Beta also paints both category
metadata icon tokens using its generic circle-plus fallback.

The five source actions and `cmd+t` / `cmd+i` descriptors are preserved. Beta's
list and action overlay do not dispatch arbitrary shortcut descriptors. Use the
actions menu (Cmd+K) for Show RGB/HEX and Show/Hide Details. Beta handles arrows
and Enter; its Cmd+K is specifically Meta-based, with no guaranteed Ctrl mapping
on Windows. Selection belongs to the renderer's index: search clamps the index,
and a dropdown click resets it to zero. The shown row, details and action IDs
stay coherent, but preserving the same selected color across reordered results
is not supported by this beta. No unsupported `selectedItemId` field is invented.
The renderer retains an in-progress local query draft when a same-query view
refresh arrives; search text is never trimmed by this adapter.

## Reproducible validation

From this directory, with Rust 1.85+ and Node 22+:

```sh
cargo test --locked
cargo +1.85.0 test --locked
cargo fmt --check
rustfmt --edition 2021 --check src/tezbar_adapter.rs
cargo clippy --locked --all-targets -- -D warnings
cargo +1.85.0 clippy --locked --all-targets -- -D warnings
cargo check --locked --all-targets --target x86_64-pc-windows-gnu
cargo clippy --locked --all-targets --target x86_64-pc-windows-gnu -- -D warnings
npm ci --ignore-scripts
npm test
cargo package --locked --allow-dirty
```

Rust fixtures are kept byte-exact and excluded from rustfmt traversal. The npm
validation dependencies are build/test tools only. `scripts/oracle.mjs` executes
the pinned **original TS/TSX files** with their actual vendored locked Fuse;
Raycast and React hooks are replaced only at the UI/effect boundary. It checks
all color literals, four views, ranked raw scores and order, category merging,
shade section order, thousands of normal/typo/HEX/RGB/Unicode/long queries, source
TSX views, SVGs, metadata and all three original copy handlers. Expected search
logic is not reimplemented in JavaScript. `scripts/renderer.mjs` executes the
byte-exact beta list, detail, Markdown sanitizer and action overlay, with OS
methods forbidden. `scripts/provenance.mjs` validates the full pinned schema,
all fixture hashes and original icon bytes. Rust tests also verify native SDK
copy envelopes, retries, actions, staged-search rejection and lifecycle safety.

The package includes all fixtures, licenses, validation scripts and locks; it
can be extracted and tested independently. Dependency installation/build may
need network access; executing the Rust command does not. No real clipboard,
browser or window calls are made in tests. Real macOS/Windows UI and OS
clipboard behavior has not been run.

Recorded development checks: Rust 1.98.0 and 1.85.0 tests and strict all-target
clippy passed; 11 Rust tests include 84 source-derived offline view/query cases.
Current-toolchain Windows GNU all-target check/clippy and authored rustfmt checks
passed. The executable source oracle passed 4,406 distinct valid Unicode queries
across four views (17,624 comparisons), 14,475 raw score checks, 128 original TSX
view combinations and 5,940 original copy-handler checks. The actual beta renderer
passed three cases covering 163 swatch rows, detail metadata, dropdown actions,
search drafts, action ordering and documented keyboard limits. Full schema and
43 fixture hashes, the 32,645-byte icon and licenses passed verification. A fresh
extraction of the Cargo package passed both Rust toolchain test suites, strict
clippy and the complete npm suite. Disposable V1 passed all 17 original tests and
TypeScript 5.9.3 typecheck; tracked V1 files and root package-lock remain byte-exact
except the README discovery link.

To repeat the original V1 checks without changing its pinned fixtures, copy
`tests/fixtures/own` to an ignored disposable directory, run `npm ci --ignore-scripts`
there to use its original npm lock, then run
`pnpm --config.verify-deps-before-run=false run test` and
`pnpm --config.verify-deps-before-run=false run typecheck`.
