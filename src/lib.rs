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
        let methods: &[(&str, Method)] = &[
            ("addTrack", Self::add_track),
            ("setVolume", Self::set_volume),
            ("setPanning", Self::set_panning),
            ("getMeter", Self::get_meter),
            ("close", Self::close),
        ];
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

    fn set_volume(mut cx: MethodContext<JsObject>) -> JsResult<JsValue> {
        let value_js: JsNumber = *cx.argument(0)?;
        let value: f32 = value_js.value(&mut cx) as f32;

        unpack(&mut cx, |cx, js_engine: &JsEngine| {
            js_engine.engine.unpack(cx, |cx, engine| {
                engine.set_volume(value);
                Ok(cx.undefined().as_value(cx))
            })
        })
    }

    fn set_panning(mut cx: MethodContext<JsObject>) -> JsResult<JsValue> {
        let value_js: JsNumber = *cx.argument(0)?;
        let value: f32 = value_js.value(&mut cx) as f32;

        unpack(&mut cx, |cx, js_engine: &JsEngine| {
            js_engine.engine.unpack(cx, |cx, engine| {
                engine.set_panning(value);

                Ok(cx.undefined().as_value(cx))
            })
        })
    }

    fn get_meter(mut cx: MethodContext<JsObject>) -> JsResult<JsValue> {
        unpack(&mut cx, |cx, js_engine: &JsEngine| {
            js_engine.engine.unpack(cx, |cx, engine| {
                let [peak, long_peak, rms] = engine.get_meter();
                let peak_js = cx.empty_array();
                let long_peak_js = cx.empty_array();
                let rms_js = cx.empty_array();

                for (thing, thing_js) in [(peak, peak_js), (long_peak, long_peak_js), (rms, rms_js)]
                {
                    for (i, val) in thing.iter().enumerate() {
                        let index_js = cx.number(i as f64);
                        let val_js = cx.number(*val);
                        thing_js.set(cx, index_js, val_js)?;
                    }
                }

                let meter_js = cx.empty_object();
                meter_js.set(cx, "peak", peak_js)?;
                meter_js.set(cx, "longPeak", long_peak_js)?;
                meter_js.set(cx, "rms", rms_js)?;
                Ok(meter_js.as_value(cx))
            })
        })
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
