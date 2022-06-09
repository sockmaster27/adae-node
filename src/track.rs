use std::ops::Deref;

use neon::prelude::*;

use crate::{
    encapsulator::{encapsulate, unpack, Method},
    shared_engine::SharedEngine,
};

/// The returned object must adhere to the interface defined in the `index.d.ts` file.
pub fn construct<'a>(
    cx: &mut FunctionContext<'a>,
    key: u32,
    engine: SharedEngine,
) -> JsResult<'a, JsObject> {
    let properties = &[("key", cx.number(key).as_value(cx))];
    let object = encapsulate(cx, engine, properties, METHODS)?;

    Ok(object)
}

/// Fetch the track from the engine.
fn unpack_track<'a, F, R>(cx: &mut CallContext<'a, JsObject>, callback: F) -> NeonResult<R>
where
    F: FnOnce(&mut CallContext<'a, JsObject>, &mut ardae::Track) -> NeonResult<R>,
{
    let key_js: Handle<JsNumber> = cx.this().get(cx, "key")?;
    let key = key_js.value(cx) as u32;

    unpack(cx, |cx, shared_engine: &SharedEngine| {
        shared_engine.with_inner(cx, |cx, engine| {
            let track = match engine.track_mut(key) {
                Ok(track) => track,
                Err(_) => {
                    return cx.throw_error("Track has been deleted");
                }
            };
            callback(cx, track)
        })
    })
}

pub fn delete_track<'a>(
    cx: &mut MethodContext<'a, JsObject>,
    key: u32,
) -> JsResult<'a, JsBox<TrackDataWrapper>> {
    unpack(cx, |cx, shared_engine: &SharedEngine| {
        shared_engine.with_inner(cx, |cx, engine| {
            let track = engine
                .track(key)
                .or_else(|e| cx.throw_error(format!("{}", &e)))?;
            let data = TrackDataWrapper(track.data());
            let boxed_data = cx.boxed(data);

            engine
                .delete_track(key)
                .or_else(|e| cx.throw_error(format!("{}", &e)))?;

            Ok(boxed_data)
        })
    })
}

const METHODS: &[(&str, Method)] = &[
    ("setPanning", |mut cx| {
        let value_js: Handle<JsNumber> = cx.argument(0)?;
        let value = value_js.value(&mut cx) as f32;

        unpack_track(&mut cx, |cx, track| {
            track.panning.set(value);
            Ok(cx.undefined().as_value(cx))
        })
    }),
    ("setVolume", |mut cx| {
        let value_js: Handle<JsNumber> = cx.argument(0)?;
        let value = value_js.value(&mut cx) as f32;

        unpack_track(&mut cx, |cx, track| {
            track.volume.set(value);
            Ok(cx.undefined().as_value(cx))
        })
    }),
    ("readMeter", |mut cx| {
        unpack_track(&mut cx, |cx, track| {
            let [peak, long_peak, rms] = track.meter.read();
            let peak_js = JsArray::new(cx, 2);
            let long_peak_js = JsArray::new(cx, 2);
            let rms_js = JsArray::new(cx, 2);

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
    }),
    ("delete", |mut cx| {
        let key_js: Handle<JsNumber> = cx.this().get(&mut cx, "key")?;
        let key = key_js.value(&mut cx) as u32;
        let boxed_data = delete_track(&mut cx, key)?;
        Ok(boxed_data.as_value(&mut cx))
    }),
];

/// Allow [`TrackData`] to be boxed
#[derive(Clone)]
pub struct TrackDataWrapper(pub ardae::TrackData);
impl Deref for TrackDataWrapper {
    type Target = ardae::TrackData;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Finalize for TrackDataWrapper {}
