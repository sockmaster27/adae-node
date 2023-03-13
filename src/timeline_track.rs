use neon::prelude::*;

use crate::{
    encapsulator::{encapsulate, unpack, Method},
    shared_engine::SharedEngine,
    timestamp::timestamp,
};

/// The returned object must adhere to the interface defined in the `index.d.ts` file.
pub fn construct<'a>(
    cx: &mut FunctionContext<'a>,
    key: u32,
    engine: SharedEngine,
) -> JsResult<'a, JsObject> {
    let key_js = cx.number(key).as_value(cx);
    let object = encapsulate(cx, engine, &[("key", key_js)], METHODS)?;
    Ok(object)
}

const METHODS: &[(&str, Method)] = &[("addClip", |mut cx| {
    let clip_js: Handle<JsObject> = cx.argument(0)?;
    let clip_key_js: Handle<JsNumber> = clip_js.get(&mut cx, "key")?;
    let clip_key = clip_key_js.value(&mut cx) as u32;

    let start_js = cx.argument(1)?;
    let start = timestamp(&mut cx, start_js)?;

    let end = match cx.argument_opt(2) {
        None => None,
        Some(val) => {
            let obj = val.downcast_or_throw(&mut cx)?;
            Some(timestamp(&mut cx, obj)?)
        }
    };

    let track_key_js: Handle<JsNumber> = cx.this().get(&mut cx, "key")?;
    let track_key = track_key_js.value(&mut cx) as u32;

    unpack(&mut cx, |cx, shared_engine: &SharedEngine| {
        shared_engine.with_inner(cx, |cx, engine| {
            engine
                .timeline_mut()
                .add_clip(track_key, clip_key, start, end)
                .or_else(|e| cx.throw_error(format!("{}", &e)))?;

            Ok(cx.undefined().as_value(cx))
        })
    })
})];
