use std::iter::IntoIterator;
use std::rc::Rc;

use proc_macro2::TokenStream;
use quote::quote;

use crate::parse;

pub struct Counter(u32);

impl Counter {
    pub fn new() -> Self {
        Self(0)
    }

    fn next(&mut self) -> u32 {
        let next = self.0;
        self.0 += 1;
        next
    }
}

pub struct NodeId {
    pub parent: Option<Rc<NodeId>>,
    pub id: u32,
    pub local_index: u16,
}

pub struct Parent(Option<Rc<NodeId>>);

impl Parent {
    pub fn root() -> Self {
        Parent(None)
    }

    fn id(&self) -> Rc<NodeId> {
        self.0.clone().unwrap()
    }

    fn child(&self, index: usize, counter: &mut Counter) -> Parent {
        Self(Some(Rc::new(NodeId {
            parent: self.0.as_ref().map(|id| id.clone()),
            id: counter.next(),
            local_index: index as u16,
        })))
    }
}

pub enum Push {
    Lit(syn::LitStr),
    Bind(syn::Expr),
    Empty,
}

pub struct Branch {
    pub keywords: TokenStream,
    pub cond: Option<Box<syn::Expr>>,
    pub then: Vec<Block>,
}

pub enum Op {
    Push(Vec<Push>),
    // "Flattened" branch - the length of the vec is the number of possibilities:
    Branch(Vec<Branch>),
}

pub struct Block {
    pub op: Op,
    pub id: Rc<NodeId>,
}

pub fn create_blocks(
    constituents: Vec<parse::Constituent>,
    parent: Parent,
    counter: &mut Counter,
) -> Vec<Block> {
    let mut peek_ast = constituents.into_iter().peekable();
    let mut pushes = vec![];
    let mut blocks = vec![];

    loop {
        match peek_ast.peek() {
            None => {
                if !pushes.is_empty() {
                    blocks.push(Block {
                        op: Op::Push(pushes),
                        id: parent.child(blocks.len(), counter).id(),
                    });
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
                    parse::Constituent::Bind(expr) => Push::Bind(expr),
                    _ => panic!(),
                });
            }
            Some(parse::Constituent::Block(_)) => {
                pushes.push(match peek_ast.next().unwrap() {
                    parse::Constituent::Block(_) => Push::Empty,
                    _ => panic!(),
                });
            }
            Some(parse::Constituent::If(_)) => {
                if !pushes.is_empty() {
                    blocks.push(Block {
                        op: Op::Push(pushes),
                        id: parent.child(blocks.len(), counter).id(),
                    });
                    pushes = vec![];
                }
                let branch_parent = parent.child(blocks.len(), counter);

                let iff = match peek_ast.next().unwrap() {
                    parse::Constituent::If(iff) => iff,
                    _ => panic!(),
                };
                let if_token = iff.if_token;

                let mut branches = vec![];
                branches.push(Branch {
                    keywords: quote! { #if_token },
                    cond: Some(iff.cond),
                    then: create_blocks(
                        iff.then_branch.constituents,
                        branch_parent.child(branches.len(), counter),
                        counter,
                    ),
                });

                let mut next = iff.else_branch;

                while let Some(else_branch) = next {
                    match else_branch {
                        parse::Else::If(else_token, iff) => {
                            let if_token = iff.if_token;
                            branches.push(Branch {
                                keywords: quote! { #else_token #if_token },
                                cond: Some(iff.cond),
                                then: create_blocks(
                                    iff.then_branch.constituents,
                                    branch_parent.child(branches.len(), counter),
                                    counter,
                                ),
                            });
                            next = iff.else_branch;
                        }
                        parse::Else::Block(else_token, block) => {
                            branches.push(Branch {
                                keywords: quote! { #else_token },
                                cond: None,
                                then: create_blocks(
                                    block.constituents,
                                    branch_parent.child(branches.len(), counter),
                                    counter,
                                ),
                            });
                            break;
                        }
                    }
                }

                blocks.push(Block {
                    op: Op::Branch(branches),
                    id: branch_parent.id(),
                });
            }
            Some(parse::Constituent::Match(_)) => {
                panic!("match is not supported yet");
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
        let blocks = create_blocks(ast.constituents, Parent::root(), &mut Counter::new());
        assert_eq!(blocks.len(), 3);
    }
}
