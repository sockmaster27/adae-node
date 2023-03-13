use std::sync::{Arc, Mutex, MutexGuard};

use neon::{
    context::{CallContext, Context},
    result::Throw,
    types::{Finalize, JsObject},
};

pub struct SharedEngine(
    // Arc allows each track to also have a reference
    // Mutex allows the value to be borrowed mutably from one place at a time
    // Option allows the engine to be dropped which stops audio
    Arc<Mutex<Option<ardae::Engine>>>,
);
impl SharedEngine {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Some(ardae::Engine::new()))))
    }

    fn lock(
        &self,
        cx: &mut CallContext<JsObject>,
    ) -> Result<MutexGuard<Option<ardae::Engine>>, Throw> {
        self.0
            .lock()
            .or_else(|_| cx.throw_error("A panic has ocurred while holding a lock on the engine."))
    }

    pub fn with_inner<'a, R, F>(
        &self,
        cx: &mut CallContext<'a, JsObject>,
        callback: F,
    ) -> Result<R, Throw>
    where
        F: FnOnce(&mut CallContext<'a, JsObject>, &mut ardae::Engine) -> Result<R, Throw>,
    {
        let mut option_guard = self.lock(cx)?;
        let engine = match *option_guard {
            Some(ref mut engine) => engine,
            None => {
                return cx.throw_error("Engine has already been closed.");
            }
        };

        callback(cx, engine)
    }

    pub fn close(&self, cx: &mut CallContext<JsObject>) -> Result<(), Throw> {
        let mut option = self.lock(cx)?;
        option.take();
        Ok(())
    }
}
impl Clone for SharedEngine {
    /// Clones engine via Arc.
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
impl Finalize for SharedEngine {}
