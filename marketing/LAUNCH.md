# Kensaku launch kit (1.0.16)

Everything needed to announce the app. Tone rules: concrete over adjectives,
honest limits included, every post ends with a question or an ask for feedback.
Never crosspost the same day to overlapping audiences.

> Updated for 1.0.16. Re-read the numbers against
> [docs/BENCHMARKS.md](../docs/BENCHMARKS.md) before posting: the Windows and
> Linux figures are different machines, and the thread count changes the
> filename-search result.

---

## 1. Reddit

### Titles (ranked)

1. `You can't Ctrl+F a screenshot. Now you can.`
2. `I stopped naming files and started searching what's inside them`
3. `Spotlight for Windows — and it reads inside your screenshots (free, MIT)`

Title 1 for r/SideProject and r/productivity (curiosity + familiar pain).
Title 3 for r/windowsapps (Mac-envy phrase converts there). Title 2 is the
safe personal-story fallback anywhere.

### Body (all subs except r/opensource)

```
I could never find my own screenshots. I knew the error said "CUDA out of
memory". I knew I screenshotted it. I did not know it was called
"Screenshot 2026-07-14 094231.png".

So I built Kensaku (検索 = search). It indexes your files and reads inside
them — PDFs, DOCX, code, logs, and with OCR, the actual text in your
screenshots. It also finds by meaning: "the PDF about database indexing"
locates notes-final-2.pdf. And it parses "screenshots from last week".

  • 100% local. No account, no telemetry, nothing to the cloud.
  • Alt+Space to summon. ~12 MB of metadata for 100,000 files; the 1.0.16
    Windows setup is 14.3 MB.
  • Free and open source, MIT: github.com/yjid/kensaku
  • Bonus: Claude or Cursor can search the same index over MCP, read-only.

Windows is the build I've beaten on most; macOS and Linux come straight
from CI and need real-world testing — that's exactly the feedback I'm
here for.

What's the first thing you'd search for?
```

### r/opensource variant

Same body, plus this line before the final question:

```
License: MIT. Stack: Rust + Tauri, SQLite (WAL) as durable truth, Tantivy
for full text, a static embedding table for semantic search. Docs live
in /docs — ARCHITECTURE.md is the honest map. Extractors are a documented
extension point (docs/EXTRACTORS.md) if you want to teach it a new format.
```

### Attach

`assets/search-at-full-speed.png` to every Reddit post (visual ≈ 2× engagement).

### Per-sub first comment (post it yourself immediately after submitting)

- r/SideProject: "The single most requested thing so far was 'search my
  screenshots' — OCR runs with PP-OCRv6 by default (bundled runtime; the
  ~6.1 MB model pack downloads once), and the optional Indra engine handles
  handwriting-ish and low-quality captures."
- r/productivity: "The workflow trick: don't organize. Search 'invoice
  modified:30d' beats the folder you were going to create."
- r/windowsapps: "Alt+Space is the default summon (rebindable in Settings).
  It starts hidden in the tray; closing the window destroys the webview and
  keeps the index warm instead of rebuilding it."
- r/opensource: "Good first issues are labeled in the tracker; extractors
  and agent tooling are the two areas where outside PRs land fastest."

---

## 2. Show HN

### Title (mechanism-forward, zero hype — HN rule)

```
Show HN: Kensaku – local file search that reads inside your screenshots
```

### Body (~110 words)

```
Kensaku (検索) is local-first file search and a launcher for
Windows/macOS/Linux. It indexes file names (milliseconds across 100k
files, benchmarks with methodology in the repo), full text in
PDFs/DOCX/PPTX and code, OCR'd text inside screenshots via PP-OCRv6 by
default (bundled runtime, Tesseract fallback), and semantic matches —
"the PDF about database indexing" finds notes-final-2.pdf. No cloud, no
account, MIT.

I built it because I kept losing screenshots of errors I could describe
but not name. Indexing is a single pass — each file feeds both the name
and text indexes once — with incremental updates on change.

Honest limits: builds are unsigned, thumbnails are images-only, and
visual search is still a beta download rather than part of results.
Windows is the most-tested build; macOS/Linux feedback is the thing I
want most: github.com/yjid/kensaku
```

### Author first comment (post immediately — the technical deep-dive)

```
Author here. Architecture notes for the technically curious (full map in
docs/ARCHITECTURE.md):

- SQLite with WAL is the durable truth; an in-memory snapshot built at
  startup serves queries and is mutated incrementally. Chose this over a
  custom mmap format for crash safety and free migrations; the mmap
  Everything-style format is the documented next step if cold load ever
  exceeds 400ms at 1M files.
- Name search is served from the in-memory tier. On the 100k Windows
  corpus that is 17–19 ms with the default 16-thread pool, 3.1 ms at 4
  threads, and 1.3 ms on the warm Linux reference — the char-bloom
  prefilter makes each entry cheap enough that oversubscription costs
  more than it saves. Full text is Tantivy (batched). We benchmarked
  SQLite FTS5 trigram as a replacement and rejected it: 918 MB and much
  slower broad queries at 1M documents (docs/BENCHMARKS.md).
- Semantic tier: a static embedding table (potion-base-8M-style, 256-dim,
  i8-quantized, ~7.3 MiB). No ONNX and no neural graph at query time —
  tokenize, table lookup, weighted mean, normalize. An optional smart
  pack exists behind a feature flag; its quality and footprint are not
  yet measured, so the static table remains the shipped path.
- OCR: PP-OCRv6 by default (bundled runtime); optional Indra engine
  (higher quality, ~27 MB pack) and Tesseract fallback (system binary,
  used on Intel Macs where the bundled runtime is unavailable).
- Closing the launcher destroys its webview and keeps the Rust index
  warm; reopening does not rebuild metadata or Tantivy. The 100k-file
  metadata layer is 128 B per entry, about 12.2 MB.
- Agent access is read-only over MCP/loopback HTTP, budgeted in size,
  content returned as data never instructions.

Known tradeoffs we argued about: unsigned binaries (signing costs are
real for an independent app), thumbnails image-only for now, and visual
search not yet wired into results. Updates ship in-app: size-checked and
verified against the release SHA256SUMS before install. What would you
want an extractor for next?
```

---

## 3. X / Twitter

### Main post

```
You can't Ctrl+F a screenshot.

So we made Kensaku — local file search that reads inside your files:
PDFs, code, logs, and the text in your screenshots.

Describe it: "the PDF about database indexing". It finds notes-final-2.pdf.

100% local. Free, MIT.
github.com/yjid/kensaku
```

### Follow-up (reply to own post)

```
Alt+Space to summon, type, Enter. The 1.0.16 Windows setup is 14.3 MB;
about 12 MB of metadata covers 100,000 files. Your AI agent can search
the same index over MCP — read-only, nothing leaves the machine.
```

---

## 4. Playbook

| When | Where | Notes |
|---|---|---|
| Day 1, 8–11am ET | r/SideProject | Title 1, hero image attached, own first comment |
| Day 1 or 2, 7–10am ET | Show HN | Tue–Thu only. First comment immediately, answer everything for 24h |
| Day 3 | r/opensource | Title 2 + license/stack line, contribution-focused first comment |
| Day 4 | r/productivity, r/windowsapps | Crosspost variants, own per-sub comments |
| Same days | X | Main post morning ET, follow-up reply after 4–6h |

Rules that matter more than copy:

1. **Reply speed is the growth engine.** A bug reported in comments and
   fixed with a pushed commit the same day outperforms any headline.
2. **Never argue feature gaps.** Quote the honest-limits paragraph.
3. **One metric to watch:** downloads on the release page per 1,000 views.
   If it's low, the README hero is the thing to fix, not the posts.
4. **If HN flags the domain** (new domain, low karma), submit the deep
   GitHub URL, not the marketing landing page.
5. Update the [latest release](https://github.com/yjid/kensaku/releases/latest)
   notes only via new releases — never edit a published release silently.

---

## 5. Product Hunt

**Tagline (≤60 chars) — pick one, test in A/B:**

1. `Ctrl+F for your screenshots` ← the one to lead with
2. `Find files by what's inside them, not their names`
3. `The file search that reads your screenshots`

**Description (~260 chars):**

```
Kensaku indexes your files and reads inside them — PDFs, code, logs, and the
text in your screenshots. Find by meaning ("the PDF about database
indexing"), parse "screenshots from last week". 100% local, free, MIT.
Your AI agent can search it too (MCP).
```

**Topics to tag:** Productivity, Developer Tools, Open Source, Privacy, AI

**Maker first comment (post the second the listing goes live):**

```
Hey PH 👋 Maker here.

I built Kensaku because I could never find my own screenshots. I knew the
error said "CUDA out of memory", I knew I'd screenshotted it, and I had no
idea it was called "Screenshot 2026-07-14 094231.png".

So it reads the pixels. OCR, built in. Plus meaning-based search, natural
language time filters, and MCP so your AI agent can search the same index
read-only. Nothing leaves the machine. MIT.

Honest limits: unsigned builds, visual search still a beta download, and
Windows is the most-tested platform. macOS/Linux come from CI and need
real-world testing — that's the feedback I'm here for. What would you
search for first?
```

**Rules that matter on PH:** launch at 00:01 PT (Tuesday–Thursday), the maker
comment must be the first comment, and reply to every comment in the first
24h. A short demo GIF in the gallery outperforms any screenshot.

---

## 6. Indie Hackers

**Title:** `I'm launching Kensaku — local file search that reads your screenshots`

**Post:**

```
I'm launching Kensaku (検索 = search) today.

I built it because I kept losing screenshots of errors I could describe but
not name. It indexes your files and reads inside them — PDFs, code, logs,
and the text in screenshots via OCR. It also finds by meaning and parses
things like "screenshots from last week".

Numbers that matter to me right now: about 12 MB of metadata for 100,000
files, a 14.3 MB Windows setup, filename search in 3–19 ms across that
corpus depending on thread count, and 0 bytes to the cloud. MIT, free.

I'm not charging for it. The goal is to build the launcher people
actually want to use, then figure out the business. macOS and Linux come
from CI and need real-world testing — if you're on either, I'd love a
test report.

What's the first thing you'd search for?
```

---

## 7. LinkedIn

**Post (professional, first-person, no emoji spam):**

```
I shipped Kensaku today — a local file search that reads inside your files.

The itch: I could never find my own screenshots. I knew the error said
"CUDA out of memory", I knew I'd captured it, and I had no idea what I'd
named the file.

So I built one that reads the pixels. OCR is built in, along with full
text search across PDFs/DOCX/code, meaning-based matches, and natural
language time filters ("screenshots from last week"). Everything stays on
your machine — no account, no telemetry, no cloud.

Stack: Rust, Tauri 2, SQLite + Tantivy, a static embedding table for
semantic search. MIT licensed, free. Windows/macOS/Linux.

If you try it and it breaks, tell me — that's the most useful thing
right now: github.com/yjid/kensaku
```

---

## 8. Mastodon / Bluesky (short post + hashtags)

```
You can't Ctrl+F a screenshot. So I built Kensaku — local file search
that reads inside your files (PDFs, code, logs, and the text in your
screenshots). Meaning search + natural language time. 100% local, free,
MIT. github.com/yjid/kensaku

#opensource #rust #tauri #privacy #productivity
```

---

## 9. Dev.to / Hashnode (long-form, SEO)

**Title:** `How I built a local file search that reads inside your screenshots (Rust + Tauri)`

**Outline (write it as a build log, not a press release):**

1. The itch — the screenshot problem, with the concrete example
2. Why "filename search" tools leave money on the table
3. The architecture: SQLite WAL as durable truth, in-memory snapshot, Tantivy
   for text, a static embedding table for meaning, PP-OCRv6 for pixels with
   Tesseract as the fallback
4. The hard parts: webview destruction on hide, OCR budgeting, keeping memory
   flat on large folders, the theme bug that wouldn't die
5. Benchmarks with methodology (link docs/BENCHMARKS.md)
6. What's next — link the roadmap Discussions
7. CTA: try it, star it, tell me what breaks

---

## 10. Demo assets (do these before posting anything)

This is the highest-ROI 30 minutes of the whole launch. Posts with a short
demo clip outperform text-only posts by a wide margin on every platform.

- [ ] **10–15s screen recording, no audio:** Alt+Space → type "cuda out of
      memory" → results appear → Enter opens the screenshot. One take.
      Save as `demo.gif` (≤8 MB for GitHub) and an MP4 for PH/X.
- [ ] **Static hero screenshot:** home screen with real recents + a search
      in progress. Refresh `assets/search-at-full-speed.png` if it predates
      the current UI (the bundled one is from before 1.0.16).
- [ ] **Social preview:** upload `assets/social-preview.png` in repo
      Settings → Social preview (GitHub doesn't allow this via API).
- [ ] **One-liner for bios:** "I build Kensaku — local search that reads
      inside your files. github.com/yjid/kensaku"

---

## 11. The universal 2-line pitch (use anywhere, verbatim)

> Kensaku is local file search that reads inside your files — documents,
> code, and even the text in your screenshots. 100% local, free, MIT.
> github.com/yjid/kensaku

---

## 12. Order of operations (one day, ~3 hours)

| Time (ET) | Action |
|---|---|
| 07:30 | Final read-through of all copy in this file. Fix anything stale. |
| 08:00 | Post to r/SideProject (Title 1, hero image attached). Drop the per-sub first comment. |
| 08:15 | Post to r/opensource with the stack line. |
| 09:00 | Submit Show HN. Post the author deep-dive as the first comment immediately. |
| 09:15 | LinkedIn post. |
| 10:00 | Mastodon / Bluesky post. |
| 12:00 | X main post + follow-up reply 4–6h later. |
| 14:00 | Product Hunt (if launching there — 00:01 PT start, so PH is a separate day). |
| All day | Reply to every comment. Ship one small fix mentioned in a comment and say so. |
