use std::fmt::Debug;
use std::sync::{Arc, Mutex, MutexGuard};

use neon::{context::Context, result::Throw, types::Finalize};

pub struct SharedEngine(
    // Arc allows each track to also have a reference
    // Mutex allows the value to be borrowed mutably from one place at a time
    // Option allows the engine to be dropped which stops audio
    Arc<Mutex<Option<adae::Engine>>>,
);
impl SharedEngine {
    pub fn empty() -> Self {
        Self(Arc::new(Mutex::new(Some(adae::Engine::empty()))))
    }

    pub fn new(
        config: adae::config::Config,
    ) -> Result<
        (Self, impl Iterator<Item = adae::error::ImportError>),
        adae::error::InvalidConfigError,
    > {
        let (engine, import_errors) = adae::Engine::new(config, &adae::EngineState::default())?;
        Ok((Self(Arc::new(Mutex::new(Some(engine)))), import_errors))
    }

    pub fn dummy() -> Self {
        Self(Arc::new(Mutex::new(Some(adae::Engine::dummy()))))
    }

    fn lock<'a, C>(&self, cx: &mut C) -> Result<MutexGuard<Option<adae::Engine>>, Throw>
    where
        C: Context<'a>,
    {
        self.0
            .lock()
            .or_else(|_| cx.throw_error("A panic has ocurred while holding a lock on the engine."))
    }

    pub fn with_inner<'a, C, R, F>(&self, cx: &mut C, callback: F) -> Result<R, Throw>
    where
        C: Context<'a>,
        F: FnOnce(&mut C, &mut adae::Engine) -> Result<R, Throw>,
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

    pub fn close(&self) {
        let lock_result = self.0.lock();
        if let Ok(mut option) = lock_result {
            drop(option.take());
        }
        // If the lock is poisoned, then the user has already been notified of the error, and nothing more should be done.
    }
}
impl Clone for SharedEngine {
    /// Clones engine via Arc.
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
impl Debug for SharedEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedEngine").finish()
    }
}
impl Finalize for SharedEngine {}
