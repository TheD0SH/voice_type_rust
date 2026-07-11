#[tauri::command]
pub fn open_external_url(url: String) -> Result<(), String> {
    let trimmed = url.trim();
    if !trimmed.starts_with("https://") {
        return Err("Only https URLs are allowed".to_string());
    }

    if trimmed.chars().any(char::is_control) {
        return Err("URL contains invalid control characters".to_string());
    }

    open::that(trimmed)
        .map(|_| ())
        .map_err(|error| format!("Failed to open URL: {}", error))
}

#[tauri::command]
pub fn get_history() -> Vec<voice_type::history::HistoryEntry> {
    voice_type::history::load()
}

#[tauri::command]
pub fn clear_history() -> Result<(), String> {
    voice_type::history::clear().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn play_beep(frequency: u32, duration_ms: u32) -> Result<(), String> {
    std::thread::spawn(move || {
        #[cfg(target_os = "windows")]
        {
            mod win_beep {
                use std::ffi::c_int;
                unsafe extern "system" {
                    pub fn Beep(dwFreq: u32, dwDuration: u32) -> c_int;
                }
            }
            unsafe {
                win_beep::Beep(frequency, duration_ms);
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Minimal fallback audio feedback: write the ASCII BEL character
            // to stderr. This gives at least some audible signal on platforms
            // without a dedicated beep API.
            let _ = frequency;
            let _ = duration_ms;
            let _ = std::io::Write::write_all(&mut std::io::stderr(), b"\x07");
        }
    });
    Ok(())
}
