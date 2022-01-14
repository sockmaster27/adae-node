use neon::context::Context;
use neon::prelude::*;
use std::cell::RefCell;

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
    /// Construct a new JavaScript object.
    /// The returned object must adhere to the interface defined in the `index.d.ts` file.
    fn constructor(mut cx: FunctionContext) -> JsResult<JsObject> {
        let object = cx.this();
        let root = object.root(&mut cx);

        // JsBox allows a rust value to be contained in a JS object.
        // RefCell allows its value to be borrowed as mutable.
        // Option allows its value to be dropped by rust.
        let boxed_engine = cx.boxed(RefCell::new(Some(JsEngine {
            engine: ardae::Engine::new(),
            root,
        })));

        object.set(&mut cx, DATA_KEY, boxed_engine)?;

        // Insert methods
        let set_volume_js = JsFunction::new(&mut cx, Self::set_volume)?;
        object.set(&mut cx, "setVolume", set_volume_js)?;
        let get_peaks_js = JsFunction::new(&mut cx, Self::get_peaks)?;
        object.set(&mut cx, "getPeak", get_peaks_js)?;
        let close_js = JsFunction::new(&mut cx, Self::close)?;
        object.set(&mut cx, "close", close_js)?;

        Ok(object)
    }

    fn set_volume(mut cx: MethodContext<JsObject>) -> JsResult<JsUndefined> {
        let value_js: JsNumber = *cx.argument(0)?;
        let value: f32 = value_js.value(&mut cx) as f32;

        let handle = cx.this().get(&mut cx, DATA_KEY)?;
        let boxed: JsBox<RefCell<Option<JsEngine>>> =
            *handle.downcast(&mut cx).or_throw(&mut cx)?;
        let ref_cell = &*boxed;
        let option = &mut *ref_cell.borrow_mut();
        let js_engine = match option {
            Some(x) => x,
            None => return cx.throw_error("This shit empty"),
        };

        js_engine.engine.set_volume(value);

        Ok(cx.undefined())
    }

    fn get_peaks(mut cx: MethodContext<JsObject>) -> JsResult<JsNumber> {
        let handle = cx.this().get(&mut cx, DATA_KEY)?;
        let boxed: JsBox<RefCell<Option<JsEngine>>> =
            *handle.downcast(&mut cx).or_throw(&mut cx)?;
        let ref_cell = &*boxed;
        let option = &mut *ref_cell.borrow_mut();
        let js_engine = match option {
            Some(x) => x,
            None => return cx.throw_error("This shit empty"),
        };

        Ok(cx.number(js_engine.engine.get_peak()))
    }

    fn close(mut cx: MethodContext<JsObject>) -> JsResult<JsUndefined> {
        let this = cx.this();

        let handle = cx.this().get(&mut cx, DATA_KEY)?;
        let boxed: JsBox<RefCell<Option<JsEngine>>> =
            *handle.downcast(&mut cx).or_throw(&mut cx)?;
        let ref_cell = &*boxed;
        let option = &mut *ref_cell.borrow_mut();

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
