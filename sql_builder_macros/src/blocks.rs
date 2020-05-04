use std::iter::IntoIterator;
use syn::{Expr, ExprLit, Lit, LitStr};

use crate::parse;

pub enum Push {
    Lit(LitStr),
    Bind,
}

pub enum Cond {
    If(Box<Expr>),
    ElseIf(Box<Expr>),
    Else,
}

pub struct Branch {
    pub cond: Cond,
    pub then: Vec<Block>,
}

pub enum Block {
    Push(Vec<Push>),
    Branch(Vec<Branch>),
}

pub fn create_blocks(constituents: Vec<parse::Constituent>) -> Vec<Block> {
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
                    parse::Constituent::Literal(lit_str) => Push::Lit(lit_str),
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

                let iff = match peek_ast.next().unwrap() {
                    parse::Constituent::If(iff) => iff,
                    _ => panic!(),
                };

                let mut branches = vec![];
                branches.push(Branch {
                    cond: Cond::If(iff.cond),
                    then: create_blocks(iff.then_branch.constituents),
                });

                let mut next = iff.else_branch;

                while let Some(else_branch) = next {
                    match else_branch {
                        parse::Else::If(_token, iff) => {
                            branches.push(Branch {
                                cond: Cond::ElseIf(iff.cond),
                                then: create_blocks(iff.then_branch.constituents),
                            });
                            next = iff.else_branch;
                        }
                        parse::Else::Block(_token, block) => {
                            branches.push(Branch {
                                cond: Cond::Else,
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
