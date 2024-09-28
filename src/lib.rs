#![allow(warnings)]

use itertools::Itertools as _;
use quote::quote;
use quote::ToTokens;
use quote::TokenStreamExt;
use syn::{parse_macro_input, Expr, ExprMatch, Pat, PatSlice, PatIdent, Ident};
use proc_macro2;
use proc_macro2::TokenStream;

enum Rule {
    Lit(Pat),
    Skip,
}

// let v = vec![1, 2, 3];
// match 1 {
//     1 if { let mut data = v.iter(); let a = data.find_position(|&x| x == &0); true } => println!("asd"),
//     _ => (),
// };
                
// let mut v = vec![1, 2, 3, 4, 5, 6];
// let a: Vec<(u32, u32)> = v.iter()
//  .map(|x| *x)
//  .tuple_windows()
//  .collect_vec();

// a     = Value(a)
// _     = Splat((0, Some(1)), None)
// ..    = Splat((0, None), None)
// 2..   = Splat((2, None), None)
// 2..3  = Splat((2, 3), None)
// .., a = Splat((0, None), Some(a))
// a | b = Values([a, b])

enum Pattern {
    Value(TokenStream),
    Values(Vec<TokenStream>),
    Splat((Option<usize>, Option<usize>), Option<TokenStream>),
}

fn generate_patterns(mut tokens: impl Iterator<Item = Pat>) -> Vec<Pattern> {
    let mut patterns = vec![];
    let Some(elem) = tokens.next() else {
        return vec![]
    };

    match elem {
        Pat::Lit(pat) => {
            patterns.push(Pattern::Value(pat.clone().to_token_stream()));
        },
        Pat::Range(pat) => {
            match (&pat.start, &pat.end) {
                (None, None) => {
                }
                (None, Some(e)) => todo!(),
                (Some(s), None) => todo!(),
                (Some(s), Some(e)) => todo!(),
            };
        }
        Pat::Const(_) => todo!(),
        Pat::Ident(_) => todo!(),
        Pat::Macro(_) => todo!(),
        Pat::Or(_) => todo!(),
        Pat::Paren(_) => todo!(),
        Pat::Path(_) => todo!(),
        Pat::Reference(_) => todo!(),
        Pat::Rest(_) => todo!(),
        Pat::Slice(_) => todo!(),
        Pat::Struct(_) => todo!(),
        Pat::Tuple(_) => todo!(),
        Pat::TupleStruct(_) => todo!(),
        Pat::Type(_) => todo!(),
        Pat::Verbatim(_) => todo!(),
        Pat::Wild(_) => todo!(),
        _ => todo!(),
    }

    patterns
}

#[proc_macro]
pub fn generate_match(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_expr = parse_macro_input!(input as Expr);

    let Expr::Match(ExprMatch { expr, arms, .. }) = input_expr else {
        return syn::Error::new_spanned(input_expr, "Expected a match statement").to_compile_error().into()
    };

    for arm in arms.iter() {
        match &arm.pat {
            Pat::Slice(PatSlice { elems, .. }) => {
                for (i, elem) in elems.iter().enumerate() {
                }
            }
            _ => todo!(),
        }
    }

    // let arms = arms
    //     .into_iter()
    //     .map(|arm| {
    //         if let Pat::Slice(PatSlice { elems, .. }) = &arm.pat {
    //             elems
    //                 .iter()
    //                 .tuple_windows()
    //                 .enumerate()
    //                 .map(|(i, (elem1, elem2))| {
    //                     match (elem1, elem2) {
    //                         (Pat::Range(range), Pat::Lit(lit)) => todo!(),
    //                         _ => todo!(),
    //                     }
    //                 });
    //
    //             let body = &arm.body;
    //             quote! { data if true => #body }
    //         } else {
    //             quote! { #arm }
    //         }
    //     })
    //     .collect_vec();

    quote! {
        match #expr {
            //#(#arms)*
        }
    }.into()

    // let transformed_arms = arms.into_iter().map(|mut arm| {
    //     if let Pat::Slice(PatSlice { elems, .. }) = &arm.pat {
    //         let mut conditions = vec![];
    //
    //         for (i, elem) in elems.iter().enumerate() {
    //             match elem {
    //                 Pat::Lit(pat_lit) => {
    //                     conditions.push(quote! {
    //                         data.iter().nth(#i) == Some(&#pat_lit)
    //                     });
    //                 }
    //                 Pat::Ident(PatIdent { ident, .. }) => {
    //                     let var_name = ident.to_string();
    //                     conditions.push(quote! {
    //                         data.len() > #i
    //                     });
    //                 }
    //                 Pat::Rest(_) => {
    //                     conditions.push(quote! {
    //                         data.iter().skip(#i).count() > 0
    //                     });
    //                 }
    //                 _ => {}
    //             }
    //         }
    //
    //         let guard = quote! { #(#conditions)&&* };
    //
    //         arm.pat = syn::parse_quote! { data };
    //         arm.guard = Some((syn::token::If::default(), syn::parse_quote! { #guard }));
    //         arm.body = syn::parse_quote! {{
    //             #arm.body
    //         }};
    //     }
    //     arm
    // });
    //
    // let expanded = quote! {
    //     match #expr {
    //         #(#transformed_arms)*
    //     }
    // };
    //
    // expanded.into()
}
