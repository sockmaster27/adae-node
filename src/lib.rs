//! Binding for Node.js' native addon API.

mod config;
mod custom_output;
mod encapsulator;
mod panic_handling;
mod shared_engine;
mod stored_clip;
mod timestamp;
mod track;

use std::path::Path;

use neon::prelude::*;

use custom_output::get_debug;
#[cfg(feature = "custom_debug_output")]
use custom_output::output_debug;
use encapsulator::{encapsulate, prevent_gc, unpack, unpack_this, Method};
use panic_handling::{listen_for_crash, stop_listening_for_crash};
use shared_engine::SharedEngine;
use stored_clip::stored_audio_clip;
use timestamp::timestamp;
use track::{audio_track, master, AudioTrackStateWrapper};

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    #[cfg(feature = "custom_debug_output")]
    adae::set_output(output_debug);

    let engine_class = JsFunction::new(&mut cx, constructor)?;
    for (name, method) in STATIC_METHODS {
        let method_js = JsFunction::new(&mut cx, *method)?;
        engine_class.set(&mut cx, *name, method_js)?;
    }
    cx.export_value("Engine", engine_class)?;

    cx.export_function("meterScale", meter_scale)?;
    cx.export_function("inverseMeterScale", inverse_meter_scale)?;
    cx.export_function("getDebugOutput", get_debug)?;
    cx.export_function("listenForCrash", listen_for_crash)?;
    cx.export_function("stopListeningForCrash", stop_listening_for_crash)?;

    let timestamp_class = timestamp::class(&mut cx)?;
    cx.export_value("Timestamp", timestamp_class)?;

    let config_module = config::module(&mut cx)?;
    cx.export_value("config", config_module)?;

    Ok(())
}

/// The returned object must adhere to the interface defined in the `index.d.ts` file.
fn constructor(mut cx: FunctionContext) -> JsResult<JsObject> {
    let config_js = cx.argument_opt(0);
    let shared_engine = match config_js {
        Some(config_js) => {
            let config_obj: Handle<JsObject> = config_js.downcast_or_throw(&mut cx)?;
            config::config_class::unpack(&mut cx, config_obj, |_, config| {
                let (engine, import_errors) = SharedEngine::new(config);
                debug_assert!(
                    import_errors.is_empty(),
                    "Import errors: {:?}",
                    import_errors
                );
                Ok(engine)
            })?
        }
        None => SharedEngine::empty(),
    };
    let object = encapsulate(&mut cx, shared_engine, &[], METHODS)?;
    prevent_gc(&mut cx, object)?;
    Ok(object)
}

const STATIC_METHODS: &[(&str, Method)] = &[("dummy", |mut cx| {
    let shared_engine = SharedEngine::dummy();
    let object = encapsulate(&mut cx, shared_engine, &[], METHODS)?;
    prevent_gc(&mut cx, object)?;
    Ok(object.as_value(&mut cx))
})];
const METHODS: &[(&str, Method)] = &[
    ("setConfig", |mut cx| {
        let config_js = cx.argument(0)?;
        config::config_class::unpack(&mut cx, config_js, |cx, config| {
            unpack_this(cx, |cx, shared_engine: &SharedEngine| {
                shared_engine.with_inner(cx, |cx, engine| {
                    engine.set_config(config);
                    Ok(cx.undefined().as_value(cx))
                })
            })
        })
    }),
    ("play", |mut cx| {
        unpack_this(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.with_inner(cx, |cx, engine| {
                engine.play();
                Ok(cx.undefined().as_value(cx))
            })
        })
    }),
    ("pause", |mut cx| {
        unpack_this(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.with_inner(cx, |cx, engine| {
                engine.pause();
                Ok(cx.undefined().as_value(cx))
            })
        })
    }),
    ("jumpTo", |mut cx| {
        let timestamp_js: Handle<JsObject> = cx.argument(0)?;
        let timestamp = timestamp(&mut cx, timestamp_js)?;

        unpack_this(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.with_inner(cx, |cx, engine| {
                engine.jump_to(timestamp);
                Ok(cx.undefined().as_value(cx))
            })
        })
    }),
    ("getPlayheadPosition", |mut cx| {
        unpack_this(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.with_inner(cx, |cx, engine| {
                let timestamp = timestamp::construct(cx, engine.playhead_position())?;
                Ok(timestamp.as_value(cx))
            })
        })
    }),
    ("getMaster", |mut cx| {
        unpack_this(&mut cx, |cx, shared_engine: &SharedEngine| {
            let track = master::construct(cx, SharedEngine::clone(shared_engine))?;
            Ok(track.as_value(cx))
        })
    }),
    ("getAudioTracks", |mut cx| {
        unpack_this(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.with_inner(cx, |cx, engine| {
                let tracks = engine.audio_tracks();
                let js_tracks = cx.empty_array();

                for (i, track) in tracks.enumerate() {
                    let js_track = audio_track::construct(
                        cx,
                        track.clone(),
                        SharedEngine::clone(shared_engine),
                    )?;
                    js_tracks.set(cx, i as u32, js_track)?;
                }
                Ok(js_tracks.as_value(cx))
            })
        })
    }),
    ("addAudioTrack", |mut cx| {
        unpack_this(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.with_inner(cx, |cx, engine| {
                let audio_track = engine
                    .add_audio_track()
                    .or_else(|e| cx.throw_error(format!("{}", e)))?;

                let js_track =
                    audio_track::construct(cx, audio_track, SharedEngine::clone(shared_engine))?;

                Ok(js_track.as_value(cx))
            })
        })
    }),
    ("addAudioTracks", |mut cx| {
        let count_js: Handle<JsNumber> = cx.argument(0)?;
        let count = count_js.value(&mut cx) as u32;

        unpack_this(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.with_inner(cx, |cx, engine| {
                let tracks = engine
                    .add_audio_tracks(count)
                    .or_else(|e| cx.throw_error(format!("{}", &e)))?;

                let new_tracks = cx.empty_array();
                for (i, track) in tracks.enumerate() {
                    let track =
                        audio_track::construct(cx, track, SharedEngine::clone(shared_engine))?;
                    let index_js = cx.number(i as f64);
                    new_tracks.set(cx, index_js, track)?;
                }

                Ok(new_tracks.as_value(cx))
            })
        })
    }),
    ("deleteAudioTrack", |mut cx| {
        let track_js: Handle<JsObject> = cx.argument(0)?;
        let track = audio_track::unpack_audio_track(&mut cx, track_js)?;
        unpack_this(&mut cx, |cx, shared_engine: &SharedEngine| {
            let boxed_state = audio_track::delete(cx, shared_engine, track)?;
            Ok(boxed_state.as_value(cx))
        })
    }),
    ("deleteAudioTracks", |mut cx| {
        let tracks_js_array: Handle<JsArray> = cx.argument(0)?;
        let tracks_js = tracks_js_array.to_vec(&mut cx)?;

        let tracks = tracks_js
            .into_iter()
            .map(|value| {
                let track_js: Handle<JsObject> = value.downcast_or_throw(&mut cx)?;
                let track = audio_track::unpack_audio_track(&mut cx, track_js)?;
                Ok(track)
            })
            .collect::<Result<Vec<_>, _>>()?;

        unpack_this(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.with_inner(cx, |cx, engine| {
                let length = tracks
                    .len()
                    .try_into()
                    .or_else(|_| cx.throw_error("Too many tracks to fit into array"))?;
                let state_array = JsArray::new(cx, length);

                for (i, track) in tracks.iter().enumerate() {
                    let state = engine
                        .audio_track_state(track)
                        .or_else(|e| cx.throw_error(format! {"{}", &e}))?;
                    let state_js = encapsulate(cx, AudioTrackStateWrapper(state), &[], &[])?;

                    let index_js = cx.number(i as f64);
                    state_array.set(cx, index_js, state_js)?;
                }

                engine
                    .delete_audio_tracks(tracks)
                    .or_else(|e| cx.throw_error(format! {"{}", &e}))?;

                Ok(state_array.as_value(cx))
            })
        })
    }),
    ("reconstructAudioTrack", |mut cx| {
        let state_js: Handle<JsObject> = cx.argument(0)?;
        unpack(&mut cx, state_js, |cx, state: &AudioTrackStateWrapper| {
            unpack_this(cx, |cx, shared_engine: &SharedEngine| {
                shared_engine.with_inner(cx, |cx, engine| {
                    let track = engine
                        .reconstruct_audio_track(state.0.clone())
                        .or_else(|e| cx.throw_error(format! {"{}", &e}))?;
                    let track_js =
                        audio_track::construct(cx, track, SharedEngine::clone(shared_engine))?;
                    Ok(track_js.as_value(cx))
                })
            })
        })
    }),
    ("reconstructAudioTracks", |mut cx| {
        let states_js_array: Handle<JsArray> = cx.argument(0)?;
        let states_js = states_js_array.to_vec(&mut cx)?;
        let states = states_js
            .iter()
            .map(|value| {
                let state_js: Handle<JsObject> = value.downcast_or_throw(&mut cx)?;
                let state = unpack(&mut cx, state_js, |_cx, state: &AudioTrackStateWrapper| {
                    Ok(state.0.clone())
                })?;
                Ok(state)
            })
            .collect::<Result<Vec<_>, _>>()?;

        unpack_this(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.with_inner(cx, |cx, engine| {
                let tracks = engine
                    .reconstruct_audio_tracks(states)
                    .or_else(|e| cx.throw_error(format! {"{}", &e}))?;

                let tracks_js = cx.empty_array();
                for (i, track) in tracks.into_iter().enumerate() {
                    let track_js =
                        audio_track::construct(cx, track, SharedEngine::clone(shared_engine))?;
                    let index_js = cx.number(i as f64);
                    tracks_js.set(cx, index_js, track_js)?;
                }

                Ok(tracks_js.as_value(cx))
            })
        })
    }),
    ("importAudioClip", |mut cx| {
        let path_js: Handle<JsString> = cx.argument(0)?;
        let path = path_js.value(&mut cx);

        unpack_this(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.with_inner(cx, |cx, engine| {
                let clip = engine
                    .import_audio_clip(Path::new(&path))
                    .or_else(|e| cx.throw_error(format! {"{}", &e}))?;
                let clip_js =
                    stored_audio_clip::construct(cx, clip, SharedEngine::clone(shared_engine))?;
                Ok(clip_js.as_value(cx))
            })
        })
    }),
    ("close", |mut cx| {
        unpack_this(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.close(cx)?;

            Ok(cx.undefined().as_value(cx))
        })
    }),
];

fn meter_scale(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let value_js: Handle<JsNumber> = cx.argument(0)?;
    let value = value_js.value(&mut cx) as f32;

    let result = adae::meter_scale(value);
    let result_js = cx.number(result);

    Ok(result_js)
}

fn inverse_meter_scale(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let value_js: Handle<JsNumber> = cx.argument(0)?;
    let value = value_js.value(&mut cx) as f32;

    let result = adae::inverse_meter_scale(value);
    let result_js = cx.number(result);

    Ok(result_js)
}
