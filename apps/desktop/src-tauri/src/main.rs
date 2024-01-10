#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc};
use tokio::sync::Mutex;
use std::env;
use tauri::{Manager};
use tauri_plugin_positioner::{WindowExt, Position};

mod recording;
mod upload;
mod devices;
mod utils;

use recording::{RecordingState, start_dual_recording, stop_all_recordings};
use upload::upload_file;
use devices::list_devices;

use ffmpeg_sidecar::{
    command::ffmpeg_is_installed,
    download::{check_latest_version, download_ffmpeg_package, ffmpeg_download_url, unpack_ffmpeg},
    error::Result as FfmpegResult,
    paths::sidecar_dir,
    version::ffmpeg_version,
};

fn main() {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("Thread panicked: {:?}", info);
    }));

    if which::which("ffmpeg").is_err() {
        if let Err(e) = handle_ffmpeg_installation() {
            eprintln!("Failed to handle FFmpeg installation: {}", e);
        }
    }

    fn handle_ffmpeg_installation() -> FfmpegResult<()> {
        if ffmpeg_is_installed() {
            println!("FFmpeg is already installed! 🎉");
            return Ok(());
        }

        match check_latest_version() {
            Ok(version) => println!("Latest available version: {}", version),
            Err(_) => println!("Skipping version check on this platform."),
        }

        let download_url = ffmpeg_download_url()?;
        let destination = sidecar_dir()?;

        println!("Downloading from: {:?}", download_url);
        let archive_path = download_ffmpeg_package(download_url, &destination)?;
        println!("Downloaded package: {:?}", archive_path);

        println!("Extracting...");
        unpack_ffmpeg(&archive_path, &destination)?;

        let version = ffmpeg_version()?;
        println!("FFmpeg version: {}", version);

        println!("Done! 🏁");
        Ok(())
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .setup(move |app| {
            if let Some(camera_window) = app.get_window("camera") { 
              let _ = camera_window.move_window(Position::BottomRight);
            }

            if let Some(options_window) = app.get_window("options") { 
              let _ = options_window.move_window(Position::Center);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_dual_recording,
            stop_all_recordings,
            upload_file,
            list_devices
        ])
        .manage(Arc::new(Mutex::new(RecordingState { screen_process: None, video_process: None, tx: None })))
        .plugin(tauri_plugin_context_menu::init())
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}