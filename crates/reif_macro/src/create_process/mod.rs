mod tokens;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{ParseStream, Parser},
    Error, LitStr, Result,
};

use regex_syntax::{hir::HirKind, Parser as ReParser};

#[derive(Debug, Clone, Copy)]
enum ElseAction {
    Return,
    Break,
}

impl ElseAction {
    pub fn to_tokens(&self) -> TokenStream {
        match self {
            ElseAction::Return => quote! { return false; },
            ElseAction::Break => quote! { break; },
        }
    }
}

pub fn _create_process(tokens: TokenStream) -> TokenStream {
    rfmatch_parse
        .parse2(tokens)
        .unwrap_or_else(Error::into_compile_error)
}

fn rfmatch_parse(input: ParseStream) -> Result<TokenStream> {
    let lit_str: LitStr = input.parse()?;

    let ast = match ReParser::new().parse(&lit_str.value()) {
        Ok(v) => v,
        Err(err) => return Err(Error::new(lit_str.span(), err)),
    };

    let tokens: Vec<TokenStream> = match ast.kind() {
        HirKind::Empty => vec![quote! {}],
        HirKind::Literal(literal) => {
            vec![tokens::literal_tokens(literal, ElseAction::Return)
                .map_err(|err| Error::new(lit_str.span(), err))?]
        }
        HirKind::Class(class) => vec![tokens::class_tokens(class, ElseAction::Return)],
        HirKind::Look(_look) => {
            return Err(Error::new(lit_str.span(), "'Look' alone cannot be used."))
        }
        HirKind::Repetition(repetition) => {
            vec![tokens::repetition_tokens(repetition)
                .map_err(|err| Error::new(lit_str.span(), err))?]
        }
        HirKind::Capture(capture) => {
            vec![tokens::capture_tokens(capture, ElseAction::Return)
                .map_err(|err| Error::new(lit_str.span(), err))?]
        }
        HirKind::Concat(vec) => tokens::concat_tokens(vec, ElseAction::Return)
            .map_err(|err| Error::new(lit_str.span(), err))?,
        HirKind::Alternation(vec) => vec![tokens::alternation_tokens(vec, ElseAction::Return)
            .map_err(|err| Error::new(lit_str.span(), err))?],
    };

    Ok(quote! {
        |input: &str| -> bool {
            // start
            let mut rest = input;

            #(#tokens)*

            // end
            let _ = rest;
            if rest.is_empty() {
                true
            } else {
                false
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use regex_syntax::Parser;

    use super::_create_process;

    #[test]
    fn debug() {
        let tokens =
            _create_process(quote! {r"$abc^"});
        dbg!(tokens);
    }
}
