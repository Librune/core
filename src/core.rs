use boa_engine::{Context, Source};
use tokio::runtime::Runtime;

use crate::runtime::init_runtime;

pub struct BookCore {
    pub context: Context,
}

impl BookCore {
    pub fn new() -> Self {
        let mut core = Self {
            context: Context::default(),
        };
        init_runtime(&mut core.context);
        core
    }

    pub fn eval(&mut self, code: String) -> Result<String, String> {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let ctx = &mut self.context;
            ctx.eval(Source::from_bytes(code.as_bytes()))
                .map(|value| {
                    if value.is_object() {
                        value.to_json(ctx).unwrap().to_string()
                    } else {
                        value.to_string(ctx).unwrap().to_std_string_escaped()
                    }
                })
                .map_err(|err| err.to_string())
        })
    }
}
