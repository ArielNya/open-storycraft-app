//! Tauri 2 shell. Commands here are the IPC contract for desktop and Android.

#![allow(missing_docs)]
#![allow(clippy::print_stdout)]

mod android;
mod commands;
mod error;
mod host;
mod paths;
mod settings;
mod window;

use storycraft_auth::DeviceCode;

/// Shared process state.
pub struct AppState {
    /// In-flight device-code login, if any.
    pub pending_device: std::sync::Mutex<Option<DeviceCode>>,
}

/// Start the Tauri runtime.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if std::env::var_os("RUST_LOG").is_some() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_writer(std::io::stderr)
            .try_init();
    }

    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            pending_device: std::sync::Mutex::new(None),
        })
        .setup(|app| {
            window::fit_to_work_area(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::list_models,
            commands::discover_projects,
            commands::get_status,
            commands::list_files,
            commands::read_project_file,
            commands::list_jobs,
            commands::get_job,
            commands::save_job,
            commands::job_diff,
            commands::reject_job,
            commands::list_skills,
            commands::run_skill,
            commands::export_project,
            commands::burstiness_report,
            commands::generate_names,
            commands::pick_folder,
            commands::auth_login,
            commands::auth_poll,
            commands::auth_status,
            commands::auth_logout,
        ])
        .run(tauri::generate_context!());

    if let Err(err) = result {
        eprintln!("storycraft-app failed: {err}");
        std::process::exit(1);
    }
}
