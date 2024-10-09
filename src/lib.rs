#![allow(warnings)]

use std::collections::VecDeque;

use itertools::Itertools as _;
use quote::quote;
use quote::ToTokens;
use quote::TokenStreamExt;
use syn::{parse_macro_input, Expr, ExprMatch, Pat, PatSlice, PatIdent, Ident};
use proc_macro2;
use proc_macro2::TokenStream;
use syn::spanned::Spanned;

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

#[derive(Debug)]
enum SplatTerm {
    Value(TokenStream),
    Values(Vec<TokenStream>),
}

#[derive(Debug)]
enum Pattern {
    Value(TokenStream),
    Values(Vec<TokenStream>),
    Splat((Option<usize>, Option<usize>), Option<SplatTerm>),
}

fn parse_number(expr: &Expr) -> Result<usize, syn::Error> {
    expr
        .to_token_stream()
        .to_string()
        .parse()
        .map_err(|err| {
            syn::Error::new(expr.span(), format!("expected number but got: {:?}", err))
        })
}

fn generate_patterns<'a>(mut pats: impl Iterator<Item = &'a Pat>) -> Result<Vec<Pattern>, syn::Error> {
    let Some(elem) = pats.next() else {
        return Ok(vec![])
    };

    let mut patterns = generate_patterns(pats)?;
    match elem {
        Pat::Lit(pat) => patterns.push(Pattern::Value(pat.clone().to_token_stream())),
        Pat::Range(pat) => {
            let term = match patterns.pop() {
                Some(Pattern::Value(tokens))  => Some(SplatTerm::Value(tokens)),
                Some(Pattern::Values(tokens)) => Some(SplatTerm::Values(tokens)),
                Some(pattern) => {
                    patterns.push(pattern);
                    None
                },
                None => None,
            };

            match (&pat.start, &pat.end) {
                (None, None)           => patterns.push(Pattern::Splat((None, None), term)),
                (None, Some(max))      => patterns.push(Pattern::Splat((None, Some(parse_number(max)?)), term)),
                (Some(min), None)      => patterns.push(Pattern::Splat((Some(parse_number(min)?), None), term)),
                (Some(min), Some(max)) => patterns.push(Pattern::Splat((Some(parse_number(min)?), Some(parse_number(max)?)), term)),
            }
        }
        Pat::Rest(_) => patterns.push(Pattern::Splat((None, None), None)),
        _ => (),
    }

    Ok(patterns)
}

fn generate_match_type(pattern: Pattern, n: usize) -> TokenStream {
    match pattern {
        Pattern::Value(tokens) => quote! { MatchType::Value(#tokens) },
        Pattern::Values(tokens) => {
            let mut tokens = tokens.clone();
            let tokens     = (0..n)
                .into_iter()
                .map(|_| quote! {tokens.pop() })
                .collect_vec();

            quote! { MatchType::Values([#(#tokens),*]) }
        },
        Pattern::Splat((min, max), tokens) => quote! {  },
    }
}

#[proc_macro]
pub fn generate_match(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_expr = parse_macro_input!(input as Expr);

    let Expr::Match(ExprMatch { expr, arms, .. }) = input_expr else {
        return syn::Error::new_spanned(input_expr, "Expected a match statement").to_compile_error().into();
    };

    let typ = if let Expr::Path(expr_path) = &*expr {
        let var_type = expr_path.path.get_ident();
        if var_type.unwrap() == "Vec" {
            let syn::PathArguments::AngleBracketed(args) = &expr_path.path.segments.last().unwrap().arguments else {
                unreachable!()
            };

            let Some(syn::GenericArgument::Type(syn::Type::Path(syn::TypePath { path, .. }))) = args.args.first() else {
                unreachable!()
            };

            quote! { #path }
        } else { unreachable!() }
    } else { unreachable!() };

    let (slice_pats, other_pats): (Vec<_>, Vec<_>) = arms
        .iter()
        .partition_map(|arm| {
            match &arm.pat {
                Pat::Slice(pat) => itertools::Either::Left(pat),
                pat             => itertools::Either::Right(pat),
            }
        });

    // #[derive(Debug)]
    // enum MatchState {
    //     Unmatched,
    //     Matched,
    // }
    //
    // #[derive(Debug)]
    // enum MatchType<const N: usize, T> {
    //     Value(T),
    //     Values([T; N]),
    //     Splat((usize, Option<usize>), Option<T>),
    // }
    //
    // #[derive(Debug)]
    // struct Find<const N: usize, const N2: usize, T> {
    //     matches: [Option<MatchType<N2, T>>; N],
    //     checked: bool,
    // }
    //
    // impl<const N: usize, const N2: usize, T> Find<N, N2, T> {
    //     fn new(matches: [Option<MatchType<N2, T>>; N]) -> Self {
    //         Find { matches, checked: false }
    //     }
    //
    //     fn front(&mut self) -> Option<&mut MatchType<N2, T>> {
    //         let mut found = None;
    //         let len = self.matches.len();
    //         let mut i = 0;
    //         while i < len {
    //             if self.matches[i].is_some() {
    //                 found = Some(i);
    //                 break;
    //             }
    //
    //             i += 1;
    //         }
    //
    //         if let Some(i) = found {
    //             self.matches[i].as_mut()
    //         } else {
    //             None
    //         }
    //     }
    //
    //     fn remove_front(&mut self) {
    //         let mut i = 0;
    //         while i < self.matches.len() {
    //             if self.matches[i].is_some() {
    //                 self.matches[i] = None;
    //                 return;
    //             }
    //
    //             i += 1;
    //         }
    //     }
    //
    //     fn is_empty(&self) -> bool {
    //         self.matches
    //             .iter()
    //             .filter(|&item| item.is_some())
    //             .count() == 0
    //     }
    // }

    // let mut f0 = Find::new([Some(MatchType::Splat((2, None), Some(2))), Some(MatchType::Value(3)), None]);                       // true  | true
    // let mut f1 = Find::new([Some(MatchType::Value(0)), Some(MatchType::Value(1)), Some(MatchType::Value(9))]);                   // false | false
    // let mut f2 = Find::new([Some(MatchType::Value(0)), Some(MatchType::Splat((0, None), Some(9))), Some(MatchType::Value(10))]); // false | false
    // let mut f3 = Find::new([Some(MatchType::Splat((0, None), Some(2))), Some(MatchType::Value(3)), None]);                       // true  | false
    // let mut f4 = Find::new([Some(MatchType::Splat((0, None), Some(10))), Some(MatchType::Value(11)), None]);                     // false | false
    // let mut f5 = Find::new([Some(MatchType::Splat((0, None), None)), None, None]);                                               // true  | true
    // let mut f6 = Find::new([None, None, None]);                                                                                  // true  | true
    // let mut fs: [Find<3, 2>; 7] = [f0, f1, f2, f3, f4, f5, f6];

    let x_patterns = match slice_pats
        .iter()
        .map(|&pat| generate_patterns(pat.elems.iter()))
        .process_results(|patterns_iter| patterns_iter.collect_vec()) {
            Ok(patterns) => patterns,
            Err(err) => return err.to_compile_error().into(),
        };

    let Some(matchtype_arr_len) = x_patterns.iter().map(|patterns| patterns.len()).max() else {
        return syn::Error::new(proc_macro2::Span::call_site(), "Expected at least one pattern").to_compile_error().into();
    };

    let values_arr_len = x_patterns.iter().map(|patterns| patterns.iter().map(|pattern| if let Pattern::Values(values) = pattern { values.len() } else { 0 }).max().unwrap_or(0)).max().unwrap_or(0);

    assert!(matchtype_arr_len > 0);

    let matches = x_patterns
        .into_iter()
        .map(|patterns| patterns.into_iter().map(|pattern| generate_match_type(pattern, values_arr_len)))
        .map(|mut pattern| {
            (0..matchtype_arr_len)
                .into_iter()
                .map(|_| pattern.next().map_or(quote! { None }, |pattern| quote! { Some(#pattern) }))
                .collect_vec()
        })
        .map(|matches| {
            // let var_ident = Ident::new(&format!("v{}", i), proc_macro2::Span::call_site());
            // quote! { let mut #var_ident = Find::<#matchtype_arr_len, #values_arr_len, _>::new([#(#matches),*]); }
            quote! { Find::<#matchtype_arr_len, #values_arr_len, _>::new([#(#matches),*]) }
        })
        .collect_vec();

    quote! {
        enum MatchState {
            Unmatched,
            Matched,
        }

        enum MatchType<const N: usize, T> {
            Value(T),
            Values([T; N]),
            Splat((usize, Option<usize>), Option<T>),
        }

        struct Find<const N: usize, const N2: usize, T> {
            matches: [Option<MatchType<N2, T>>; N],
            checked: bool,
        }

        impl<const N: usize, const N2: usize, T> Find<N, N2, T> {
            fn new(matches: [Option<MatchType<N2, T>>; N]) -> Self {
                Find { matches, checked: false }
            }

            fn front(&mut self) -> Option<&mut MatchType<N2, T>> {
                let mut found = None;
                let len = self.matches.len();
                let mut i = 0;
                while i < len {
                    if self.matches[i].is_some() {
                        found = Some(i);
                        break;
                    }

                    i += 1;
                }

                if let Some(i) = found {
                    self.matches[i].as_mut()
                } else {
                    None
                }
            }

            fn remove_front(&mut self) {
                let mut i = 0;
                while i < self.matches.len() {
                    if self.matches[i].is_some() {
                        self.matches[i] = None;
                        return;
                    }

                    i += 1;
                }
            }

            fn is_empty(&self) -> bool {
                self.matches
                    .iter()
                    .filter(|&item| item.is_some())
                    .count() == 0
            }
        }

        let vars = [#(#matches),*];
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
