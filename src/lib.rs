mod encapsulator;
mod mixer_track;
mod shared_engine;

use neon::prelude::*;

use encapsulator::{encapsulate, prevent_gc, unpack, Method};
use mixer_track::JsTrack;
use shared_engine::SharedEngine;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("Engine", JsEngine::constructor)?;
    Ok(())
}

/// A wrapper around the `ardae::Engine` compatible with Neon's API.
///
/// Note that even though its functions are organized as being associated to the `JsEngine`,
/// most of them are supposed to be exposed directly by Neon, and do not follow rust conventions.
struct JsEngine {
    engine: SharedEngine,
}
impl JsEngine {
    /// The returned object must adhere to the interface defined in the `index.d.ts` file.
    fn constructor(mut cx: FunctionContext) -> JsResult<JsObject> {
        let engine = SharedEngine::new();

        let js_engine = JsEngine { engine };
        let tracks = Self::construct_tracks(&mut cx, &js_engine)?;

        let properties = &[("tracks", tracks)];
        let methods: &[(&str, Method)] = &[("addTrack", Self::add_track), ("close", Self::close)];
        let object = encapsulate(&mut cx, js_engine, properties, methods)?;
        prevent_gc(&mut cx, object)?;
        Ok(object)
    }

    fn construct_tracks<'a>(
        cx: &mut FunctionContext<'a>,
        js_engine: &JsEngine,
    ) -> JsResult<'a, JsValue> {
        let track_keys = js_engine.engine.unpack(cx, |_cx, engine| {
            let tracks = engine.tracks();
            let track_keys: Vec<u32> = tracks.iter().map(|track| track.key()).collect();
            Ok(track_keys)
        })?;

        let js_tracks = JsArray::new(cx, track_keys.len() as u32);
        for (i, &key) in track_keys.iter().enumerate() {
            let js_track = JsTrack::construct(cx, key, SharedEngine::clone(&js_engine.engine))?;
            js_tracks.set(cx, i as u32, js_track)?;
        }
        Ok(js_tracks.as_value(cx))
    }

    fn add_track(mut cx: MethodContext<JsObject>) -> JsResult<JsValue> {
        let js_track = unpack(&mut cx, |cx, js_engine: &JsEngine| {
            let key = js_engine.engine.unpack(cx, |cx, engine| {
                let result = engine.add_track();
                match result {
                    Ok(track) => Ok(track.key()),
                    Err(_) => cx.throw_error("Max number of tracks reached"),
                }
            })?;

            JsTrack::construct(cx, key, SharedEngine::clone(&js_engine.engine))
        })?;

        let object = cx.this();
        let tracks: Handle<JsArray> = object.get(&mut cx, "tracks")?.downcast_or_throw(&mut cx)?;
        let end = tracks.len(&mut cx);
        tracks.set(&mut cx, end, js_track)?;

        Ok(js_track.as_value(&mut cx))
    }

    fn close(mut cx: MethodContext<JsObject>) -> JsResult<JsValue> {
        unpack(&mut cx, |cx, js_engine: &JsEngine| {
            js_engine.engine.close(cx)?;

            Ok(cx.undefined().as_value(cx))
        })
    }
}

// This just has to be here.
impl Finalize for JsEngine {}
