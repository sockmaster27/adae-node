use neon::prelude::*;

use ardae::Timestamp;

use crate::encapsulator::{encapsulate, unpack, Method};

pub fn class<'a, C>(cx: &mut C) -> JsResult<'a, JsValue>
where
    C: Context<'a>,
{
    let class_obj = cx.empty_object();
    for (name, method) in STATIC_METHODS {
        let method_js = JsFunction::new(cx, *method)?;
        class_obj.set(cx, *name, method_js)?;
    }
    Ok(class_obj.as_value(cx))
}

pub fn construct<'a, C>(cx: &mut C, timestamp: Timestamp) -> JsResult<'a, JsValue>
where
    C: Context<'a>,
{
    Ok(encapsulate(cx, TimestampWrapper(timestamp), &[], METHODS)?.as_value(cx))
}

pub fn timestamp<'a>(
    cx: &mut MethodContext<'a, JsObject>,
    obj: Handle<JsObject>,
) -> NeonResult<Timestamp> {
    unpack(cx, |cx, timestamp: &TimestampWrapper| Ok(timestamp.0))
}

struct TimestampWrapper(Timestamp);
impl Finalize for TimestampWrapper {}

const STATIC_METHODS: &[(&str, Method)] = &[
    ("zero", |mut cx| construct(&mut cx, Timestamp::zero())),
    ("fromBeatUnits", |mut cx| {
        let beat_units_js: Handle<JsNumber> = cx.argument(0)?;
        let beat_units_f64 = beat_units_js.value(&mut cx);

        if beat_units_f64 < 0.0 {
            return cx.throw_range_error("Timestamp with beat unit value under 0 is not valid");
        }
        if (u32::MAX as f64) < beat_units_f64 {
            return cx.throw_range_error("Timestamp must have beat unit value less than 2^32");
        }

        let beat_units = beat_units_f64 as u32;
        construct(&mut cx, Timestamp::from_beat_units(beat_units))
    }),
];

const METHODS: &[(&str, Method)] = &[];
