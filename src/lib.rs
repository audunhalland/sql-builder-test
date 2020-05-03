use proc_macro::TokenStream;
use quote::quote;

#[allow(dead_code)]
mod builder;
mod parse;

#[proc_macro]
pub fn build_query(_input: TokenStream) -> TokenStream {
    let dummy = quote! { "lol" };
    dummy.into()
}
