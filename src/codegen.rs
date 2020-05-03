use proc_macro2::TokenStream;
use std::iter::IntoIterator;
use syn::{Expr, ExprLit, Lit};

use crate::blocks;
use crate::parse;

pub fn codegen(ast: parse::BuilderAST) -> TokenStream {
    let blocks = blocks::create_blocks(ast.constituents);

    panic!()
}
