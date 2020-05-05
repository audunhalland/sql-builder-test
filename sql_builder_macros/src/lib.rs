#![allow(dead_code)]

use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;

mod blocks;
mod builder;
mod codegen;
mod parse;

#[proc_macro]
pub fn build_query(input: TokenStream) -> TokenStream {
    let ast: parse::BuilderAST = syn::parse_macro_input!(input);
    codegen::codegen(ast).into()
}

#[proc_macro_hack]
pub fn build_query2(input: TokenStream) -> TokenStream {
    let ast: parse::BuilderAST = syn::parse_macro_input!(input);
    codegen::codegen2(ast).into()
}
