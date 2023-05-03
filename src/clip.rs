use neon::prelude::*;

use crate::encapsulator::encapsulate;
use crate::encapsulator::unpack_this;
use crate::encapsulator::Method;

pub mod audio_clip {
    use super::*;

    /// The returned object must adhere to the interface defined in the `index.d.ts` file.
    pub fn construct<'a>(
        cx: &mut FunctionContext<'a>,
        clip_key: ardae::AudioClipKey,
    ) -> JsResult<'a, JsObject> {
        let object = encapsulate(cx, clip_key, &[], METHODS)?;
        Ok(object)
    }

    const METHODS: &[(&str, Method)] = &[("key", |mut cx| {
        unpack_this(&mut cx, |cx, &clip_key: &ardae::AudioClipKey| {
            Ok(cx.number(clip_key).as_value(cx))
        })
    })];
}
