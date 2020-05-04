pub extern crate sql_builder_macros;

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
