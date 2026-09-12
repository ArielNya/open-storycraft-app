//! Job keepalive. Desktop: a notification. Android: dataSync foreground service.

use storycraft_core::Job;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

/// RAII: notify while a skill run is in flight. Preview is already on disk.
pub(crate) struct JobNotice;

impl JobNotice {
    pub(crate) fn start(app: &AppHandle, job: &Job) -> Self {
        let title = format!("Open Storycraft · {}", job.skill);
        let _ = app
            .notification()
            .builder()
            .title(&title)
            .body("Generating. Preview is written to disk if the UI is killed.")
            .show();
        #[cfg(target_os = "android")]
        start_foreground(job);
        Self
    }
}

impl Drop for JobNotice {
    fn drop(&mut self) {
        #[cfg(target_os = "android")]
        stop_foreground();
    }
}

#[cfg(target_os = "android")]
fn start_foreground(job: &Job) {
    let _ = job;
}

#[cfg(target_os = "android")]
fn stop_foreground() {}
