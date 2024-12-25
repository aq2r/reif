use std::str::from_utf8;

use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::quote;
use regex_syntax::hir::{Capture, Class, Hir, HirKind, Literal, Repetition};

use super::ElseAction;

pub fn literal_tokens(literal: &Literal, else_action: ElseAction) -> anyhow::Result<TokenStream> {
    let literal_val = from_utf8(&literal.0)?;
    let else_token = else_action.to_tokens();

    let if_tokens = quote! {
        if rest.starts_with( #literal_val ) {
            rest = &rest[ #literal_val .len()..];
        }
    };

    Ok(quote! {
        {
            #if_tokens

            else {
                #else_token
            }
        }
    })
}

pub fn class_tokens(class: &Class, else_action: ElseAction) -> TokenStream {
    let mut start_chars = vec![];
    let mut end_chars = vec![];

    match class {
        Class::Unicode(class_unicode) => {
            for i in class_unicode.ranges() {
                start_chars.push(i.start());
                end_chars.push(i.end());
            }
        }
        Class::Bytes(class_bytes) => {
            for i in class_bytes.ranges() {
                start_chars.push(i.start() as char);
                end_chars.push(i.end() as char);
            }
        }
    }

    let match_arms: Vec<_> = start_chars
        .iter()
        .zip(end_chars.iter())
        .map(|(s, e)| {
            quote! {
                #s ..= #e => true,
            }
        })
        .collect();

    let else_token_false = match else_action {
        ElseAction::Return => quote! { return false; },
        ElseAction::Break => quote! { break; },
    };

    quote! {
        {
            let Some(c) = rest.chars().next() else {
                #else_token_false
            };

            let result = match c {
                #(#match_arms)*
                _ => false,
            };

            if result {
                rest = &rest[c.len_utf8()..];
            } else {
                #else_token_false
            }
        }
    }
}

pub fn repetition_tokens(repetition: &Repetition) -> anyhow::Result<TokenStream> {
    let repetition_min = repetition.min;
    let repetition_max = match repetition.max {
        Some(v) => v,
        None => u32::MAX,
    };

    let inner_tokens = match repetition.sub.kind() {
        regex_syntax::hir::HirKind::Empty => todo!(),
        regex_syntax::hir::HirKind::Literal(literal) => {
            vec![literal_tokens(literal, ElseAction::Break)?]
        }
        regex_syntax::hir::HirKind::Class(class) => {
            vec![class_tokens(class, ElseAction::Break)]
        }
        regex_syntax::hir::HirKind::Look(_look) => return Err(anyhow!("Look is unsupported")),
        regex_syntax::hir::HirKind::Repetition(repetition) => {
            vec![repetition_tokens(repetition)?]
        }
        regex_syntax::hir::HirKind::Capture(capture) => {
            vec![capture_tokens(capture, ElseAction::Break)?]
        }
        regex_syntax::hir::HirKind::Concat(vec) => concat_tokens(vec, ElseAction::Break)?,
        regex_syntax::hir::HirKind::Alternation(vec) => {
            vec![alternation_tokens(vec, ElseAction::Break)?]
        }
    };

    Ok(quote! {
        {
            let mut cycle_count = 0;

            loop {
                if cycle_count >= #repetition_max {
                    break;
                }

                // 内部のtoken
                #(#inner_tokens)*


                cycle_count += 1;
            }

            if !( #repetition_min <= cycle_count && cycle_count <= #repetition_max ) {
                return false;
            }
        }
    })
}

pub fn concat_tokens(vec: &Vec<Hir>, else_action: ElseAction) -> anyhow::Result<Vec<TokenStream>> {
    let mut tokens_vec = vec![];

    for i in vec {
        match i.kind() {
            HirKind::Empty => todo!(),
            HirKind::Literal(literal) => tokens_vec.push(literal_tokens(literal, else_action)?),
            HirKind::Class(class) => tokens_vec.push(class_tokens(class, else_action)),
            HirKind::Look(_look) => return Err(anyhow!("Look is unsupported")),
            HirKind::Repetition(repetition) => tokens_vec.push(repetition_tokens(repetition)?),
            HirKind::Capture(capture) => tokens_vec.push(capture_tokens(capture, else_action)?),
            HirKind::Concat(vec) => tokens_vec.append(&mut concat_tokens(vec, else_action)?),
            HirKind::Alternation(vec) => tokens_vec.push(alternation_tokens(vec, else_action)?),
        };
    }

    Ok(tokens_vec)
}

pub fn alternation_tokens(vec: &Vec<Hir>, else_action: ElseAction) -> anyhow::Result<TokenStream> {
    let mut tokens_vec_vec = vec![];

    for i in vec {
        match i.kind() {
            HirKind::Empty => todo!(),
            HirKind::Literal(literal) => {
                tokens_vec_vec.push(vec![literal_tokens(literal, else_action)?])
            }
            HirKind::Class(class) => tokens_vec_vec.push(vec![class_tokens(class, else_action)]),
            HirKind::Look(_look) => return Err(anyhow!("Look is unsupported")),
            HirKind::Repetition(repetition) => {
                tokens_vec_vec.push(vec![repetition_tokens(repetition)?])
            }
            HirKind::Capture(capture) => {
                tokens_vec_vec.push(vec![capture_tokens(capture, else_action)?])
            }
            HirKind::Concat(vec) => tokens_vec_vec.push(concat_tokens(vec, else_action)?),
            HirKind::Alternation(vec) => {
                tokens_vec_vec.push(vec![alternation_tokens(vec, else_action)?])
            }
        };
    }

    let else_token = match else_action {
        ElseAction::Return => quote! { return false; },
        ElseAction::Break => quote! { break; },
    };

    let tokens_vec: Vec<TokenStream> = tokens_vec_vec
        .into_iter()
        .map(|tokens_vec| {
            quote! {
                (|| {#(#tokens_vec)* true} )()
            }
        })
        .collect();

    Ok(quote! {
        {
            if {  #( #tokens_vec ) ||*}  {
                ()
            }

            else {
                #else_token
            }
        }
    })
}

pub fn capture_tokens(capture: &Capture, else_action: ElseAction) -> anyhow::Result<TokenStream> {
    let tokens_vec = match capture.sub.kind() {
        HirKind::Empty => todo!(),
        HirKind::Literal(literal) => {
            vec![literal_tokens(literal, ElseAction::Return)?]
        }
        HirKind::Class(class) => vec![class_tokens(class, ElseAction::Return)],
        HirKind::Look(_look) => return Err(anyhow!("Look is unsupported")),
        HirKind::Repetition(repetition) => vec![repetition_tokens(repetition)?],
        HirKind::Capture(capture) => vec![capture_tokens(capture, ElseAction::Return)?],
        HirKind::Concat(vec) => concat_tokens(vec, ElseAction::Return)?,
        HirKind::Alternation(vec) => vec![alternation_tokens(vec, ElseAction::Return)?],
    };

    let if_tokens = quote! {
        if {
            || -> bool {
                #(#tokens_vec)*

                true
            }()
        }
    };

    let else_token = match else_action {
        ElseAction::Return => quote! { return false; },
        ElseAction::Break => quote! { break; },
    };

    Ok(quote! {
        #if_tokens {

        } else {
            #else_token
        }
    })
}
