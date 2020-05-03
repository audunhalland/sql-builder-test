use proc_macro2::{Ident, Span};

use quote::{format_ident, ToTokens};
use syn::braced;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, ExprLit, Lit};
use syn::{ExprGroup, ExprIf, LitStr, Token};

pub struct SqlBlock {
    pub brace_token: syn::token::Brace,
    pub constituents: Vec<Constituent>,
}

impl Parse for SqlBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(SqlBlock {
            brace_token: braced!(content in input),
            constituents: parse_constituents(&content)?,
        })
    }
}

pub struct If {
    pub if_token: Token![if],
    pub cond: Box<Expr>,
    pub then_branch: SqlBlock,
    pub else_branch: Option<Else>,
}

impl Parse for If {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(If {
            if_token: input.parse()?,
            cond: Box::new(input.parse()?),
            then_branch: input.parse()?,
            else_branch: {
                if input.peek(Token![else]) {
                    Some(input.parse()?)
                } else {
                    None
                }
            },
        })
    }
}

pub enum Else {
    If(Token![else], Box<If>),
    Block(Token![else], SqlBlock),
}

impl Parse for Else {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let else_token: Token![else] = input.parse()?;
        let lookahead = input.lookahead1();

        if input.peek(Token![if]) {
            Ok(Else::If(else_token, Box::new(input.parse()?)))
        } else if input.peek(syn::token::Brace) {
            Ok(Else::Block(else_token, input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

pub enum Constituent {
    Literal(LitStr),
    Bind(Expr),
    Block(SqlBlock),
    If(If),
}

pub struct BuilderAST {
    pub constituents: Vec<Constituent>,
}

fn parse_next_constituent(input: ParseStream) -> syn::Result<Constituent> {
    if input.peek(LitStr) {
        return Ok(Constituent::Literal(input.parse()?));
    }

    if input.peek(syn::token::Brace) {
        return Ok(Constituent::Block(input.parse()?));
    }

    if input.peek(Token!(if)) {
        return Ok(Constituent::If(input.parse()?));
    }

    let expr = input.parse::<Expr>()?;

    Ok(Constituent::Bind(expr))
}

fn parse_constituents(input: ParseStream) -> syn::Result<Vec<Constituent>> {
    let mut constituents = vec![];
    loop {
        if input.is_empty() {
            return Ok(constituents);
        }

        constituents.push(parse_next_constituent(&input)?);
    }
}

impl Parse for BuilderAST {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let constituents = parse_constituents(&input)?;
        Ok(BuilderAST { constituents })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use quote::quote;

    #[test]
    fn parse_sql_block() {
        let ast: SqlBlock = syn::parse2(quote! {
            { "SELECT" }
        })
        .unwrap();
        assert_eq!(ast.constituents.len(), 1);
    }

    #[test]
    fn parse_if() {
        syn::parse2::<If>(quote! {
            if 2 { "SELECT" }
        })
        .unwrap();
    }

    #[test]
    fn parse_else() {
        syn::parse2::<Else>(quote! {
            else if 2 { "A" "B" }
        })
        .unwrap();
    }

    #[test]
    fn parse_if_else() {
        syn::parse2::<If>(quote! {
            if 2 { "A" } else { "B" }
        })
        .unwrap();
    }

    #[test]
    fn parse_if_else_if() {
        syn::parse2::<If>(quote! {
            if 2 { "A" "B" } else if 3 { "C" "D" }
        })
        .unwrap();
    }

    #[test]
    fn parse_ast_with_block() {
        let ast: BuilderAST = syn::parse2(quote! {
            "SELECT col FROM table "
            "WHERE " {
                "LOLG"
            }
        })
        .unwrap();
    }

    #[test]
    fn parse_ast_with_try_block() {
        let a = Some(42_i32);
        let ast: BuilderAST = syn::parse2(quote! {
            "SELECT * " { "WHERE " #a? }
        })
        .unwrap();
        assert_eq!(ast.constituents.len(), 2);
    }

    #[test]
    fn parse_ast_bind() {
        let two = 2i32;
        let ast: BuilderAST = syn::parse2(quote! {
            "SELECT col FROM table WHERE a = " #two + #two " AND b IS NOT NULL"
        })
        .unwrap();

        assert_eq!(ast.constituents.len(), 3);
    }

    #[test]
    fn parse_ast_if() {
        let test = true;
        let ast: BuilderAST = syn::parse2(quote! {
            "SELECT yo FROM saft WHERE"
            if #test { "bar = " "yo" } else { "TRUE" }
            "YO"
        })
        .unwrap();

        assert_eq!(ast.constituents.len(), 3);
    }
}
