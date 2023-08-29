use neon::prelude::*;

use crate::encapsulator;
use crate::encapsulator::encapsulate;
use crate::encapsulator::unpack_this;
use crate::encapsulator::Method;
use crate::shared_engine::SharedEngine;
use crate::timestamp;

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
                        .expect("AudioTrackWrapper should have a clip");
                    callback(cx, clip)
                })
            },
        )
    }

    const METHODS: &[(&str, Method)] = &[
        ("key", |mut cx| {
            unpack_this(&mut cx, |cx, (_, _, key): &(SharedEngine, 
                adae::TimelineTrackKey,adae::AudioClipKey)| {
                Ok(cx.number(*key).as_value(cx))
            })
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
    ];
}
