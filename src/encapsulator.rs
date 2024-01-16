use neon::prelude::*;

use std::fmt::Debug;

/// Property name of the rust struct on the JS-object resulting from [`encapsulate`].
pub const DATA_KEY: &str = "data";
/// Property name of the [`Root`] on the JS-object having gone through [`prevent_gc`].
const ROOT_KEY: &str = "root";

pub type Method = fn(MethodContext<JsObject>) -> JsResult<JsValue>;

/// Encapsulate the `data` in a JavaScript object, with the given properties and methods exposed.
///
/// To access this data again in the methods see [`unpack`].
pub fn encapsulate<'a, C, D>(
    cx: &mut C,
    data: D,
    properties: &[(&str, Handle<JsValue>)],
    methods: &'static [(&str, Method)],
) -> JsResult<'a, JsObject>
where
    C: Context<'a>,
    D: 'static + Finalize + Send,
{
    let object = cx.empty_object();

    // JsBox allows a rust value to be contained in a JS object.
    let boxed_data = cx.boxed(data);

    object.set(cx, DATA_KEY, boxed_data)?;

    for (property_name, property) in properties {
        object.set(cx, *property_name, *property)?;
    }
    for (method_name, method) in methods {
        let method_js = JsFunction::new(cx, *method)?;
        object.set(cx, *method_name, method_js)?;
    }

    Ok(object)
}

/// Places a [`Root`] on the object, ensuring that it isn't garbage collected.
pub fn prevent_gc(cx: &mut FunctionContext, object: Handle<JsObject>) -> NeonResult<()> {
    let root = object.root(cx);
    let boxed_root = cx.boxed(root);
    object.set(cx, ROOT_KEY, boxed_root)?;
    Ok(())
}

/// Used in methods exposed by [`encapsulate`] to access the `data` stored in the context's `this`.
///
/// The function hands the data to the closure, which must explicitly state its expected type.
/// If the data is not of this type, a JavaScript exception is thrown.
//
// Would ideally return the data, but that doesn't appear to be possibly due to the nesting of references.
pub fn unpack_this<'a, D, R, F>(cx: &mut MethodContext<'a, JsObject>, callback: F) -> NeonResult<R>
where
    D: 'static + Finalize + Send,
    F: FnOnce(&mut MethodContext<'a, JsObject>, &D) -> NeonResult<R>,
{
    let boxed: Handle<JsBox<D>> = cx.this().get(cx, DATA_KEY)?;
    let data = &**boxed;

    callback(cx, data)
}

pub fn unpack<'a, C, D, F, R>(cx: &mut C, obj: Handle<'a, JsObject>, callback: F) -> NeonResult<R>
where
    C: Context<'a>,
    D: 'static + Finalize + Send + Debug,
    F: FnOnce(&mut C, &D) -> NeonResult<R>,
{
    let boxed: Handle<JsBox<D>> = obj.get(cx, DATA_KEY)?;
    let data = &**boxed;

    callback(cx, data)
}

/// Update the data stored in the object via a closure.
pub fn update_data<'a, C, D, F>(
    cx: &mut C,
    obj: Handle<'a, JsObject>,
    callback: F,
) -> NeonResult<()>
where
    C: Context<'a>,
    D: 'static + Finalize + Send + Debug,
    F: FnOnce(&mut C, &D) -> NeonResult<D>,
{
    let old_boxed: Handle<JsBox<D>> = obj.get(cx, DATA_KEY)?;
    let old_data = &**old_boxed;

    let new_data = callback(cx, old_data)?;
    let new_boxed = cx.boxed(new_data);

    obj.set(cx, DATA_KEY, new_boxed)?;

    Ok(())
}
