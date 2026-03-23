use js_sys::Array;
use leptos::leptos_dom::ev::MouseEvent;
use leptos::*;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use std::path::Path;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
        async fn invoke(cmd: &str, args: JsValue) -> JsValue;

        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "dialog"])]
        async fn open(options: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct SoundDescriptor {
        id: String,
        label: String,
        path: String,
}

#[derive(Serialize, Deserialize)]
struct RegisterSoundArgs<'a> {
        label: &'a str,
        path: &'a str,
}

#[derive(Serialize, Deserialize)]
struct PlaySoundArgs<'a> {
        sound_id: &'a str,
        path: &'a str,
}

#[derive(Serialize, Deserialize)]
struct StopSoundArgs<'a> {
        sound_id: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SoundEntry {
        descriptor: SoundDescriptor,
        is_playing: bool,
}

fn file_stem_as_label(path: &str) -> String {
        Path::new(path)
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("Untitled")
                .to_string()
}

fn extract_selected_path(selection: JsValue) -> Option<String> {
        if selection.is_null() || selection.is_undefined() {
                return None;
        }

        if let Some(path) = selection.as_string() {
                return Some(path);
        }

        if let Ok(array) = selection.dyn_into::<Array>() {
                return array.iter().next().and_then(|value| value.as_string());
        }

        None
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
        let sounds = create_rw_signal(cx, Vec::<SoundEntry>::new());
        let status = create_rw_signal(cx, String::new());
        let is_busy = create_rw_signal(cx, false);

        let choose_sound = move |ev: MouseEvent| {
                ev.prevent_default();
                if is_busy.get() {
                        return;
                }

                spawn_local(async move {
                        is_busy.set(true);
                        status.set(String::new());

                        let dialog_options = js_sys::Object::new();
                        let filters = Array::new();
                        let filter = js_sys::Object::new();
                        js_sys::Reflect::set(&filter, &"name".into(), &"Audio".into()).ok();
                        let extensions = Array::new();
                        ["mp3", "wav", "flac", "ogg", "aac", "m4a", "mp4"]
                                .iter()
                                .for_each(|ext| {
                                        extensions.push(&JsValue::from_str(ext));
                                });
                        js_sys::Reflect::set(&filter, &"extensions".into(), &extensions.into()).ok();
                        filters.push(&filter);
                        js_sys::Reflect::set(&dialog_options, &"filters".into(), &filters.into()).ok();

                        let selection = open(dialog_options.into()).await;
                        let Some(path) = extract_selected_path(selection) else {
                                status.set("No file selected".to_string());
                                is_busy.set(false);
                                return;
                        };

                        let label = file_stem_as_label(&path);
                        let args = to_value(&RegisterSoundArgs { label: &label, path: &path }).unwrap();

                        let Ok(new_sound) = invoke("register_sound", args)
                                .await
                                .into_serde::<SoundDescriptor>()
                        else {
                                status.set("Unable to register sound".to_string());
                                is_busy.set(false);
                                return;
                        };

                        sounds.update(|entries| {
                                entries.push(SoundEntry { descriptor: new_sound, is_playing: false });
                        });
                        status.set("Sound added".to_string());
                        is_busy.set(false);
                });
        };

        let play_sound = move |sound: SoundEntry| {
            let sounds = sounds.clone();
            let status = status.clone();
            spawn_local(async move {
                    let args = to_value(&PlaySoundArgs { sound_id: &sound.descriptor.id, path: &sound.descriptor.path }).unwrap();
                    if invoke("play_sound", args).await.is_undefined() {
                            status.set("Failed to start playback".to_string());
                            return;
                    }

                    sounds.update(|entries| {
                            if let Some(entry) = entries.iter_mut().find(|entry| entry.descriptor.id == sound.descriptor.id) {
                                    entry.is_playing = true;
                            }
                    });
                    status.set(format!("Playing {}", sound.descriptor.label));
            });
        };

        let stop_sound = move |sound: SoundEntry| {
            let sounds = sounds.clone();
            let status = status.clone();
            spawn_local(async move {
                    let args = to_value(&StopSoundArgs { sound_id: &sound.descriptor.id }).unwrap();
                    if invoke("stop_sound", args).await.is_undefined() {
                            status.set("Failed to stop sound".to_string());
                            return;
                    }

                    sounds.update(|entries| {
                            if let Some(entry) = entries.iter_mut().find(|entry| entry.descriptor.id == sound.descriptor.id) {
                                    entry.is_playing = false;
                            }
                    });
                    status.set(format!("Stopped {}", sound.descriptor.label));
            });
        };

        let stop_all = move |ev: MouseEvent| {
                ev.prevent_default();
                let sounds = sounds.clone();
                let status = status.clone();
                spawn_local(async move {
                        if invoke("stop_all_sounds", JsValue::NULL).await.is_undefined() {
                                status.set("No sounds to stop".to_string());
                                return;
                        }
                        sounds.update(|entries| {
                                for entry in entries.iter_mut() {
                                        entry.is_playing = false;
                                }
                        });
                        status.set("Stopped every sound".to_string());
                });
        };

        let remove_sound = move |sound: SoundEntry| {
                let sounds = sounds.clone();
                let status = status.clone();
                spawn_local(async move {
                        let args = to_value(&StopSoundArgs { sound_id: &sound.descriptor.id }).unwrap();
                        let _ = invoke("stop_sound", args).await;
                        sounds.update(|entries| {
                                entries.retain(|entry| entry.descriptor.id != sound.descriptor.id);
                        });
                        status.set(format!("Removed {}", sound.descriptor.label));
                });
        };

        view! { cx,
                <main class="page">
                        <section class="hero">
                                <div>
                                        <p class="eyebrow">"Cross-platform soundboard"</p>
                                        <h1>"Tsukiko"<span class="accent">" Soundboard"</span></h1>
                                        <p class="lede">
                                                "Drop in your favorite audio clips and play them instantly with a layout designed for focus."
                                        </p>
                                        <div class="actions">
                                                <button class="primary" on:click=choose_sound disabled=move || is_busy.get()>
                                                        { move || if is_busy.get() { "Loading..." } else { "Add sound" } }
                                                </button>
                                                <button class="ghost" on:click=stop_all>"Stop all"</button>
                                        </div>
                                        <Show when=move || !status.get().is_empty() fallback=|| view! { cx, () }>
                                                <p class="status">{ move || status.get() }</p>
                                        </Show>
                                </div>
                        </section>

                        <section class="sound-grid">
                                <Show
                                        when=move || sounds.get().is_empty()
                                        fallback=move || {
                                                view! { cx,
                                                        <For
                                                                each=move || sounds.get()
                                                                key=|sound| sound.descriptor.id.clone()
                                                                view=move |cx, sound: SoundEntry| {
                                                                        let is_playing = sound.is_playing;
                                                                        let sound_clone_for_toggle = sound.clone();
                                                                        let sound_clone_for_remove = sound.clone();
                                                                        let on_toggle = move |ev: MouseEvent| {
                                                                                ev.prevent_default();
                                                                                if is_playing {
                                                                                        stop_sound(sound_clone_for_toggle.clone());
                                                                                } else {
                                                                                        play_sound(sound_clone_for_toggle.clone());
                                                                                }
                                                                        };
                                                                        let on_remove = move |ev: MouseEvent| {
                                                                                ev.prevent_default();
                                                                                remove_sound(sound_clone_for_remove.clone());
                                                                        };
                                                                        view! { cx,
                                                                                <article class="sound-card">
                                                                                        <div class="sound-card__header">
                                                                                                <div>
                                                                                                        <p class="eyebrow">{ if is_playing { "Playing" } else { "Ready" } }</p>
                                                                                                        <h3>{ sound.descriptor.label.clone() }</h3>
                                                                                                </div>
                                                                                                <span class=move || format!("pill {}", if is_playing { "pill--playing" } else { "pill--idle" })>
                                                                                                        { if is_playing { "Live" } else { "Idle" } }
                                                                                                </span>
                                                                                        </div>
                                                                                        <p class="sound-card__path">{ sound.descriptor.path.clone() }</p>
                                                                                        <div class="sound-card__actions">
                                                                                                <button class=move || if is_playing { "secondary danger" } else { "primary" } on:click=on_toggle>
                                                                                                        { if is_playing { "Stop" } else { "Play" } }
                                                                                                </button>
                                                                                                <button class="ghost" on:click=on_remove>"Remove"</button>
                                                                                        </div>
                                                                                </article>
                                                                        }
                                                                }
                                                        />
                                                }
                                        }
                                >
                                        <div class="empty">
                                                <h3>"No sounds yet"</h3>
                                                <p>"Tap \"Add sound\" to browse for audio files and build your board."</p>
                                        </div>
                                </Show>
                        </section>
                </main>
        }
}
