// use boa_engine::{Context, JsError, JsResult};
// use boa_gc::{Finalize, Trace};
// use boa_runtime::{ConsoleState, Logger};
// use std::io::Write;

// #[derive(Debug, Trace, Finalize)]
// pub struct DefaultLogger;

// impl Logger for DefaultLogger {
//     #[inline]
//     fn log(&self, msg: String, state: &ConsoleState, _context: &mut Context) -> JsResult<()> {
//         let indent = state.indent();
//         writeln!(std::io::stdout(), "{msg:>indent$}").map_err(JsError::from_rust)
//     }

//     #[inline]
//     fn info(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()> {
//         self.log(msg, state, context)
//     }

//     #[inline]
//     fn warn(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()> {
//         self.log(msg, state, context)
//     }

//     #[inline]
//     fn error(&self, msg: String, state: &ConsoleState, _context: &mut Context) -> JsResult<()> {
//         let indent = state.indent();
//         writeln!(std::io::stderr(), "{msg:>indent$}").map_err(JsError::from_rust)
//     }
// }
