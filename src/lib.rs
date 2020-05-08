pub extern crate sql_builder_macros;

use proc_macro_hack::proc_macro_hack;

pub struct Builder {
    sql: String,
    args_count: usize,
    args_size: usize,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            sql: String::new(),
            args_count: 0,
            args_size: 0,
        }
    }

    #[inline]
    pub fn push_sql(&mut self, sql: &'static str) {
        use std::fmt::Write;
        write!(self.sql, "{}", sql).unwrap();
    }

    pub fn push_sql_arg(&mut self) {
        use std::fmt::Write;
        write!(self.sql, "${}", self.args_count).unwrap();
        self.args_count += 1;
    }

    #[inline]
    pub fn push_bind_arg<T>(&mut self, arg: T)
    where
        T: Sized,
    {
        self.args_size += std::mem::size_of_val(&arg);
    }

    pub fn build(self) -> Query {
        Query {
            sql: self.sql,
            args_count: self.args_count,
            args_size: self.args_size,
        }
    }
}

pub struct Query {
    pub sql: String,
    pub args_count: usize,
    pub args_size: usize,
}

#[proc_macro_hack]
pub use sql_builder_macros::build_query;
