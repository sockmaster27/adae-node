#[macro_use]
extern crate lazy_static;

mod custom_output;
mod encapsulator;
mod mixer_track;
mod shared_engine;

use mixer_track::TrackDataWrapper;
use neon::prelude::*;

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
    cx.export_function("getDebugOutput", get_debug)?;
    Ok(())
}

/// The returned object must adhere to the interface defined in the `index.d.ts` file.
fn constructor(mut cx: FunctionContext) -> JsResult<JsObject> {
    fn construct_tracks<'a>(
        cx: &mut FunctionContext<'a>,
        engine: &SharedEngine,
    ) -> JsResult<'a, JsValue> {
        let track_keys = engine.unpack(cx, |_cx, engine| {
            let tracks = engine.tracks();
            let track_keys: Vec<u32> = tracks.iter().map(|track| track.key()).collect();
            Ok(track_keys)
        })?;

        let js_tracks = JsArray::new(cx, track_keys.len() as u32);
        for (i, &key) in track_keys.iter().enumerate() {
            let js_track = mixer_track::construct(cx, key, SharedEngine::clone(engine))?;
            js_tracks.set(cx, i as u32, js_track)?;
        }
        Ok(js_tracks.as_value(cx))
    }

    let engine = SharedEngine::new();
    let tracks = construct_tracks(&mut cx, &engine)?;

    let properties = &[("tracks", tracks)];
    let object = encapsulate(&mut cx, engine, properties, METHODS)?;
    prevent_gc(&mut cx, object)?;
    Ok(object)
}

// Closures are used to put declarations inside list, but they should be coerced to fns.
const METHODS: &[(&str, Method)] = &[
    ("getTrack", |mut cx| {
        let key_js: Handle<JsNumber> = cx.argument(0)?;
        let key = key_js.value(&mut cx) as u32;

        unpack(&mut cx, |cx, engine: &SharedEngine| {
            let track = mixer_track::construct(cx, key, SharedEngine::clone(engine))?;
            Ok(track.as_value(cx))
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

        let js_track = unpack(&mut cx, |cx, engine: &SharedEngine| {
            let key = engine.unpack(cx, |cx, engine| {
                let track = match data_option {
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
                Ok(track.key())
            })?;

            mixer_track::construct(cx, key, SharedEngine::clone(engine))
        })?;

        let object = cx.this();
        let tracks: Handle<JsArray> = object.get(&mut cx, "tracks")?;

        let end = tracks.len(&mut cx);
        tracks.set(&mut cx, end, js_track)?;

        Ok(js_track.as_value(&mut cx))
    }),
    ("close", |mut cx| {
        unpack(&mut cx, |cx, engine: &SharedEngine| {
            engine.close(cx)?;

            Ok(cx.undefined().as_value(cx))
        })
    }),
];
