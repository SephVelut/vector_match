#![allow(warnings)]

use itertools::Itertools as _;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr, ExprMatch, Pat, PatSlice, PatIdent, Ident};

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

// enum MatchType {
//     Lit(u32),
//     Range(Option<u32>),
// }
//
// struct Find {
//     index:   usize,
//     matches: Vec<MatchType>,
// }
//
// impl Find {
//     fn new(matches: Vec<MatchType>) -> Self {
//         Find { index: 0, matches: matches.into_iter().rev().collect_vec() }
//     }
// }
//
// let mut v  = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
// let mut m1 = Find::new(vec![MatchType::Lit(0), MatchType::Lit(1), MatchType::Lit(9)]);          // false
// let mut m2 = Find::new(vec![MatchType::Lit(0), MatchType::Range(Some(9)), MatchType::Lit(10)]); // false
// let mut m4 = Find::new(vec![MatchType::Range(Some(2)), MatchType::Lit(3)]);                     // true
// let mut m3 = Find::new(vec![MatchType::Range(Some(10)), MatchType::Lit(11)]);                   // false
// let mut m5 = Find::new(vec![]);                                                                 // true
// let mut ms = vec![m1, m2, m3, m4, m5];
// for item in v.iter() {
// }

#[proc_macro]
pub fn generate_match(input: TokenStream) -> TokenStream {
    let input_expr = parse_macro_input!(input as Expr);

    let Expr::Match(ExprMatch { expr, arms, .. }) = input_expr else {
        return syn::Error::new_spanned(input_expr, "Expected a match statement").to_compile_error().into()
    };

    let arms = arms
        .into_iter()
        .map(|arm| {
            if let Pat::Slice(PatSlice { elems, .. }) = &arm.pat {
                elems
                    .iter()
                    .tuple_windows()
                    .enumerate()
                    .map(|(i, (elem1, elem2))| {
                        match (elem1, elem2) {
                            (Pat::Range(range), Pat::Lit(lit)) => todo!(),
                            _ => todo!(),
                        }
                    });

                let body = &arm.body;
                quote! { data if true => #body }
            } else {
                quote! { #arm }
            }
        })
        .collect_vec();

    quote! {
        match #expr {
            #(#arms)*
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
