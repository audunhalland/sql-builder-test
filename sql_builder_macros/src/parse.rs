use syn::parse::{Parse, ParseStream};

pub struct SqlBlock {
    pub brace_token: syn::token::Brace,
    pub constituents: Vec<Constituent>,
}

impl Parse for SqlBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(SqlBlock {
            brace_token: syn::braced!(content in input),
            constituents: parse_constituents(&content)?,
        })
    }
}

pub struct If {
    pub if_token: syn::Token![if],
    pub cond: Box<syn::Expr>,
    pub then_branch: SqlBlock,
    pub else_branch: Option<Else>,
}

impl Parse for If {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(If {
            if_token: input.parse()?,
            cond: Box::new(input.parse()?), // BUG: https://github.com/dtolnay/syn/issues/789
            then_branch: input.parse()?,
            else_branch: {
                if input.peek(syn::Token![else]) {
                    Some(input.parse()?)
                } else {
                    None
                }
            },
        })
    }
}

pub enum Else {
    If(syn::Token![else], Box<If>),
    Block(syn::Token![else], SqlBlock),
}

impl Parse for Else {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let else_token: syn::Token![else] = input.parse()?;
        let lookahead = input.lookahead1();

        if input.peek(syn::Token![if]) {
            Ok(Else::If(else_token, Box::new(input.parse()?)))
        } else if input.peek(syn::token::Brace) {
            Ok(Else::Block(else_token, input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

pub struct Match {
    pub match_token: syn::Token![match],
    pub expr: Box<syn::Expr>,
    pub brace_token: syn::token::Brace,
    pub arms: Vec<MatchArm>,
}

impl Parse for Match {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let match_token: syn::Token![match] = input.parse()?;
        let expr = input.parse()?; // BUG: https://github.com/dtolnay/syn/issues/789

        let content;
        let brace_token = syn::braced!(content in input);

        let mut arms = Vec::new();
        while !content.is_empty() {
            arms.push(content.call(MatchArm::parse)?);
        }

        Ok(Match {
            match_token,
            expr: Box::new(expr),
            brace_token,
            arms,
        })
    }
}

pub struct MatchArm {
    pub pat: syn::Pat,
    pub guard: Option<(syn::Token![if], Box<syn::Expr>)>,
    pub fat_arrow_token: syn::Token![=>],
    pub body: SqlBlock,
}

impl Parse for MatchArm {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(MatchArm {
            pat: {
                let leading_vert: Option<syn::Token![|]> = input.parse()?;
                let pat: syn::Pat = input.parse()?;
                if leading_vert.is_some() || input.peek(syn::Token![|]) {
                    let mut cases = syn::punctuated::Punctuated::new();
                    cases.push_value(pat);
                    while input.peek(syn::Token![|]) {
                        let punct = input.parse()?;
                        cases.push_punct(punct);
                        let pat: syn::Pat = input.parse()?;
                        cases.push_value(pat);
                    }
                    syn::Pat::Or(syn::PatOr {
                        attrs: Vec::new(),
                        leading_vert,
                        cases,
                    })
                } else {
                    pat
                }
            },
            guard: {
                if input.peek(syn::Token![if]) {
                    let if_token: syn::Token![if] = input.parse()?;
                    let guard: syn::Expr = input.parse()?;
                    Some((if_token, Box::new(guard)))
                } else {
                    None
                }
            },
            fat_arrow_token: input.parse()?,
            body: input.parse()?,
        })
    }
}

pub enum Constituent {
    Literal(syn::LitStr),
    Bind(syn::Expr),
    Block(SqlBlock),
    If(If),
    Match(Match),
}

pub struct BuilderAST {
    pub constituents: Vec<Constituent>,
}

fn parse_next_constituent(input: ParseStream) -> syn::Result<Constituent> {
    if input.peek(syn::LitStr) {
        return Ok(Constituent::Literal(input.parse()?));
    }

    if input.peek(syn::token::Brace) {
        return Ok(Constituent::Block(input.parse()?));
    }

    if input.peek(syn::Token!(if)) {
        return Ok(Constituent::If(input.parse()?));
    }

    if input.peek(syn::Token!(match)) {
        return Ok(Constituent::Match(input.parse()?));
    }

    let expr = input.parse::<syn::Expr>()?;

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
    fn parse_match_arm() {
        syn::parse2::<MatchArm>(quote! {
            Some(2) => {}
        })
        .unwrap();
    }

    #[test]
    fn parse_match_empty() {
        syn::parse2::<Match>(quote! {
            match Some(42) {}
        })
        .unwrap();
    }

    #[test]
    fn parse_match_var_with_arms() {
        let var = quote::format_ident!("var");
        let match_ = syn::parse2::<Match>(quote! {
            match #var {
                Some(37) => { "A" "B" }
                Some(83) => { "A" "C" }
            }
        })
        .unwrap();

        assert_eq!(match_.arms.len(), 2);
    }

    #[test]
    fn parse_ast_with_block() {
        syn::parse2::<BuilderAST>(quote! {
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

    #[test]
    fn parse_ast_match() {
        let test = Some(42);
        let ast: BuilderAST = syn::parse2(quote! {
            "SELECT yo FROM saft WHERE"
            match #test {
                Some(42) => { "TRUE" }
            }
        })
        .unwrap();

        assert_eq!(ast.constituents.len(), 2);
    }
}
