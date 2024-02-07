use neon::prelude::*;

use crate::encapsulator;
use crate::encapsulator::encapsulate;
use crate::encapsulator::unpack_this;
use crate::encapsulator::Method;
use crate::shared_engine::SharedEngine;
use crate::stored_clip::stored_audio_clip;
use crate::timestamp;
use std::ops::Deref;

pub mod audio_clip {
    use crate::{track::audio_track, utils::ResultExt};

    use super::*;

    pub fn construct<'a>(
        cx: &mut FunctionContext<'a>,
        clip_key: adae::AudioClipKey,
        engine: SharedEngine,
    ) -> JsResult<'a, JsObject> {
        let object = encapsulate(cx, (engine, AudioClipKeyWrapper(clip_key)), &[], METHODS)?;
        Ok(object)
    }

    fn unpack_this_clip<'a, R, F>(
        cx: &mut MethodContext<'a, JsObject>,
        callback: F,
    ) -> NeonResult<R>
    where
        F: FnOnce(&mut MethodContext<'a, JsObject>, &adae::AudioClip) -> NeonResult<R>,
    {
        encapsulator::unpack_this(
            cx,
            |cx, (shared_engine, clip_key): &(SharedEngine, AudioClipKeyWrapper)| {
                shared_engine.with_inner(cx, |cx, engine| {
                    let clip = engine.audio_clip(**clip_key).or_throw(cx)?;
                    callback(cx, clip)
                })
            },
        )
    }

    pub fn encapsulate_state<'a, C>(
        cx: &mut C,
        state: adae::AudioClipState,
    ) -> JsResult<'a, JsObject>
    where
        C: Context<'a>,
    {
        let state_js = encapsulate(cx, AudioClipStateWrapper(state), &[], &[])?;
        Ok(state_js)
    }

    pub fn unpack_state<'a, C>(
        cx: &mut C,
        state_obj: Handle<'a, JsObject>,
    ) -> NeonResult<adae::AudioClipState>
    where
        C: Context<'a>,
    {
        let state = encapsulator::unpack(cx, state_obj, |_cx, state: &AudioClipStateWrapper| {
            Ok(state.0.clone())
        })?;
        Ok(state)
    }

    const METHODS: &[(&str, Method)] = &[
        ("getKey", |mut cx| {
            unpack_this(
                &mut cx,
                |cx, (_, clip_key): &(SharedEngine, AudioClipKeyWrapper)| {
                    let key: u32 = (**clip_key).into();
                    Ok(cx.number(key).as_value(cx))
                },
            )
        }),
        ("getStart", |mut cx| {
            unpack_this_clip(&mut cx, |cx, clip| timestamp::construct(cx, clip.start()))
        }),
        ("getLength", |mut cx| {
            encapsulator::unpack_this(
                &mut cx,
                |cx, (shared_engine, clip_key): &(SharedEngine, AudioClipKeyWrapper)| {
                    shared_engine.with_inner(cx, |cx, engine| {
                        let bpm_cents = engine.bpm_cents();

                        let clip = engine.audio_clip(**clip_key).or_throw(cx)?;
                        timestamp::construct(cx, clip.length(bpm_cents))
                    })
                },
            )
        }),
        ("move", |mut cx| {
            let new_start_js = cx.argument::<JsObject>(0)?;
            let new_start = timestamp(&mut cx, new_start_js)?;

            encapsulator::unpack_this(
                &mut cx,
                |cx, (shared_engine, clip_key): &(SharedEngine, AudioClipKeyWrapper)| {
                    shared_engine.with_inner(cx, |cx, engine| {
                        engine
                            .audio_clip_move(**clip_key, new_start)
                            .or_else(|e| cx.throw_error(format!("Failed to move clip: {e}")))?;
                        Ok(cx.undefined().as_value(cx))
                    })
                },
            )
        }),
        ("moveToTrack", |mut cx| {
            let new_start_js = cx.argument::<JsObject>(0)?;
            let new_start = timestamp(&mut cx, new_start_js)?;

            let new_audio_track_js = cx.argument::<JsObject>(1)?;
            let new_audio_track_key =
                audio_track::unpack_audio_track_key(&mut cx, new_audio_track_js)?;

            encapsulator::unpack_this(
                &mut cx,
                |cx, (shared_engine, clip_key): &(SharedEngine, AudioClipKeyWrapper)| {
                    shared_engine.with_inner(cx, |cx, engine| {
                        let new_timeline_track_key = engine
                            .audio_timeline_track_key(new_audio_track_key)
                            .or_else(|e| {
                                cx.throw_error(format!("Failed to get timeline track: {e}"))
                            })?;

                        engine
                            .audio_clip_move_to_track(**clip_key, new_start, new_timeline_track_key)
                            .or_else(|e| cx.throw_error(format!("Failed to move clip: {e}")))?;

                        Ok(cx.undefined().as_value(cx))
                    })
                },
            )
        }),
        ("cropStart", |mut cx| {
            let new_length_js = cx.argument::<JsObject>(0)?;
            let new_length = timestamp(&mut cx, new_length_js)?;

            encapsulator::unpack_this(
                &mut cx,
                |cx, (shared_engine, clip_key): &(SharedEngine, AudioClipKeyWrapper)| {
                    shared_engine.with_inner(cx, |cx, engine| {
                        engine
                            .audio_clip_crop_start(**clip_key, new_length)
                            .or_else(|e| {
                                cx.throw_error(format!("Failed to crop start of clip: {e}"))
                            })?;
                        Ok(cx.undefined().as_value(cx))
                    })
                },
            )
        }),
        ("cropEnd", |mut cx| {
            let new_length_js = cx.argument::<JsObject>(0)?;
            let new_length = timestamp(&mut cx, new_length_js)?;

            encapsulator::unpack_this(
                &mut cx,
                |cx, (shared_engine, clip_key): &(SharedEngine, AudioClipKeyWrapper)| {
                    shared_engine.with_inner(cx, |cx, engine| {
                        engine
                            .audio_clip_crop_end(**clip_key, new_length)
                            .or_else(|e| {
                                cx.throw_error(format!("Failed to crop end of clip: {e}"))
                            })?;
                        Ok(cx.undefined().as_value(cx))
                    })
                },
            )
        }),
        ("getStoredClip", |mut cx| {
            encapsulator::unpack_this(
                &mut cx,
                |cx, (shared_engine, clip_key): &(SharedEngine, AudioClipKeyWrapper)| {
                    shared_engine.with_inner(cx, |cx, engine| {
                        let clip = engine.audio_clip(**clip_key).or_throw(cx)?;

                        Ok(stored_audio_clip::construct(
                            cx,
                            clip.stored_clip(),
                            shared_engine.clone(),
                        )?
                        .as_value(cx))
                    })
                },
            )
        }),
        ("delete", |mut cx| {
            encapsulator::unpack_this(
                &mut cx,
                |cx, (shared_engine, clip_key): &(SharedEngine, AudioClipKeyWrapper)| {
                    shared_engine.with_inner(cx, |cx, engine| {
                        let state = engine.delete_audio_clip(**clip_key).or_throw(cx)?;
                        let state_js = encapsulate_state(cx, state)?;
                        Ok(state_js.as_value(cx))
                    })
                },
            )
        }),
    ];

    #[derive(Debug)]
    pub struct AudioClipKeyWrapper(pub adae::AudioClipKey);
    impl Deref for AudioClipKeyWrapper {
        type Target = adae::AudioClipKey;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl Finalize for AudioClipKeyWrapper {}

    #[derive(Debug)]
    pub struct AudioClipStateWrapper(pub adae::AudioClipState);
    impl Deref for AudioClipStateWrapper {
        type Target = adae::AudioClipState;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl Finalize for AudioClipStateWrapper {}
}
