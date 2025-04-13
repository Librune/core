use boa_engine::{js_error, js_string, Context, JsArgs, JsValue, NativeFunction};
use quickxml_to_serde::{xml_string_to_json, Config, NullValue};

pub fn regist_xml_to_json(ctx: &mut Context) {
    let function = NativeFunction::from_fn_ptr(|_this, args, context| {
        let xml = args.get_or_undefined(0);
        if xml.is_null_or_undefined() {
            return Err(js_error!("XMLString is undefined"));
        }
        let xml_str = xml.to_string(context).unwrap().to_std_string_escaped();
        let conf = Config::new_with_custom_values(false, "", "text", NullValue::Null);
        let json = xml_string_to_json(xml_str.to_owned(), &conf).expect("Malformed XML");
        let js_value = JsValue::from_json(&json, context)?;
        let js_object = js_value.as_object().unwrap();
        Ok(JsValue::Object(js_object.clone()))
    });
    ctx.register_global_builtin_callable(js_string!("xml2Json"), 1, function)
        .expect("Failed to register xmlToJson");
}
