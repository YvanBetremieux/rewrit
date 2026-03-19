use std::time::Duration;

#[allow(dead_code)]
pub struct DelayConfig {
    pub window_hide: Duration,
    pub restore_clipboard: Duration,
    pub clipboard_ready: Duration,
    pub keyboard_short_delay: Duration,
    pub after_copy: Duration,
    pub clipboard_retry: Duration,
    pub after_refocus: Duration,
}

impl DelayConfig {
    pub fn load() -> Self {
        let _ = dotenv::dotenv();
        Self {
            // After hiding preview window, before refocusing previous app
            window_hide: Duration::from_millis(Self::env("DELAY_WINDOW_HIDE", 150)),
            // Before restoring original clipboard content
            restore_clipboard: Duration::from_millis(Self::env("DELAY_RESTORE_CLIPBOARD", 500)),
            // After writing to clipboard, before simulating paste
            clipboard_ready: Duration::from_millis(Self::env("DELAY_CLIPBOARD_READY", 100)),
            // Between modifier key press/release and character key
            keyboard_short_delay: Duration::from_millis(Self::env("DELAY_KEYBOARD_SHORT_DELAY", 60)),
            // After Cmd+C/X, before reading clipboard
            after_copy: Duration::from_millis(Self::env("DELAY_AFTER_COPY", 150)),
            // Between clipboard read retry attempts
            clipboard_retry: Duration::from_millis(Self::env("DELAY_CLIPBOARD_RETRY", 50)),
            // After refocusing previous app, before pasting
            after_refocus: Duration::from_millis(Self::env("DELAY_AFTER_REFOCUS", 200)),
        }
    }

    fn env(key: &str, default: u64) -> u64 {
        std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
    }
}

lazy_static::lazy_static! {
    pub static ref DELAY_CONFIG: DelayConfig = DelayConfig::load();
}
