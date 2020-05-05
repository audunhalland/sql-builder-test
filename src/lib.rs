pub extern crate sql_builder_macros;

use proc_macro_hack::proc_macro_hack;

#[macro_export]
macro_rules! build_query {
    ($($tokens:tt)*) => {{
        #[macro_use]
        mod _macro_result {
            $crate::sql_builder_macros::build_query!($($tokens)*);
        }
        macro_result!()
    }};
}

#[proc_macro_hack]
pub use sql_builder_macros::build_query2;
