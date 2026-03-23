// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
        all(not(debug_assertions), target_os = "windows"),
        windows_subsystem = "windows"
)]

use once_cell::sync::Lazy;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use serde::Serialize;
use std::{collections::HashMap, fs::File, io::BufReader, path::Path, sync::Mutex};
use thiserror::Error;
use uuid::Uuid;

#[derive(Serialize, Clone, Debug)]
struct SoundDescriptor {
        id: String,
        label: String,
        path: String,
}

#[derive(Debug, Error)]
enum AudioError {
        #[error("Failed to access default output: {0}")]
        Output(String),
        #[error("Could not open {0}: {1}")]
        FileOpen(String, String),
        #[error("Failed to decode {0}: {1}")]
        Decode(String, String),
        #[error("Could not create audio sink: {0}")]
        Sink(String),
        #[error("Audio engine poisoned" )]
        Poisoned,
}

struct AudioState {
        _stream: OutputStream,
        handle: OutputStreamHandle,
        sinks: HashMap<String, Sink>,
}

impl AudioState {
        fn new() -> Result<Self, AudioError> {
                let (stream, handle) = OutputStream::try_default()
                        .map_err(|err| AudioError::Output(err.to_string()))?;

                Ok(Self {
                        _stream: stream,
                        handle,
                        sinks: HashMap::new(),
                })
        }

        fn play(&mut self, sound_id: &str, path: &str) -> Result<(), AudioError> {
                self.stop(sound_id);
                let file = File::open(path)
                        .map_err(|err| AudioError::FileOpen(path.to_string(), err.to_string()))?;
                let source = Decoder::new(BufReader::new(file))
                        .map_err(|err| AudioError::Decode(path.to_string(), err.to_string()))?;
                let sink = Sink::try_new(&self.handle).map_err(|err| AudioError::Sink(err.to_string()))?;
                sink.append(source);
                sink.play();
                self.sinks.insert(sound_id.to_string(), sink);
                Ok(())
        }

        fn stop(&mut self, sound_id: &str) {
                if let Some(sink) = self.sinks.remove(sound_id) {
                        sink.stop();
                }
        }

        fn stop_all(&mut self) {
                for (_, sink) in self.sinks.drain() {
                        sink.stop();
                }
        }
}

static AUDIO_ENGINE: Lazy<Mutex<AudioState>> = Lazy::new(|| {
        let state = AudioState::new().expect("Failed to start audio engine");
        Mutex::new(state)
});

#[tauri::command]
fn register_sound(label: String, path: String) -> Result<SoundDescriptor, String> {
        if !Path::new(&path).exists() {
                        return Err("Audio file not found at that path".to_string());
        }

        let display_label = if label.trim().is_empty() {
                Path::new(&path)
                        .file_stem()
                        .and_then(|name| name.to_str())
                        .unwrap_or("Untitled")
                        .to_string()
        } else {
                label
        };

        Ok(SoundDescriptor {
                id: Uuid::new_v4().to_string(),
                label: display_label,
                path,
        })
}

#[tauri::command]
fn play_sound(sound_id: String, path: String) -> Result<(), String> {
        let mut guard = AUDIO_ENGINE.lock().map_err(|_| AudioError::Poisoned.to_string())?;
        guard.play(&sound_id, &path).map_err(|err| err.to_string())
}

#[tauri::command]
fn stop_sound(sound_id: String) -> Result<(), String> {
        let mut guard = AUDIO_ENGINE.lock().map_err(|_| AudioError::Poisoned.to_string())?;
        guard.stop(&sound_id);
        Ok(())
}

#[tauri::command]
fn stop_all_sounds() -> Result<(), String> {
        let mut guard = AUDIO_ENGINE.lock().map_err(|_| AudioError::Poisoned.to_string())?;
        guard.stop_all();
        Ok(())
}

fn main() {
        tauri::Builder::default()
                .invoke_handler(tauri::generate_handler![
                        register_sound,
                        play_sound,
                        stop_sound,
                        stop_all_sounds
                ])
                .run(tauri::generate_context!())
                .expect("error while running tsukiko application");
}
