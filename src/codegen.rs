use proc_macro2::TokenStream;
use std::iter::IntoIterator;
use syn::{Expr, ExprLit, Lit};

use crate::parse;

enum Push {
    Lit,
    Bind,
}

struct Branch {
    cond: Option<Box<Expr>>,
    then: Vec<Block>,
}

enum Block {
    Push(Vec<Push>),
    Branch(Vec<Branch>),
}

fn create_blocks(constituents: Vec<parse::Constituent>) -> Vec<Block> {
    let mut peek_ast = constituents.into_iter().peekable();
    let mut pushes = vec![];
    let mut blocks = vec![];

    loop {
        match peek_ast.peek() {
            None => {
                if !pushes.is_empty() {
                    blocks.push(Block::Push(pushes));
                }
                return blocks;
            }
            Some(parse::Constituent::Literal(_)) => {
                pushes.push(match peek_ast.next().unwrap() {
                    parse::Constituent::Literal(_) => Push::Lit,
                    _ => panic!(),
                });
            }
            Some(parse::Constituent::Bind(_)) => {
                pushes.push(match peek_ast.next().unwrap() {
                    parse::Constituent::Bind(_) => Push::Bind,
                    _ => panic!(),
                });
            }
            Some(parse::Constituent::Block(_)) => {
                pushes.push(match peek_ast.next().unwrap() {
                    parse::Constituent::Block(_) => Push::Bind,
                    _ => panic!(),
                });
            }
            Some(parse::Constituent::If(_)) => {
                if !pushes.is_empty() {
                    blocks.push(Block::Push(pushes));
                    pushes = vec![];
                }

                let if_expr = match peek_ast.next().unwrap() {
                    parse::Constituent::If(if_expr) => if_expr,
                    _ => panic!(),
                };

                let mut branches = vec![];
                branches.push(Branch {
                    cond: Some(if_expr.cond),
                    then: create_blocks(if_expr.then_branch.constituents),
                });

                let mut next = if_expr.else_branch;

                while let Some(else_branch) = next {
                    match else_branch {
                        parse::Else::If(_token, iff) => {
                            branches.push(Branch {
                                cond: Some(iff.cond),
                                then: create_blocks(iff.then_branch.constituents),
                            });
                            next = iff.else_branch;
                        }
                        parse::Else::Block(_token, block) => {
                            branches.push(Branch {
                                cond: None,
                                then: create_blocks(block.constituents),
                            });
                            break;
                        }
                    }
                }

                blocks.push(Block::Branch(branches));
            }
        }
    }
}

pub fn codegen(ast: parse::BuilderAST) -> TokenStream {
    panic!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;
    use quote::quote;

    #[test]
    fn conditional() {
        let foo = Some(1u32);
        let ast: parse::BuilderAST = syn::parse2(quote! {
            "SELECT yo FROM stuff "
            "WHERE "
            if let Some(bar) = #foo {
                "id IN (" bar ") "
            } else {
                "TRUE "
            }
            "AND "
            "ORDER BY "
            "date"
        })
        .unwrap();
        let blocks = create_blocks(ast.constituents);
        assert_eq!(blocks.len(), 3);
    }
}
