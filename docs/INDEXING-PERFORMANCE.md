# Indexing performance: plan

Written after two field reports: "indexing happens every time I open the app", and
`Building semantic search` re-running for the same files. This is the plan, the reasoning
behind each item, and what was deliberately left out.

## What is actually wrong

Five separate things, in order of how much pain they cause.

**1. A launch re-reads files it should not.** Two defects, both found and fixed this week:
an unreadable stat was stored as `(0, 0)` and then re-read on every launch, and the OCR
signature was written as "no engine" when a startup probe failed, so the next launch
re-OCR'd every image. Both are fixed; #123's `Last index scan` row names the cause from
now on.

**2. The crawl does not yield to the user.** The phash backfill checks a `query_in_progress`
flag before each file, but the crawl's extraction pool does not look at it at all. On a
25k-file corpus with a 2-8 thread extraction pool, every search during a crawl competes
with every core. This is why the app feels bad exactly when it is doing the most work.

**3. Search rebuilds the whole corpus on every keystroke.** `search/mod.rs:843` calls
`metadata.materialize(i)` for every entry, every query. That reconstructs a path from the
node arena, joins a `PathBuf`, and copies the name. At 25k files it is noticeable; the
scan you pasted had 108,175 semantic units and 211 MB of index, so it is a 25k-entry
corpus, and this runs 25,000 times per keystroke. The arena exists precisely so the query
path does not copy names and paths, and search undoes that.

**4. OCR and embedding run inline in the crawl.** A crawl is walk → diff → extract → OCR →
embed, all in one synchronous pass. OCR is minutes of work on a screenshot-heavy corpus.
The window stays responsive because the crawl is on its own thread, but the status bar says
"Indexing" for the whole thing, and nothing else can proceed.

**5. The watcher is stopped for the whole crawl.** Anything changed during a scan is
invisible until the next launch, which then legitimately re-reads it, which looks like
"it reindexed on its own".

## Plan

### Phase A — do now (this change)

**A1. Cache the materialized entry list, keyed on the metadata generation.**

`MetadataStore` already bumps a `generation` counter on every mutation that can change the
entry list. Search already reads it (`search/mod.rs:855`). Cache `Arc<Vec<FileEntry>>`
against that counter.

- Where: a small `EntrySnapshot` in `search/mod.rs`, holding `Mutex<Option<(u64, Arc<Vec<FileEntry>>)>>`.
- Invalidation: generation mismatch. No timers, no TTL, one comparison per query.
- Risk: a mutation that changes entries without bumping the generation would serve stale
  rows. Mitigated by auditing the bump sites (14 in `metadata.rs`) and a test that inserts,
  queries, inserts again, queries again, and asserts the second query sees the new row.
- Cost: the first query after a change still materializes once, which is the current
  behaviour; every subsequent query is a pointer bump.

**A2. Make extraction yield to in-flight queries.**

The extraction producer checks `query_in_flight` before pulling the next job and waits
while a query is running, the same way the phash backfill does (`indexer/mod.rs:191`).

- Wait is bounded (200 ms polls) so a stuck flag cannot stall the crawl forever.
- Only the *producer* yields: the consumer that writes to Tantivy is not blocked, so the
  index still converges.
- Trade-off: a crawl takes longer in wall-clock terms while the user is actively searching.
  That is the right trade: the user is searching, not waiting for a scan.

**A3. Report it, so the next field report is not a guess.**

`Last index scan` already names why files were re-read. Add the OCR signature transition
(the previous commit) and keep the crawl's own console line. No behaviour change.

### Phase B — the real fix, next change

**B1. Split the crawl into reconcile and drain.**

- Launch does: walk, stat, diff, write metadata, mark `content_pending`. Nothing else.
- A separate `drain` worker takes pending files in priority order (text before images,
  newest first), extracts, OCRs, embeds, and yields to queries between files.
- Status becomes two independent things: `Preparing` (seconds) and
  `Background: N files` (visible, pausable, ignorable).
- The drain is resumable because `content_pending` is already persisted per file. A restart
  continues instead of restarting.

This is the change that makes a launch cheap regardless of what changed. It is also the
riskiest: it moves the one place that writes both indexes onto a second thread lifecycle.
Do it behind the existing crawl gate, with the ordering guarantees the current pipeline
already has, and land it with a benchmark comparing launch time before and after.

**B2. Keep the watcher alive during the crawl.**

Feed watcher events into the same `content_pending` set instead of stopping the watcher.
Depends on B1: while the crawl owns the metadata write lock per file, an event racing it
can only mark a file pending, which the drain picks up.

### Phase C — measured, not guessed

**C1. Benchmarks as a gate.** `examples/*_bench.rs` exists and is never run in CI. Add a
job that runs the index and facet benches on a fixed corpus and fails on a >20% regression.
Without this, every future change is a guess.

**C2. OCR cost.** Stream PDF pages (done, #111), and size the Indra engine pool to
`cores / threads` rather than `cores / 2 / threads`, which gives one engine on a 4-core
machine while extraction runs 2-8 jobs.

## Review of this plan

Checked against the code before committing to it.

- **Is A1 worth it?** The scan you sent has 25,143 entries. At 25k entries the materialize
  pass is roughly 100k allocations per keystroke. The 40 ms debounce hides some of it; it
  does not hide it on a slower machine or a larger corpus. Yes, worth it, and it is
  self-contained.
- **Does A1 need to handle the daemon path?** No: the daemon owns its own store, and the
  cache lives in the query path of one process. Each process caches its own view.
- **Does A2 risk a deadlock?** The flag is set and cleared around the query in
  `commands.rs` with no early returns between them. The yield waits on a timer, not a
  lock, so a leaked flag costs a slower crawl, not a hang. Acceptable, and it matches what
  the phash backfill already does.
- **Should A2 reduce thread count instead of yielding?** No. Yielding is strictly better:
  during a query the pool stops, and between queries it uses every core.
- **Why not do B1 now?** It changes which thread owns the index writer and the transaction
  boundaries. Landing that in the same change as a cache and a yield policy would make a
  regression impossible to attribute. It deserves its own change and its own benchmark.
- **Is the plan missing the obvious?** One thing: a launch on a corpus with no changes
  should show nothing at all, not a walk. It cannot: the walk *is* how changes are found.
  With B1 the walk stays but takes a second and does not put a progress bar on the UI.

## What this change does not claim

- It does not make a first scan fast. The first scan is inherently the whole corpus.
- It does not fix the OCR backlog loop by itself; that is the signature fix in #125.
- It does not add the persisted drain queue. That is B1.
