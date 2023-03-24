use std::path::Path;

use neon::prelude::*;

use crate::{
    encapsulator::{encapsulate, unpack, Method},
    shared_engine::SharedEngine,
    timeline_track,
};

pub fn construct<'a, C>(cx: &mut C, shared_engine: SharedEngine) -> JsResult<'a, JsValue>
where
    C: Context<'a>,
{
    Ok(encapsulate(cx, shared_engine, &[], METHODS)?.as_value(cx))
}

const METHODS: &[(&str, Method)] = &[
    ("getTracks", |mut cx| {
        unpack(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.with_inner(cx, |cx, engine| {
                let timeline = engine.timeline_mut();
                let array = JsArray::new(cx, timeline.used_keys());

                for (i, key) in timeline.tracks().enumerate() {
                    let track =
                        timeline_track::construct(cx, key, SharedEngine::clone(shared_engine))?;
                    array.set(cx, i as u32, track)?;
                }

                Ok(array.as_value(cx))
            })
        })
    }),
    ("getTrack", |mut cx| {
        let key_js: Handle<JsNumber> = cx.argument(0)?;
        let key = key_js.value(&mut cx) as u32;

        unpack(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.with_inner(cx, |cx, engine| {
                let exists = engine.timeline().key_in_use(key);
                if !exists {
                    cx.throw_error(format!("No timeline track with key: {}", key))?;
                }

                let track = timeline_track::construct(cx, key, SharedEngine::clone(shared_engine))?;

                Ok(track.as_value(cx))
            })
        })
    }),
    ("importAudioClip", |mut cx| {
        let path_js: Handle<JsString> = cx.argument(0)?;
        let path = path_js.value(&mut cx);

        unpack(&mut cx, |cx, shared_engine: &SharedEngine| {
            shared_engine.with_inner(cx, |cx, engine| {
                let clip_key = engine
                    .timeline_mut()
                    .import_audio_clip(&Path::new(&path))
                    .or_else(|e| cx.throw_error(format! {"{}", &e}))?;

                Ok(cx.number(clip_key).as_value(cx))
            })
        })
    }),
];
