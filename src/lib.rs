use neon::context::Context;
use neon::prelude::*;
use std::sync::Mutex;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("Engine", JsEngine::constructor)?;
    Ok(())
}

const DATA_KEY: &str = "data";

/// A wrapper around the `ardae::Engine` compatible with Neon's API.
///
/// Note that even though these functions are organized as being associated to the `JsEngine`,
/// they're supposed to be exposed directly by Neon, and do not follow rust conventions.
struct JsEngine {
    engine: ardae::Engine,

    /// Holding a `Root` to the owning object ensures that it isn't garbage collected.
    /// This means that the audio won't unexpectedly stop.
    #[allow(dead_code)]
    root: Root<JsObject>,
}
impl JsEngine {
    /// Utility function for unpacking the JsEngine from all the stuff it's wrapped in.
    ///
    /// In contrast to all other functions on this struct, this shouldn't be a method on the JS object.
    //
    // Would ideally return the JsEngine, but that doesn't appear to be possibly due to the nesting of references.
    fn unpack_engine<
        'a,
        R: Value,
        F: FnOnce(CallContext<'a, JsObject>, &mut JsEngine) -> JsResult<'a, R>,
    >(
        mut cx: CallContext<'a, JsObject>,
        callback: F,
    ) -> JsResult<R> {
        let handle = cx.this().get(&mut cx, DATA_KEY)?;
        let boxed: JsBox<Mutex<Option<JsEngine>>> = *handle.downcast(&mut cx).or_throw(&mut cx)?;
        let mutex = &*boxed;
        let option = &mut *match mutex.lock() {
            Ok(x) => x,
            Err(_) => {
                return cx.throw_error("Another thread panicked while holding a lock on the engine")
            }
        };
        let js_engine = match option {
            Some(x) => x,
            None => return cx.throw_error("Engine has been closed"),
        };

        callback(cx, js_engine)
    }

    /// Construct a new JavaScript object.
    /// The returned object must adhere to the interface defined in the `index.d.ts` file.
    fn constructor(mut cx: FunctionContext) -> JsResult<JsObject> {
        let object = cx.this();
        let root = object.root(&mut cx);

        // JsBox allows a rust value to be contained in a JS object.
        // Mutex allows its value to be borrowed as mutable, blocking until it's available.
        // Option allows its value to be dropped by rust.
        let boxed_engine = cx.boxed(Mutex::new(Some(JsEngine {
            engine: ardae::Engine::new(),
            root,
        })));

        object.set(&mut cx, DATA_KEY, boxed_engine)?;

        /// Macro for inserting a function as a method on the object.
        ///
        /// Takes the name that the method should be exposed with, and the function itself (has to be a function pointer).
        macro_rules! insert_method {
            ($name:expr, $function:expr) => {
                let function_js = JsFunction::new(&mut cx, $function)?;
                object.set(&mut cx, $name, function_js)?;
            };
        }

        insert_method!("setVolume", Self::set_volume);
        insert_method!("getPeak", Self::get_peak);
        insert_method!("close", Self::close);

        Ok(object)
    }

    fn set_volume(cx: MethodContext<JsObject>) -> JsResult<JsUndefined> {
        Self::unpack_engine(cx, |mut cx, js_engine| {
            let value_js: JsNumber = *cx.argument(0)?;
            let value: f32 = value_js.value(&mut cx) as f32;

            js_engine.engine.set_volume(value);

            Ok(cx.undefined())
        })
    }

    fn get_peak(cx: MethodContext<JsObject>) -> JsResult<JsNumber> {
        Self::unpack_engine(cx, |mut cx, js_engine| {
            Ok(cx.number(js_engine.engine.get_peak()))
        })
    }

    fn close(mut cx: MethodContext<JsObject>) -> JsResult<JsUndefined> {
        let this = cx.this();

        let handle = this.get(&mut cx, DATA_KEY)?;
        let boxed: JsBox<Mutex<Option<JsEngine>>> = *handle.downcast(&mut cx).or_throw(&mut cx)?;
        let mutex = &*boxed;
        let option = &mut *match mutex.lock() {
            Ok(x) => x,
            Err(_) => {
                return cx.throw_error("Another thread panicked while holding a lock on the engine")
            }
        };

        let error_thrower = |mut cx: MethodContext<JsObject>| -> JsResult<JsUndefined> {
            cx.throw_error("Engine has been closed")
        };
        let error_thrower_js = JsFunction::new(&mut cx, error_thrower)?;

        let prop_names = this.get_own_property_names(&mut cx)?.to_vec(&mut cx)?;
        for key in prop_names {
            this.set(&mut cx, key, error_thrower_js)?;
        }

        let undefined = cx.undefined();
        this.set(&mut cx, DATA_KEY, undefined)?;

        // Drop the JsEngine, and free the Root to allow the object to be garbage collected.
        drop(option.take());

        Ok(cx.undefined())
    }
}

// This just has to be here.
impl Finalize for JsEngine {}
