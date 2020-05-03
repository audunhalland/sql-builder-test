#![allow(dead_code)]

use proc_macro::TokenStream;

mod blocks;
mod builder;
mod codegen;
mod parse;

#[proc_macro]
pub fn build_query(input: TokenStream) -> TokenStream {
    let ast: parse::BuilderAST = syn::parse_macro_input!(input);
    codegen::codegen(ast).into()
}
