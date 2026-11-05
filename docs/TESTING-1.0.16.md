# Manual test checklist: 1.0.16

Defects that CI cannot cover, because they need a real window manager, a real IME, or a
real filesystem. Each item names the issue or PR behind it and what the old behaviour was,
so a pass means "this no longer happens" rather than "this looks fine".

Run on the platform in brackets. Items marked (all) apply everywhere.

## Launcher dismissal (#95, PR #98) [Linux especially]

The window used to hide on any blur, so a native context menu or an OS-rendered popup
made it disappear under the cursor.

1. Search, then right-click empty space in the results area. **Palette stays.**
   Previously: hid immediately.
2. Right-click a result. Kensaku's own menu opens; palette stays.
3. Open the sort control, pick "Newest". Palette stays. Keyboard pass on the same control:
   focus it, Enter to open, arrows to move, Enter to choose, Escape to cancel.
4. Click another application. **Palette hides.** This is the behaviour that must not
   regress: the guards added for 1-3 must not keep the launcher on screen.
5. Escape with an empty query hides it; Escape with a query clears the query first.

## Input [all]

6. With a Chinese or Japanese IME: type, then Enter to commit the candidate.
   **No file opens.** Arrow keys page the candidate window and do not move the result
   selection. Previously: the committing Enter also opened the selected result.
7. Type a query and press Enter immediately, before results settle. It must not open a file
   from the previous query.
8. From the search field, Tab moves into the results; Shift+Tab leaves to the toolbar; once
   focus is on a control, Tab behaves natively. Previously: Tab was swallowed everywhere.

## Index [all]

9. Rename a folder containing files, then search for a phrase from inside one of them.
   Found under the new path; the old path does not appear. Previously: the old documents
   stayed in the content index forever.
10. Add a distinctive word to a file under an ignored path (`dist/`, `.venv/`, a
    `.cursorignore` entry) and save it. It must not become searchable.
11. Create an unreadable folder (`chmod 000`) inside a watched root, trigger a re-scan, and
    check that files elsewhere are still indexed and the locked folder's files did not
    vanish from results. Previously: the whole subtree was deleted from the index.
    (Skip on Windows, where the permission model differs.)
12. Add an extension to the excluded list in Settings, save, let it crawl. Files with that
    extension leave the results. Previously: exclusions only applied to files indexed after
    the change.

## OCR [all with Tesseract data installed]

13. Settings → Search → OCR: select a non-English language. Index an image containing text
    in it, then search for a word from it. Previously: every image was read as English
    because `-l` was never passed.
14. Index a scanned PDF of 20+ pages while watching memory. Usage should stay near one
    page's worth, not climb by gigabytes.
15. Health check on a clean profile: no warnings at all. Previously: three, one of which
    was the app's own startup log.

## Shell [Windows for 16-17, all for 18-19]

16. Settings → About → check for updates. With no update available it must return promptly
    and not leave the download state stuck on "busy".
17. From the **portable** zip: the update action must refuse and tell you to replace the
    folder. Previously: it installed a second copy into Program Files while the portable
    folder kept running the old binary.
18. Settings → About → Reset all, confirm. The app must come back up. Previously: it could
    close and never reopen, because the replacement process lost the instance-lock race.
19. Launch the app twice. The second launch summons the existing window; there are not two
    tray icons or two windows. Previously: two GUIs could run when the daemon was present.

## What CI already covers

- `cargo test --all-targets` and `cargo clippy -D warnings` on Linux.
- `tsc --noEmit`, `vitest run`, and the production build.
- PP-OCR model tests on macOS, Linux and Windows.
- Release packaging: the portable zip is asserted to contain `onnxruntime.dll`, and the
  release workflow fails if it is missing.

## Verified against the v1.0.16-rc1 artifacts

- `kensaku-portable-1.0.16-x64.zip`: `onnxruntime.dll` present, PE32+ x86-64, 17 MB.
- `kensaku-portable-1.0.16-arm64.zip`: `onnxruntime.dll` present, PE32+ ARM64, 17 MB.
- Zip checksum matches the published `SHA256SUMS`.
