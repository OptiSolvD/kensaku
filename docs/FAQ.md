# FAQ

**Is anything sent to the cloud?**
No. There is no telemetry and no account, and search queries or file contents
never leave your machine. Kensaku makes two kinds of outbound requests: a GitHub
release check about once every 12 hours, and model/OCR pack downloads. The
default text model downloads automatically after setup, and OCR data for the
enabled engine may download automatically while OCR is on; the optional Indra,
smart-embedding, and visual packs are user-initiated.

**Why is the first launch slow?**
It builds the index. The guided first run completes the initial scan before
opening the main window, showing live counts and ending with a Done summary;
later launches stay search-first and instant.

**What happens during setup?**
The guided flow covers folders, exclusions, preferences (theme, interface
language, window mode, shortcut), optional OCR/model downloads, and the first
scan, ending with a Done summary before the main window opens. Downloads are
acquired there; Settings shows installed status and **About → Run setup again**.

**Why does search return nothing for a word I know is in a file?**
Check: is the folder watched? Is the extension in the excluded list? Is the file
over the content cap (extraction up to 100 MB, 4 MB of text indexed)? Is it
binary? Run Settings → Health → Doctor. The Search syntax card shows filters
that may be active.

**Does it index hidden files?**
No by default. `skip_hidden` is on; you can opt in via config.

**Does it index PDFs and Office documents?**
Yes — PDF, DOCX/PPTX/XLSX, EPUB/ODT, RTF. Scanned PDFs need OCR; encrypted PDFs are
flagged, not indexed.

**Does it search images?**
By filename, and by OCR text: PP-OCRv6 by default (runs inside Kensaku on a
bundled ONNX Runtime), or the optional higher-quality Indra engine (~27 MB pack,
deskew and multi-script recognition), or Tesseract where neither can run (Intel
Macs) or if you pick it. Scanned PDFs are OCR'd too, one page at a time. Results
also group perceptual near-duplicates.

Natural-language visual search ("red cap", "system design diagram"),
similar-image lookup, and an optional local "this is me" identity are **not**
part of the 1.0 search path. The Beta section can download a CLIP pack, but no
production visual encoder is wired in, so `similar_images` self-hides.

**Can I just type "latest resume" or use `/` and `@`?**
Yes. A deterministic, local intent compiler turns recency, size, type, and place
phrases into the normal filters and sort, and shows each inference as a removable
chip. `/` opens the command menu and `@` inserts a directory scope. The explicit
`key:value` grammar (for example `type:pdf` and `modified:<7d`) always works too.

**OCR is on but no image text is indexed.**
Open Settings → Search → OCR: it shows which engine is actually reading
images and, if PP-OCRv6 cannot run, why (pack still downloading, Intel Mac).
"Verified" means the selected engine read a test image correctly. With
Tesseract, make sure language data is installed (`tesseract-ocr-eng`) or
download it in setup.
Existing images are re-scanned. Doctor repeats the check. The setup pack button
shows download state, success, or the backend error.

**Can it search across multiple drives?**
Yes. Add each root in Settings → Folders. Unplugged drives are preserved rather
than pruned.

**How much memory does it use?**
The metadata layer keeps 128 bytes per `FileEntry` (12.2 MB measured for 100,201
entries), plus a path index and the ~7.3 MiB i8 static embedding table when the
semantic pack is installed (the f32 switch is ~30 MB). The crawl is bounded:
extracted text is written through a small queue instead of accumulating in
memory, perceptual image hashing runs in a separate background pass that decodes
one image at a time, and the Tantivy writer arena is 16 MB. On the development
machine the indexer process peaked at roughly 72–73 MB for both text-only and
mixed image/text corpora (before adding the embedding table). Total app memory
is dominated by the OS webview that hosts the desktop window.

**Does it conflict with Spotlight / Everything / Recoll?**
No. It builds its own index and does not read or write theirs.

**Can two instances run?**
No. A GUI lock makes a second launch hand its request to the running app and
exit; the first window is shown (or toggled) instead of a second copy starting.

**Is the app signed?**
Not yet. macOS needs right-click → Open (or `xattr -dr com.apple.quarantine`),
Windows shows SmartScreen. See [`INSTALL.md`](INSTALL.md#the-builds-are-unsigned).

**Does it update itself?**
Yes. Background checks run about every 12 hours (`auto_update`, on by default).
Settings → About shows release notes, asset size, and download progress; the
download is size-checked and verified against the release `SHA256SUMS` before
the installer runs. Portable installs are told to replace the folder instead.
You can turn checks off in Settings → About.

**Where is my data?**
`~/.config/kensaku/` on Linux, `~/Library/Application Support/kensaku/` on macOS,
`%APPDATA%\kensaku\` on Windows. Settings → About → Reset everything wipes the
index, database, and settings; downloaded model and OCR packs are kept unless you
tick the box to remove them.

**Can I use it from an AI agent?**
Yes — that's the point of `kensaku-cli` and the MCP server. See
[`AGENTS.md`](AGENTS.md). The app itself contains no LLM.

**Why not bundle an LLM / "ask my files"?**
A local LLM is a multi-gigabyte dependency and a different product. Kensaku is
the fast, trustworthy index: plain-language intent is compiled deterministically
on-device, and there is no chat panel and no `/ask` command. Agents bring their
own model through MCP and get the same read-only index. File contents never leave
the machine during search.

**How do I contribute?**
See [`CONTRIBUTING.md`](CONTRIBUTING.md). Tests and clippy must pass.

**Why is the app bigger than the early 18 MB plan?**
The early plan assumed a much smaller dependency set and no bundled ONNX Runtime.
The 1.0.16 Windows x64 downloads are a 14.3 MB NSIS setup and a 20.1 MB MSI;
the installed app binary is about 36 MB. The base search path uses the compact
i8 static table rather than
a neural graph at query time; ONNX is present for OCR, and the smart embedding
tier remains optional.
