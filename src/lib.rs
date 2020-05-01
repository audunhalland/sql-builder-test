use proc_macro2::{Ident, Span};

use quote::{format_ident, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Group;
use syn::{Expr, ExprLit, Lit};
use syn::{ExprGroup, Token};

#[allow(dead_code)]
mod builder;

struct Input {
    source: String,
    source_span: Span,
    // `arg0 .. argN` for N arguments
    arg_names: Vec<Ident>,
    arg_exprs: Vec<Expr>,
}

impl Input {
    fn from_exprs(input: ParseStream, mut args: impl Iterator<Item = Expr>) -> syn::Result<Self> {
        fn lit_err<T>(span: Span, unexpected: Expr) -> syn::Result<T> {
            Err(syn::Error::new(
                span,
                format!(
                    "expected string literal, got {}",
                    unexpected.to_token_stream()
                ),
            ))
        }

        let (source, source_span) = match args.next() {
            Some(Expr::Lit(ExprLit {
                lit: Lit::Str(sql), ..
            })) => (sql.value(), sql.span()),
            Some(Expr::Group(ExprGroup {
                expr,
                group_token: Group { span },
                ..
            })) => {
                // this duplication with the above is necessary because `expr` is `Box<Expr>` here
                // which we can't directly pattern-match without `box_patterns`
                match *expr {
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(sql), ..
                    }) => (sql.value(), span),
                    other_expr => return lit_err(span, other_expr),
                }
            }
            Some(other_expr) => return lit_err(other_expr.span(), other_expr),
            None => return Err(input.error("expected SQL string literal")),
        };

        let arg_exprs: Vec<_> = args.collect();
        let arg_names = (0..arg_exprs.len())
            .map(|i| format_ident!("arg{}", i))
            .collect();

        Ok(Self {
            source,
            source_span,
            arg_exprs,
            arg_names,
        })
    }
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args = Punctuated::<Expr, Token![,]>::parse_terminated(input)?.into_iter();

        Self::from_exprs(input, args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use quote::quote;

    #[test]
    fn test() {
        let input: Input = syn::parse2(quote! {
            "lol",
            "lolz"
        })
        .unwrap();
        assert_eq!(input.source, "lol");

        let ident_strings: Vec<String> = input.arg_names.iter().map(ToString::to_string).collect();
        assert_eq!(ident_strings, vec!["arg0".to_owned()]);
    }
}
