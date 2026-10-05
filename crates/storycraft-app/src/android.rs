//! Job keepalive. Desktop: a notification. Android: dataSync foreground service.

use storycraft_core::Job;
use tauri::AppHandle;
use tauri::plugin::TauriPlugin;
#[cfg(not(target_os = "android"))]
use tauri_plugin_notification::NotificationExt;

/// Handle to the Kotlin side (`StorycraftPlugin.kt`).
#[cfg(target_os = "android")]
pub(crate) struct Bridge(tauri::plugin::PluginHandle<tauri::Wry>);

/// Registers the Kotlin plugin on Android; a no-op on desktop.
pub(crate) fn plugin() -> TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new("storycraft")
        .setup(|app, api| {
            #[cfg(target_os = "android")]
            {
                use tauri::Manager;
                let handle =
                    api.register_android_plugin("dev.openstorycraft.app", "StorycraftPlugin")?;
                app.manage(Bridge(handle));
            }
            #[cfg(not(target_os = "android"))]
            let _ = (app, api);
            Ok(())
        })
        .build()
}

/// RAII: notify while a skill run is in flight. Preview is already on disk.
///
/// Must be created off the main thread: on Android it makes a blocking call
/// into Kotlin, which runs on the main looper.
pub(crate) struct JobNotice {
    #[cfg(target_os = "android")]
    app: AppHandle,
}

impl JobNotice {
    pub(crate) fn start(app: &AppHandle, job: &Job) -> Self {
        #[cfg(target_os = "android")]
        {
            // The service's ongoing notification is the notice on Android.
            call(app, "startJob", serde_json::json!({ "skill": job.skill }));
            Self { app: app.clone() }
        }
        #[cfg(not(target_os = "android"))]
        {
            let title = format!("Open Storycraft · {}", job.skill);
            let _ = app
                .notification()
                .builder()
                .title(&title)
                .body("Generating. Preview is written to disk if the UI is killed.")
                .show();
            Self {}
        }
    }
}

impl Drop for JobNotice {
    fn drop(&mut self) {
        #[cfg(target_os = "android")]
        call(&self.app, "stopJob", serde_json::json!({}));
    }
}

/// Open the OAuth verification page: a Custom Tab on Android (the login must
/// not run inside the app's `WebView`), the default browser elsewhere.
pub(crate) fn open_auth_url(app: &AppHandle, url: &str) {
    #[cfg(target_os = "android")]
    call(app, "openAuthTab", serde_json::json!({ "url": url }));
    #[cfg(not(target_os = "android"))]
    {
        use tauri_plugin_opener::OpenerExt;
        let _ = app.opener().open_url(url, None::<&str>);
    }
}

/// Fire a plugin command. A failure costs the keepalive, not the job, so it is
/// logged rather than returned.
#[cfg(target_os = "android")]
fn call(app: &AppHandle, command: &str, payload: serde_json::Value) {
    use tauri::Manager;
    let Some(bridge) = app.try_state::<Bridge>() else {
        tracing::warn!(command, "storycraft Android plugin not registered");
        return;
    };
    if let Err(err) = bridge
        .0
        .run_mobile_plugin::<serde_json::Value>(command, payload)
    {
        tracing::warn!(%err, command, "Android plugin call failed");
    }
}
