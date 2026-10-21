#[cfg(feature = "desktop")]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            commands::initialize(app)?;
            #[cfg(feature="verification")]
            {use tauri::Manager;if std::env::var_os("IELTS_STUDIO_TEST_CONFIG_DIR").is_some(){if let Some(window)=app.get_webview_window("main"){let _=window.hide();}}}
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_roots,
            commands::set_roots,
            commands::import_paper,
            commands::list_papers,
            commands::load_paper,
            commands::read_asset,
            commands::read_asset_binary,
            commands::save_attempt,
            commands::load_attempt,
            commands::list_attempts,
            commands::submit_attempt,
            commands::migrate_roots,
            commands::get_config,
            commands::save_config,
            commands::credential_status,
            commands::save_credential,
            commands::clear_credential,
            commands::read_runtime_model,
            speech_commands::save_speaking_recording,
            speech_commands::read_speaking_recording,
            speech_commands::recover_speaking_recording,
            speech_commands::speech_synthesize,
            speech_commands::speech_recognize,
            speech_commands::speech_examiner,
            speech_commands::cancel_speech
            ,cloud_commands::cloud_create
            ,cloud_commands::cloud_list
            ,cloud_commands::cloud_progress
            ,cloud_commands::cloud_read
            ,cloud_commands::cloud_step
            ,cloud_commands::cloud_cancel
            ,cloud_commands::cloud_publish
            ,cloud_commands::cloud_recheck
            ,cloud_commands::cloud_audio_start
            ,cloud_commands::cloud_retry_audio
            ,cloud_commands::cloud_fix_text
            ,cloud_commands::cloud_retry_text
            ,cloud_commands::cloud_review
            ,cloud_commands::cloud_add_media
            ,cloud_commands::cloud_test
            ,cloud_commands::cloud_asset
            ,source_commands::source_action
            ,source_commands::source_step
            ,source_commands::source_asset
            ,task_commands::task_enqueue
            ,task_commands::task_list
            ,task_commands::task_read
            ,task_commands::task_pause
            ,task_commands::task_cancel
            ,task_commands::task_resume
            ,task_commands::task_policy_read
            ,task_commands::task_policy_save
            ,task_commands::task_exam_active
            ,task_commands::calibration_import
            ,task_commands::calibration_read
        ])
        .run(tauri::generate_context!())
        .expect("IELTS Studio failed to start");
}
pub mod attempts;
#[cfg(feature = "desktop")]
mod commands;
#[cfg(feature = "desktop")]
mod speech_commands;
#[cfg(feature = "desktop")]
mod cloud_commands;
pub mod configuration;
pub mod contracts;
pub mod credentials;
pub mod migration;
pub mod calibration;
pub mod papers;
pub mod runtime;
pub mod storage;
pub mod task_queue;
pub mod task_policy;
#[cfg(feature = "desktop")]
mod task_commands;
pub mod speech;
pub mod speech_cache;
pub mod cloud;
pub mod source_library;
#[cfg(feature = "desktop")]
mod source_commands;
