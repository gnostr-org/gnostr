use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, RangeLimits, Token,
};

struct RandomBuilder {
    min:       Option<Box<Expr>>,
    max:       Option<Box<Expr>>,
    rng:       Option<Box<Expr>>,
    exclusive: bool,
    cmp:       bool,
}

impl Parse for RandomBuilder {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        if input.is_empty() {
            Ok(RandomBuilder {
                min:       None,
                max:       None,
                rng:       None,
                exclusive: false,
                cmp:       false,
            })
        } else {
            let expr: Expr = input.parse()?;

            if let Expr::Range(range) = expr {
                let exclusive = match range.limits {
                    RangeLimits::HalfOpen(_) => true,
                    RangeLimits::Closed(_) => false,
                };

                let min = range.start;
                let max = range.end;

                if input.is_empty() {
                    Ok(RandomBuilder {
                        min,
                        max,
                        rng: None,
                        exclusive,
                        cmp: false,
                    })
                } else {
                    input.parse::<Token!(,)>()?;

                    let expr: Expr = input.parse()?;

                    Ok(RandomBuilder {
                        min,
                        max,
                        rng: Some(Box::new(expr)),
                        exclusive,
                        cmp: false,
                    })
                }
            } else if input.lookahead1().peek(Token!(,)) {
                input.parse::<Token!(,)>()?;

                let expr2: Expr = input.parse()?;

                if input.is_empty() {
                    Ok(RandomBuilder {
                        min:       Some(Box::from(expr)),
                        max:       Some(Box::from(expr2)),
                        rng:       None,
                        exclusive: false,
                        cmp:       true,
                    })
                } else {
                    input.parse::<Token!(,)>()?;

                    let expr3: Expr = input.parse()?;

                    Ok(RandomBuilder {
                        min:       Some(Box::from(expr)),
                        max:       Some(Box::from(expr2)),
                        rng:       Some(Box::new(expr3)),
                        exclusive: false,
                        cmp:       true,
                    })
                }
            } else {
                Ok(RandomBuilder {
                    min:       None,
                    max:       None,
                    rng:       Some(Box::new(expr)),
                    exclusive: false,
                    cmp:       false,
                })
            }
        }
    }
}

#[proc_macro_hack]
pub fn random(input: TokenStream) -> TokenStream {
    let rb = parse_macro_input!(input as RandomBuilder);

    let random = match rb.min.as_ref() {
        Some(min) => match rb.max.as_ref() {
            Some(max) => {
                if rb.exclusive {
                    match rb.rng.as_ref() {
                        Some(rng) => {
                            quote! {
                                $crate::random_exclusively_with_rng(#min, #max, &mut #rng)
                            }
                        },
                        None => {
                            quote! {
                                $crate::random_exclusively(#min, #max)
                            }
                        },
                    }
                } else if rb.cmp {
                    match rb.rng.as_ref() {
                        Some(rng) => {
                            quote! {
                                $crate::random_inclusively_cmp_with_rng(#min, #max, &mut #rng)
                            }
                        },
                        None => {
                            quote! {
                                $crate::random_inclusively_cmp(#min, #max)
                            }
                        },
                    }
                } else {
                    match rb.rng.as_ref() {
                        Some(rng) => {
                            quote! {
                                $crate::random_inclusively_with_rng(#min, #max, &mut #rng)
                            }
                        },
                        None => {
                            quote! {
                                $crate::random_inclusively(#min, #max)
                            }
                        },
                    }
                }
            },
            None => match rb.rng.as_ref() {
                Some(rng) => {
                    quote! {
                        $crate::random_at_least_with_rng(#min, &mut #rng)
                    }
                },
                None => {
                    quote! {
                        $crate::random_at_least(#min)
                    }
                },
            },
        },
        None => match rb.max.as_ref() {
            Some(max) => {
                if rb.exclusive {
                    match rb.rng.as_ref() {
                        Some(rng) => {
                            quote! {
                                $crate::random_at_most_exclusively_with_rng(#max, &mut #rng)
                            }
                        },
                        None => {
                            quote! {
                                $crate::random_at_most_exclusively(#max)
                            }
                        },
                    }
                } else {
                    match rb.rng.as_ref() {
                        Some(rng) => {
                            quote! {
                                $crate::random_at_most_with_rng(#max, &mut #rng)
                            }
                        },
                        None => {
                            quote! {
                                $crate::random_at_most(#max)
                            }
                        },
                    }
                }
            },
            None => match rb.rng.as_ref() {
                Some(rng) => {
                    quote! {
                        $crate::random_with_rng(&mut #rng)
                    }
                },
                None => {
                    quote! {
                        $crate::random()
                    }
                },
            },
        },
    };

    random.into()
}

// TODO -----------------------------

struct RandomFillBuilder {
    out: Box<Expr>,
    rb:  RandomBuilder,
}

impl Parse for RandomFillBuilder {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let expr: Expr = input.parse()?;

        let out = Box::from(expr);

        if input.lookahead1().peek(Token!(,)) {
            input.parse::<Token!(,)>()?;
        }

        let rb: RandomBuilder = input.parse()?;

        Ok(RandomFillBuilder {
            out,
            rb,
        })
    }
}

#[proc_macro_hack::proc_macro_hack]
pub fn random_fill(input: TokenStream) -> TokenStream {
    let rfb = parse_macro_input!(input as RandomFillBuilder);

    let out = rfb.out;

    let rb = rfb.rb;

    let random_fill = match rb.min.as_ref() {
        Some(min) => match rb.max.as_ref() {
            Some(max) => {
                if rb.exclusive {
                    match rb.rng.as_ref() {
                        Some(rng) => {
                            quote! {
                                $crate::random_fill_exclusively_with_rng(#out.as_mut(), #min, #max, &mut #rng)
                            }
                        },
                        None => {
                            quote! {
                                $crate::random_fill_exclusively(#out.as_mut(), #min, #max)
                            }
                        },
                    }
                } else if rb.cmp {
                    match rb.rng.as_ref() {
                        Some(rng) => {
                            quote! {
                                $crate::random_fill_inclusively_cmp_with_rng(#out.as_mut(), #min, #max, &mut #rng)
                            }
                        },
                        None => {
                            quote! {
                                $crate::random_fill_inclusively_cmp(#out.as_mut(), #min, #max)
                            }
                        },
                    }
                } else {
                    match rb.rng.as_ref() {
                        Some(rng) => {
                            quote! {
                                $crate::random_fill_inclusively_with_rng(#out.as_mut(), #min, #max, &mut #rng)
                            }
                        },
                        None => {
                            quote! {
                                $crate::random_fill_inclusively(#out.as_mut(), #min, #max)
                            }
                        },
                    }
                }
            },
            None => match rb.rng.as_ref() {
                Some(rng) => {
                    quote! {
                        $crate::random_fill_at_least_with_rng(#out.as_mut(), #min, &mut #rng)
                    }
                },
                None => {
                    quote! {
                        $crate::random_fill_at_least(#out.as_mut(), #min)
                    }
                },
            },
        },
        None => match rb.max.as_ref() {
            Some(max) => {
                if rb.exclusive {
                    match rb.rng.as_ref() {
                        Some(rng) => {
                            quote! {
                                $crate::random_fill_at_most_exclusively_with_rng(#out.as_mut(), #max, &mut #rng)
                            }
                        },
                        None => {
                            quote! {
                                $crate::random_fill_at_most_exclusively(#out.as_mut(), #max)
                            }
                        },
                    }
                } else {
                    match rb.rng.as_ref() {
                        Some(rng) => {
                            quote! {
                                $crate::random_fill_at_most_with_rng(#out.as_mut(), #max, &mut #rng)
                            }
                        },
                        None => {
                            quote! {
                                $crate::random_fill_at_most(#out.as_mut(), #max)
                            }
                        },
                    }
                }
            },
            None => match rb.rng.as_ref() {
                Some(rng) => {
                    quote! {
                        $crate::random_fill_with_rng(#out.as_mut(), &mut #rng)
                    }
                },
                None => {
                    quote! {
                        $crate::random_fill(#out.as_mut())
                    }
                },
            },
        },
    };

    random_fill.into()
}
