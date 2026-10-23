// Prevents additional console window on Windows in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api_client;
mod device_id;
mod config;
mod llm_client;
mod updater;

use api_client::ApiClient;
use llm_client::{LlmClient, LlmProvider};
use enigo::{Enigo, Key, Keyboard, Settings};
use std::sync::Mutex;
use std::thread;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    pub id: String,
    pub shortcut: String,
    pub prompt: String,
}

// NOTE: Mutex .unwrap() calls throughout this file are intentional.
// The global shortcut handler wraps all shortcut logic in std::panic::catch_unwind
// (see the with_handler closure in setup), so if a mutex is poisoned by a panic,
// the catch_unwind boundary prevents crash propagation and resets is_processing.
// The only mutex that uses unwrap_or_else(|e| e.into_inner()) is is_processing
// itself, inside that catch_unwind recovery path.
struct AppState {
    api_client: Mutex<Option<ApiClient>>,
    llm_client: Mutex<Option<LlmClient>>,
    device_id: Mutex<String>,
    pending_translation: Mutex<Option<String>>,
    original_text: Mutex<Option<String>>,
    last_prompt: Mutex<Option<String>>,
    previous_app: Mutex<Option<String>>,
    show_preview: Mutex<bool>,
    is_processing: Mutex<bool>,
    shortcuts: Mutex<HashMap<String, ShortcutConfig>>,
    quick_select_enabled: Mutex<bool>,
    quick_select_open: Mutex<bool>,
    quick_select_shortcut: Mutex<String>,
    stream_response: Mutex<bool>,
    // Progress indicator state (polled by the progress window)
    progress_state: Mutex<ProgressState>,
}

#[derive(Debug, Clone, Serialize)]
struct ProgressState {
    active: bool,
    input_chars: usize,
    output_chars: usize,
    done: bool,
    error: Option<String>,
    start_ms: u64,
}

// --- macOS helpers using osascript (reliable keystroke simulation) ---

#[cfg(target_os = "macos")]
fn macos_keystroke(app: &AppHandle, key: &str) {
    use tauri_plugin_shell::ShellExt;
    let script = format!(
        r#"tell application "System Events" to keystroke "{}" using command down"#,
        key
    );
    let output = app.shell().command("osascript").args(["-e", &script]).output();
    if let Ok(result) = tauri::async_runtime::block_on(output) {
        if !result.stderr.is_empty() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            log::error!("[rewrit] keystroke error: {}", stderr);
            if is_accessibility_error(&stderr) {
                report_accessibility_missing(app);
            }
        }
    }
}

/// osascript errors raised when rewrit is not allowed to control the keyboard
#[cfg(target_os = "macos")]
fn is_accessibility_error(stderr: &str) -> bool {
    ["(1002)", "(-1719)", "(-25211)"].iter().any(|code| stderr.contains(code))
}

#[cfg(target_os = "macos")]
static ACCESSIBILITY_DIALOG_OPEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Explain the missing Accessibility permission (macOS drops it after each unsigned update)
#[cfg(target_os = "macos")]
fn report_accessibility_missing(app: &AppHandle) {
    use std::sync::atomic::Ordering;
    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
    if ACCESSIBILITY_DIALOG_OPEN.swap(true, Ordering::SeqCst) {
        return;
    }
    app.dialog()
        .message(
            "rewrit is not allowed to copy and paste for you.\n\n\
             In System Settings › Privacy & Security › Accessibility, remove rewrit with \"–\", \
             add it again and turn it on, then restart rewrit.\n\n\
             macOS often drops this permission after an update.",
        )
        .title("Accessibility permission needed")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom("Open Accessibility Settings".into(), "Later".into()))
        .show(|open| {
            ACCESSIBILITY_DIALOG_OPEN.store(false, Ordering::SeqCst);
            if open {
                let _ = open_accessibility_settings();
            }
        });
}

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

/// Whether rewrit may control the keyboard; None on systems without this permission
#[tauri::command]
fn accessibility_granted() -> Option<bool> {
    #[cfg(target_os = "macos")]
    return Some(unsafe { AXIsProcessTrusted() });
    #[cfg(not(target_os = "macos"))]
    None
}

/// Open System Settings › Privacy & Security › Accessibility
#[tauri::command]
fn open_accessibility_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn()
            .map_err(|e| format!("Failed to open System Settings: {}", e))?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    Err("Accessibility settings only exist on macOS".to_string())
}

#[cfg(target_os = "macos")]
fn refocus_previous_app(app: &AppHandle, app_name: &str) {
    use tauri_plugin_shell::ShellExt;
    log::info!("[rewrit] refocusing: {}", app_name);
    // Sanitize app_name to prevent AppleScript injection
    let safe_name = app_name.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        r#"tell application "System Events" to set frontmost of process "{}" to true"#,
        safe_name
    );
    let output = app.shell().command("osascript").args(["-e", &script]).output();
    if let Ok(result) = tauri::async_runtime::block_on(output) {
        if !result.stderr.is_empty() {
            log::error!("[rewrit] refocus error: {}", String::from_utf8_lossy(&result.stderr));
        }
    }
    thread::sleep(config::DELAY_CONFIG.after_refocus);
    log::info!("[rewrit] refocus done");
}

// --- Non-macOS helpers using Enigo ---

#[cfg(not(target_os = "macos"))]
fn simulate_key_combo(key: char) {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    let delay = config::DELAY_CONFIG.keyboard_short_delay;
    let _ = enigo.key(Key::Control, enigo::Direction::Press);
    thread::sleep(delay);
    let _ = enigo.key(Key::Unicode(key), enigo::Direction::Click);
    thread::sleep(delay);
    let _ = enigo.key(Key::Control, enigo::Direction::Release);
}

// --- Tauri Commands ---

#[tauri::command]
async fn get_device_id(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.device_id.lock().unwrap().clone())
}

#[tauri::command]
async fn fetch_quota(state: State<'_, AppState>) -> Result<api_client::QuotaInfo, String> {
    let client = state.api_client.lock().unwrap().as_ref().cloned();
    match client {
        Some(client) => client.fetch_quota().await.map(|r| r.quota),
        None => Err("API client not initialized".to_string()),
    }
}

#[tauri::command]
async fn reset_quota(state: State<'_, AppState>) -> Result<api_client::QuotaInfo, String> {
    let client = state.api_client.lock().unwrap().as_ref().cloned();
    match client {
        Some(client) => client.reset_quota().await,
        None => Err("API client not initialized".to_string()),
    }
}

#[tauri::command]
fn execute_paste(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let (translated_text, original_text, previous_app) = {
        let mut pending = state.pending_translation.lock().unwrap();
        let original = state.original_text.lock().unwrap();
        let prev_app = state.previous_app.lock().unwrap();
        (pending.take(), original.clone(), prev_app.clone())
    };

    let text = translated_text.ok_or("No translated text available")?;
    log::info!("[rewrit] pasting {} chars", text.len());

    // Write to clipboard before hiding window
    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard().write_text(text.clone())
        .map_err(|e| format!("Failed to write to clipboard: {}", e))?;

    // Hide preview window
    if let Some(w) = app.get_webview_window("preview") {
        let _ = w.hide();
    }
    thread::sleep(config::DELAY_CONFIG.window_hide);

    // Refocus previous app and paste
    #[cfg(target_os = "macos")]
    if let Some(ref name) = previous_app {
        refocus_previous_app(&app, name);
    }

    #[cfg(target_os = "macos")]
    macos_keystroke(&app, "v");
    #[cfg(not(target_os = "macos"))]
    simulate_key_combo('v');

    // Restore original clipboard in background
    let app_clone = app.clone();
    let restore_delay = config::DELAY_CONFIG.restore_clipboard;
    thread::spawn(move || {
        thread::sleep(restore_delay);
        if let Some(orig) = original_text {
            use tauri_plugin_clipboard_manager::ClipboardExt;
            let _ = app_clone.clipboard().write_text(orig);
        }
    });

    Ok(())
}

#[tauri::command]
fn cancel_preview(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    {
        let mut pending = state.pending_translation.lock().unwrap();
        *pending = None;
    }

    if let Some(w) = app.get_webview_window("preview") {
        let _ = w.hide();
    }

    #[cfg(target_os = "macos")]
    {
        let previous_app = state.previous_app.lock().unwrap();
        if let Some(ref name) = *previous_app {
            refocus_previous_app(&app, name);
        }
    }

    Ok(())
}

#[tauri::command]
fn retry_translation(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let original_text = state.original_text.lock().unwrap().clone();
    let text = original_text.ok_or("No original text available")?;
    let last_prompt = state.last_prompt.lock().unwrap().clone();
    let prompt = last_prompt.ok_or("No prompt available")?;

    tauri::async_runtime::spawn(async move {
        let state: State<AppState> = app.state();
        let llm = state.llm_client.lock().unwrap().as_ref().cloned();
        let use_stream = *state.stream_response.lock().unwrap();

        if let Some(llm) = llm {
            match run_llm_preview(llm, text.clone(), prompt, use_stream, app.clone()).await {
                Ok(response) if response.success => {
                    if let Some(translated) = response.translated_text {
                        *state.pending_translation.lock().unwrap() = Some(translated.clone());
                        if let Some(window) = app.get_webview_window("preview") {
                            let _ = window.emit("show-preview", serde_json::json!({
                                "original": text,
                                "translated": translated
                            }));
                        }
                    }
                }
                Ok(response) => {
                    let err = response.error.unwrap_or_else(|| "Translation failed".to_string());
                    report_llm_error(&app, &err);
                }
                Err(e) => report_llm_error(&app, &e),
            }
        } else {
            report_llm_error(&app, &missing_key_message());
        }
    });

    Ok(())
}

#[tauri::command]
fn get_show_preview(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.show_preview.lock().unwrap())
}

#[tauri::command]
fn set_show_preview(state: State<'_, AppState>, show: bool) -> Result<(), String> {
    *state.show_preview.lock().unwrap() = show;
    let _ = device_id::set_show_preview(show);
    Ok(())
}

#[tauri::command]
fn get_progress_state(state: State<'_, AppState>) -> Result<ProgressState, String> {
    Ok(state.progress_state.lock().unwrap().clone())
}

#[tauri::command]
fn get_stream_response(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.stream_response.lock().unwrap())
}

#[tauri::command]
fn set_stream_response(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    *state.stream_response.lock().unwrap() = enabled;
    device_id::set_stream_response(enabled)?;
    log::info!("[rewrit] stream response: {}", enabled);
    Ok(())
}

#[tauri::command]
fn get_quick_select_enabled(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.quick_select_enabled.lock().unwrap())
}

#[tauri::command]
fn set_quick_select_enabled(state: State<'_, AppState>, app: AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

    *state.quick_select_enabled.lock().unwrap() = enabled;
    let _ = device_id::set_quick_select_enabled(enabled);

    let qs_shortcut = state.quick_select_shortcut.lock().unwrap().clone();
    let shortcut: Shortcut = qs_shortcut.parse()
        .map_err(|e| format!("Invalid shortcut: {}", e))?;

    if enabled {
        let _ = app.global_shortcut().register(shortcut);
    } else {
        let _ = app.global_shortcut().unregister(shortcut);
    }

    log::info!("[rewrit] quick select enabled: {}", enabled);
    Ok(())
}

#[tauri::command]
fn get_quick_select_shortcut(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.quick_select_shortcut.lock().unwrap().clone())
}

#[tauri::command]
fn set_quick_select_shortcut(state: State<'_, AppState>, app: AppHandle, shortcut: String) -> Result<(), String> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

    let new_shortcut = shortcut.trim().to_string();

    // Validate
    let new_parsed: Shortcut = new_shortcut.parse()
        .map_err(|e| format!("Invalid shortcut format: {}", e))?;

    // Unregister old
    let old_shortcut = state.quick_select_shortcut.lock().unwrap().clone();
    if let Ok(old_parsed) = old_shortcut.parse::<Shortcut>() {
        let _ = app.global_shortcut().unregister(old_parsed);
    }

    // Register new (if enabled)
    let enabled = *state.quick_select_enabled.lock().unwrap();
    if enabled {
        app.global_shortcut().register(new_parsed)
            .map_err(|e| format!("Failed to register shortcut: {}", e))?;
    }

    // Persist and update state
    device_id::set_quick_select_shortcut(&new_shortcut)?;
    *state.quick_select_shortcut.lock().unwrap() = new_shortcut.clone();

    log::info!("[rewrit] Quick Select shortcut changed to: {}", new_shortcut);
    Ok(())
}

#[tauri::command]
fn cancel_quick_select(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    *state.quick_select_open.lock().unwrap() = false;

    if let Some(w) = app.get_webview_window("quickselect") {
        let _ = w.hide();
    }

    #[cfg(target_os = "macos")]
    {
        let previous_app = state.previous_app.lock().unwrap();
        if let Some(ref name) = *previous_app {
            refocus_previous_app(&app, name);
        }
    }

    Ok(())
}

#[tauri::command]
fn execute_quick_select(
    language: String,
    tone: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    // Close the quick select window
    *state.quick_select_open.lock().unwrap() = false;
    if let Some(w) = app.get_webview_window("quickselect") {
        let _ = w.hide();
    }

    let original_text = state.original_text.lock().unwrap().clone();
    let text = original_text.ok_or("No text captured")?;

    let prompt = build_quick_select_prompt(&language, &tone);
    *state.last_prompt.lock().unwrap() = Some(prompt.clone());

    let show_preview = *state.show_preview.lock().unwrap();
    log::info!(
        "[rewrit] quick_select: lang={}, tone={}, preview={}",
        language, tone, show_preview
    );

    if show_preview {
        // Open preview window and run translation
        let preview_window = if let Some(w) = app.get_webview_window("preview") {
            w
        } else {
            use tauri::WebviewWindowBuilder;
            match WebviewWindowBuilder::new(
                &app,
                "preview",
                tauri::WebviewUrl::App("src/preview.html".into()),
            )
            .title("rewrit - Preview")
            .inner_size(650.0, 450.0)
            .center()
            .decorations(false)
            .always_on_top(true)
            .resizable(false)
            .skip_taskbar(true)
            .build()
            {
                Ok(w) => w,
                Err(_) => {
                    *state.is_processing.lock().unwrap() = false;
                    return Err("Failed to create preview window".to_string());
                }
            }
        };

        let _ = preview_window.show();
        let _ = preview_window.set_focus();

        #[cfg(target_os = "macos")]
        {
            use tauri_plugin_shell::ShellExt;
            let script =
                r#"tell application "System Events" to set frontmost of process "rewrit" to true"#;
            let output = app.shell().command("osascript").args(["-e", script]).output();
            let _ = tauri::async_runtime::block_on(output);
        }

        let _ = preview_window.emit("translation-started", ());

        let app_clone = app.clone();
        let text_clone = text.clone();
        tauri::async_runtime::spawn(async move {
            let state: State<AppState> = app_clone.state();
            let llm = state.llm_client.lock().unwrap().as_ref().cloned();
            let use_stream = *state.stream_response.lock().unwrap();

            if let Some(llm) = llm {
                match run_llm_preview(llm, text_clone.clone(), prompt, use_stream, app_clone.clone()).await {
                    Ok(response) if response.success => {
                        if let Some(translated) = response.translated_text {
                            *state.pending_translation.lock().unwrap() = Some(translated.clone());
                            if let Some(window) = app_clone.get_webview_window("preview") {
                                let _ = window.emit("show-preview", serde_json::json!({
                                    "original": text_clone,
                                    "translated": translated
                                }));
                            }
                            report_usage_to_backend(&state, &app_clone).await;
                        }
                    }
                    Ok(response) => {
                        let err = response.error.unwrap_or_else(|| "Translation failed".to_string());
                        report_llm_error(&app_clone, &err);
                    }
                    Err(e) => report_llm_error(&app_clone, &e),
                }
            } else {
                report_llm_error(&app_clone, &missing_key_message());
            }
            *state.is_processing.lock().unwrap() = false;
        });
    } else {
        // Direct paste mode — show progress indicator
        show_progress(&app, text.len());

        let app_clone = app.clone();
        let text_clone = text.clone();
        tauri::async_runtime::spawn(async move {
            let state: State<AppState> = app_clone.state();
            let llm = state.llm_client.lock().unwrap().as_ref().cloned();
            let use_stream = *state.stream_response.lock().unwrap();

            if let Some(llm) = llm {
                log::info!("[rewrit] direct paste: use_stream={}", use_stream);
                let result = if use_stream {
                    let app_for_chunks = app_clone.clone();
                    llm.execute_prompt_stream(text_clone, prompt, move |accumulated| {
                        update_progress_chunks(&app_for_chunks, accumulated.len());
                    }).await
                } else {
                    llm.execute_prompt(text_clone, prompt).await
                };

                match result {
                    Ok(response) if response.success => {
                        if let Some(translated) = response.translated_text {
                            finish_progress(&app_clone, None);

                            use tauri_plugin_clipboard_manager::ClipboardExt;
                            let _ = app_clone.clipboard().write_text(translated);
                            tokio::time::sleep(config::DELAY_CONFIG.clipboard_ready).await;

                            // Refocus previous app first
                            #[cfg(target_os = "macos")]
                            {
                                let prev = state.previous_app.lock().unwrap().clone();
                                if let Some(ref name) = prev {
                                    let app_for_refocus = app_clone.clone();
                                    let name = name.clone();
                                    let _ = tokio::task::spawn_blocking(move || {
                                        refocus_previous_app(&app_for_refocus, &name);
                                    })
                                    .await;
                                }
                            }

                            hide_progress(&app_clone);

                            let app_for_paste = app_clone.clone();
                            let _ = tokio::task::spawn_blocking(move || {
                                #[cfg(target_os = "macos")]
                                macos_keystroke(&app_for_paste, "v");
                                #[cfg(not(target_os = "macos"))]
                                simulate_key_combo('v');
                            })
                            .await;

                            report_usage_to_backend(&state, &app_clone).await;
                        }
                    }
                    Ok(response) => {
                        let err = response.error.unwrap_or_else(|| "Translation failed".to_string());
                        report_llm_error(&app_clone, &err);
                    }
                    Err(e) => report_llm_error(&app_clone, &e),
                }
            } else {
                report_llm_error(&app_clone, &missing_key_message());
            }
            *state.is_processing.lock().unwrap() = false;
        });
    }

    Ok(())
}

fn build_quick_select_prompt(language: &str, tone: &str) -> String {
    let lang_name = match language {
        "fr" => "French",
        "en" => "English",
        "es" => "Spanish",
        _ => "English",
    };

    let tone_desc = match tone {
        "professional" => "with a professional, formal tone",
        "casual" => "with a casual, relaxed tone",
        "concise" => "making it concise and to the point",
        _ => "with a professional, formal tone",
    };

    format!("Rewrite this text in {} {}", lang_name, tone_desc)
}

pub fn handle_quick_select_trigger(app_handle: AppHandle) {
    let state: State<AppState> = app_handle.state();

    // Check if feature is enabled
    if !*state.quick_select_enabled.lock().unwrap() {
        return;
    }

    // Toggle: if already open, close it
    {
        let mut is_open = state.quick_select_open.lock().unwrap();
        if *is_open {
            *is_open = false;
            if let Some(w) = app_handle.get_webview_window("quickselect") {
                let _ = w.hide();
            }
            #[cfg(target_os = "macos")]
            {
                let previous_app = state.previous_app.lock().unwrap().clone();
                if let Some(ref name) = previous_app {
                    refocus_previous_app(&app_handle, name);
                }
            }
            return;
        }
    }

    // Prevent concurrent triggers
    {
        let mut is_processing = state.is_processing.lock().unwrap();
        if *is_processing {
            log::info!("[rewrit] already processing, skipping quick select");
            return;
        }
        *is_processing = true;
    }

    // Release modifier keys
    {
        let mut enigo = Enigo::new(&Settings::default()).unwrap();
        let _ = enigo.key(Key::Meta, enigo::Direction::Release);
        let _ = enigo.key(Key::Shift, enigo::Direction::Release);
        let _ = enigo.key(Key::Control, enigo::Direction::Release);
        thread::sleep(std::time::Duration::from_millis(150));
    }

    // Clear clipboard before copy
    use tauri_plugin_clipboard_manager::ClipboardExt;
    let _ = app_handle.clipboard().write_text("".to_string());

    // Get the active application and copy text (always copy, never cut)
    #[cfg(target_os = "macos")]
    {
        use tauri_plugin_shell::ShellExt;

        let script = r#"tell application "System Events" to get name of first application process whose frontmost is true"#;
        let output = app_handle
            .shell()
            .command("osascript")
            .args(["-e", script])
            .output();
        if let Ok(result) = tauri::async_runtime::block_on(output) {
            if let Ok(name) = String::from_utf8(result.stdout) {
                let name = name.trim().to_string();
                if !name.is_empty() && name != "rewrit" {
                    log::info!("[rewrit] previous app: {}", name);
                    *state.previous_app.lock().unwrap() = Some(name);
                }
            }
        }

        // Always copy (never cut) for quick select
        log::info!("[rewrit] quick select: simulating Cmd+C");
        macos_keystroke(&app_handle, "c");
        thread::sleep(config::DELAY_CONFIG.after_copy);
    }

    #[cfg(not(target_os = "macos"))]
    {
        simulate_key_combo('c');
        thread::sleep(config::DELAY_CONFIG.after_copy);
    }

    // Read clipboard with retries
    let clipboard_text = {
        let mut result = None;
        let retry_delay = config::DELAY_CONFIG.clipboard_retry;
        for _ in 0..8 {
            thread::sleep(retry_delay);
            if let Ok(text) = app_handle.clipboard().read_text() {
                if !text.is_empty() {
                    result = Some(text);
                    break;
                }
            }
        }
        result
    };

    let clipboard_text = match clipboard_text {
        Some(t) => {
            log::info!("[rewrit] quick select clipboard: {} chars", t.len());
            t
        }
        None => {
            log::info!("[rewrit] quick select: no text selected");
            *state.is_processing.lock().unwrap() = false;
            return;
        }
    };

    // Store original text
    *state.original_text.lock().unwrap() = Some(clipboard_text);

    // Open/show QuickSelect window
    let qs_window = if let Some(w) = app_handle.get_webview_window("quickselect") {
        w
    } else {
        use tauri::WebviewWindowBuilder;
        match WebviewWindowBuilder::new(
            &app_handle,
            "quickselect",
            tauri::WebviewUrl::App("src/quickselect.html".into()),
        )
        .title("rewrit - Quick Select")
        .inner_size(340.0, 280.0)
        .center()
        .decorations(false)
        .always_on_top(true)
        .resizable(false)
        .skip_taskbar(true)
        .build()
        {
            Ok(w) => w,
            Err(e) => {
                log::error!("[rewrit] failed to create quickselect window: {}", e);
                *state.is_processing.lock().unwrap() = false;
                return;
            }
        }
    };

    let _ = qs_window.show();
    let _ = qs_window.set_focus();

    #[cfg(target_os = "macos")]
    {
        use tauri_plugin_shell::ShellExt;
        let script =
            r#"tell application "System Events" to set frontmost of process "rewrit" to true"#;
        let output = app_handle
            .shell()
            .command("osascript")
            .args(["-e", script])
            .output();
        let _ = tauri::async_runtime::block_on(output);
    }

    *state.quick_select_open.lock().unwrap() = true;
    let _ = qs_window.emit("quick-select-open", ());

    // Release is_processing - the actual translation will happen when user picks options
    *state.is_processing.lock().unwrap() = false;
}

#[tauri::command]
async fn get_shortcuts(state: State<'_, AppState>) -> Result<Vec<ShortcutConfig>, String> {
    Ok(state.shortcuts.lock().unwrap().values().cloned().collect())
}

#[tauri::command]
async fn add_shortcut(
    shortcut: String,
    prompt: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ShortcutConfig, String> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

    if prompt.is_empty() {
        return Err("Prompt cannot be empty".to_string());
    }
    if prompt.len() > 500 {
        return Err("Prompt too long (max 500 characters)".to_string());
    }

    let id = format!("shortcut_{}", &uuid::Uuid::new_v4().to_string()[..8]);

    let parsed = shortcut.parse::<Shortcut>()
        .map_err(|e| format!("Invalid shortcut format: {}", e))?;

    app_handle.global_shortcut().register(parsed)
        .map_err(|e| format!("Failed to register shortcut: {}", e))?;

    let config = ShortcutConfig { id: id.clone(), shortcut, prompt };
    let mut shortcuts = state.shortcuts.lock().unwrap();
    shortcuts.insert(id, config.clone());
    persist_shortcuts(&shortcuts);
    Ok(config)
}

#[tauri::command]
async fn remove_shortcut(
    id: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

    let mut shortcuts = state.shortcuts.lock().unwrap();
    let config = shortcuts.get(&id).ok_or("Shortcut not found")?;

    let parsed = config.shortcut.parse::<Shortcut>()
        .map_err(|e| format!("Invalid shortcut format: {}", e))?;

    app_handle.global_shortcut().unregister(parsed)
        .map_err(|e| format!("Failed to unregister shortcut: {}", e))?;

    shortcuts.remove(&id);
    persist_shortcuts(&shortcuts);
    Ok(())
}

#[tauri::command]
async fn update_shortcut(
    id: String,
    shortcut: Option<String>,
    prompt: Option<String>,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ShortcutConfig, String> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

    if let Some(ref p) = prompt {
        if p.is_empty() {
            return Err("Prompt cannot be empty".to_string());
        }
        if p.len() > 500 {
            return Err("Prompt too long (max 500 characters)".to_string());
        }
    }

    let mut shortcuts = state.shortcuts.lock().unwrap();
    let config = shortcuts.get_mut(&id).ok_or("Shortcut not found")?;

    if let Some(new_shortcut) = shortcut {
        let old_parsed = config.shortcut.parse::<Shortcut>()
            .map_err(|e| format!("Invalid shortcut format: {}", e))?;
        app_handle.global_shortcut().unregister(old_parsed)
            .map_err(|e| format!("Failed to unregister old shortcut: {}", e))?;

        let new_parsed = new_shortcut.parse::<Shortcut>()
            .map_err(|e| format!("Invalid shortcut format: {}", e))?;
        app_handle.global_shortcut().register(new_parsed)
            .map_err(|e| format!("Failed to register new shortcut: {}", e))?;

        config.shortcut = new_shortcut;
    }

    if let Some(new_prompt) = prompt {
        config.prompt = new_prompt;
    }

    let result = config.clone();
    persist_shortcuts(&shortcuts);
    Ok(result)
}

#[tauri::command]
async fn get_llm_provider() -> Result<String, String> {
    Ok(device_id::get_llm_provider())
}

#[tauri::command]
async fn set_llm_provider(provider: String, state: State<'_, AppState>) -> Result<(), String> {
    let provider_enum = LlmProvider::from_str(&provider);
    device_id::set_llm_provider(provider_enum.as_str())?;

    // Ollama doesn't need an API key
    if !provider_enum.requires_api_key() {
        *state.llm_client.lock().unwrap() = Some(LlmClient::new(provider_enum.clone(), String::new()));
        log::info!("✅ Switched to {} provider (local)", provider_enum.display_name());
        return Ok(());
    }

    // Reinitialize LLM client with the new provider's key
    let api_key = device_id::get_api_key(provider_enum.as_str())
        .or_else(|| {
            let env_var = format!("{}_API_KEY", provider_enum.as_str().to_uppercase());
            std::env::var(&env_var).ok().filter(|k| !k.is_empty())
        });

    match api_key {
        Some(key) => {
            *state.llm_client.lock().unwrap() = Some(LlmClient::new(provider_enum.clone(), key));
            log::info!("✅ Switched to {} provider", provider_enum.display_name());
        }
        None => {
            *state.llm_client.lock().unwrap() = None;
            log::info!("⚠️  Switched to {} but no API key configured", provider_enum.display_name());
        }
    }
    Ok(())
}

#[tauri::command]
async fn get_api_key(provider: String) -> Result<String, String> {
    let provider_enum = LlmProvider::from_str(&provider);
    Ok(device_id::get_api_key(provider_enum.as_str()).unwrap_or_default())
}

#[tauri::command]
async fn set_api_key(provider: String, key: String, state: State<'_, AppState>) -> Result<(), String> {
    let provider_enum = LlmProvider::from_str(&provider);
    let key = key.trim().to_string();

    if key.is_empty() {
        device_id::clear_api_key(provider_enum.as_str())?;
    } else {
        device_id::set_api_key(provider_enum.as_str(), &key)?;
    }

    // If this is the active provider, reinitialize the client
    let active_provider = LlmProvider::from_str(&device_id::get_llm_provider());
    if active_provider == provider_enum {
        if key.is_empty() {
            // Try .env fallback
            let env_var = format!("{}_API_KEY", provider_enum.as_str().to_uppercase());
            if let Some(env_key) = std::env::var(&env_var).ok().filter(|k| !k.is_empty()) {
                *state.llm_client.lock().unwrap() = Some(LlmClient::new(provider_enum, env_key));
                log::info!("✅ LLM client reverted to .env key");
            } else {
                *state.llm_client.lock().unwrap() = None;
            }
        } else {
            *state.llm_client.lock().unwrap() = Some(LlmClient::new(provider_enum, key));
            log::info!("✅ LLM client updated with user API key");
        }
    }

    Ok(())
}

#[tauri::command]
async fn init_app(state: State<'_, AppState>) -> Result<(), String> {
    let provider = LlmProvider::from_str(&device_id::get_llm_provider());

    // Ollama doesn't need an API key
    if !provider.requires_api_key() {
        *state.llm_client.lock().unwrap() = Some(LlmClient::new(provider.clone(), String::new()));
        log::info!("✅ {} client initialized (local)", provider.display_name());
        return Ok(());
    }

    // Priority: user-saved key > .env key
    let api_key = device_id::get_api_key(provider.as_str())
        .or_else(|| {
            let env_var = format!("{}_API_KEY", provider.as_str().to_uppercase());
            std::env::var(&env_var).ok().filter(|k| !k.is_empty())
        });

    match api_key {
        Some(key) => {
            *state.llm_client.lock().unwrap() = Some(LlmClient::new(provider.clone(), key));
            log::info!("✅ {} client initialized", provider.display_name());
            Ok(())
        }
        None => {
            log::info!("⚠️  No API key configured for {} — waiting for user to set one in Settings", provider.display_name());
            Ok(())
        }
    }
}

// --- Error reporting ---

fn open_settings(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    use tauri::WebviewWindowBuilder;
    let url = if cfg!(debug_assertions) {
        let dev_url = std::env::var("TAURI_DEV_URL")
            .unwrap_or_else(|_| "http://localhost:5173".to_string());
        tauri::WebviewUrl::External(dev_url.parse().unwrap())
    } else {
        tauri::WebviewUrl::App("index.html".into())
    };
    if let Ok(w) = WebviewWindowBuilder::new(app, "main", url)
        .title("rewrit - Settings")
        .inner_size(800.0, 600.0)
        .center()
        .build()
    {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn missing_key_message() -> String {
    let provider = LlmProvider::from_str(&device_id::get_llm_provider());
    format!("No API key configured for {}. Add one in Settings › AI Provider.", provider.display_name())
}

/// Log an AI error, close the preview/progress windows and tell the user what happened.
fn report_llm_error(app: &AppHandle, error: &str) {
    log::error!("[rewrit] transformation failed: {}", error);
    let _ = app.emit("translation-error", serde_json::json!({"error": error}));
    if let Some(w) = app.get_webview_window("preview") {
        let _ = w.hide();
    }
    hide_progress(app);

    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
    let handle = app.clone();
    app.dialog()
        .message(format!("{}\n\nError logs: Settings › Diagnostics › Copy error logs.", error))
        .title("rewrit could not transform your text")
        .kind(MessageDialogKind::Error)
        .buttons(MessageDialogButtons::OkCancelCustom("Open Settings".into(), "OK".into()))
        .show(move |open| {
            if open {
                open_settings(&handle);
            }
        });
}

// --- Diagnostics ---

#[derive(Debug, Clone, Serialize)]
struct Diagnostics {
    version: String,
    os: String,
    arch: String,
    provider: String,
    model: Option<String>,
    api_key_configured: bool,
    config_dir: String,
    log_dir: String,
}

#[tauri::command]
fn get_diagnostics(app: AppHandle, state: State<'_, AppState>) -> Diagnostics {
    let provider = LlmProvider::from_str(&device_id::get_llm_provider());
    let model = state.llm_client.lock().unwrap().as_ref().map(|c| c.model().to_string());
    Diagnostics {
        version: app.package_info().version.to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        provider: provider.display_name().to_string(),
        api_key_configured: model.is_some(),
        model,
        config_dir: device_id::get_config_dir().display().to_string(),
        log_dir: app.path().app_log_dir().map(|p| p.display().to_string()).unwrap_or_default(),
    }
}

#[derive(Debug, Clone, Serialize)]
struct ConnectionTest {
    ok: bool,
    duration_ms: u128,
    output: Option<String>,
    error: Option<String>,
}

/// Send a tiny prompt to the active provider and report the raw result or error.
#[tauri::command]
async fn test_llm_connection(state: State<'_, AppState>) -> Result<ConnectionTest, String> {
    let client = state.llm_client.lock().unwrap().clone()
        .ok_or("No API key configured for the selected provider")?;
    log::info!("[rewrit] diagnostics: testing LLM connection");
    let start = std::time::Instant::now();
    let result = client.execute_prompt("Hello world".to_string(), "Translate to French".to_string()).await;
    let duration_ms = start.elapsed().as_millis();
    Ok(match result {
        Ok(r) => ConnectionTest { ok: true, duration_ms, output: r.translated_text, error: None },
        Err(e) => {
            log::error!("[rewrit] diagnostics: LLM connection failed: {}", e);
            ConnectionTest { ok: false, duration_ms, output: None, error: Some(e) }
        }
    })
}

#[tauri::command]
fn open_log_dir(app: AppHandle) -> Result<(), String> {
    let dir = app.path().app_log_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let opener = if cfg!(target_os = "macos") { "open" } else if cfg!(target_os = "windows") { "explorer" } else { "xdg-open" };
    std::process::Command::new(opener).arg(&dir).spawn().map_err(|e| e.to_string())?;
    Ok(())
}

/// Last lines of errors.log, to paste in a bug report
#[tauri::command]
fn get_error_logs(app: AppHandle) -> Result<String, String> {
    const MAX_LINES: usize = 50;
    let path = app.path().app_log_dir().map_err(|e| e.to_string())?.join("errors.log");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(String::new()),
        Err(e) => return Err(format!("Failed to read {}: {}", path.display(), e)),
    };
    let lines: Vec<&str> = content.lines().collect();
    Ok(lines[lines.len().saturating_sub(MAX_LINES)..].join("\n"))
}

// --- Model selection ---

fn resolve_api_key(provider: &LlmProvider) -> Option<String> {
    device_id::get_api_key(provider.as_str()).or_else(|| {
        let env_var = format!("{}_API_KEY", provider.as_str().to_uppercase());
        std::env::var(&env_var).ok().filter(|k| !k.is_empty())
    })
}

#[derive(Debug, Clone, Serialize)]
struct ModelSetting {
    model: String,
    default_model: String,
}

#[tauri::command]
fn get_model(provider: String) -> ModelSetting {
    let provider = LlmProvider::from_str(&provider);
    let default_model = provider.default_model().to_string();
    ModelSetting {
        model: device_id::get_model(provider.as_str()).unwrap_or_else(|| default_model.clone()),
        default_model,
    }
}

#[tauri::command]
async fn list_models(provider: String) -> Result<Vec<String>, String> {
    let provider = LlmProvider::from_str(&provider);
    let key = if provider.requires_api_key() {
        resolve_api_key(&provider)
            .ok_or_else(|| format!("Add a {} API key first", provider.display_name()))?
    } else {
        String::new()
    };
    llm_client::list_models(&provider, &key).await
}

#[tauri::command]
fn set_model(provider: String, model: String, state: State<'_, AppState>) -> Result<(), String> {
    let provider = LlmProvider::from_str(&provider);
    device_id::set_model(provider.as_str(), &model)?;
    log::info!("[rewrit] {} model set to {}", provider.display_name(), if model.is_empty() { "default" } else { &model });

    // Rebuild the client if this is the active provider
    if LlmProvider::from_str(&device_id::get_llm_provider()) == provider {
        let key = if provider.requires_api_key() { resolve_api_key(&provider) } else { Some(String::new()) };
        if let Some(key) = key {
            *state.llm_client.lock().unwrap() = Some(LlmClient::new(provider, key));
        }
    }
    Ok(())
}

// --- Ollama model management ---

#[derive(Debug, Clone, Serialize)]
struct OllamaModelStatus {
    installed: bool,
    ollama_running: bool,
}

#[tauri::command]
async fn ollama_status() -> Result<OllamaModelStatus, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| e.to_string())?;

    // Check if Ollama is running
    let running = client.get("http://localhost:11434/api/tags")
        .send()
        .await
        .is_ok();

    if !running {
        return Ok(OllamaModelStatus { installed: false, ollama_running: false });
    }

    // Check if model is installed
    let resp = client.get("http://localhost:11434/api/tags")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status().is_success() {
        let body: serde_json::Value = resp.json().await.unwrap_or_default();
        let models = body["models"].as_array();
        let installed = models.map_or(false, |m| {
            m.iter().any(|model| {
                model["name"].as_str().map_or(false, |n| n.starts_with("llama3.2:3b"))
            })
        });
        Ok(OllamaModelStatus { installed, ollama_running: true })
    } else {
        Ok(OllamaModelStatus { installed: false, ollama_running: true })
    }
}

#[tauri::command]
async fn ollama_pull() -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.post("http://localhost:11434/api/pull")
        .json(&serde_json::json!({"name": "llama3.2:3b", "stream": false}))
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

    if resp.status().is_success() {
        Ok("Model downloaded successfully".to_string())
    } else {
        let body = resp.text().await.unwrap_or_default();
        Err(format!("Failed to pull model: {}", body))
    }
}

#[tauri::command]
async fn ollama_delete() -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.delete("http://localhost:11434/api/delete")
        .json(&serde_json::json!({"name": "llama3.2:3b"}))
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

    if resp.status().is_success() {
        Ok("Model deleted".to_string())
    } else {
        let body = resp.text().await.unwrap_or_default();
        Err(format!("Failed to delete model: {}", body))
    }
}

// --- Progress indicator helpers ---

fn show_progress(app: &AppHandle, input_chars: usize) {
    // Update shared progress state
    {
        let state: State<AppState> = app.state();
        let mut ps = state.progress_state.lock().unwrap();
        ps.active = true;
        ps.input_chars = input_chars;
        ps.output_chars = 0;
        ps.done = false;
        ps.error = None;
        ps.start_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }

    if let Some(w) = app.get_webview_window("progress") {
        let _ = w.show();
        if let Ok(Some(monitor)) = w.current_monitor() {
            let size = monitor.size();
            let scale = monitor.scale_factor();
            let x = (size.width as f64 / scale) - 260.0;
            let _ = w.set_position(tauri::LogicalPosition::new(x, 40.0));
        }
    }
}

fn update_progress_chunks(app: &AppHandle, output_chars: usize) {
    let state: State<AppState> = app.state();
    state.progress_state.lock().unwrap().output_chars = output_chars;
}

fn finish_progress(app: &AppHandle, error: Option<String>) {
    let state: State<AppState> = app.state();
    let mut ps = state.progress_state.lock().unwrap();
    if let Some(err) = error {
        ps.error = Some(err);
    } else {
        ps.done = true;
    }
}

fn hide_progress(app: &AppHandle) {
    {
        let state: State<AppState> = app.state();
        state.progress_state.lock().unwrap().active = false;
    }
    if let Some(w) = app.get_webview_window("progress") {
        let _ = w.hide();
    }
}

// --- LLM preview helper (streaming or not) ---

async fn run_llm_preview(
    llm: llm_client::LlmClient,
    text: String,
    prompt: String,
    stream: bool,
    app: AppHandle,
) -> Result<api_client::TranslateResponse, String> {
    if stream {
        let app_for_chunks = app.clone();
        let original_text = text.clone();
        llm.execute_prompt_stream(text, prompt, move |accumulated| {
            let _ = app_for_chunks.emit("stream-chunk", serde_json::json!({
                "text": accumulated,
                "original": original_text,
            }));
        }).await.map(|response| {
            let _ = app.emit("stream-done", ());
            response
        })
    } else {
        llm.execute_prompt(text, prompt).await
    }
}

// --- Quota reporting ---

async fn report_usage_to_backend(state: &State<'_, AppState>, app: &AppHandle) {
    let api_client = state.api_client.lock().unwrap().as_ref().cloned();
    if let Some(client) = api_client {
        match client.report_usage().await {
            Ok(quota) => {
                log::info!("[rewrit] quota: {}/{}", quota.used, quota.limit);
                let _ = app.emit("quota-updated", &quota);
                let _ = app.emit("translation-completed", serde_json::json!({"quota": quota}));
            }
            Err(e) if e == "quota_exceeded" => {
                log::info!("[rewrit] quota exceeded");
                let _ = app.emit("quota-exceeded", ());
            }
            Err(e) => {
                log::info!("[rewrit] quota report failed (non-critical): {}", e);
            }
        }
    }
}

// --- Shortcut handling ---

fn persist_shortcuts(shortcuts: &HashMap<String, ShortcutConfig>) {
    let entries: Vec<(String, String)> = shortcuts.values()
        .map(|s| (s.shortcut.clone(), s.prompt.clone()))
        .collect();
    if let Err(e) = device_id::set_shortcuts(&entries) {
        log::error!("[rewrit] failed to persist shortcuts: {}", e);
    }
}

fn normalize_shortcut(shortcut: &str) -> String {
    let normalized = shortcut.to_lowercase()
        .replace("super", "commandorcontrol")
        .replace("cmd", "commandorcontrol")
        .replace("command", "commandorcontrol")
        .replace("ctrl", "commandorcontrol")
        .replace("control", "commandorcontrol")
        .replace("key", "")
        .replace(" ", "");

    let mut parts: Vec<&str> = normalized.split('+')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();
    parts.sort();
    parts.join("+")
}

fn find_prompt_for_shortcut(shortcuts: &HashMap<String, ShortcutConfig>, shortcut_str: &str) -> Option<String> {
    let normalized = normalize_shortcut(shortcut_str);
    shortcuts.values()
        .find(|s| normalize_shortcut(&s.shortcut) == normalized)
        .map(|s| s.prompt.clone())
}

fn handle_global_shortcut(app_handle: &AppHandle, shortcut_str: &str) {
    log::info!("[rewrit] shortcut: {}", shortcut_str);
    let state: State<AppState> = app_handle.state();

    // Find prompt for this shortcut
    let prompt = {
        let shortcuts = state.shortcuts.lock().unwrap();
        find_prompt_for_shortcut(&shortcuts, shortcut_str)
    };

    let prompt = match prompt {
        Some(p) => p,
        None => {
            log::info!("[rewrit] no prompt for shortcut");
            return;
        }
    };

    // Prevent concurrent triggers
    {
        let mut is_processing = state.is_processing.lock().unwrap();
        if *is_processing {
            log::info!("[rewrit] already processing, skipping");
            return;
        }
        *is_processing = true;
    }

    let show_preview = *state.show_preview.lock().unwrap();
    log::info!("[rewrit] prompt='{}', preview={}", &prompt[..prompt.len().min(50)], show_preview);

    // Wait for user to release modifier keys (Cmd+Alt from the shortcut)
    // Without this, simulating Cmd+C while Alt is still held sends Cmd+Alt+C
    {
        log::info!("[rewrit] waiting for modifier key release...");
        let mut enigo = Enigo::new(&Settings::default()).unwrap();
        // Release all modifiers to ensure clean state
        let _ = enigo.key(Key::Meta, enigo::Direction::Release);
        let _ = enigo.key(Key::Shift, enigo::Direction::Release);
        let _ = enigo.key(Key::Control, enigo::Direction::Release);
        thread::sleep(std::time::Duration::from_millis(150));
    }

    // Clear clipboard before copy to ensure we detect fresh content
    use tauri_plugin_clipboard_manager::ClipboardExt;
    let _ = app_handle.clipboard().write_text("".to_string());

    // Get the active application and copy/cut text
    #[cfg(target_os = "macos")]
    {
        use tauri_plugin_shell::ShellExt;

        // Store previous app
        let script = r#"tell application "System Events" to get name of first application process whose frontmost is true"#;
        let output = app_handle.shell().command("osascript").args(["-e", script]).output();
        if let Ok(result) = tauri::async_runtime::block_on(output) {
            if let Ok(name) = String::from_utf8(result.stdout) {
                let name = name.trim().to_string();
                if !name.is_empty() && name != "rewrit" {
                    log::info!("[rewrit] previous app: {}", name);
                    *state.previous_app.lock().unwrap() = Some(name);
                }
            }
        }

        // Copy (preview) or Cut (direct paste)
        let key = if show_preview { "c" } else { "x" };
        log::info!("[rewrit] simulating Cmd+{}", key);
        macos_keystroke(app_handle, key);
        thread::sleep(config::DELAY_CONFIG.after_copy);
    }

    #[cfg(not(target_os = "macos"))]
    {
        let key = if show_preview { 'c' } else { 'x' };
        simulate_key_combo(key);
        thread::sleep(config::DELAY_CONFIG.after_copy);
    }

    // Read clipboard - wait for non-empty content (fresh from the copy above)
    let clipboard_text = {
        let mut result = None;
        let retry_delay = config::DELAY_CONFIG.clipboard_retry;
        for _ in 0..8 {
            thread::sleep(retry_delay);
            if let Ok(text) = app_handle.clipboard().read_text() {
                if !text.is_empty() {
                    result = Some(text);
                    break;
                }
            }
        }
        result
    };

    let clipboard_text = match clipboard_text {
        Some(t) => {
            log::info!("[rewrit] clipboard: {} chars", t.len());
            t
        }
        None => {
            log::info!("[rewrit] clipboard empty after retries");
            *state.is_processing.lock().unwrap() = false;
            return;
        }
    };

    // Store original text and prompt for retry
    *state.original_text.lock().unwrap() = Some(clipboard_text.clone());
    *state.last_prompt.lock().unwrap() = Some(prompt.clone());

    if show_preview {
        // --- Preview mode ---
        let preview_window = if let Some(w) = app_handle.get_webview_window("preview") {
            w
        } else {
            use tauri::WebviewWindowBuilder;
            match WebviewWindowBuilder::new(
                app_handle,
                "preview",
                tauri::WebviewUrl::App("src/preview.html".into())
            )
            .title("rewrit - Preview")
            .inner_size(650.0, 450.0)
            .center()
            .decorations(false)
            .always_on_top(true)
            .resizable(false)
            .skip_taskbar(true)
            .build() {
                Ok(w) => w,
                Err(_) => {
                    *state.is_processing.lock().unwrap() = false;
                    return;
                }
            }
        };

        let _ = preview_window.show();
        let _ = preview_window.set_focus();

        // On macOS, also activate our app to ensure the preview window gets keyboard focus
        #[cfg(target_os = "macos")]
        {
            use tauri_plugin_shell::ShellExt;
            let script = r#"tell application "System Events" to set frontmost of process "rewrit" to true"#;
            let output = app_handle.shell().command("osascript").args(["-e", script]).output();
            let _ = tauri::async_runtime::block_on(output);
        }

        let _ = preview_window.emit("translation-started", ());

        let app_clone = app_handle.clone();
        let text = clipboard_text.clone();
        let prompt_clone = prompt.clone();
        tauri::async_runtime::spawn(async move {
            let state: State<AppState> = app_clone.state();
            let llm = state.llm_client.lock().unwrap().as_ref().cloned();
            let use_stream = *state.stream_response.lock().unwrap();

            if let Some(llm) = llm {
                match run_llm_preview(llm, text.clone(), prompt_clone, use_stream, app_clone.clone()).await {
                    Ok(response) if response.success => {
                        if let Some(translated) = response.translated_text {
                            *state.pending_translation.lock().unwrap() = Some(translated.clone());
                            if !use_stream {
                                if let Some(window) = app_clone.get_webview_window("preview") {
                                    let _ = window.emit("show-preview", serde_json::json!({
                                        "original": text,
                                        "translated": translated
                                    }));
                                }
                            } else {
                                // Streaming already emitted chunks; emit final show-preview for pending_translation
                                if let Some(window) = app_clone.get_webview_window("preview") {
                                    let _ = window.emit("show-preview", serde_json::json!({
                                        "original": text,
                                        "translated": translated
                                    }));
                                }
                            }
                            report_usage_to_backend(&state, &app_clone).await;
                        }
                    }
                    Ok(response) => {
                        let err = response.error.unwrap_or_else(|| "Translation failed".to_string());
                        report_llm_error(&app_clone, &err);
                    }
                    Err(e) => report_llm_error(&app_clone, &e),
                }
            } else {
                report_llm_error(&app_clone, &missing_key_message());
            }
            *state.is_processing.lock().unwrap() = false;
        });
    } else {
        // --- Direct paste mode — show progress indicator ---
        show_progress(app_handle, clipboard_text.len());

        let app_clone = app_handle.clone();
        let text = clipboard_text.clone();
        let prompt_clone = prompt.clone();
        tauri::async_runtime::spawn(async move {
            let state: State<AppState> = app_clone.state();
            let llm = state.llm_client.lock().unwrap().as_ref().cloned();
            let use_stream = *state.stream_response.lock().unwrap();

            if let Some(llm) = llm {
                let result = if use_stream {
                    let app_for_chunks = app_clone.clone();
                    llm.execute_prompt_stream(text.clone(), prompt_clone, move |accumulated| {
                        let _ = app_for_chunks.emit("progress-chunk", serde_json::json!({
                            "chars": accumulated.len(),
                        }));
                    }).await
                } else {
                    llm.execute_prompt(text.clone(), prompt_clone).await
                };

                match result {
                    Ok(response) if response.success => {
                        if let Some(translated) = response.translated_text {
                            let _ = app_clone.emit("progress-done", ());

                            use tauri_plugin_clipboard_manager::ClipboardExt;
                            let _ = app_clone.clipboard().write_text(translated.clone());
                            tokio::time::sleep(config::DELAY_CONFIG.clipboard_ready).await;

                            hide_progress(&app_clone);

                            let app_for_paste = app_clone.clone();
                            let _ = tokio::task::spawn_blocking(move || {
                                #[cfg(target_os = "macos")]
                                macos_keystroke(&app_for_paste, "v");
                                #[cfg(not(target_os = "macos"))]
                                simulate_key_combo('v');
                            }).await;

                            report_usage_to_backend(&state, &app_clone).await;
                        }
                    }
                    Ok(response) => {
                        let err = response.error.unwrap_or_else(|| "Translation failed".to_string());
                        report_llm_error(&app_clone, &err);
                    }
                    Err(e) => report_llm_error(&app_clone, &e),
                }
            } else {
                report_llm_error(&app_clone, &missing_key_message());
            }
            *state.is_processing.lock().unwrap() = false;
        });
    }
}

// --- App setup ---

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Panic hook to capture crash info (panics on handler threads kill the app silently)
    std::panic::set_hook(Box::new(|panic_info| {
        let bt = std::backtrace::Backtrace::force_capture();
        eprintln!("[rewrit] PANIC: {}\n{}", panic_info, bt);
        log::error!("[rewrit] PANIC: {}\nBacktrace:\n{}", panic_info, bt);
    }));

    dotenv::dotenv().ok();

    let device_id = device_id::get_device_id().expect("Failed to get device ID");
    log::info!("[rewrit] Device ID: {}...{}", &device_id[..8], &device_id[device_id.len()-4..]);

    let api_client = ApiClient::new(device_id.clone());

    let app_state = AppState {
        api_client: Mutex::new(Some(api_client)),
        llm_client: Mutex::new(None),
        device_id: Mutex::new(device_id.clone()),
        pending_translation: Mutex::new(None),
        original_text: Mutex::new(None),
        last_prompt: Mutex::new(None),
        previous_app: Mutex::new(None),
        show_preview: Mutex::new(device_id::get_show_preview()),
        is_processing: Mutex::new(false),
        shortcuts: Mutex::new(HashMap::new()),
        quick_select_enabled: Mutex::new(device_id::get_quick_select_enabled()),
        quick_select_open: Mutex::new(false),
        quick_select_shortcut: Mutex::new(device_id::get_quick_select_shortcut()),
        stream_response: Mutex::new(device_id::get_stream_response()),
        progress_state: Mutex::new(ProgressState {
            active: false, input_chars: 0, output_chars: 0,
            done: false, error: None, start_ms: 0,
        }),
    };

    tauri::Builder::default()
        // Logs go to stdout, <log dir>/rewrit.log and errors only to <log dir>/errors.log
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("rewrit".to_string()),
                    }),
                    // Errors only, for "Copy error logs" in Settings
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("errors".to_string()),
                    })
                    .filter(|metadata| metadata.level() <= log::Level::Error),
                ])
                .level(log::LevelFilter::Info)
                .max_file_size(2_000_000)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepOne)
                .build(),
        )
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_device_id,
            fetch_quota,
            reset_quota,
            execute_paste,
            cancel_preview,
            retry_translation,
            get_show_preview,
            set_show_preview,
            get_progress_state,
            get_stream_response,
            set_stream_response,
            init_app,
            get_llm_provider,
            set_llm_provider,
            get_api_key,
            set_api_key,
            ollama_status,
            ollama_pull,
            ollama_delete,
            get_shortcuts,
            add_shortcut,
            remove_shortcut,
            update_shortcut,
            get_quick_select_enabled,
            set_quick_select_enabled,
            get_quick_select_shortcut,
            set_quick_select_shortcut,
            cancel_quick_select,
            execute_quick_select,
            get_diagnostics,
            test_llm_connection,
            open_log_dir,
            get_error_logs,
            accessibility_granted,
            open_accessibility_settings,
            get_model,
            list_models,
            set_model,
            updater::get_update_settings,
            updater::set_update_auto_check,
            updater::set_update_auto_install,
            updater::check_for_update,
            updater::install_update
        ])
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Regular);
            }

            updater::spawn_background_checks(app.handle().clone());

            // Pre-warm osascript to avoid cold start delay on first shortcut use
            #[cfg(target_os = "macos")]
            {
                use tauri_plugin_shell::ShellExt;
                let script = r#"return"#;
                let output = app.handle().shell().command("osascript").args(["-e", script]).output();
                let _ = tauri::async_runtime::block_on(output);
                log::info!("✅ osascript warmed up");
            }

            // Global shortcuts
            // IMPORTANT: The handler callback runs on a thread within the tokio runtime
            // context (via App::run → runtime.enter()). Calling block_on from that thread
            // would panic ("Cannot start a runtime from within a runtime"). We spawn a
            // dedicated thread for each shortcut trigger so block_on works safely.
            let app_handle = app.handle().clone();
            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |_app, shortcut, event| {
                        if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                            let handle = app_handle.clone();
                            let shortcut_str = shortcut.to_string();
                            let qs_shortcut: String = handle.state::<AppState>().quick_select_shortcut.lock().unwrap().clone();
                            let is_qs = normalize_shortcut(&shortcut_str) == normalize_shortcut(&qs_shortcut);
                            std::thread::spawn(move || {
                                if let Err(e) = std::panic::catch_unwind(
                                    std::panic::AssertUnwindSafe(|| {
                                        if is_qs {
                                            handle_quick_select_trigger(handle.clone());
                                        } else {
                                            handle_global_shortcut(&handle, &shortcut_str);
                                        }
                                    }),
                                ) {
                                    log::error!("[rewrit] handler panicked: {:?}", e);
                                    let state: State<AppState> = handle.state();
                                    *state
                                        .is_processing
                                        .lock()
                                        .unwrap_or_else(|e| e.into_inner()) = false;
                                }
                            });
                        }
                    })
                    .build()
            )?;

            // System tray
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

            let tray_icon = {
                use tauri::image::Image;
                let icon_bytes = include_bytes!("../icons/tray-icon.png");
                match image::load_from_memory(icon_bytes) {
                    Ok(img) => {
                        let rgba = img.to_rgba8();
                        let (w, h) = rgba.dimensions();
                        log::info!("✅ Loaded tray-icon.png ({}x{})", w, h);
                        Image::new_owned(rgba.into_raw(), w, h)
                    }
                    Err(_) => app.default_window_icon().unwrap().clone(),
                }
            };

            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .icon_as_template(true)
                .tooltip("rewrit")
                .menu(&menu)
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "quit" => app.exit(0),
                        "settings" => open_settings(app),
                        _ => {}
                    }
                })
                .build(app)?;

            log::info!("✅ Tray icon created");

            // Register shortcuts (load persisted, or use defaults)
            {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
                let state: State<AppState> = app.handle().state();
                let mut shortcuts = state.shortcuts.lock().unwrap();

                let persisted = device_id::get_shortcuts();
                let entries: Vec<(String, String)> = if persisted.is_empty() {
                    vec![
                        ("CommandOrControl+Alt+Y".to_string(), "Translate this text from French to English".to_string()),
                        ("CommandOrControl+Alt+T".to_string(), "Translate this text from English to French".to_string()),
                    ]
                } else {
                    persisted
                };

                for (shortcut_str, prompt) in &entries {
                    if let Ok(shortcut) = shortcut_str.parse::<Shortcut>() {
                        if app.global_shortcut().register(shortcut).is_ok() {
                            let id = format!("shortcut_{}", &uuid::Uuid::new_v4().to_string()[..8]);
                            shortcuts.insert(id.clone(), ShortcutConfig {
                                id,
                                shortcut: shortcut_str.to_string(),
                                prompt: prompt.to_string(),
                            });
                        }
                    }
                }
            }

            // Pre-create hidden windows so Vite compiles them at startup
            // (avoids race condition on first shortcut use where the event
            // is emitted before the Svelte component has mounted its listener)
            {
                use tauri::WebviewWindowBuilder;
                if let Ok(w) = WebviewWindowBuilder::new(
                    app.handle(),
                    "preview",
                    tauri::WebviewUrl::App("src/preview.html".into()),
                )
                .title("rewrit - Preview")
                .inner_size(650.0, 450.0)
                .center()
                .decorations(false)
                .always_on_top(true)
                .resizable(false)
                .skip_taskbar(true)
                .visible(false)
                .build()
                {
                    log::info!("✅ Preview window pre-created (hidden)");
                    let _ = w;
                }

                if let Ok(w) = WebviewWindowBuilder::new(
                    app.handle(),
                    "quickselect",
                    tauri::WebviewUrl::App("src/quickselect.html".into()),
                )
                .title("rewrit - Quick Select")
                .inner_size(340.0, 280.0)
                .center()
                .decorations(false)
                .always_on_top(true)
                .resizable(false)
                .skip_taskbar(true)
                .visible(false)
                .build()
                {
                    log::info!("✅ Quick Select window pre-created (hidden)");
                    let _ = w;
                }

                if let Ok(w) = WebviewWindowBuilder::new(
                    app.handle(),
                    "progress",
                    tauri::WebviewUrl::App("src/progress.html".into()),
                )
                .title("rewrit")
                .inner_size(240.0, 64.0)
                .position(
                    (tauri::LogicalPosition::<f64>::new(0.0, 0.0)).x,
                    40.0,
                )
                .decorations(false)
                .always_on_top(true)
                .resizable(false)
                .skip_taskbar(true)
                .visible(false)
                .build()
                {
                    if let Ok(Some(monitor)) = w.current_monitor() {
                        let size = monitor.size();
                        let scale = monitor.scale_factor();
                        let x = (size.width as f64 / scale) - 260.0;
                        let _ = w.set_position(tauri::LogicalPosition::new(x, 40.0));
                    }
                    log::info!("✅ Progress window pre-created (hidden)");
                    let _ = w;
                }
            }

            // Register Quick Select shortcut
            {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
                let state: State<AppState> = app.handle().state();
                let qs_shortcut = state.quick_select_shortcut.lock().unwrap().clone();
                if let Ok(shortcut) = qs_shortcut.parse::<Shortcut>() {
                    if app.global_shortcut().register(shortcut).is_ok() {
                        log::info!("✅ Quick Select shortcut registered ({})", qs_shortcut);
                    } else {
                        log::error!("⚠️  Failed to register Quick Select shortcut");
                    }
                }
            }

            // Initialize LLM client
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state: State<AppState> = app_handle.state();
                if let Err(e) = init_app(state).await {
                    log::error!("Failed to initialize LLM client: {}", e);
                }
            });

            // Register with backend (non-blocking, non-critical)
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state: State<AppState> = app_handle.state();
                let client = state.api_client.lock().unwrap().as_ref().cloned();
                if let Some(client) = client {
                    let platform = if cfg!(target_os = "macos") { "macos" }
                        else if cfg!(target_os = "windows") { "windows" }
                        else { "linux" };
                    match client.register(platform.to_string()).await {
                        Ok(r) => log::info!("Registered with backend. Quota: {:?}", r.quota),
                        Err(e) => log::info!("Backend registration skipped: {}", e),
                    }
                }
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            match event {
                tauri::RunEvent::Ready => {
                    #[cfg(target_os = "macos")]
                    {
                        #[allow(deprecated)]
                        unsafe {
                            use cocoa::appkit::{NSApplication, NSImage};
                            use cocoa::base::{nil, id};
                            use cocoa::foundation::NSData;
                            use objc::{msg_send, sel, sel_impl};

                            let icon_bytes = include_bytes!("../icons/icon.png");
                            let data = NSData::dataWithBytes_length_(
                                nil,
                                icon_bytes.as_ptr() as *const std::os::raw::c_void,
                                icon_bytes.len() as u64,
                            );
                            let image = NSImage::initWithData_(NSImage::alloc(nil), data);
                            if image != nil {
                                let ns_app = cocoa::appkit::NSApp();
                                ns_app.setApplicationIconImage_(image);
                                // Force dock tile refresh so Cmd+Tab picks up the new icon
                                let dock_tile: id = msg_send![ns_app, dockTile];
                                let _: () = msg_send![dock_tile, display];
                                log::info!("✅ macOS app icon set (Ready event)");
                            } else {
                                log::error!("❌ Failed to create NSImage from icon data");
                            }
                        }
                    }
                }
                tauri::RunEvent::ExitRequested { api, code, .. } => {
                    // code == None: triggered by "no visible windows" → prevent (tray app)
                    // code == Some(_): triggered by explicit quit/signal → let it exit
                    if code.is_none() {
                        api.prevent_exit();
                    }
                }
                _ => {}
            }
        });
}

fn main() {
    run();
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::is_accessibility_error;

    #[test]
    fn detects_keystroke_permission_errors() {
        assert!(is_accessibility_error("36:68: execution error: Erreur dans System Events : osascript n’est pas autorisé à envoyer de saisies. (1002)"));
        assert!(is_accessibility_error("System Events got an error: rewrit is not allowed assistive access. (-25211)"));
        assert!(!is_accessibility_error("execution error: Can’t get process \"Foo\". (-1728)"));
    }
}
