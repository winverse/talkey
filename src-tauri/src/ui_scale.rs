//! Window size presets for the main window, like a game client's "window
//! size" option: the page is zoomed and the window grows by the same ratio.

use tauri::Manager;

/// Main window size at 100%, matching tauri.conf.json.
const BASE_WIDTH: f64 = 1280.0;
const BASE_HEIGHT: f64 = 860.0;
/// Space kept free around the window when a preset is larger than the screen.
const SCREEN_MARGIN: f64 = 48.0;

pub const MIN_SCALE: f64 = 0.8;
pub const MAX_SCALE: f64 = 1.6;

/// Clamps a stored scale to the supported range; invalid values become 100%.
pub fn normalize(scale: f64) -> f64 {
    if scale.is_finite() && scale > 0.0 {
        scale.clamp(MIN_SCALE, MAX_SCALE)
    } else {
        1.0
    }
}

/// Window size for `scale`, shrunk to fit a screen of `screen` logical pixels.
fn window_size(scale: f64, screen: Option<(f64, f64)>) -> (f64, f64) {
    let (mut width, mut height) = (BASE_WIDTH * scale, BASE_HEIGHT * scale);
    if let Some((screen_width, screen_height)) = screen {
        width = width.min(screen_width - SCREEN_MARGIN);
        height = height.min(screen_height - SCREEN_MARGIN);
    }
    (width.round(), height.round())
}

/// Zooms the main window's page to `scale`. With `resize`, also resizes the
/// window by the same ratio (fitted to the screen) and centers it.
pub fn apply(app: &tauri::AppHandle, scale: f64, resize: bool) {
    let scale = normalize(scale);
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if let Err(error) = window.set_zoom(scale) {
        tracing::warn!("Failed to set UI scale {scale}: {error}");
    }
    if !resize {
        return;
    }
    let screen = window.current_monitor().ok().flatten().map(|monitor| {
        let factor = monitor.scale_factor();
        let area = monitor.work_area();
        (
            f64::from(area.size.width) / factor,
            f64::from(area.size.height) / factor,
        )
    });
    let (width, height) = window_size(scale, screen);
    let _ = window.set_size(tauri::LogicalSize::new(width, height));
    let _ = window.center();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_keeps_presets_and_rejects_bad_values() {
        assert_eq!(normalize(1.25), 1.25);
        assert_eq!(normalize(3.0), MAX_SCALE);
        assert_eq!(normalize(0.1), MIN_SCALE);
        assert_eq!(normalize(f64::NAN), 1.0);
        assert_eq!(normalize(0.0), 1.0);
    }

    #[test]
    fn window_grows_with_scale_and_fits_the_screen() {
        assert_eq!(window_size(1.0, None), (1280.0, 860.0));
        assert_eq!(window_size(1.25, Some((2560.0, 1440.0))), (1600.0, 1075.0));
        // A 14-inch MacBook Pro work area is about 1512 x 945.
        assert_eq!(window_size(1.4, Some((1512.0, 945.0))), (1464.0, 897.0));
    }
}
