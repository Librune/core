// 不需要构建原生对象，直接用 js 糊一个 object 就行了，反正内容都是存放在 boa context 中，不需要在 rust 中持有

use boa_engine::{Context, Source};

// #[derive(Debug, Trace, Finalize, JsData)]
// struct ENVS {
//     #[unsafe_ignore_trace]
//     values: HashMap<String, Value>,
// }

// impl ENVS {
//     fn get_values(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
//         if let Some(object) = this.as_object() {
//             if let Some(envs) = object.downcast_ref::<ENVS>() {
//                 let json_value = json!(envs.values.clone());
//                 return Ok(JsValue::from_json(&json_value, context)?);
//             }
//         }
//         Err(JsNativeError::typ()
//             .with_message("Invalid this value")
//             .into())
//     }

//     fn set_values(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
//         if let Some(object) = this.as_object() {
//             if let Some(mut envs) = object.downcast_mut::<ENVS>() {
//                 if let Some(env) = args.get(0) {
//                     if let Ok(env_map) = env.to_json(context) {
//                         envs.values.extend(env_map.as_object().unwrap().clone());
//                         return Ok(JsValue::undefined());
//                     }
//                 }
//             }
//         }
//         Err(JsNativeError::typ()
//             .with_message("Invalid this value")
//             .into())
//     }

//     fn get_value(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
//         if let Some(object) = this.as_object() {
//             if let Some(envs) = object.downcast_ref::<ENVS>() {
//                 if let Some(key) = args.get(0) {
//                     if let Ok(key_str) = key.to_string(context) {
//                         if let Some(value) = envs.values.get(&key_str.to_std_string_escaped()) {
//                             return Ok(JsValue::from_json(value, context)?);
//                         }
//                     }
//                 }
//             }
//         }
//         Err(JsNativeError::typ()
//             .with_message("Invalid this value")
//             .into())
//     }

//     fn set_value(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
//         if let Some(object) = this.as_object() {
//             if let Some(mut envs) = object.downcast_mut::<ENVS>() {
//                 if let Some(key) = args.get(0) {
//                     if let Ok(key_str) = key.to_string(context) {
//                         if let Some(value) = args.get(1) {
//                             envs.values
//                                 .insert(key_str.to_std_string_escaped(), value.to_json(context)?);
//                             return Ok(JsValue::undefined());
//                         }
//                     }
//                 }
//             }
//         }
//         Err(JsNativeError::typ()
//             .with_message("Invalid this value")
//             .into())
//     }

//     fn clear(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
//         if let Some(object) = this.as_object() {
//             if let Some(mut envs) = object.downcast_mut::<ENVS>() {
//                 envs.values.clear();
//                 return Ok(JsValue::undefined());
//             }
//         }
//         Err(JsNativeError::typ()
//             .with_message("Invalid this value")
//             .into())
//     }
// }

// impl Class for ENVS {
//     const NAME: &'static str = "ENVS";
//     const LENGTH: usize = 0;

//     fn data_constructor(
//         _this: &JsValue,
//         _args: &[JsValue],
//         _context: &mut Context,
//     ) -> JsResult<Self> {
//         let values = HashMap::new();
//         Ok(ENVS { values })
//     }

//     fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
//         class
//             .method(
//                 js_string!("getValues"),
//                 0,
//                 NativeFunction::from_fn_ptr(Self::get_values),
//             )
//             .method(
//                 js_string!("setValues"),
//                 1,
//                 NativeFunction::from_fn_ptr(Self::set_values),
//             )
//             .method(
//                 js_string!("getValue"),
//                 1,
//                 NativeFunction::from_fn_ptr(Self::get_value),
//             )
//             .method(
//                 js_string!("setValue"),
//                 2,
//                 NativeFunction::from_fn_ptr(Self::set_value),
//             )
//             .method(
//                 js_string!("clear"),
//                 0,
//                 NativeFunction::from_fn_ptr(Self::clear),
//             );
//         Ok(())
//     }
// }

// pub fn define_envs(context: &mut Context) {
//     context
//         .register_global_class::<ENVS>()
//         .expect("the ENVS builtin shouldn't exist");
//     context
//         .eval(Source::from_bytes(
//             r#"
//       const __ENVS__ = (new ENVS()).envs;
//       const setEnv = (key, value) => {
//         __ENVS__.setValue(key, value);
//       };
//       const getEnv = (key) => {
//         return __ENVS__.getValue(key);
//       };
//       const setEnvs = (envs) => {
//         __ENVS__.setValues(envs);
//       };
//       const getEnvs = () => {
//         return __ENVS__.getValues();
//       };
//       const clearEnvs = () => {
//         __ENVS__.clear();
//       };
//   "#,
//         ))
//         .unwrap();
// }

pub fn regist_envs(ctx: &mut Context) {
    ctx.eval(Source::from_bytes(
        r#"
      let __ENVS__ = {};
      function setEnv(key, value){
        __ENVS__[key] = value;
      }
      function getEnv(key){
        console.log(__ENVS__, key,__ENVS__[0]);
        return __ENVS__[key];
      }
      function setEnvs(envs){ __ENVS__ = envs}
      function getEnvs(){return __ENVS__;}
      function clearEnvs(){
        __ENVS__ = {};
      }
    "#,
    ))
    .unwrap();
}
