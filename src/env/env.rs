use std::collections::HashMap;

use boa_engine::{
    class::{Class, ClassBuilder},
    js_string, Context, JsData, JsNativeError, JsResult, JsValue, NativeFunction,
};
use boa_gc::{Finalize, Trace};
use serde_json::{json, Value};

#[derive(Debug, Trace, Finalize, JsData)]
struct ENVS {
    #[unsafe_ignore_trace]
    envs: HashMap<String, Value>,
}

impl ENVS {
    fn get_values(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(envs) = object.downcast_ref::<ENVS>() {
                let json_value = json!(envs.envs.clone());
                return Ok(JsValue::from_json(&json_value, context)?);
            }
        }
        Err(JsNativeError::typ()
            .with_message("Invalid this value")
            .into())
    }

    fn set_values(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(mut envs) = object.downcast_mut::<ENVS>() {
                if let Some(env) = args.get(0) {
                    if let Ok(env_map) = env.to_json(context) {
                        envs.envs.extend(env_map.as_object().unwrap().clone());
                        return Ok(JsValue::undefined());
                    }
                }
            }
        }
        Err(JsNativeError::typ()
            .with_message("Invalid this value")
            .into())
    }

    fn get_value(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(envs) = object.downcast_ref::<ENVS>() {
                if let Some(key) = args.get(0) {
                    if let Ok(key_str) = key.to_string(context) {
                        if let Some(value) = envs.envs.get(&key_str.to_std_string_escaped()) {
                            return Ok(JsValue::from_json(value, context)?);
                        }
                    }
                }
            }
        }
        Err(JsNativeError::typ()
            .with_message("Invalid this value")
            .into())
    }

    fn set_value(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(mut envs) = object.downcast_mut::<ENVS>() {
                if let Some(key) = args.get(0) {
                    if let Ok(key_str) = key.to_string(context) {
                        if let Some(value) = args.get(1) {
                            envs.envs
                                .insert(key_str.to_std_string_escaped(), value.to_json(context)?);
                            return Ok(JsValue::undefined());
                        }
                    }
                }
            }
        }
        Err(JsNativeError::typ()
            .with_message("Invalid this value")
            .into())
    }

    fn clear(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(mut envs) = object.downcast_mut::<ENVS>() {
                envs.envs.clear();
                return Ok(JsValue::undefined());
            }
        }
        Err(JsNativeError::typ()
            .with_message("Invalid this value")
            .into())
    }
}

impl Class for ENVS {
    const NAME: &'static str = "__ENVS__";
    const LENGTH: usize = 0;

    fn data_constructor(
        _this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<Self> {
        let envs = HashMap::new();
        Ok(ENVS { envs })
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        class
            .method(
                js_string!("getValues"),
                0,
                NativeFunction::from_fn_ptr(Self::get_values),
            )
            .method(
                js_string!("setValues"),
                1,
                NativeFunction::from_fn_ptr(Self::set_values),
            )
            .method(
                js_string!("getValue"),
                1,
                NativeFunction::from_fn_ptr(Self::get_value),
            )
            .method(
                js_string!("setValue"),
                2,
                NativeFunction::from_fn_ptr(Self::set_value),
            )
            .method(
                js_string!("clear"),
                0,
                NativeFunction::from_fn_ptr(Self::clear),
            );
        Ok(())
    }
}

pub fn define_envs(context: &mut Context) {
    context
        .register_global_class::<ENVS>()
        .expect("the ENVS builtin shouldn't exist");
}
