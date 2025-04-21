use std::io::Write;

use boa_engine::JsError;
use boa_engine::{Context, JsResult};
use boa_gc::{Finalize, Trace};
use boa_runtime::{ConsoleState, Logger};
use book_core::BookCore;

#[derive(Debug, Trace, Finalize)]
pub struct CustLogger;

impl Logger for CustLogger {
    #[inline]
    fn log(&self, msg: String, state: &ConsoleState, _context: &mut Context) -> JsResult<()> {
        let indent = state.indent();
        writeln!(std::io::stdout(), "这是自定义的日志打印器：{msg:>indent$}")
            .map_err(JsError::from_rust)
    }

    #[inline]
    fn info(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()> {
        self.log(msg, state, context)
    }

    #[inline]
    fn warn(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()> {
        self.log(msg, state, context)
    }

    #[inline]
    fn error(&self, msg: String, state: &ConsoleState, _context: &mut Context) -> JsResult<()> {
        let indent = state.indent();
        writeln!(std::io::stderr(), "{msg:>indent$}").map_err(JsError::from_rust)
    }
}

#[test]
fn test() {
    let code = include_str!("./wk8.js");
    let mut wk8 = BookCore::init(code.to_string());
    wk8.regist_cust_logger(CustLogger);
    let res = wk8.eval("test();".to_string());
    println!("{}", res.unwrap());
}
