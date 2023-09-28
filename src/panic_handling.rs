use std::{
    backtrace::Backtrace,
    cell::Cell,
    panic,
    sync::{Mutex, OnceLock},
};

use neon::{prelude::*, types::Deferred};

static CHANNEL: OnceLock<Mutex<Option<Channel>>> = OnceLock::new();
static DEFERREDS: OnceLock<Mutex<Vec<Deferred>>> = OnceLock::new();
thread_local! {
    static ENABLED: Cell<bool> = Cell::new(true);
}

pub fn listen_for_crash(mut cx: FunctionContext) -> JsResult<JsPromise> {
    CHANNEL.get_or_init(|| Mutex::new(Some(cx.channel())));

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

    // Deactive for the main thread, since Neon's panic handling is more precise here.
    ENABLED.with(|e| e.set(false));

    Ok(promise)
}

pub fn stop_listening_for_crash(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // Drop the channel
    CHANNEL.get().map(|m| m.lock().unwrap().take());

    let deferreds_opt = DEFERREDS.get();

    if let Some(deferreds_mutex) = deferreds_opt {
        let mut deferreds = deferreds_mutex.lock().unwrap();
        for deferred in deferreds.drain(..) {
            let undefined = cx.undefined();
            deferred.resolve(&mut cx, undefined);
        }
    }

    Ok(cx.undefined())
}

fn panic_hook(info: &panic::PanicInfo) {
    let enabled = ENABLED.with(|e| e.get());
    if !enabled {
        return;
    }

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

    // 😔
    let channel_mutex_opt = CHANNEL.get();
    if let Some(channel_mutex) = channel_mutex_opt {
        let channel_opt = channel_mutex.lock().unwrap();
        if let Some(ref channel) = *channel_opt {
            channel.send(move |mut cx| {
                let deferreds_opt = DEFERREDS.get();
                if let Some(deferreds_mutex) = deferreds_opt {
                    let mut deferreds = deferreds_mutex.lock().unwrap();

                    for deferred in deferreds.drain(..) {
                        let error = cx.error(&error_msg)?;
                        deferred.reject(&mut cx, error);
                    }
                }

                Ok(())
            });
        }
    }
}
