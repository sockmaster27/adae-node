use neon::prelude::*;

use adae::Timestamp;

use crate::encapsulator::{self, encapsulate, Method};

pub fn class<'a, C>(cx: &mut C) -> JsResult<'a, JsValue>
where
    C: Context<'a>,
{
    let class = JsFunction::new(cx, |mut cx| {
        cx.throw_error::<_, Handle<JsValue>>(
            "Timestamp cannot be constructed directly. Use the static methods instead.",
        )
    })?;
    for (name, method) in STATIC_METHODS {
        let method_js = JsFunction::new(cx, *method)?;
        class.set(cx, *name, method_js)?;
    }
    Ok(class.as_value(cx))
}

pub fn construct<'a, C>(cx: &mut C, timestamp: Timestamp) -> JsResult<'a, JsValue>
where
    C: Context<'a>,
{
    Ok(encapsulate(cx, TimestampWrapper(timestamp), &[], METHODS)?.as_value(cx))
}

pub fn timestamp(
    cx: &mut MethodContext<'_, JsObject>,
    obj: Handle<JsObject>,
) -> NeonResult<Timestamp> {
    let boxed: Handle<JsBox<TimestampWrapper>> = obj.get(cx, encapsulator::DATA_KEY)?;
    let wrapper = &*boxed;
    Ok(wrapper.0)
}

struct TimestampWrapper(Timestamp);
impl Finalize for TimestampWrapper {}

/// Example: `err_msg("beat", "greater than zero", -1.0)` -> `"Timestamp must have beat value greater than zero. Got -1"`
fn err_msg(property: &str, expected_to_be: &str, value: f64) -> String {
    format!("Timestamp must have {property} value {expected_to_be}. Got {value}")
}

const STATIC_METHODS: &[(&str, Method)] = &[
    ("min", |mut cx| {
        let a_js = cx.argument::<JsObject>(0)?;
        let a = timestamp(&mut cx, a_js)?;

        let b_js = cx.argument::<JsObject>(1)?;
        let b = timestamp(&mut cx, b_js)?;

        construct(&mut cx, Timestamp::min(a, b))
    }),
    ("max", |mut cx| {
        let a_js = cx.argument::<JsObject>(0)?;
        let a = timestamp(&mut cx, a_js)?;

        let b_js = cx.argument::<JsObject>(1)?;
        let b = timestamp(&mut cx, b_js)?;

        construct(&mut cx, Timestamp::max(a, b))
    }),
    ("eq", |mut cx| {
        let a_js = cx.argument::<JsObject>(0)?;
        let a = timestamp(&mut cx, a_js)?;

        let b_js = cx.argument::<JsObject>(1)?;
        let b = timestamp(&mut cx, b_js)?;

        Ok(cx.boolean(a == b).as_value(&mut cx))
    }),
    ("add", |mut cx| {
        let a_js = cx.argument::<JsObject>(0)?;
        let a = timestamp(&mut cx, a_js)?;

        let b_js = cx.argument::<JsObject>(1)?;
        let b = timestamp(&mut cx, b_js)?;

        let r = match a.checked_add(b) {
            Some(r) => Ok(r),
            None => cx.throw_error(format!("Timestamp addition with overflow: {a:?} + {b:?}")),
        }?;

        construct(&mut cx, r)
    }),
    ("sub", |mut cx| {
        let a_js = cx.argument::<JsObject>(0)?;
        let a = timestamp(&mut cx, a_js)?;

        let b_js = cx.argument::<JsObject>(1)?;
        let b = timestamp(&mut cx, b_js)?;

        let r = match a.checked_sub(b) {
            Some(r) => Ok(r),
            None => cx.throw_error(format!(
                "Timestamp subtraction with overflow: {a:?} - {b:?}"
            )),
        }?;

        construct(&mut cx, r)
    }),
    ("mul", |mut cx| {
        let ts_js = cx.argument::<JsObject>(0)?;
        let ts = timestamp(&mut cx, ts_js)?;

        let s_js = cx.argument::<JsNumber>(1)?;
        let s_f64 = s_js.value(&mut cx);

        if s_f64 < 0.0 {
            return cx.throw_range_error(format!(
                "Timestamp must be multiplied by a value greater than zero. Got {s_f64}"
            ));
        }
        if (u32::MAX as f64) < s_f64 {
            return cx.throw_range_error(format!(
                "Timestamp must be multiplied by a value smaller than 2^32. Got {s_f64}"
            ));
        }

        let s = s_f64 as u32;

        construct(&mut cx, ts * s)
    }),
    ("zero", |mut cx| construct(&mut cx, Timestamp::zero())),
    ("infinity", |mut cx| {
        construct(&mut cx, Timestamp::infinity())
    }),
    ("fromBeatUnits", |mut cx| {
        let beat_units_js: Handle<JsNumber> = cx.argument(0)?;
        let beat_units_f64 = beat_units_js.value(&mut cx);

        if beat_units_f64 < 0.0 {
            return cx.throw_range_error(err_msg("beat unit", "greater than zero", beat_units_f64));
        }
        if (u32::MAX as f64) < beat_units_f64 {
            return cx.throw_range_error(err_msg("beat unit", "smaller than 2^32", beat_units_f64));
        }

        let beat_units = beat_units_f64 as u32;
        construct(&mut cx, Timestamp::from_beat_units(beat_units))
    }),
    ("fromBeats", |mut cx| {
        let beats_js: Handle<JsNumber> = cx.argument(0)?;
        let beats_f64 = beats_js.value(&mut cx);

        if beats_f64 < 0.0 {
            return cx.throw_range_error(err_msg("beat", "greater than zero", beats_f64));
        }
        if (u32::MAX as f64) < beats_f64 {
            return cx.throw_range_error(err_msg("beat", "smaller than 2^32", beats_f64));
        }

        let beats = beats_f64 as u32;
        construct(&mut cx, Timestamp::from_beats(beats))
    }),
    ("fromSamples", |mut cx| {
        let samples_js: Handle<JsNumber> = cx.argument(0)?;
        let samples_f64 = samples_js.value(&mut cx);
        if samples_f64 < 0.0 {
            return cx.throw_range_error(err_msg("sample", "greater than zero", samples_f64));
        }
        if (u64::MAX as f64) < samples_f64 {
            return cx.throw_range_error(err_msg("sample", "smaller than 2^64", samples_f64));
        }
        let samples = samples_f64 as u64;

        let sample_rate_js: Handle<JsNumber> = cx.argument(1)?;
        let sample_rate_f64 = sample_rate_js.value(&mut cx);
        if sample_rate_f64 < 0.0 {
            return cx.throw_range_error(err_msg(
                "sample rate",
                "greater than zero",
                sample_rate_f64,
            ));
        }
        if (u32::MAX as f64) < sample_rate_f64 {
            return cx.throw_range_error(err_msg(
                "sample rate",
                "smaller than 2^32",
                sample_rate_f64,
            ));
        }
        let sample_rate = sample_rate_f64 as u32;

        let bpm_js: Handle<JsNumber> = cx.argument(2)?;
        let bpm_f64 = bpm_js.value(&mut cx);
        let bpm_cents_f64 = bpm_f64 * 100.0;
        if bpm_cents_f64 < 0.0 {
            return cx.throw_range_error(err_msg("BPM", "greater than zero", bpm_f64));
        }
        if (u16::MAX as f64) < bpm_cents_f64 {
            return cx.throw_range_error(err_msg("BPM", "smaller than 2^16 / 100", bpm_f64));
        }
        let bpm_cents = bpm_cents_f64 as u16;

        construct(
            &mut cx,
            Timestamp::from_samples(samples, sample_rate, bpm_cents),
        )
    }),
];

const METHODS: &[(&str, Method)] = &[
    ("getBeatUnits", |mut cx| {
        let this = cx.this();
        let timestamp = timestamp(&mut cx, this)?;
        let beat_units = timestamp.beat_units();
        Ok(cx.number(beat_units as f64).as_value(&mut cx))
    }),
    ("getBeats", |mut cx| {
        let this = cx.this();
        let timestamp = timestamp(&mut cx, this)?;
        let beat_units = timestamp.beats();
        Ok(cx.number(beat_units as f64).as_value(&mut cx))
    }),
    ("getSamples", |mut cx| {
        let this = cx.this();

        let sample_rate_js: Handle<JsNumber> = cx.argument(0)?;
        let sample_rate_f64 = sample_rate_js.value(&mut cx);
        if sample_rate_f64 < 0.0 {
            return cx.throw_range_error(err_msg(
                "sample rate",
                "greater than zero",
                sample_rate_f64,
            ));
        }
        if (u32::MAX as f64) < sample_rate_f64 {
            return cx.throw_range_error(err_msg(
                "sample rate",
                "smaller than 2^32",
                sample_rate_f64,
            ));
        }
        let sample_rate = sample_rate_f64 as u32;

        let bpm_js: Handle<JsNumber> = cx.argument(1)?;
        let bpm_f64 = bpm_js.value(&mut cx);
        let bpm_cents_f64 = bpm_f64 * 100.0;
        if bpm_cents_f64 < 0.0 {
            return cx.throw_range_error(err_msg("BPM", "greater than zero", bpm_f64));
        }
        if (u16::MAX as f64) < bpm_cents_f64 {
            return cx.throw_range_error(err_msg("BPM", "smaller than 2^16 / 100", bpm_f64));
        }
        let bpm_cents = bpm_cents_f64 as u16;

        let timestamp = timestamp(&mut cx, this)?;
        let beat_units = timestamp.samples(sample_rate, bpm_cents);
        Ok(cx.number(beat_units as f64).as_value(&mut cx))
    }),
];
