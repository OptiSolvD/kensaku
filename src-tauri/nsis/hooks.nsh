; Kensaku NSIS installer hooks.
;
; The stock Tauri uninstaller removes the install directory and shortcuts but
; knows nothing about the data Kensaku writes at runtime. These hooks make
; uninstalling a full cleanup:
;
;   - the app keeps running in the background when its window is hidden, so it
;     has to be stopped before its files are deleted;
;   - the runtime data lives in %APPDATA%\kensaku (config, index, metadata DB,
;     thumbnails, OCR packs, model packs, agent HTTP token);
;   - WebView2 keeps its own profile per Tauri identifier under
;     %LOCALAPPDATA%\com.gochi.kensaku;
;   - the auto-launch crate registers HKCU\...\Run with the display name
;     "Kensaku".

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Stopping Kensaku..."
  nsExec::Exec 'taskkill /IM "kensaku.exe" /F'
  Pop $R9
  nsExec::Exec 'taskkill /IM "kensaku-daemon.exe" /F'
  Pop $R9
  nsExec::Exec 'taskkill /IM "kensaku-quick.exe" /F'
  Pop $R9
  Sleep 500
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  DetailPrint "Removing Kensaku data..."
  ; Runtime data: config, index, metadata.db, thumbs, tessdata, ocr packs,
  ; model packs, agent HTTP token, logs, crawl report.
  RMDir /r "$APPDATA\kensaku"
  ; A portable install keeps its data beside the exe. That folder is the
  ; user's own, so uninstalling the installed copy must not touch it.
  ; Tauri identifier-scoped directories, used for the WebView2 profile and any
  ; $APPCONFIG-pathed artifacts. Removed whichever of them exist.
  RMDir /r "$APPDATA\com.gochi.kensaku"
  RMDir /r "$LOCALAPPDATA\com.gochi.kensaku"
  RMDir /r "$LOCALAPPDATA\kensaku"
  ; Installers and partial downloads staged by the in-app updater. These live
  ; in the temp directory and are not covered by anything above: a few upgrades
  ; leave hundreds of megabytes here.
  RMDir /r "$TEMP\kensaku-updates"
  ; Autostart entry written by the auto-launch crate, under both names it has
  ; used so an older install is cleaned up too.
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Kensaku"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "kensaku"
  DetailPrint "Kensaku data removed."
!macroend
