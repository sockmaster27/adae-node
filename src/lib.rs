//! Binding for Node.js' native addon API.

#[macro_use]
extern crate lazy_static;

mod custom_output;
mod encapsulator;
mod shared_engine;
mod timeline_track;
mod timestamp;
mod track;

use std::path::Path;

use neon::prelude::*;
use track::{delete_track, TrackDataWrapper};

use custom_output::get_debug;
#[cfg(feature = "custom_debug_output")]
use custom_output::output_debug;
use encapsulator::{encapsulate, prevent_gc, unpack, Method};
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
    let timeline_object = encapsulate(
        &mut cx,
        SharedEngine::clone(&shared_engine),
        &[],
        TIMELINE_METHODS,
    )?
    .as_value(&mut cx);
    let object = encapsulate(
        &mut cx,
        shared_engine,
        &[("timeline", timeline_object)],
        METHODS,
    )?;
    prevent_gc(&mut cx, object)?;
    Ok(object)
}

// Closures are used to put declarations inside list, but they should be coerced to fns.
const METHODS: &[(&str, Method)] = &[
    ("getMaster", |mut cx| {
        unpack(&mut cx, |cx, shared_engine: &SharedEngine| {
            let track = track::construct_master(cx, SharedEngine::clone(shared_engine))?;
            Ok(track.as_value(cx))
        })
    }),
    ("getTracks", |mut cx| {
        unpack(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.with_inner(cx, |cx, engine| {
                let tracks = engine.tracks();
                let track_keys = tracks.iter().map(|track| track.key());

                let length = track_keys
                    .len()
                    .try_into()
                    .or_else(|_| cx.throw_error("Too many tracks to fit into array"))?;
                let js_tracks = JsArray::new(cx, length);

                for (i, key) in track_keys.enumerate() {
                    let js_track =
                        track::construct_track(cx, key, SharedEngine::clone(shared_engine))?;
                    js_tracks.set(cx, i as u32, js_track)?;
                }
                Ok(js_tracks.as_value(cx))
            })
        })
    }),
    ("getTrack", |mut cx| {
        let key_js: Handle<JsNumber> = cx.argument(0)?;
        let key = key_js.value(&mut cx) as u32;

        unpack(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.with_inner(cx, |cx, engine| {
                // Check if track exists
                engine
                    .track(key)
                    .or_else(|e| cx.throw_error(format!("{}", e)))?;

                let track = track::construct_track(cx, key, SharedEngine::clone(shared_engine))?;
                Ok(track.as_value(cx))
            })
        })
    }),
    ("addTrack", |mut cx| {
        let mut data_option = cx.argument_opt(0);
        if let Some(data) = data_option {
            let is_undefined = data.is_a::<JsUndefined, _>(&mut cx);
            let is_null = data.is_a::<JsNull, _>(&mut cx);
            if is_undefined || is_null {
                data_option = None;
            }
        }

        unpack(&mut cx, |cx, shared_engine: &SharedEngine| {
            let key = shared_engine.with_inner(cx, |cx, engine| {
                let key = match data_option {
                    None => engine
                        .add_track()
                        .or_else(|e| cx.throw_error(format!("{}", e))),
                    Some(data_value) => {
                        let boxed_data: Handle<JsBox<TrackDataWrapper>> =
                            data_value.downcast_or_throw(cx)?;
                        let data = &**boxed_data;
                        engine
                            .reconstruct_track(data)
                            .or_else(|e| cx.throw_error(format!("{}", e)))
                    }
                }?;
                Ok(key)
            })?;

            let js_track = track::construct_track(cx, key, SharedEngine::clone(shared_engine))?;

            Ok(js_track.as_value(cx))
        })
    }),
    ("addAudioTrack", |mut cx| {
        unpack(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.with_inner(cx, |cx, engine| {
                let (tl_track_key, track_key) = engine
                    .add_audio_track()
                    .or_else(|e| cx.throw_error(format! {"{}", &e}))?;

                let tl_track_obj = timeline_track::construct(
                    cx,
                    tl_track_key,
                    SharedEngine::clone(&shared_engine),
                )?;
                let track_obj =
                    track::construct_track(cx, track_key, SharedEngine::clone(&shared_engine))?;

                let res = JsArray::new(cx, 2);
                res.set(cx, 0, tl_track_obj)?;
                res.set(cx, 1, track_obj)?;
                Ok(res.as_value(cx))
            })
        })
    }),
    ("addTracks", |mut cx| {
        // Determine which overload is used
        // (count: number or data: TrackData[])
        let arg: Handle<JsValue> = cx.argument(0)?;
        let is_count = arg.is_a::<JsNumber, _>(&mut cx);
        let is_data = arg.is_a::<JsArray, _>(&mut cx);
        unpack(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.with_inner(cx, |cx, engine| {
                let keys = if is_count {
                    let count_js: Handle<JsNumber> = arg.downcast_or_throw(cx)?;
                    let count = count_js.value(cx) as u32;

                    engine
                        .add_tracks(count)
                        .or_else(|e| cx.throw_error(format!("{}", &e)))
                } else if is_data {
                    let data_js_array: Handle<JsArray> = arg.downcast_or_throw(cx)?;
                    let data_js = data_js_array.to_vec(cx)?;

                    let mut data = Vec::with_capacity(data_js.len());
                    for value in data_js {
                        let boxed_data: Handle<JsBox<TrackDataWrapper>> =
                            value.downcast_or_throw(cx)?;
                        data.push(ardae::TrackData::clone(&***boxed_data));
                    }

                    engine
                        .reconstruct_tracks(data.iter())
                        .or_else(|e| cx.throw_error(format!("{}", &e)))
                } else {
                    cx.throw_type_error("Argument not of type `number` or `TrackData[]`")
                }?;

                let length = keys
                    .len()
                    .try_into()
                    .or_else(|_| cx.throw_error("Too many tracks to fit into array"))?;
                let new_tracks = JsArray::new(cx, length);
                for (i, &key) in keys.iter().enumerate() {
                    let track =
                        track::construct_track(cx, key, SharedEngine::clone(shared_engine))?;
                    let index_js = cx.number(i as f64);
                    new_tracks.set(cx, index_js, track)?;
                }

                Ok(new_tracks.as_value(cx))
            })
        })
    }),
    ("deleteTrack", |mut cx| {
        let track_js: Handle<JsObject> = cx.argument(0)?;
        let key_js: Handle<JsNumber> = track_js.get(&mut cx, "key")?;
        let key = key_js.value(&mut cx) as u32;
        let boxed_data = delete_track(&mut cx, key)?;
        Ok(boxed_data.as_value(&mut cx))
    }),
    ("deleteTracks", |mut cx| {
        let tracks_js_array: Handle<JsArray> = cx.argument(0)?;
        let tracks_js = tracks_js_array.to_vec(&mut cx)?;

        let mut keys = Vec::with_capacity(tracks_js.len());
        for value in tracks_js {
            let track_js: Handle<JsObject> = value.downcast_or_throw(&mut cx)?;
            let key_js: Handle<JsNumber> = track_js.get(&mut cx, "key")?;
            let key = key_js.value(&mut cx) as u32;
            keys.push(key);
        }

        unpack(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.with_inner(cx, |cx, engine| {
                let length = keys
                    .len()
                    .try_into()
                    .or_else(|_| cx.throw_error("Too many tracks to fit into array"))?;
                let data_array = JsArray::new(cx, length);

                for (i, &key) in keys.iter().enumerate() {
                    let track = engine
                        .track(key)
                        .or_else(|e| cx.throw_error(format! {"{}", &e}))?;
                    let data = TrackDataWrapper(track.data());
                    let boxed_data = cx.boxed(data);

                    let index_js = cx.number(i as f64);
                    data_array.set(cx, index_js, boxed_data)?;
                }

                engine
                    .delete_tracks(keys)
                    .or_else(|e| cx.throw_error(format! {"{}", &e}))?;

                Ok(data_array.as_value(cx))
            })
        })
    }),
    ("close", |mut cx| {
        unpack(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.close(cx)?;

            Ok(cx.undefined().as_value(cx))
        })
    }),
];

const TIMELINE_METHODS: &[(&str, Method)] = &[("importAudioClip", |mut cx| {
    unpack(&mut cx, |cx, shared_engine: &SharedEngine| {
        shared_engine.with_inner(cx, |cx, engine| {
            let path_js: Handle<JsString> = cx.argument(0)?;
            let path = path_js.value(cx);

            let clip_key = engine
                .timeline_mut()
                .import_audio_clip(&Path::new(&path))
                .or_else(|e| cx.throw_error(format! {"{}", &e}))?;

            Ok(cx.number(clip_key).as_value(cx))
        })
    })
})];

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
