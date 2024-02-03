use neon::prelude::*;
use std::ops::Deref;

use crate::clip::audio_clip::AudioClipKeyWrapper;
use crate::stored_clip::stored_audio_clip::StoredAudioClipKeyWrapper;
use crate::utils::ResultExt;
use crate::{clip::audio_clip, encapsulator::unpack, timestamp::timestamp};
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
        F: FnOnce(&mut CallContext<'a, JsObject>, &mut adae::MixerTrack) -> NeonResult<R>,
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
    use super::*;

    /// The returned object must adhere to the interface defined in the `index.d.ts` file.
    pub fn construct<'a>(
        cx: &mut FunctionContext<'a>,
        audio_track_key: adae::AudioTrackKey,
        engine: SharedEngine,
    ) -> JsResult<'a, JsObject> {
        let object = encapsulate(
            cx,
            (engine, AudioTrackKeyWrapper(audio_track_key)),
            &[],
            METHODS,
        )?;

        Ok(object)
    }

    pub fn unpack_audio_track_key<'a, C>(
        cx: &mut C,
        obj: Handle<'a, JsObject>,
    ) -> NeonResult<adae::AudioTrackKey>
    where
        C: Context<'a>,
    {
        unpack(
            cx,
            obj,
            |_, (_, audio_track_key): &(SharedEngine, AudioTrackKeyWrapper)| Ok(**audio_track_key),
        )
    }

    fn unpack_this_audio_track_key(
        cx: &mut CallContext<'_, JsObject>,
    ) -> NeonResult<adae::AudioTrackKey> {
        let this = cx.this();
        unpack_audio_track_key(cx, this)
    }

    fn unpack_this_mixer_track<'a, F, R>(
        cx: &mut CallContext<'a, JsObject>,
        callback: F,
    ) -> NeonResult<R>
    where
        F: FnOnce(&mut CallContext<'a, JsObject>, &mut adae::MixerTrack) -> NeonResult<R>,
    {
        unpack_this(cx, |cx, data: &(SharedEngine, AudioTrackKeyWrapper)| {
            let (shared_engine, audio_track_key) = data;

            shared_engine.with_inner(cx, |cx, engine| {
                let mixer_track_key = engine
                    .audio_mixer_track_key(**audio_track_key)
                    .or_throw(cx)?;
                let track = engine.mixer_track_mut(mixer_track_key).or_throw(cx)?;
                callback(cx, track)
            })
        })
    }

    pub fn delete<'a, C>(
        cx: &mut C,
        shared_engine: &SharedEngine,
        audio_track_key: adae::AudioTrackKey,
    ) -> JsResult<'a, JsObject>
    where
        C: Context<'a>,
    {
        shared_engine.with_inner(cx, |cx, engine| {
            let state = engine.audio_track_state(audio_track_key).or_throw(cx)?;
            engine.delete_audio_track(audio_track_key).or_throw(cx)?;

            encapsulate(cx, AudioTrackStateWrapper(state), &[], &[])
        })
    }

    /// Throws an error if the audio track represented by `cx.this()` has been deleted.
    fn assert_this_not_deleted(cx: &mut MethodContext<JsObject>) -> NeonResult<()> {
        unpack_this(
            cx,
            |cx, (shared_engine, audio_track_key): &(SharedEngine, AudioTrackKeyWrapper)| {
                shared_engine.with_inner(cx, |cx, engine| {
                    if !engine.has_audio_track(**audio_track_key) {
                        return cx.throw_error("Audio track has been deleted.");
                    }

                    Ok(())
                })
            },
        )
    }

    const METHODS: &[(&str, Method)] = &[
        ("getKey", |mut cx| {
            assert_this_not_deleted(&mut cx)?;

            let audio_track_key = unpack_this_audio_track_key(&mut cx)?;
            let key: u32 = audio_track_key.into();
            Ok(cx.number(key).as_value(&mut cx))
        }),
        ("getPanning", |mut cx| {
            assert_this_not_deleted(&mut cx)?;
            unpack_this_mixer_track(&mut cx, get_panning)
        }),
        ("setPanning", |mut cx| {
            assert_this_not_deleted(&mut cx)?;
            unpack_this_mixer_track(&mut cx, set_panning)
        }),
        ("getVolume", |mut cx| {
            assert_this_not_deleted(&mut cx)?;
            unpack_this_mixer_track(&mut cx, get_volume)
        }),
        ("setVolume", |mut cx| {
            assert_this_not_deleted(&mut cx)?;
            unpack_this_mixer_track(&mut cx, set_volume)
        }),
        ("readMeter", |mut cx| {
            assert_this_not_deleted(&mut cx)?;
            unpack_this_mixer_track(&mut cx, read_meter)
        }),
        ("snapMeter", |mut cx| {
            assert_this_not_deleted(&mut cx)?;
            unpack_this_mixer_track(&mut cx, snap_meter)
        }),
        ("getClips", |mut cx| {
            assert_this_not_deleted(&mut cx)?;
            unpack_this(
                &mut cx,
                |cx, (shared_engine, audio_track_key): &(SharedEngine, AudioTrackKeyWrapper)| {
                    shared_engine.with_inner(cx, |cx, engine| {
                        let timeline_track_key = engine
                            .audio_timeline_track_key(**audio_track_key)
                            .or_throw(cx)?;
                        let clips = engine.audio_clips(timeline_track_key).or_throw(cx)?;

                        let clips_js = JsArray::new(cx, clips.size_hint().0 as u32);
                        for (i, clip) in clips.enumerate() {
                            let clip_js =
                                audio_clip::construct(cx, clip.key, shared_engine.clone())?;
                            clips_js.set(cx, i as u32, clip_js)?;
                        }

                        Ok(clips_js.as_value(cx))
                    })
                },
            )
        }),
        ("addClip", |mut cx| {
            assert_this_not_deleted(&mut cx)?;

            let audio_clip_js = cx.argument::<JsObject>(0)?;
            let audio_clip_key = unpack(
                &mut cx,
                audio_clip_js,
                |_, (_, stored_clip_key): &(SharedEngine, StoredAudioClipKeyWrapper)| {
                    Ok(**stored_clip_key)
                },
            )?;

            let start_js = cx.argument::<JsObject>(1)?;
            let start = timestamp(&mut cx, start_js)?;

            let length_js_val = cx.argument_opt(2);
            let length_js = match length_js_val {
                Some(val) => {
                    let is_null = val.is_a::<JsNull, _>(&mut cx);
                    let is_undefined = val.is_a::<JsUndefined, _>(&mut cx);
                    if is_null || is_undefined {
                        None
                    } else {
                        Some(val.downcast_or_throw(&mut cx)?)
                    }
                }
                None => None,
            };
            let length = match length_js {
                Some(length_js) => Some(timestamp(&mut cx, length_js)?),
                None => None,
            };

            unpack_this(
                &mut cx,
                |cx, (shared_engine, audio_track_key): &(SharedEngine, AudioTrackKeyWrapper)| {
                    shared_engine.with_inner(cx, |cx, engine| {
                        let timeline_track_key = engine
                            .audio_timeline_track_key(**audio_track_key)
                            .or_throw(cx)?;
                        let key = engine
                            .add_audio_clip(timeline_track_key, audio_clip_key, start, length)
                            .or_throw(cx)?;

                        Ok(audio_clip::construct(cx, key, shared_engine.clone())?.as_value(cx))
                    })
                },
            )
        }),
        ("deleteClip", |mut cx| {
            assert_this_not_deleted(&mut cx)?;

            let clip_js = cx.argument::<JsObject>(0)?;
            let state = audio_clip::state_of(&mut cx, clip_js)?;
            unpack(
                &mut cx,
                clip_js,
                |cx, (shared_engine, clip_key): &(SharedEngine, AudioClipKeyWrapper)| {
                    shared_engine.with_inner(cx, |cx, engine| {
                        engine.delete_audio_clip(**clip_key).or_throw(cx)?;

                        Ok(state.as_value(cx))
                    })
                },
            )
        }),
        // TODO: Move to Engine (?)
        ("deleteClips", |mut cx| {
            assert_this_not_deleted(&mut cx)?;

            let clips_js_array = cx.argument::<JsArray>(0)?;
            let clips_js = clips_js_array.to_vec(&mut cx)?;
            let clip_keys = clips_js
                .iter()
                .map(|clip_js| {
                    let clip_obj = clip_js.downcast_or_throw::<JsObject, _>(&mut cx)?;
                    unpack(
                        &mut cx,
                        clip_obj,
                        |_, (_, clip_key): &(SharedEngine, AudioClipKeyWrapper)| Ok(**clip_key),
                    )
                })
                .collect::<NeonResult<Vec<_>>>()?;

            let clip_states_js_array = JsArray::new(&mut cx, clips_js.len() as u32);
            for (i, clip_js) in clips_js.iter().enumerate() {
                let clip_obj = clip_js.downcast_or_throw::<JsObject, _>(&mut cx)?;
                let clip_state_js = audio_clip::state_of(&mut cx, clip_obj)?;
                clip_states_js_array.set(&mut cx, i as u32, clip_state_js)?;
            }

            unpack_this(
                &mut cx,
                |cx, (shared_engine, _): &(SharedEngine, AudioTrackKeyWrapper)| {
                    shared_engine.with_inner(cx, |cx, engine| {
                        engine.delete_audio_clips(clip_keys).or_throw(cx)?;
                        Ok(clip_states_js_array.as_value(cx))
                    })
                },
            )
        }),
        ("reconstructClip", |mut cx| {
            assert_this_not_deleted(&mut cx)?;

            let state_js = cx.argument::<JsObject>(0)?;
            let state = audio_clip::unpack_state(&mut cx, state_js)?;

            unpack_this(
                &mut cx,
                |cx, (shared_engine, audio_track_key): &(SharedEngine, AudioTrackKeyWrapper)| {
                    shared_engine.with_inner(cx, |cx, engine| {
                        let timeline_track_key = engine
                            .audio_timeline_track_key(**audio_track_key)
                            .or_throw(cx)?;
                        let clip_key = engine
                            .reconstruct_audio_clip(timeline_track_key, state)
                            .or_throw(cx)?;

                        let clip_js = audio_clip::construct(cx, clip_key, shared_engine.clone())?;
                        Ok(clip_js.as_value(cx))
                    })
                },
            )
        }),
        ("reconstructClips", |mut cx| {
            assert_this_not_deleted(&mut cx)?;

            let states_js_array = cx.argument::<JsArray>(0)?;
            let states_js = states_js_array.to_vec(&mut cx)?;
            let states = states_js
                .iter()
                .map(|state_js| {
                    let state_obj = state_js.downcast_or_throw(&mut cx)?;
                    audio_clip::unpack_state(&mut cx, state_obj)
                })
                .collect::<NeonResult<Vec<_>>>()?;

            unpack_this(
                &mut cx,
                |cx, (shared_engine, audio_track_key): &(SharedEngine, AudioTrackKeyWrapper)| {
                    shared_engine.with_inner(cx, |cx, engine| {
                        let timeline_track_key = engine
                            .audio_timeline_track_key(**audio_track_key)
                            .or_throw(cx)?;

                        let clip_keys = engine
                            .reconstruct_audio_clips(timeline_track_key, states)
                            .or_throw(cx)?;

                        let (lower_size_hint, upper_size_hint) = clip_keys.size_hint();
                        let size_hint = upper_size_hint.unwrap_or(lower_size_hint);

                        let clips_js = JsArray::new(cx, size_hint as u32);
                        for (i, clip_key) in clip_keys.enumerate() {
                            let clip_js =
                                audio_clip::construct(cx, clip_key, shared_engine.clone())?;
                            clips_js.set(cx, i as u32, clip_js)?;
                        }

                        Ok(clips_js.as_value(cx))
                    })
                },
            )
        }),
        ("delete", |mut cx| {
            assert_this_not_deleted(&mut cx)?;
            unpack_this(
                &mut cx,
                |cx, data: &(SharedEngine, AudioTrackKeyWrapper)| {
                    let (shared_engine, audio_track_key) = data;

                    let boxed_state = delete(cx, shared_engine, **audio_track_key)?;
                    Ok(boxed_state.as_value(cx))
                },
            )
        }),
    ];

    #[derive(Clone, Debug)]
    pub struct AudioTrackKeyWrapper(pub adae::AudioTrackKey);
    impl Deref for AudioTrackKeyWrapper {
        type Target = adae::AudioTrackKey;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl Finalize for AudioTrackKeyWrapper {}

    #[derive(Debug)]
    pub struct AudioTrackStateWrapper(pub adae::AudioTrackState);
    impl Deref for AudioTrackStateWrapper {
        type Target = adae::AudioTrackState;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl Finalize for AudioTrackStateWrapper {}
}

// Shared methods
fn get_panning<'a>(
    cx: &mut CallContext<'a, JsObject>,
    track: &mut adae::MixerTrack,
) -> JsResult<'a, JsValue> {
    let panning = track.panning();
    let panning_js = cx.number(panning);
    Ok(panning_js.as_value(cx))
}
fn set_panning<'a>(
    cx: &mut CallContext<'a, JsObject>,
    track: &mut adae::MixerTrack,
) -> JsResult<'a, JsValue> {
    let value_js: Handle<JsNumber> = cx.argument(0)?;
    let value = value_js.value(cx) as f32;

    track.set_panning(value);
    Ok(cx.undefined().as_value(cx))
}
fn get_volume<'a>(
    cx: &mut CallContext<'a, JsObject>,
    track: &mut adae::MixerTrack,
) -> JsResult<'a, JsValue> {
    let volume = track.volume();
    let volume_js = cx.number(volume);
    Ok(volume_js.as_value(cx))
}
fn set_volume<'a>(
    cx: &mut CallContext<'a, JsObject>,
    track: &mut adae::MixerTrack,
) -> JsResult<'a, JsValue> {
    let value_js: Handle<JsNumber> = cx.argument(0)?;
    let value = value_js.value(cx) as f32;

    track.set_volume(value);
    Ok(cx.undefined().as_value(cx))
}
fn read_meter<'a>(
    cx: &mut CallContext<'a, JsObject>,
    track: &mut adae::MixerTrack,
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
    track: &mut adae::MixerTrack,
) -> JsResult<'a, JsValue> {
    track.snap_rms();
    Ok(cx.undefined().as_value(cx))
}
