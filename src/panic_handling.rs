use std::{panic, sync::Mutex};

use neon::{prelude::*, types::Deferred};

pub fn listen_for_crash(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();

    let h = PanicHandler {
        channel,
        deferred: Mutex::new(Some(deferred)),
    };

    let old = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        h.call(info);
        old(info);
    }));

    Ok(promise)
}

pub fn stop_listening_for_crash(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    drop(panic::take_hook());
    Ok(cx.undefined())
}

struct PanicHandler {
    channel: Channel,
    deferred: Mutex<Option<Deferred>>,
}
impl PanicHandler {
    fn settle_with<'a, F>(&self, f: F)
    where
        F: FnOnce(Deferred, TaskContext) -> NeonResult<()> + Send + 'static,
    {
        let mutex_res = self.deferred.lock();
        if let Ok(mut def_opt) = mutex_res {
            if let Some(deferred) = def_opt.take() {
                self.channel.send(move |cx| f(deferred, cx));
            }
        }
    }

    fn call(&self, info: &panic::PanicInfo) {
        let msg;
        if let Some(m) = info.payload().downcast_ref::<&str>() {
            msg = m.to_string();
        } else if let Some(m) = info.payload().downcast_ref::<String>() {
            msg = m.clone();
        } else {
            msg = "Engine crashed with no message".to_string();
        }

        let loc = info.location().unwrap();
        let error_msg = format!("{}\n{}:{}:{}", msg, loc.file(), loc.line(), loc.column());

        self.settle_with(|deferred, mut cx| {
            let error = cx.error(error_msg)?;
            deferred.reject(&mut cx, error);
            Ok(())
        })
    }
}
impl Drop for PanicHandler {
    fn drop(&mut self) {
        self.settle_with(|deferred, mut cx| {
            let undefined = cx.undefined();
            deferred.resolve(&mut cx, undefined);
            Ok(())
        });
    }
}
