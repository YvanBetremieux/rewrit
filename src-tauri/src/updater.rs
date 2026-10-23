// Auto-update: background checks, settings and Tauri commands for the Settings window.
// Updates come from latest.json on the latest GitHub release (see tauri.conf.json).

use crate::device_id;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_updater::{Update, UpdaterExt};

/// How often the background task checks for updates
const CHECK_INTERVAL_HOURS: i64 = 24;
/// How often the background task wakes up to see if a check is due
/// (a plain 24h sleep would drift while the Mac is asleep)
const TICK: Duration = Duration::from_secs(60 * 60);

static INSTALLING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize)]
pub struct UpdateSettings {
    current_version: String,
    auto_check: bool,
    auto_install: bool,
    last_check: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateCheck {
    current_version: String,
    available: bool,
    version: Option<String>,
    notes: Option<String>,
}

fn current_version(app: &AppHandle) -> String {
    app.package_info().version.to_string()
}

async fn check(app: &AppHandle) -> Result<Option<Update>, String> {
    let updater = app.updater().map_err(|e| format!("Updater init failed: {}", e))?;
    match updater.check().await {
        Ok(update) => {
            let _ = device_id::set_update_last_check(&chrono::Utc::now().to_rfc3339());
            match &update {
                Some(u) => log::info!("[rewrit] update available: v{} (current v{})", u.version, u.current_version),
                None => log::info!("[rewrit] app is up to date (v{})", current_version(app)),
            }
            Ok(update)
        }
        Err(e) => {
            log::error!("[rewrit] update check failed: {}", e);
            Err(format!("Update check failed: {}", e))
        }
    }
}

async fn install(app: &AppHandle, update: Update) -> Result<(), String> {
    if INSTALLING.swap(true, Ordering::SeqCst) {
        return Err("An update is already being installed".to_string());
    }
    log::info!("[rewrit] downloading and installing v{}", update.version);
    let _ = app.emit("update-status", "installing");
    let result = update.download_and_install(|_, _| {}, || {}).await;
    INSTALLING.store(false, Ordering::SeqCst);
    match result {
        Ok(()) => {
            log::info!("[rewrit] update v{} installed", update.version);
            let _ = app.emit("update-status", "installed");
            Ok(())
        }
        Err(e) => {
            log::error!("[rewrit] update install failed: {}", e);
            let _ = app.emit("update-status", "failed");
            Err(format!("Update install failed: {}", e))
        }
    }
}

fn check_is_due() -> bool {
    let Some(last) = device_id::get_update_last_check() else { return true };
    match chrono::DateTime::parse_from_rfc3339(&last) {
        Ok(last) => chrono::Utc::now().signed_duration_since(last) >= chrono::Duration::hours(CHECK_INTERVAL_HOURS),
        Err(_) => true,
    }
}

fn ask_restart(app: &AppHandle, version: &str) {
    let handle = app.clone();
    app.dialog()
        .message(format!("rewrit v{} has been installed. Restart now to use it?", version))
        .title("rewrit update")
        .buttons(MessageDialogButtons::OkCancelCustom("Restart now".into(), "Later".into()))
        .show(move |restart| {
            if restart {
                handle.restart();
            }
        });
}

fn ask_install(app: &AppHandle, update: Update) {
    let handle = app.clone();
    app.dialog()
        .message(format!(
            "rewrit v{} is available (you have v{}). Install it and restart now?",
            update.version, update.current_version
        ))
        .title("rewrit update")
        .buttons(MessageDialogButtons::OkCancelCustom("Install".into(), "Later".into()))
        .show(move |accepted| {
            if accepted {
                tauri::async_runtime::spawn(async move {
                    if install(&handle, update).await.is_ok() {
                        handle.restart();
                    }
                });
            }
        });
}

async fn background_check(app: &AppHandle) {
    let Ok(Some(update)) = check(app).await else { return };
    let _ = app.emit("update-status", "available");
    if device_id::get_update_auto_install() {
        let version = update.version.clone();
        if install(app, update).await.is_ok() {
            ask_restart(app, &version);
        }
    } else {
        ask_install(app, update);
    }
}

/// Check at launch, then every CHECK_INTERVAL_HOURS while auto-check is on.
pub fn spawn_background_checks(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        if device_id::get_update_auto_check() {
            background_check(&app).await;
        }
        loop {
            tokio::time::sleep(TICK).await;
            if device_id::get_update_auto_check() && check_is_due() {
                background_check(&app).await;
            }
        }
    });
}

#[tauri::command]
pub fn get_update_settings(app: AppHandle) -> UpdateSettings {
    UpdateSettings {
        current_version: current_version(&app),
        auto_check: device_id::get_update_auto_check(),
        auto_install: device_id::get_update_auto_install(),
        last_check: device_id::get_update_last_check(),
    }
}

#[tauri::command]
pub fn set_update_auto_check(enabled: bool) -> Result<(), String> {
    device_id::set_update_auto_check(enabled)
}

#[tauri::command]
pub fn set_update_auto_install(enabled: bool) -> Result<(), String> {
    device_id::set_update_auto_install(enabled)
}

#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> Result<UpdateCheck, String> {
    let update = check(&app).await?;
    Ok(UpdateCheck {
        current_version: current_version(&app),
        available: update.is_some(),
        version: update.as_ref().map(|u| u.version.clone()),
        notes: update.and_then(|u| u.body),
    })
}

/// Download and install the latest version, then restart the app.
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let update = check(&app).await?.ok_or("rewrit is already up to date")?;
    install(&app, update).await?;
    app.restart();
}
