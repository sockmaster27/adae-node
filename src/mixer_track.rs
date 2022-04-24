use neon::{prelude::*, result::Throw};

use crate::{
    encapsulator::{encapsulate, unpack, Method},
    shared_engine::SharedEngine,
};

pub struct JsTrack {
    key: u32,
    engine: SharedEngine,
}
impl JsTrack {
    /// The returned object must adhere to the interface defined in the `index.d.ts` file.
    pub fn construct<'a>(
        cx: &mut FunctionContext<'a>,
        key: u32,
        engine: SharedEngine,
    ) -> JsResult<'a, JsObject> {
        let js_track = JsTrack { key, engine };
        let properties = &[("key", cx.number(key).as_value(cx))];
        let methods: &[(&str, Method)] = &[
            ("setPanning", Self::set_panning),
            ("setVolume", Self::set_volume),
            ("readMeter", Self::read_meter),
            ("delete", Self::delete),
        ];
        let object = encapsulate(cx, js_track, properties, methods)?;

        Ok(object)
    }

    /// Fetch the track from the engine.
    fn unpack_track<'a, F, R>(cx: &mut CallContext<'a, JsObject>, callback: F) -> Result<R, Throw>
    where
        F: FnOnce(&mut CallContext<'a, JsObject>, &mut ardae::MixerTrack) -> Result<R, Throw>,
    {
        unpack(cx, |cx, js_track: &JsTrack| {
            let key = js_track.key;

            js_track.engine.unpack(cx, |cx, engine| {
                let track = match engine.track_mut(key) {
                    Ok(track) => track,
                    Err(_) => {
                        return cx.throw_error("Invalid track key");
                    }
                };
                callback(cx, track)
            })
        })
    }

    fn set_panning(mut cx: MethodContext<JsObject>) -> JsResult<JsValue> {
        let value_js: JsNumber = *cx.argument(0)?;
        let value: f32 = value_js.value(&mut cx) as f32;

        Self::unpack_track(&mut cx, |cx, track| {
            track.panning.set(value);
            Ok(cx.undefined().as_value(cx))
        })
    }

    fn set_volume(mut cx: MethodContext<JsObject>) -> JsResult<JsValue> {
        let value_js: JsNumber = *cx.argument(0)?;
        let value: f32 = value_js.value(&mut cx) as f32;

        Self::unpack_track(&mut cx, |cx, track| {
            track.volume.set(value);
            Ok(cx.undefined().as_value(cx))
        })
    }

    fn read_meter(mut cx: MethodContext<JsObject>) -> JsResult<JsValue> {
        Self::unpack_track(&mut cx, |cx, track| {
            let [peak, long_peak, rms] = track.meter.read();
            let peak_js = cx.empty_array();
            let long_peak_js = cx.empty_array();
            let rms_js = cx.empty_array();

            for (thing, thing_js) in [(peak, peak_js), (long_peak, long_peak_js), (rms, rms_js)] {
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
    }

    fn delete(mut cx: MethodContext<JsObject>) -> JsResult<JsValue> {
        unpack(&mut cx, |cx, js_track: &JsTrack| {
            let key = js_track.key;

            js_track.engine.unpack(cx, |cx, engine| {
                let result = engine.delete_track(key);
                if let Err(_) = result {
                    return cx.throw_error("Invalid track key");
                }
                Ok(cx.undefined().as_value(cx))
            })
        })
    }
}

impl Finalize for JsTrack {}
