use std::{
    backtrace::Backtrace,
    panic,
    sync::{Mutex, OnceLock},
};

use neon::{prelude::*, types::Deferred};

static CHANNEL: OnceLock<Channel> = OnceLock::new();
static DEFERREDS: OnceLock<Mutex<Vec<Deferred>>> = OnceLock::new();

pub fn listen_for_crash(mut cx: FunctionContext) -> JsResult<JsPromise> {
    CHANNEL.get_or_init(|| cx.channel());

    let (deferred, promise) = cx.promise();

    DEFERREDS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap()
        .push(deferred);

    let old = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        panic_hook(info);
        old(info);
    }));

    Ok(promise)
}

pub fn stop_listening_for_crash(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let (deferred, promise) = cx.promise();

    DEFERREDS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap()
        .push(deferred);

    settle_all_with(|deferred, cx| {
        let undefined = cx.undefined();
        deferred.resolve(cx, undefined);
        Ok(())
    });

    Ok(promise)
}

fn panic_hook(info: &panic::PanicInfo) {
    let msg = if let Some(m) = info.payload().downcast_ref::<&str>() {
        m.to_string()
    } else if let Some(m) = info.payload().downcast_ref::<String>() {
        m.clone()
    } else {
        "Engine crashed with no message".to_string()
    };

    let loc = info.location().unwrap();
    let error_msg = format!(
        "{}\n{}:{}:{}\n{}",
        msg,
        loc.file(),
        loc.line(),
        loc.column(),
        Backtrace::force_capture()
    );

    settle_all_with(move |deferred, cx| {
        let error = cx.error(&error_msg)?;
        deferred.reject(cx, error);
        Ok(())
    })
}

fn settle_all_with<F>(mut f: F)
where
    F: FnMut(Deferred, &mut TaskContext) -> NeonResult<()> + Send + 'static,
{
    let channel_opt = CHANNEL.get();
    if let Some(channel) = channel_opt {
        channel.send(move |mut cx| {
            let deferreds_opt = DEFERREDS.get();
            if let Some(deferreds_mutex) = deferreds_opt {
                let mut deferreds = deferreds_mutex.lock().unwrap();
                deferreds.drain(..).try_for_each(|d| f(d, &mut cx))?;
            }

            Ok(())
        });
    }
}
