use boa_engine::Context;
use console::regist_console;
use uuid::{regist_is_uuid, regist_uuid};
use xml2json::regist_xml_to_json;

pub mod console;
pub mod uuid;
pub mod xml2json;

pub fn set_global_extra_bindings(ctx: &mut Context) {
    regist_console(ctx);
    regist_xml_to_json(ctx);
    regist_uuid(ctx);
    regist_is_uuid(ctx);
}
