//! Binding for Node.js' native addon API.

#[macro_use]
extern crate lazy_static;

mod custom_output;
mod encapsulator;
mod shared_engine;
mod timestamp;
mod track;

use neon::prelude::*;
use track::{audio_track, master, AudioTrackStateWrapper};

use custom_output::get_debug;
#[cfg(feature = "custom_debug_output")]
use custom_output::output_debug;
use encapsulator::{encapsulate, prevent_gc, unpack, unpack_this, Method};
use shared_engine::SharedEngine;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    #[cfg(feature = "custom_debug_output")]
    ardae::set_output(output_debug);

    cx.export_function("Engine", constructor)?;

    cx.export_function("meterScale", meter_scale)?;
    cx.export_function("inverseMeterScale", inverse_meter_scale)?;
    cx.export_function("getDebugOutput", get_debug)?;

    let timestamp_class = timestamp::class(&mut cx)?;
    cx.export_value("Timestamp", timestamp_class)?;

    Ok(())
}

/// The returned object must adhere to the interface defined in the `index.d.ts` file.
fn constructor(mut cx: FunctionContext) -> JsResult<JsObject> {
    let shared_engine = SharedEngine::new();
    let object = encapsulate(&mut cx, shared_engine, &[], METHODS)?;
    prevent_gc(&mut cx, object)?;
    Ok(object)
}

// Closures are used to put declarations inside list, but they should be coerced to fns.
const METHODS: &[(&str, Method)] = &[
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
    ("importAudioClip", |mut _cx| todo!("importAudioClip")),
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

    let result = ardae::meter_scale(value);
    let result_js = cx.number(result);

    Ok(result_js)
}

fn inverse_meter_scale(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let value_js: Handle<JsNumber> = cx.argument(0)?;
    let value = value_js.value(&mut cx) as f32;

    let result = ardae::inverse_meter_scale(value);
    let result_js = cx.number(result);

    Ok(result_js)
}
