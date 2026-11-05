# Scale design: bounded memory and index size

Tracking issue: #129. This document is the design. The implementation is the PR that links it.

## 1. Problem

Kensaku was built and tuned on a developer's corpus of tens of thousands of files. Large
corpora, and corpora dominated by images and scans, expose three costs that the design does
not bound:

1. **Peak memory is not budgeted.** Decoding an image allocates `width x height x 4` bytes.
   Work was scheduled by CPU count, so peak memory was "images in flight times their size".
2. **Index size grows faster than the text it indexes.** Measured on a 25,147-entry corpus: a
   192 to 199 MB full-text index, about 8 KB per file.
3. **Memory is not attributable.** The health view reports one resident-memory figure.

These are design problems, not tuning problems. One user report of about 7 GB of resident
memory has not been reproduced. The mechanisms above would produce numbers of that order on
a large image-heavy corpus, which is why they are bounded by design.

## 2. Measured facts and code references

Only facts that were measured or read from the code appear here.

| Fact | Where |
|---|---|
| The folded trigram field stored term positions | `indexer/content.rs`, `folded_indexing` |
| No query reads positions: folded search builds boolean term queries; no phrase query exists | `indexer/content.rs`, `search_folded_filtered` |
| The stored raw text is required: folded search verifies candidates against it | `indexer/content.rs`, `search_folded_filtered` |
| OCR decoded full pages with no budget; the hashing path had a pixel guard | `ocr.rs`, `ocr_indra/mod.rs`, `indexer/extract/image.rs` |
| Extraction threads: `clamp(cores, 2, 8)`, no memory term | `indexer/extract/mod.rs` |
| Default per-file text cap is 4 MB | `config.rs`, `default_content_max_text_bytes` |
| Filename search scans the materialized entry list per query | `search/mod.rs` |

## 3. Scale tiers and targets

These are targets to be verified, not measurements.

| Tier | Files | Resident memory (target) | Index (target) |
|---|---|---|---|
| Small | up to 100k | under 400 MB | under 1 KB per file |
| Medium | 100k to 1M | under 1 GB | under 1 KB per file |
| Large | 1M and above | under 2 GB, bounded by configuration | under 1 KB per file |

The per-file index target is an order of magnitude below today's measured figure. It is only
reachable with a change to what is indexed, not just how it is stored, so it is a target for
Phase 2 and later, not for this change.

## 4. Design

### 4.1 Decode budget (this change)

Every full-image decode that feeds recognition reserves its RGBA size first, and holds the
reservation until recognition has finished with the pixels.

- `util::budget::DecodeBudget` is a counting semaphore measured in bytes.
- Default budget: 768 MiB. Override with `KENSAKU_DECODE_BUDGET_MB`.
- A request larger than the budget is clamped to the budget and runs alone. It waits until
  nothing else is reserved, so it cannot deadlock the queue, and it is the only thing in
  memory while it runs.
- Covered paths: OCR (accurate engine and Indra), visual embedding, and scanned-PDF page
  decodes. The reservation is taken before the decoder allocates.
- Not covered, deliberately: thumbnails, which are capped at 240 pixels, and perceptual
  hashing, which already runs behind the pixel guard.

Peak decoded-image memory is now bounded by the budget plus one clamped request. It no
longer depends on the number of cores.

### 4.2 Positionless trigram field (this change, schema v4)

- The folded trigram field stores frequencies only.
- The schema version becomes 4. Existing indexes rebuild once through the migration path that
  already exists, and the reason is recorded: "trigram posting format change (schema v3 -> v4)".
- The raw text stays. Folded search needs it for verification.
- Tests: a v3 index rebuilds once; a v4 index does not rebuild on launch; folded search works
  on the rebuilt index.

Cost of the upgrade: one full re-read of the corpus per install, and the previous index is
kept as `index.legacy-<stamp>` until the next cleanup, so disk use is temporarily higher.

### 4.3 Attributable memory (next)

Health reports resident memory per subsystem: entry list, index, semantic units, decode
budget in use, and loaded model packs. Without this, a memory report cannot be diagnosed.

### 4.4 Benchmark corpus and CI gate (next)

A generator produces a deterministic synthetic corpus of text, code, PDF, and image files at
chosen sizes. Benchmarks record peak resident memory, index bytes per file, cold-launch time,
and query latency at each tier. A regression beyond a set threshold fails CI.

Until this exists, every target in section 3 is unverified.

### 4.5 Name index for filename search (later)

Filename search scans the entry list per query. At 1M entries that is a linear pass per
keystroke. The planned replacement is a sorted name index (prefix search by binary search,
substring by a trigram posting set), built from the entry list and updated on mutation.

### 4.6 Lazy semantic loading (later)

Semantic units are loaded at startup regardless of whether semantic search is used. Loading
them on first semantic query, or only when semantic search is enabled, removes that cost
from every launch.

### 4.7 Model session memory (later)

ONNX sessions use greedy default allocators and are not released between crawls. Options:
bounded arenas, session reuse with explicit release, and fewer engine instances.

### 4.8 Legacy index cleanup (later)

After a successful rebuild, the superseded index can be removed once the new index has
completed a crawl. Today the newest backup is kept indefinitely.

## 5. Risks

- **One-time reindex** for every existing install (section 4.2). Mitigation: the rebuild is
  automatic, logged, and does not repeat on later launches.
- **Budget too small** makes OCR slower on large images. Mitigation: the environment override,
  and the budget reports its use so a bottleneck is visible.
- **Budget deadlock** if a holder waits for a second reservation. Mitigation: no holder
  acquires a second permit; recognition never decodes. This is a documented invariant of the
  code, covered by review, and should gain a test.
- **Trigram recall change.** Dropping positions must not change which documents match. The
  tests cover folded search on a rebuilt index; a recall comparison on a real corpus is part
  of the benchmark work (section 4.4).

## 6. Verification

- Unit tests: budget accounting, contention, oversized requests, zero requests, decode
  reservation size, corrupt input, schema v3 to v4 upgrade, and no rebuild after upgrade.
- CI: the existing Rust test, clippy with warnings denied, and frontend jobs.
- Not yet verified: resident memory and index size at each tier. That is section 4.4.

## 7. Rollback

The budget has no persistent state, so it is rolled back by reverting the commit.

The schema change can be rolled back by restoring the previous version constant. An index
written at v4 would then rebuild once more into the older format, which is safe but costly,
so it should be avoided unless necessary.
