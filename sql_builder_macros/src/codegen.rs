use proc_macro2::TokenStream;
use quote::quote;
use std::iter::IntoIterator;

use crate::blocks;
use crate::parse;

struct GenData {
    sql_str_ident: proc_macro2::Ident,
    query_args_ident: proc_macro2::Ident,
}

impl Default for GenData {
    fn default() -> Self {
        GenData {
            sql_str_ident: quote::format_ident!("sql_str"),
            query_args_ident: quote::format_ident!("query_args"),
        }
    }
}

fn gen_push_stmt(push: blocks::Push, gen_data: &GenData) -> TokenStream {
    match push {
        blocks::Push::Lit(lit_str) => {
            let sql_str_ident = &gen_data.sql_str_ident;
            quote! {
                write!(&mut #sql_str_ident, #lit_str).unwrap();
            }
        }
        blocks::Push::Bind => {
            let sql_str_ident = &gen_data.sql_str_ident;
            let query_args_ident = &gen_data.query_args_ident;
            quote! {
                write!(&mut #sql_str_ident, "${}", #query_args_ident.len()).unwrap();
            }
        }
    }
}

fn gen_pushes(pushes: Vec<blocks::Push>, gen_data: &GenData) -> TokenStream {
    let stmts: Vec<_> = pushes
        .into_iter()
        .map(|push| gen_push_stmt(push, gen_data))
        .collect();

    quote! {
        #(#stmts)*
    }
}

fn gen_branch(branch: blocks::Branch, gen_data: &GenData) -> TokenStream {
    let then = gen_blocks(branch.then, &gen_data);
    match branch.cond {
        blocks::Cond::If(cond) => quote! {
            if #cond { #then }
        },
        blocks::Cond::ElseIf(cond) => quote! {
            else if #cond { #then }
        },
        blocks::Cond::Else => quote! {
            else { #then }
        },
    }
}

fn gen_branches(branches: Vec<blocks::Branch>, gen_data: &GenData) -> TokenStream {
    let stmts: Vec<_> = branches
        .into_iter()
        .map(|branch| gen_branch(branch, gen_data))
        .collect();

    quote! {
        #(#stmts)*
    }
}

fn gen_block(block: blocks::Block, gen_data: &GenData) -> TokenStream {
    match block {
        blocks::Block::Push(pushes) => gen_pushes(pushes, gen_data),
        blocks::Block::Branch(branches) => gen_branches(branches, gen_data),
    }
}

fn gen_blocks(blocks: Vec<blocks::Block>, gen_data: &GenData) -> TokenStream {
    let output: Vec<_> = blocks
        .into_iter()
        .map(|block| gen_block(block, &gen_data))
        .collect();

    quote! {
        #(#output)*
    }
}

pub fn codegen(ast: parse::BuilderAST) -> TokenStream {
    let blocks = blocks::create_blocks(ast.constituents);
    let gen_data = GenData::default();
    let statements = gen_blocks(blocks, &gen_data);

    let sql_str_ident = &gen_data.sql_str_ident;
    let query_args_ident = &gen_data.query_args_ident;

    quote! {
        macro_rules! macro_result {
            ($($tokens:tt)*) => {{
                use std::fmt::Write;
                let mut #sql_str_ident = String::new();
                let mut #query_args_ident: Vec<bool> = Vec::new();

                #statements

                #sql_str_ident
            }}
        }
    }
}

pub fn codegen2(ast: parse::BuilderAST) -> TokenStream {
    let blocks = blocks::create_blocks(ast.constituents);
    let gen_data = GenData::default();
    let statements = gen_blocks(blocks, &gen_data);

    let sql_str_ident = &gen_data.sql_str_ident;
    let query_args_ident = &gen_data.query_args_ident;

    quote! {
        {
            use std::fmt::Write;
            let mut #sql_str_ident = String::new();
            let mut #query_args_ident: Vec<bool> = Vec::new();

            #statements

            #sql_str_ident
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_gen_blocks(ast: parse::BuilderAST) -> TokenStream {
        let blocks = blocks::create_blocks(ast.constituents);
        gen_blocks(blocks, &GenData::default())
    }

    #[test]
    fn gen_blocks_push_lit() {
        let stream = test_gen_blocks(
            syn::parse2(quote! {
                "SELECT"
            })
            .unwrap(),
        );
        assert_eq!(
            format!("{}", stream),
            "write ! ( sql_str , \"SELECT\" ) ? ;"
        );
    }

    #[test]
    fn gen_blocks_push_bind() {
        let stream = test_gen_blocks(
            syn::parse2(quote! {
                42
            })
            .unwrap(),
        );
        assert_eq!(
            format!("{}", stream),
            "write ! ( sql_str , \"${}\" , sql_args . len ( ) ) ? ;"
        );
    }

    #[test]
    fn gen_blocks_branches() {
        let stream = test_gen_blocks(
            syn::parse2(quote! {
                if true {
                    "SELECT"
                } else {
                    "DELETE"
                }
            })
            .unwrap(),
        );
        assert_eq!(
            format!("{}", stream),
            "if true { write ! ( sql_str , \"SELECT\" ) ? ; } else { write ! ( sql_str , \"DELETE\" ) ? ; } else"
        );
    }
}
