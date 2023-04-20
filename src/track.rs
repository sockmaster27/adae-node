use std::ops::Deref;

use neon::prelude::*;

use crate::{
    encapsulator::{encapsulate, unpack_this, Method},
    shared_engine::SharedEngine,
};

pub mod master {
    use super::*;

    /// The returned object must adhere to the interface defined in the `index.d.ts` file.
    pub fn construct<'a>(
        cx: &mut FunctionContext<'a>,
        engine: SharedEngine,
    ) -> JsResult<'a, JsObject> {
        let object = encapsulate(cx, engine, &[], METHODS)?;
        Ok(object)
    }

    /// Fetch the master track from the engine.
    fn unpack_this_track<'a, F, R>(cx: &mut CallContext<'a, JsObject>, callback: F) -> NeonResult<R>
    where
        F: FnOnce(&mut CallContext<'a, JsObject>, &mut ardae::Track) -> NeonResult<R>,
    {
        unpack_this(cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.with_inner(cx, |cx, engine| callback(cx, engine.master_mut()))
        })
    }

    const METHODS: &[(&str, Method)] = &[
        ("getPanning", |mut cx| {
            unpack_this_track(&mut cx, get_panning)
        }),
        ("setPanning", |mut cx| {
            unpack_this_track(&mut cx, set_panning)
        }),
        ("getVolume", |mut cx| unpack_this_track(&mut cx, get_volume)),
        ("setVolume", |mut cx| unpack_this_track(&mut cx, set_volume)),
        ("readMeter", |mut cx| unpack_this_track(&mut cx, read_meter)),
        ("snapMeter", |mut cx| unpack_this_track(&mut cx, snap_meter)),
    ];
}

pub mod audio_track {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use crate::encapsulator::unpack;

    use super::*;

    /// The returned object must adhere to the interface defined in the `index.d.ts` file.
    pub fn construct<'a>(
        cx: &mut FunctionContext<'a>,
        audio_track: ardae::AudioTrack,
        engine: SharedEngine,
    ) -> JsResult<'a, JsObject> {
        let object = encapsulate(cx, (engine, AudioTrackWrapper(audio_track)), &[], METHODS)?;

        Ok(object)
    }

    pub fn unpack_audio_track<'a, C>(
        cx: &mut C,
        obj: Handle<'a, JsObject>,
    ) -> NeonResult<ardae::AudioTrack>
    where
        C: Context<'a>,
    {
        let audio_track = unpack(cx, obj, |_cx, data: &(SharedEngine, AudioTrackWrapper)| {
            let (_, audio_track) = data;
            Ok(audio_track.0.clone())
        })?;

        Ok(audio_track)
    }

    fn unpack_this_audio_track<'a, F, R>(
        cx: &mut CallContext<'a, JsObject>,
        callback: F,
    ) -> NeonResult<R>
    where
        F: FnOnce(&mut CallContext<'a, JsObject>, &mut ardae::AudioTrack) -> NeonResult<R>,
    {
        let mut audio_track = unpack_this(cx, |_cx, data: &(SharedEngine, AudioTrackWrapper)| {
            let (_, audio_track) = data;
            Ok(audio_track.0.clone())
        })?;

        callback(cx, &mut audio_track)
    }

    fn unpack_this_track<'a, F, R>(cx: &mut CallContext<'a, JsObject>, callback: F) -> NeonResult<R>
    where
        F: FnOnce(&mut CallContext<'a, JsObject>, &mut ardae::Track) -> NeonResult<R>,
    {
        unpack_this(cx, |cx, data: &(SharedEngine, AudioTrackWrapper)| {
            let (shared_engine, audio_track) = data;

            shared_engine.with_inner(cx, |cx, engine| {
                let track = match engine.track_mut(audio_track.track_key()) {
                    Ok(track) => track,
                    Err(_) => {
                        return cx.throw_error("Audio track has been deleted");
                    }
                };
                callback(cx, track)
            })
        })
    }

    pub fn delete<'a, C>(
        cx: &mut C,
        shared_engine: &SharedEngine,
        audio_track: ardae::AudioTrack,
    ) -> JsResult<'a, JsObject>
    where
        C: Context<'a>,
    {
        shared_engine.with_inner(cx, |cx, engine| {
            let state = engine
                .audio_track_state(&audio_track)
                .or_else(|e| cx.throw_error(format!("{}", &e)))?;

            engine
                .delete_audio_track(audio_track.clone())
                .or_else(|e| cx.throw_error(format!("{}", &e)))?;

            encapsulate(cx, AudioTrackStateWrapper(state), &[], &[])
        })
    }

    const METHODS: &[(&str, Method)] = &[
        ("getPanning", |mut cx| {
            unpack_this_track(&mut cx, get_panning)
        }),
        ("setPanning", |mut cx| {
            unpack_this_track(&mut cx, set_panning)
        }),
        ("getVolume", |mut cx| unpack_this_track(&mut cx, get_volume)),
        ("setVolume", |mut cx| unpack_this_track(&mut cx, set_volume)),
        ("readMeter", |mut cx| unpack_this_track(&mut cx, read_meter)),
        ("snapMeter", |mut cx| unpack_this_track(&mut cx, snap_meter)),
        ("key", |mut cx| {
            unpack_this_audio_track(&mut cx, |cx, audio_track| {
                let mut s = DefaultHasher::new();
                audio_track.hash(&mut s);
                let key = s.finish();
                Ok(cx.number(key as f64).as_value(cx))
            })
        }),
        ("addClip", |mut _cx| todo!("addClip")),
        ("delete", |mut cx| {
            unpack_this(&mut cx, |cx, data: &(SharedEngine, AudioTrackWrapper)| {
                let (shared_engine, audio_track) = data;

                let boxed_state = delete(cx, shared_engine, audio_track.0.clone())?;
                Ok(boxed_state.as_value(cx))
            })
        }),
    ];
}

#[derive(Clone, Debug)]
pub struct AudioTrackWrapper(pub ardae::AudioTrack);
impl Deref for AudioTrackWrapper {
    type Target = ardae::AudioTrack;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Finalize for AudioTrackWrapper {}

#[derive(Debug)]
pub struct AudioTrackStateWrapper(pub ardae::AudioTrackState);
impl Deref for AudioTrackStateWrapper {
    type Target = ardae::AudioTrackState;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Finalize for AudioTrackStateWrapper {}

// Shared methods
fn get_panning<'a>(
    cx: &mut CallContext<'a, JsObject>,
    track: &mut ardae::Track,
) -> JsResult<'a, JsValue> {
    let panning = track.panning();
    let panning_js = cx.number(panning);
    Ok(panning_js.as_value(cx))
}
fn set_panning<'a>(
    cx: &mut CallContext<'a, JsObject>,
    track: &mut ardae::Track,
) -> JsResult<'a, JsValue> {
    let value_js: Handle<JsNumber> = cx.argument(0)?;
    let value = value_js.value(cx) as f32;

    track.set_panning(value);
    Ok(cx.undefined().as_value(cx))
}
fn get_volume<'a>(
    cx: &mut CallContext<'a, JsObject>,
    track: &mut ardae::Track,
) -> JsResult<'a, JsValue> {
    let volume = track.volume();
    let volume_js = cx.number(volume);
    Ok(volume_js.as_value(cx))
}
fn set_volume<'a>(
    cx: &mut CallContext<'a, JsObject>,
    track: &mut ardae::Track,
) -> JsResult<'a, JsValue> {
    let value_js: Handle<JsNumber> = cx.argument(0)?;
    let value = value_js.value(cx) as f32;

    track.set_volume(value);
    Ok(cx.undefined().as_value(cx))
}
fn read_meter<'a>(
    cx: &mut CallContext<'a, JsObject>,
    track: &mut ardae::Track,
) -> JsResult<'a, JsValue> {
    let [peak, long_peak, rms] = track.read_meter();
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
}
fn snap_meter<'a>(
    cx: &mut CallContext<'a, JsObject>,
    track: &mut ardae::Track,
) -> JsResult<'a, JsValue> {
    track.snap_rms();
    Ok(cx.undefined().as_value(cx))
}
