use boa_engine::{
    js_string,
    object::FunctionObjectBuilder,
    property::{PropertyDescriptor, PropertyKey},
    Context, JsError, JsString, JsValue, NativeFunction,
};

fn register(ctx: &mut Context, name: &str, func: NativeFunction) -> Result<bool, JsError> {
    let object_proto = ctx.intrinsics().constructors().object().prototype();
    let func = FunctionObjectBuilder::new(ctx.realm(), func).build();
    object_proto.define_property_or_throw(
        PropertyKey::from(js_string!(name)),
        PropertyDescriptor::builder()
            .value(func)
            .writable(true)
            .enumerable(false)
            .configurable(true),
        ctx,
    )
}

fn regist_to_query(ctx: &mut Context) -> Result<bool, JsError> {
    let func = NativeFunction::from_fn_ptr(|this, _args, context| {
        let this_obj = this.to_object(context)?;
        let mut result = String::new();
        let keys = this_obj.own_property_keys(context)?;
        keys.iter().for_each(|key| {
            let key_str = key.to_string();
            let value = this_obj
                .get::<JsString>(js_string!(key_str.clone()), context)
                .unwrap();
            let value_str = value.to_string(context).unwrap();
            result.push_str(&format!(
                "{}={}&",
                key_str,
                value_str.to_std_string_escaped()
            ));
        });
        result.pop();
        Ok(JsValue::String(js_string!(result)))
    });
    register(ctx, "toQuery", func)
}

pub fn extend_object(ctx: &mut Context) {
    // Register the toQuery function to the Object prototype
    regist_to_query(ctx).expect("Failed to register toQuery function");
}
