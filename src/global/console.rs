use boa_engine::{property::Attribute, Context};
use boa_runtime::Console;

pub fn regist_console(ctx: &mut Context) {
    let console = Console::init(ctx);
    ctx.register_global_property(Console::NAME, console, Attribute::all())
        .expect("the console builtin shouldn't exist");
}
