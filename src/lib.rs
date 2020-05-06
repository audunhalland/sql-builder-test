pub extern crate sql_builder_macros;

use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack]
pub use sql_builder_macros::build_query;
