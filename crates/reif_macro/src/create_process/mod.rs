mod tokens;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{ParseStream, Parser},
    Error, LitStr, Result,
};

use regex_syntax::{
    hir::{HirKind, Look},
    Parser as ReParser,
};

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

    let mut is_start_only = false;
    let mut is_end_only = false;

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
        HirKind::Concat(vec) => {
            let mut vec = vec.clone();
            // change is_start_only val
            match vec[0].kind() {
                HirKind::Look(look) => match look {
                    Look::Start => {
                        is_start_only = true;
                        vec.remove(0);
                    }
                    Look::End => {
                        return Err(Error::new(
                            lit_str.span(),
                            "'End' cannot be placed at this position.",
                        ))
                    }
                    _ => return Err(Error::new(lit_str.span(), "This 'Look' is unsupported")),
                },
                _ => (),
            }

            match vec.last().map(|i| i.kind()) {
                Some(HirKind::Look(look)) => match look {
                    Look::Start => {
                        return Err(Error::new(
                            lit_str.span(),
                            "'Start' cannot be placed at this position.",
                        ))
                    }
                    Look::End => {
                        is_end_only = true;
                        vec.pop();
                    }
                    _ => return Err(Error::new(lit_str.span(), "This 'Look' is unsupported")),
                },
                _ => (),
            }

            tokens::concat_tokens(&vec, ElseAction::Return)
                .map_err(|err| Error::new(lit_str.span(), err))?
        }
        HirKind::Alternation(vec) => vec![tokens::alternation_tokens(vec, ElseAction::Return)
            .map_err(|err| Error::new(lit_str.span(), err))?],
    };

    let mut for_max_tokens = match is_start_only {
        true => quote! { (0..1 ) },
        false => quote! { (0..input.len() - 1) },
    };

    if !is_start_only && is_end_only {
        for_max_tokens = quote! { #for_max_tokens.rev() }
    }

    let end_only_tokens = match is_end_only {
        true => {
            quote! {
                let matched_part = &input[0..(input.len() - rest.len())];
                if !input.ends_with(matched_part) {
                    return false;
                }
            }
        }
        false => quote! {},
    };

    Ok(quote! {
        |input: &str| -> bool {
            // start
            for i in #for_max_tokens {
                let mut rest = &input[i..];

                let result = (|| -> bool {
                    #(#tokens)*

                    #end_only_tokens

                    true
                })();

                if result {
                    return true;
                }
            }

            // end
            false
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
        let hir = Parser::new().parse(r"^abc");
        dbg!(&hir);

        let tokens = _create_process(quote! {r"^abc"});
        dbg!(tokens);
    }
}
