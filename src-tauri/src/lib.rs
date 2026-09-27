mod audio;
mod config;
mod hotkey;
mod inject;
mod llm;
mod resample;
mod stt;

use audio::Recorder;
use config::Config;
use hotkey::PushToTalk;
//use llm::Cleaner;
use stt::Transcriber;
use tauri::{AppHandle, Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();

            if let Some(window) = app.get_webview_window("main") {
                if let Ok(Some(monitor)) = window.primary_monitor() {
                    let screen_size = monitor.size();
                    let scale = monitor.scale_factor();
                    let window_width = 240.0;
                    let window_height = 64.0;

                    let x = (screen_size.width as f64 / scale - window_width) / 2.0;
                    let y = screen_size.height as f64 / scale - window_height - 40.0; // 40px above bottom

                    let _ = window.set_position(tauri::LogicalPosition::new(x, y));
                }
            }

            std::thread::spawn(move || run_dictation_loop(handle));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn run_dictation_loop(app: AppHandle) {
    let config = Config::load_or_create("config.toml");

    println!("Loading whisper model from {}...", config.model_path);
    let transcriber = Transcriber::load(&config.model_path);

    let ptt = PushToTalk::new(&config.hotkey).expect("failed to set up hotkey listener");
    println!("Ready. Hold {} to talk.", config.hotkey);
    let mut has_typed_before = false;

    loop {
        ptt.wait_for_press();
        let _ = app.emit("recording-started", ());

        let recorder = Recorder::start_default_input();
        let channels = recorder.config.channels();

        // Stream live level to the UI while recording.
        let level_handle = recorder.level.clone();
        let app_for_level = app.clone();
        let stop_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();

        std::thread::spawn(move || {
            while !stop_flag_clone.load(std::sync::atomic::Ordering::Relaxed) {
                let level = level_handle.load(std::sync::atomic::Ordering::Relaxed);
                let _ = app_for_level.emit("audio-level", level);
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });

        ptt.wait_for_release();
        stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = app.emit("recording-stopped", ());
        let samples = recorder.snapshot();
        drop(recorder);

        let audio = resample::resample(&samples, channels);
        let raw_text = transcriber.transcribe(&audio);

        //if is_silence_or_empty(&raw_text) {
        //    let _ = app.emit("transcription-empty", ());
        //    continue;
        //}
        //
        let _ = app.emit("transcribing", ());
        //let cleaned_text = cleaner.clean(&raw_text);

        let _ = app.emit("transcription-done", raw_text.clone());

        inject::type_text(&raw_text, has_typed_before);
        has_typed_before = true;
    }
}
