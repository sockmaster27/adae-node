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
    use super::*;

    pub fn construct<'a>(
        cx: &mut FunctionContext<'a>,
        track_key: adae::TimelineTrackKey,
        clip_key: adae::AudioClipKey,
        engine: SharedEngine,
    ) -> JsResult<'a, JsObject> {
        let object = encapsulate(cx, (engine, track_key, clip_key), &[], METHODS)?;
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
            |cx,
             (shared_engine, track_key, clip_key): &(
                SharedEngine,
                adae::TimelineTrackKey,
                adae::AudioClipKey,
            )| {
                shared_engine.with_inner(cx, |cx, engine| {
                    let clip = engine
                        .audio_clip(*track_key, *clip_key)
                        .expect("AudioClip references invalid clip");
                    callback(cx, clip)
                })
            },
        )
    }

    pub fn state_of<'a, C>(cx: &mut C, clip_obj: Handle<'a, JsObject>) -> JsResult<'a, JsObject>
    where
        C: Context<'a>,
    {
        encapsulator::unpack(
            cx,
            clip_obj,
            |cx,
             (shared_engine, track_key, clip_key): &(
                SharedEngine,
                adae::TimelineTrackKey,
                adae::AudioClipKey,
            )| {
                shared_engine.with_inner(cx, |cx, engine| {
                    let state = engine
                        .audio_clip(*track_key, *clip_key)
                        .or_else(|e| cx.throw_error(format!("{e}")))?
                        .state();

                    let state_js = encapsulate(cx, AudioClipStateWrapper(state), &[], &[])?;
                    Ok(state_js)
                })
            },
        )
    }

    pub fn unapck_state<'a, C>(
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
        ("key", |mut cx| {
            unpack_this(
                &mut cx,
                |cx, (_, _, key): &(SharedEngine, adae::TimelineTrackKey, adae::AudioClipKey)| {
                    Ok(cx.number(*key).as_value(cx))
                },
            )
        }),
        ("start", |mut cx| {
            unpack_this_clip(&mut cx, |cx, clip| timestamp::construct(cx, clip.start))
        }),
        ("length", |mut cx| {
            unpack_this_clip(&mut cx, |cx, clip| match clip.length {
                Some(length) => timestamp::construct(cx, length),
                None => Ok(cx.null().upcast()),
            })
        }),
        ("storedClip", |mut cx| {
            encapsulator::unpack_this(
                &mut cx,
                |cx,
                 (shared_engine, track_key, clip_key): &(
                    SharedEngine,
                    adae::TimelineTrackKey,
                    adae::AudioClipKey,
                )| {
                    shared_engine.with_inner(cx, |cx, engine| {
                        let clip = engine
                            .audio_clip(*track_key, *clip_key)
                            .expect("AudioClip references invalid clip");

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
                |cx,
                 (shared_engine, track_key, clip_key): &(
                    SharedEngine,
                    adae::TimelineTrackKey,
                    adae::AudioClipKey,
                )| {
                    let this = cx.this();
                    let state = state_of(cx, this)?;

                    shared_engine.with_inner(cx, |cx, engine| {
                        engine
                            .delete_audio_clip(*track_key, *clip_key)
                            .or_else(|e| cx.throw_error(format!("{e}")))?;
                        Ok(state.as_value(cx))
                    })
                },
            )
        }),
    ];

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
