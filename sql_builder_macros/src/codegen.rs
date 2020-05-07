use proc_macro2::TokenStream;
use quote::quote;
use std::iter::IntoIterator;

use crate::blocks;
use crate::parse;

struct GenData {
    builder_ident: proc_macro2::Ident,
}

impl Default for GenData {
    fn default() -> Self {
        GenData {
            builder_ident: quote::format_ident!("builder"),
        }
    }
}

fn gen_push_stmt(push: blocks::Push, gen_data: &GenData) -> TokenStream {
    let builder_ident = &gen_data.builder_ident;
    match push {
        blocks::Push::Lit(lit_str) => {
            quote! {
                #builder_ident.push_sql(#lit_str);
            }
        }
        blocks::Push::Bind(expr) => {
            quote! {
                #builder_ident.push_arg(#expr);
            }
        }
        blocks::Push::Empty => {
            quote! {}
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
    let keywords = branch.keywords;
    if let Some(cond) = branch.cond {
        quote! {
            #keywords #cond { #then }
        }
    } else {
        quote! {
            #keywords { #then }
        }
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

    let builder_ident = &gen_data.builder_ident;

    let builder_path: syn::Path = syn::parse_str("sql_builder_test::Builder").unwrap();

    quote! {
        {
            use std::fmt::Write;

            let mut #builder_ident = #builder_path::new();

            #statements

            #builder_ident.build()
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
            "write ! ( & mut sql_str , \"SELECT\" ) . unwrap ( ) ;"
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
            "write ! ( & mut sql_str , \"${}\" , query_args . len ( ) ) . unwrap ( ) ;"
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
            "if true { write ! ( & mut sql_str , \"SELECT\" ) . unwrap ( ) ; } else { write ! ( & mut sql_str , \"DELETE\" ) . unwrap ( ) ; }"
        );
    }

    #[test]
    fn experiment() {
        use std::fmt::Write;

        struct Builder {
            sql: String,
            args: Vec<String>,
            arg_count: usize,
        }

        fn fmt0(b: &mut Builder) {
            write!(b.sql, "SELECT ").unwrap();
        }
        fn fmt1_b0(b: &mut Builder) {
            write!(b.sql, "arg ").unwrap();
            write!(b.sql, "${}", b.arg_count).unwrap();
            b.arg_count += 1;
        }
        fn fmt1_b0_0(b: &mut Builder) {
            write!(b.sql, "SUB ").unwrap();
        }

        let a = Some("value".to_owned());

        let f0 = move |b: &mut Builder| {
            fmt0(b);
        };
        let f1_0 = move |b: &mut Builder| {
            fmt1_b0_0(b);
        };
        let f1 = move |b: &mut Builder| {
            if let Some(arg) = a {
                fmt1_b0(b);
                b.args.push(arg);
                f1_0(b);
            }
        };

        let mut b = Builder {
            sql: String::new(),
            args: Vec::new(),
            arg_count: 0,
        };

        if true {
            // Dynamic
            f0(&mut b);
            f1(&mut b);
        } else {
            // Static
            fmt0(&mut b);
            fmt1_b0(&mut b);
            fmt1_b0_0(&mut b);
        }

        assert_eq!(b.sql, "SELECT arg SUB ");
    }
}
