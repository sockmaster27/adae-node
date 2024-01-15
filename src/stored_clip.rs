use std::sync::Arc;

use neon::prelude::*;

use crate::encapsulator::encapsulate;
use crate::encapsulator::unpack_this;
use crate::encapsulator::Method;
use crate::shared_engine::SharedEngine;

pub mod stored_audio_clip {

    use super::*;

    /// The returned object must adhere to the interface defined in the `index.d.ts` file.
    pub fn construct<'a>(
        cx: &mut FunctionContext<'a>,
        clip_key: adae::StoredAudioClipKey,
        engine: SharedEngine,
    ) -> JsResult<'a, JsObject> {
        let object = encapsulate(cx, (engine, clip_key), &[], METHODS)?;
        Ok(object)
    }

    fn unpack_this_stored_clip<'a, F, R>(
        cx: &mut CallContext<'a, JsObject>,
        callback: F,
    ) -> NeonResult<R>
    where
        F: FnOnce(&mut CallContext<'a, JsObject>, Arc<adae::StoredAudioClip>) -> NeonResult<R>,
    {
        unpack_this(cx, |cx, data: &(SharedEngine, adae::StoredAudioClipKey)| {
            let (shared_engine, clip_key) = data;
            shared_engine.with_inner(cx, |cx, engine| {
                let clip = engine.stored_audio_clip(*clip_key).unwrap();

                callback(cx, clip)
            })
        })
    }

    const METHODS: &[(&str, Method)] = &[
        ("getKey", |mut cx| {
            unpack_this(
                &mut cx,
                |cx, data: &(SharedEngine, adae::StoredAudioClipKey)| {
                    let &(_, clip_key) = data;
                    Ok(cx.number(clip_key).as_value(cx))
                },
            )
        }),
        ("getSampleRate", |mut cx| {
            unpack_this_stored_clip(&mut cx, |cx, clip| {
                Ok(cx.number(clip.sample_rate() as f64).as_value(cx))
            })
        }),
        ("getLength", |mut cx| {
            unpack_this_stored_clip(&mut cx, |cx, clip| {
                Ok(cx.number(clip.length() as f64).as_value(cx))
            })
        }),
    ];
}
