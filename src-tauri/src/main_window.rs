//! The one place the main search window is created, shown, and hidden.
//!
//! Hiding destroys the webview, so the WebKit processes exit and the hidden
//! app keeps only the Rust side (indexer, watcher, daemon client). Showing
//! creates the window again from the same settings. The last query is kept in
//! Rust so the search comes back where the user left it.

use std::sync::Mutex;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

pub const LABEL: &str = "main";

static LAST_QUERY: Mutex<String> = Mutex::new(String::new());

pub fn remember_query(query: &str) {
    if let Ok(mut q) = LAST_QUERY.lock() {
        q.clear();
        q.push_str(query);
    }
}

pub fn last_query() -> String {
    LAST_QUERY.lock().map(|q| q.clone()).unwrap_or_default()
}

fn existing(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(LABEL)
}

fn create(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    // In "app" mode the window is a normal application window: it shows in
    // the dock/taskbar and is not pinned above other windows. In "launcher"
    // mode (default) it stays off the taskbar and floats like a palette.
    let app_mode = crate::config::KensakuConfig::load().app_mode;
    let launcher = app_mode != "app";
    let window = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()))
        .title("Kensaku")
        .inner_size(920.0, 640.0)
        .min_inner_size(720.0, 520.0)
        .decorations(false)
        .transparent(true)
        .always_on_top(launcher)
        .skip_taskbar(launcher)
        .visible_on_all_workspaces(launcher)
        .center()
        .shadow(true)
        .resizable(true)
        .visible(false)
        .build()?;

    // Launcher mode: close hides the window and the app keeps running in the
    // tray. App mode: close exits the application like a normal desktop app.
    // Read the mode inside the handler so a Settings change applies without
    // recreating the window.
    let app_handle = app.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            let app_mode = crate::config::KensakuConfig::load().app_mode;
            if app_mode == "app" {
                app_handle.exit(0);
                return;
            }
            api.prevent_close();
            hide(&app_handle);
        }
    });
    Ok(window)
}

/// Applies the parts of `app_mode` that can change on a live window. Window
/// decorations are fixed at creation; everything users notice (taskbar,
/// floating, other workspaces) updates immediately.
pub fn apply_mode(window: &WebviewWindow, app_mode: &str) {
    let launcher = app_mode != "app";
    let _ = window.set_skip_taskbar(launcher);
    let _ = window.set_always_on_top(launcher);
    let _ = window.set_visible_on_all_workspaces(launcher);
}

/// Tells the user why the app has no window, even when release builds run
/// without a console. The log path is included so a bug report is actionable.
fn report_creation_failure(app: &AppHandle, error: &str) {
    crate::crashlog::record_error("main window creation", error);
    eprintln!("[Kensaku] could not create the window: {error}");

    use tauri_plugin_dialog::DialogExt;
    let hint = if cfg!(windows) {
        "\n\nThis usually means the Microsoft Edge WebView2 runtime is missing or broken. Install it from Microsoft, then try again."
    } else {
        ""
    };
    let _ = app
        .dialog()
        .message(format!(
            "Kensaku could not open its window.\n\n{error}{hint}\n\nDetails were written to:\n{}",
            crate::crashlog::path().display()
        ))
        .title("Kensaku could not start")
        .blocking_show();
}

/// Shows the search window, creating it if it was hidden (destroyed).
pub fn show(app: &AppHandle) {
    let window = match existing(app) {
        Some(w) => w,
        None => match create(app) {
            Ok(w) => w,
            Err(e) => {
                report_creation_failure(app, &e.to_string());
                return;
            }
        },
    };
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
}

/// Hides the search window by destroying it, then returns freed memory.
pub fn hide(app: &AppHandle) {
    if let Some(window) = existing(app) {
        // Hide first, then destroy: on Windows, tearing down a visible
        // transparent + always-on-top window while WebView2 crashes out (the
        // process list is killed below) can briefly leave a composited ghost
        // frame on screen. Once hidden, nothing of the window is visible.
        let _ = window.hide();
        let _ = window.destroy();
    }
    // WebKit/WebView helpers otherwise stay alive after the window is
    // destroyed, keeping tens to hundreds of MB resident. Kill them and trim
    // the allocator so the hidden app keeps only the Rust side.
    crate::util::memory::kill_webview_processes();
    crate::util::memory::release_memory();
}

/// Shows the window if it is hidden, hides it if it is shown.
pub fn toggle(app: &AppHandle) {
    if existing(app).is_some_and(|w| w.is_visible().unwrap_or(false)) {
        hide(app);
    } else {
        show(app);
    }
}

/// Returns freed heap pages to the OS. glibc otherwise keeps them, so the
/// process can report more memory than it still uses.
pub fn release_free_memory() {
    #[cfg(target_os = "linux")]
    unsafe {
        libc::malloc_trim(0);
    }
}
