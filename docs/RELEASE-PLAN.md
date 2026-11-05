# Kensaku: review and next-release plan

Scope: `yjid/kensaku` at `bf282df` (v1.0.15). 45.7k lines across `src-tauri/` (Rust),
`src/` (React + Tauri), `cli/`, and the release workflows.
This document is the working plan for the next release. It does not start a new
version line: the next release is the next 1.0.x patch after 1.0.15.
Every item has a file:line or a measurement behind it, and a stated verification status.

**Verification legend**
- `V` verified directly in this review (code read + logic traced)
- `M` reported by static review, code quoted, not independently re-read
- `T` needs a measurement or a reproduction before it can be fixed safely

---

## 1. Ship blockers (fix before any release)

### 1.1 Agent surface can read any file on disk `V`
`agent_path_allowed` returns `true` when the scope list is empty
(`agent/mod.rs:626`), and the default config ships an empty scope
(`config.rs:450`). `read_snippet` (`mcp.rs:789`) then reads the path straight
from disk, never consulting the extension denylist that indexing uses. Any
connected MCP client could read `~/.ssh/id_ed25519`.
**Fixed in this cycle:** `agent_read_allowed` refuses known credential files and
anything in the configured exclude list, even with an empty scope. Tests added.

### 1.2 Portable Windows build ships without OCR `V`
`release.yml:162` copies only `kensaku.exe` and the marker file into the zip; the
bundled installers get `onnxruntime.dll` as a Tauri resource, the zip does not.
Every portable install has permanently dead OCR. **Fixed:** the DLL is copied and
the build fails if it is missing.

### 1.3 Doctor reports a healthy start as a crash `V`
`lib.rs:210,245` write `INFO` milestones into `crash.log`; `doctor.rs:184` warns
on any non-empty last line. Every clean install shows a warning whose text is the
app's own startup log. **Fixed:** milestones go to `kensaku.log`, and the doctor
now returns the most recent *failure* line, ignoring legacy `INFO` entries.
Tests added.

### 1.4 Launcher hides on right-click and on sort `V`
`App.tsx:990` hides on window blur. There is no contextmenu handler, so WebKitGTK
opens a native menu; and the sort control is a native `<select>`, whose popup is a
separate GTK window on Linux. Both blur the webview and the palette vanishes.
**Fixed:** root-level contextmenu suppression, in-app sort listbox, and a blur
decision function with `document.hasFocus()` plus an interaction grace window.
7 unit tests.

### 1.5 IME breaks search for CJK users `V`
`handleKeyDown` never checks `e.isComposing`, so the Enter that commits an IME
candidate also opens the selected file, and arrow keys paging candidates move the
list selection. The app ships a `zh` locale. **Fixed:** composition guard on the
key handler and the Escape handler.

### 1.6 Enter acts on the previous query's results `V`
Rows are replaced only when the response lands; the debounce plus IPC leaves a
window where Enter, Space, Cmd+C and Cmd+I act on the previous result set.
**Fixed:** results carry the query they were produced for and those actions bail
when it does not match.

---

## 2. Correctness defects

| # | Defect | Evidence | Status |
|---|---|---|---|
| 2.1 | An unreadable subdirectory deletes its whole subtree from the index on the next crawl | `walker.rs:307` drops the error, `mod.rs:1082` treats absence as deletion | M, fix next |
| 2.2 | Directory rename orphans every descendant document in the content index | `metadata.rs:607` derives paths from names it is mutating; `old_paths` ends up with one entry | M, needs test |
| 2.3 | Watcher hard-codes a 100 KB content cap, truncating the first save of larger files | `watcher.rs:821` vs `content_max_text_bytes()` | **Fixed** |
| 2.4 | Files over 1 MB ignore the configured content budget (fixed 512K head + 512K tail) | `extract/mod.rs:726` | **Fixed**; UTF-8 boundary and seek failure also handled |
| 2.5 | Same-size edits within one second are invisible, and the extraction cache returns stale text | `mod.rs:89` compares second-resolution mtime; cache key uses the same | M, needs ns-precision change |
| 2.6 | Changing the excluded-extension list never purges already-indexed files | `mod.rs:662` marks the path crawled before the exclusion check | **Fixed** |
| 2.7 | The watcher ignores `.gitignore`/`.cursorignore`, so build artifacts flip in and out of the index | `watcher.rs:484` has no `IgnoreSet` | M, fix next |
| 2.8 | `watcher_ok` reports true before any watch is registered | `watcher.rs:202` | M, fix next |
| 2.9 | Crawl gate never released on panic; every later crawl is a no-op | `mod.rs:551` has no guard, `run_rebuild` does | M, fix next |
| 2.10 | Fuzzy prefilter uses a basename bloom but scores the full path, silently dropping path matches | `fuzzy.rs:36` vs `walker.rs:338` | M, needs a fixture test |
| 2.11 | Exhaustive paging applies the 10k cap before the cursor filter, and ignores scope in the query | `search/mod.rs:1729` | M, needs a test |
| 2.12 | MCP regex tool splits patterns at whitespace | `mcp.rs:533` | **Fixed** (quoted) |
| 2.13 | `reset_all` restarts via the mechanism the codebase retired as broken (second process loses the instance lock) | `commands.rs:1345` vs `indexer/mod.rs:1201` | M, fix next |
| 2.14 | Updater accepts a truncated download and marks it installable; no timeout, no checksum | `updater.rs:237` | M, fix next |
| 2.15 | Portable install updates itself by installing a second copy into Program Files | `updater.rs:315`, no portable check | M, fix next |
| 2.16 | Semantic store dimension is the compile-time 256 while the smart model is 384, so vectors are dropped silently | `indexer/mod.rs:297`, `embed/units.rs:114` | M, latent (feature-gated) |
| 2.17 | Tesseract is invoked without `-l`, so every non-English corpus indexes English garbage | `extract/image.rs:615` | M, plan below |
| 2.18 | Scanned PDFs decode all pages into RAM before OCR (up to ~3 GB) | `pdf_image.rs:32`, `pdf.rs:131` | M, plan below |

---

## 3. Performance

Measured/derived, ordered by expected payoff.

**P1. Stop materializing the corpus on every keystroke** `V`
`search/mod.rs:843` builds a `FileEntry` for every file per query: a path
reconstruction, a `PathBuf` join and a `name` copy each. On 1M files that is
millions of allocations per keystroke.
Fix: cache `Arc<Vec<FileEntry>>` in `MetadataStore` keyed by the existing
`generation()`, which search already reads.
Expected: the dominant cost of short queries disappears; target p50 search under
10 ms on 500k files.

**P2. Allocation-free filter predicates** `V`
`CompiledFilter::matches` called `to_lowercase()` per entry for `name`, `path`,
`extension` and type. **Fixed:** `contains_ignore_case` and `eq_ignore_ascii_case`
comparisons. `type:image` on 1M entries no longer allocates 2M strings.

**P3. Sort without allocating in the comparator** `V`
`SortOrder::Name` allocated two strings per comparison. **Fixed:**
`sort_by_cached_key`.

**P4. Missing SQLite indexes** `V`
The embedding queue scan (`metadata_db.rs:769`) planned as `SCAN nodes` plus a
temp b-tree; burst candidate lookup (`:1060`) full-scanned `nodes` once per image.
**Fixed:** two indexes plus removing a predicate that defeated the partial index.
Verify with `EXPLAIN QUERY PLAN` and a 500k-row fixture.

**P5. Filter-only browse builds a map of every candidate then truncates** `M`
`search/mod.rs:1263`. Fix: bounded heap of `offset + limit` plus a single facet
pass. Expected: browsing a 200k-file folder stops costing ~50 MB per keystroke.

**P6. Regex compilation on every query** `M`
`query/intent.rs` compiles ~20 patterns per keystroke. Hoist to `LazyLock`.

**P7. Watcher and rename cost** `M`
Directory delete/rename materializes every entry and issues one statement per
row while holding the write lock. Fix: prefix `UPDATE`, filtered cursor.

**P8. OCR** `M`
- Stream PDF pages instead of decoding all of them (peak RSS, P6 above).
- Indra pool is `cores/2/2` engines, i.e. one engine on a 4-core machine
  (`ocr_indra/mod.rs:162`) while extraction runs 2-8 OCR jobs. Measure with
  `mem_probe` before changing.
- Deskew resamples ~32M pixels per page, twice on rotated pages, single-threaded.

---

## 4. OCR and image quality

1. **Tesseract languages.** Add `ocr_languages` to config, pass `-l`, default to
  `eng` plus the UI language when that traineddata exists, and expose it in
  Settings → Search next to the engine picker. Also choose PSM per source: `6` for
  documents, `11` for screenshots. This is the single biggest recall win for
  non-English users on the default engine.
- Rasterize genuinely scanned PDFs at 300 DPI when no XObject is decodable
  (`pdf.rs:64`), instead of indexing the literal `[Scanned PDF: Image only]`.
- Length-aware confidence gate (`engine.rs:32`): one uncertain glyph currently
  erases a whole short line, which is exactly the table-cell and date text
  search needs.
- Split over-wide line boxes instead of clamping to 4096 px
  (`imageops.rs:338`).
- CLS pooling for the smart embedder (`smart.rs:176`), which currently
  mean-pools a CLS-pooled model.
- `cosine_similarity_i8` assumes both vectors are unit length
  (`embed/model.rs:293`); store norms or normalise at write.

---

## 5. UI / UX

**Fixed this cycle**
- Sort is an in-app listbox: identical on every platform, no window blur.
- Native context menu suppressed; the palette has one in-app menu.
- Blur dismissal ignores popups and focus-preserving interactions.
- IME composition no longer opens files.
- Tab no longer traps focus: from the search field it walks results, Shift+Tab
  leaves to the toolbar, and once focus is on a control Tab is native.
- Stale rows are not actionable.
- Snippets load for rows scrolled into view (viewport root + mutation observer).
- Hover details fetch on hover, not on mount (was one IPC per card per search).
- Thumbnails turning off actually removes loaded thumbnails in grid view.
- Toolbar measurement no longer rebuilds a `ResizeObserver` per render.

- Settings `Switch` controls were unnamed for screen readers; `Field` now mints a
  label id and `Switch` consumes it, covering all eight rows.
- Escape in Settings no longer discards unsaved edits silently.

**Next**
- `DetailsDialog`/`PreviewDialog`/`HelpDialog` declare `role="dialog"` without
  `aria-modal`, focus trap or focus restore.
- Alt-click facet exclusion is a no-op under StrictMode because a setter is
  called inside a state updater (`App.tsx:600`).
- Paged responses overwrite `total` with `0` (`App.tsx:504`).
- `useStats` polls every 2s with no `visibilitychange` pause.
- Platform glyphs: hardcoded `⌘` in `KeyHintStrip` and Help on Linux/Windows.
- Copyable text: `select-none` on the root prevents selecting a path in Details.

---

## 6. System design

- **Path materialization** (P1) is the structural fix behind most search cost:
  the arena exists so names and paths are not copied, and search undoes that on
  every query. Cache the view, invalidate by generation.
- **Extraction cache keyed by (path, size, mtime_secs)** cannot represent two
  edits in one second. Move to nanosecond mtime and treat watcher modify events
  as cache-bypassing.
- **One `IgnoreSet`**, used by the crawler and the watcher. Today they disagree,
  which is why the index flip-flops on builds.
- **Doctor as a table.** Three near-identical engine branches produced the
  crash-log false positive; a capability table removes the copy-paste class.
- **Path storage as BLOB** for filesystems with non-UTF-8 names
  (`metadata_db.rs:321` currently collapses distinct files).
- **Ship or delete the daemon.** `kensaku-daemon` is built, documented and never
  packaged, so the daemon architecture is dead code in every release.
- **CI**: add `cargo fmt --check`, build+test the `cli` workspace, and a
  `cargo check` matrix on macOS and Windows so `#[cfg]` code compiles before tag
  day. Add a release-artifact assertion job (the portable DLL bug class).

---

## 7. Release plan

### Phase 0: blockers (this cycle, PRs open)
1. #95 launcher dismissal + keyboard/IME correctness (frontend).
2. #94 doctor crash-log classification + milestone log split.
3. Portable Windows OCR library in the zip, asserted in CI.
4. Agent read guard for credential files.
5. Perf batch: allocation-free filters, cached sort key, two indexes, watcher
   and large-file content budgets.

### Phase 1: index correctness (1.0.16)
6. Crawl-failure containment: unreadable directories protect their subtree.
7. Rename repair: snapshot descendants before mutating; purge from Tantivy.
8. One `IgnoreSet` shared by crawler and watcher; `watcher_ok` only after
   registration; crawl gate released on panic.
9. Nanosecond mtime in `FileEntry` and the extraction cache; bypass cache on
   watcher modify.
10. Exhaustive paging honours scope in the query and pages after the cap
   correctly.

### Phase 2: search performance (1.0.16)
11. Materialized-entry cache keyed by generation (P1), measured on the golden
   set before and after.
12. Bounded heap + single-pass facets for filter-only browse (P5).
13. Hoist query-intent regexes (P6).
14. Add a latency gate to CI using the existing `examples/*_bench.rs`.

### Phase 3: OCR (1.0.16)
15. Tesseract `-l` plus PSM selection, with the settings control.
16. Stream PDF pages with an aggregate pixel budget; 300 DPI raster fallback.
17. Length-aware confidence gate; split wide lines.
18. CLS pooling and normalised cosine for the smart/visual paths.

### Phase 4: shell hardening (1.0.16 or a follow-up patch)
19. `reset_all` in-process teardown, not `app.restart()`.
20. Updater: timeouts, checksum verification, truncated-download rejection,
    portable-mode install path.
21. Instance lock and `--toggle` correctness; atomic `show()`.
22. Decide daemon: package it as a sidecar or remove it.

### Acceptance for the next release
- No warning in the doctor on a clean install on all three platforms.
- OCR works from the portable zip on a machine with no prior install.
- Search p50 ≤ 10 ms and p95 ≤ 25 ms on a 500k-file corpus, recorded in
  `docs/BENCHMARKS.md`.
- Golden-set recall@5 does not regress from 0.90; MRR reported alongside.
- IME input opens nothing until the candidate is committed.
- No file outside the index scope is readable through MCP, asserted by test.
- CI green on Linux, macOS and Windows for both workspaces.

---

## 8. Risk register

| Risk | Where | Mitigation |
|---|---|---|
| Materialized-entry cache serves stale rows | `MetadataStore` generation | Cache invalidated by the same counter the index bumps on every mutation; test that a re-crawl changes the vec |
| Read guard breaks a legitimate agent workflow | `agent_read_allowed` | Scope still wins; only credential filenames/extensions are refused; covered by tests |
| ns-precision mtime changes cache misses | extraction cache | Key change only; verify hit rate on the golden corpus |
| PDF page streaming changes OCR text | `pdf.rs` | Compare extracted text on the golden set before/after |
| Rename repair removes live documents | `metadata.rs` | Test rename → assert old paths gone and new paths present, content searchable |

## 9. Test strategy

- Rust: unit tests alongside each fix (added for the crash-log classifier, the
  agent read guard, blur dismissal, sort listbox behaviour).
- Indexer: a fixture that renames a directory and asserts document counts in
  both the metadata store and Tantivy.
- Search: latency bench gate; a fixture where the query terms exist only in the
  path, to catch the fuzzy prefilter bug.
- Release: a CI step that unzips the portable artifact and asserts
  `onnxruntime.dll` is present and non-empty.
- Frontend: `vitest` added; the dismiss decision is the first suite.


---

## 10. Manual testing

Defects CI cannot reach (window-manager behaviour, IME input, filesystem permissions, the
updater and the reset flow) are listed in [`TESTING-1.0.16.md`](TESTING-1.0.16.md). Each item
states the old behaviour, so a pass means "this no longer happens".

## 11. Branches opened from this review

| Branch | What it fixes |
|---|---|
| `fix/launcher-stability-and-input` | #95 dismissal, IME, Tab, stale rows, snippet observation, hover cost, thumbnail toggle, toolbar measurement |
| `fix/doctor-crash-classification` | #94 crash-log warning, milestone log split |
| `fix/release-portable-onnx` | #94 portable OCR library, asserted at build time |
| `fix/agent-read-scope` | MCP credential read hole, regex pattern quoting |
| `perf/search-and-index-integrity` | per-entry allocations, rename orphan purge, watcher caps, large-file budget, exclusion purge, watcher status, two indexes |
| `fix/a11y-switch-labels` | accessible names on settings switches |
| `fix/settings-discard-confirm` | unsaved settings edits are not silently discarded |
| `perf/polling-while-hidden` | stats polling pauses when hidden |
| `docs/release-plan` | this document |
