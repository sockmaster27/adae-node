use std::collections::VecDeque;
use std::sync::Mutex;

use neon::prelude::*;
use neon::types::Deferred;

static ERR_MSG: &str = "Failed to write to custom debug output";

lazy_static! {
    static ref RESOLVER: Mutex<Option<(Channel, Deferred)>> = Mutex::new(None);
    static ref UNRESOLVED: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
}

pub fn get_debug(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let (deferred, promise) = cx.promise();

    let mut unresolved = UNRESOLVED.lock().expect(ERR_MSG);

    if unresolved.is_empty() {
        let channel = cx.channel();
        *RESOLVER.lock().expect(ERR_MSG) = Some((channel, deferred));
    } else {
        let msg = cx.string(unresolved.pop_back().unwrap());
        deferred.resolve(&mut cx, msg);
    }

    Ok(promise)
}

#[cfg(feature = "custom_debug_output")]
pub fn output_debug(msg: String) {
    let awaiting = RESOLVER.lock().expect(ERR_MSG).take();
    match awaiting {
        Some((channel, deferred)) => {
            channel.send(move |mut cx| {
                let msg = cx.string(msg);
                deferred.resolve(&mut cx, msg);
                Ok(())
            });
        }

        None => {
            let mut unresolved = UNRESOLVED.lock().expect(ERR_MSG);

            // Cap length
            if unresolved.len() >= 100 {
                unresolved.pop_back();
                *unresolved.back_mut().unwrap() =
                    String::from("-- Overflow: Some elements have been removed --");
            }

            unresolved.push_front(msg);
        }
    };
}
