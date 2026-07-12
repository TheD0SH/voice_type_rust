use anyhow::{Context, Result};
use super::RuntimeSnapshot;
use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use voice_type::config::APP_DISPLAY_NAME;
use voice_type::types::Config;

pub(crate) const HUD_WINDOW: &str = "hud";
pub(crate) const HUD_MARGIN: i32 = 18;

pub(crate) fn ensure_hud_window(app: &AppHandle, config: &Config) -> Result<()> {
    if app.get_webview_window(HUD_WINDOW).is_some() {
        return Ok(());
    }

    let (init_width, init_height) = hud_window_size(config);

    let window = WebviewWindowBuilder::new(app, HUD_WINDOW, WebviewUrl::App("hud.html".into()))
        .title(APP_DISPLAY_NAME)
        .inner_size(init_width, init_height)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .closable(false)
        .visible(false)
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(false)
        .build()
        .context("Failed to build HUD webview window")?;

    let _ = apply_hud_window_layout(&window, config);

    // Position: use saved position if available, otherwise corner default.
    if let (Some(x), Some(y)) = (config.hud_position_x, config.hud_position_y) {
        let _ = window.set_position(PhysicalPosition::new(x, y));
    } else {
        let _ = position_hud_window(app, &window, config);
    }

    // Explicitly hide — the window may flash briefly on Windows during creation.
    let _ = window.hide();

    // Click-through unless pinned.  The HUD starts hidden and is only shown
    // by sync_hud_window when recording/processing starts.
    let _ = window.set_ignore_cursor_events(!config.hud_pinned);

    Ok(())
}

pub(crate) fn sync_hud_window(app: &AppHandle, snapshot: &RuntimeSnapshot) {
    let pinned = snapshot.config.hud_pinned;
    let should_show = pinned
        || (snapshot.config.hud_enabled
            && matches!(
                snapshot.app_state.as_str(),
                "recording" | "processing"
            ));

    // Lazy window creation: don't create the HUD window at startup (it causes
    // a brief flash on Windows).  Instead, create it here the first time it's
    // actually needed.
    if should_show && app.get_webview_window(HUD_WINDOW).is_none() {
        let _ = ensure_hud_window(app, &snapshot.config);
    }

    let Some(window) = app.get_webview_window(HUD_WINDOW) else {
        return;
    };

    if should_show {
        let visible = window.is_visible().unwrap_or(false);
        if !visible {
            // Only auto-position when not pinned (pinned keeps user's position).
            if !pinned {
                let _ = position_hud_window(app, &window, &snapshot.config);
            }
            let _ = window.show();
        }
    } else if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    }

    // Toggle interactivity: pinned HUD is clickable/draggable, normal is click-through.
    let _ = window.set_ignore_cursor_events(!pinned);
}

pub(crate) fn position_hud_window(
    app: &AppHandle,
    window: &WebviewWindow,
    config: &Config,
) -> tauri::Result<()> {
    let monitor = app
        .get_webview_window("main")
        .and_then(|main_window| main_window.current_monitor().ok().flatten())
        .or(window.current_monitor()?)
        .or(window.primary_monitor()?);

    let Some(monitor) = monitor else {
        return Ok(());
    };

    let work_area = monitor.work_area();
    let size = window.outer_size()?;
    let scale = monitor.scale_factor();
    let width_log = size.width as f64 / scale;
    let height_log = size.height as f64 / scale;

    let x = if config.hud_side == "left" {
        work_area.position.x + HUD_MARGIN
    } else {
        work_area.position.x + work_area.size.width as i32 - width_log as i32 - HUD_MARGIN
    };
    let y = work_area.position.y + work_area.size.height as i32 - height_log as i32 - HUD_MARGIN;

    window.set_position(PhysicalPosition::new(x, y))
}

pub(crate) fn apply_hud_window_layout(window: &WebviewWindow, config: &Config) -> tauri::Result<()> {
    let (width, height) = hud_window_size(config);
    window.set_size(LogicalSize::new(width, height))
}

pub(crate) fn hud_window_size(config: &Config) -> (f64, f64) {
    let show_topline = config.hud_show_state || config.hud_show_app_name;

    let mut height = 62.0;
    if show_topline {
        height += 18.0;
    }
    if config.hud_show_meter {
        height += 52.0;
    }

    let width = if config.hud_show_meter {
        340.0
    } else if show_topline {
        260.0
    } else {
        220.0
    };

    (width, height)
}
