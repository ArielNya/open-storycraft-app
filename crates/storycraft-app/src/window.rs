//! Initial window sizing.
//!
//! The compositor owns the final window size: a request taller than the work
//! area is silently clamped, and some compositors ignore resize requests
//! outright. Ask for something the work area can hold, so the request and the
//! result agree, and log it when they cannot.

use tauri::{AppHandle, LogicalSize, Manager, WebviewWindow};

/// Room left for window decorations. Wayland does not report panel geometry to
/// clients, so [`WebviewWindow::current_monitor`] may hand back the whole
/// screen as the work area; the margin keeps the decorated window inside it.
const DECORATION_ALLOWANCE: f64 = 64.0;

/// Smallest work area worth trusting, in logical pixels. Anything smaller is a
/// driver or backend reporting nonsense, and the monitor size is used instead.
const MIN_TRUSTED_WORK_AREA: f64 = 300.0;

/// Shrink the `main` window to the work area of the monitor it opens on.
///
/// Best effort: a missing monitor, or a window that already fits, leaves the
/// configured size alone.
pub(crate) fn fit_to_work_area(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        tracing::debug!("window sizing: no main window");
        return;
    };
    let Some(requested) = requested_size(app) else {
        tracing::debug!("window sizing: no size for the main window in the config");
        return;
    };
    let Some(work_area) = work_area(&window) else {
        tracing::debug!("window sizing: no monitor reported yet");
        return;
    };
    let fitted = clamp_to_work_area(requested, work_area);
    if fitted == requested {
        tracing::debug!(
            ?requested,
            ?work_area,
            "window sizing: request already fits"
        );
        return;
    }
    tracing::info!(
        requested = ?requested,
        ?work_area,
        fitted = ?fitted,
        "window clamped to the monitor work area"
    );
    if let Err(err) = window.set_size(LogicalSize::new(fitted.0, fitted.1)) {
        tracing::warn!(%err, "could not resize the window");
    }
}

/// The `main` window size requested in `tauri.conf.json`.
fn requested_size(app: &AppHandle) -> Option<(f64, f64)> {
    let config = app.config();
    let window = config.app.windows.iter().find(|w| w.label == "main")?;
    Some((window.width, window.height))
}

/// Work area of the window's monitor, in logical pixels.
fn work_area(window: &WebviewWindow) -> Option<(f64, f64)> {
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten())?;
    let scale = monitor.scale_factor();
    let area = monitor.size().to_logical::<f64>(scale);
    let work = monitor.work_area().size.to_logical::<f64>(scale);
    let usable = if work.height >= MIN_TRUSTED_WORK_AREA {
        work
    } else {
        area
    };
    Some((usable.width, usable.height))
}

/// Clamp `requested` to `available`, keeping [`DECORATION_ALLOWANCE`] free.
///
/// Never grows a window: a small request stays small on a large monitor.
fn clamp_to_work_area(requested: (f64, f64), available: (f64, f64)) -> (f64, f64) {
    let max_height = available.1 - DECORATION_ALLOWANCE;
    let max_width = available.0;
    (requested.0.min(max_width), requested.1.min(max_height))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn tall_request_is_clamped_to_the_work_area() {
        // 780 requested on a 1080p screen at 1.7 fractional scaling.
        let fitted = clamp_to_work_area((420.0, 780.0), (1129.0, 635.0));
        assert_eq!(fitted, (420.0, 571.0));
    }

    #[test]
    fn a_request_that_fits_is_untouched() {
        assert_eq!(
            clamp_to_work_area((420.0, 780.0), (1920.0, 1080.0)),
            (420.0, 780.0)
        );
    }

    #[test]
    fn a_small_request_is_not_grown() {
        assert_eq!(
            clamp_to_work_area((420.0, 400.0), (1920.0, 1080.0)),
            (420.0, 400.0)
        );
    }

    #[test]
    fn a_narrow_monitor_clamps_the_width_too() {
        let fitted = clamp_to_work_area((420.0, 780.0), (360.0, 700.0));
        assert_eq!(fitted, (360.0, 636.0));
    }
}
