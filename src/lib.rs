use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::__private::Default;
use syn::__private::Span;
use syn::punctuated::Punctuated;
use syn::FnArg::Typed;
use syn::{
    parse_macro_input, token, Block, Expr, ExprCall, ExprPath, FnArg, Ident, ImplItem,
    ImplItemMethod, ItemImpl, ParenthesizedGenericArguments, Pat, PatIdent, PatType, Path,
    PathArguments, PathSegment, ReturnType, Signature, Stmt, TraitBound, TraitBoundModifier, Type,
    TypeImplTrait, TypeParamBound, Visibility,
};


/// Macro which does the following: adds a function (invoke_all) which forwards all but the last
/// argument to every function matching the signature in the impl block, and consumes their results
/// with the final parameter, a closure; adds an associated constant (METHOD_COUNT) of the number of
/// available functions; adds an associated constant array (METHOD_LIST) of the names of available
/// functions. Note that the order in which the functions appear in METHOD_LIST array is the same
/// order in which they appear in the impl block.
#[proc_macro_attribute]
pub fn invoke_all(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemImpl);

    // Get a vec of references to ImplItemMethods in the impl block
    let methods = input
        .items
        .iter()
        .filter_map(|item| match item {
            ImplItem::Method(method) => Some(method),
            _ => None,
        })
        .collect::<Vec<_>>();

    // Get the number of available functions in the impl block
    let count = methods.len();

    // Get a list of identifiers for available functions:
    let names = methods
        .iter()
        .map(|iim| iim.sig.ident.to_string())
        .collect::<Vec<_>>();

    // Add invoke_all function to impl block:
    let invoke_all = create_invoke_all(&methods, get_struct_identifier_as_path(&input));
    input.items.push(invoke_all);

    // Append the number of functions (excluding those added by macro) to the impl block:
    input
        .items
        .push(
            syn::parse(
            quote!(pub const METHOD_COUNT: usize = #count;)
                .into())
                .unwrap()
        );

    // Append an array containing all function identifiers into the tokenstream
    input
        .items
        .push(
            syn::parse(
                quote!(pub const METHOD_LIST: [&'static str; #count] = [#(#names),*];)
                    .into())
                .unwrap()
        );

    input.into_token_stream().into()
}

/// Creates a function that invokes all functions in the impl block (all methods in the
/// annotated block must share the same signature). Note that rather than returning, the
/// invoke_all function will share the same parameter signature as the impl block functions but
/// also has a "consumer" FnMut(Original Return Type) -> () parameter added.
fn create_invoke_all(names: &Vec<&ImplItemMethod>, struct_path: PathSegment) -> ImplItem {
    // Get output type:
    let output_type = names[0].sig.output.clone();

    // Set up the signature for the invoke_all function.
    let mut invoke_sig = Signature {
        // Set function name to invoke_all
        ident: Ident::new("invoke_all", Span::call_site()),
        // Set return type to ()
        output: ReturnType::Default,
        ..names[0].sig.clone()
    };

    // Grab parameter identifiers to invoke_all before appending consumer closure parameter
    let param_ids = invoke_sig
        .inputs
        .iter()
        .cloned()
        .filter_map(|fnarg| match fnarg {
            FnArg::Receiver(_) => None,
            Typed(pattype) => Some(pattype),
        })
        .filter_map(|pat| match *pat.pat {
            Pat::Ident(patident) => Some(patident.ident),
            _ => None,
        })
        .collect::<Vec<_>>();

    // Specify name of closure parameter, if one will be provided:
    let closure_name = "consumer";

    // If return type is (), don't bother adding closure; otherwise do so
    let trailing_empty_return_type: ReturnType = syn::parse(quote!(-> ()).into()).unwrap();
    if output_type != trailing_empty_return_type && output_type != ReturnType::Default {
            // Use method return type to create an impl trait definition for consumer closures
            invoke_sig.inputs.push(Typed(PatType {
                attrs: vec![],
                pat: Box::new(Pat::Ident(PatIdent {
                    attrs: vec![],
                    by_ref: None,
                    mutability: Some(token::Mut {
                        span: Span::call_site(),
                    }),
                    // Parameter name to consumer
                    ident: Ident::new(closure_name, Span::call_site()),
                    subpat: None,
                })),
                colon_token: Default::default(),
                // Set parameter type to impl FnMut(IMPL FUNCTIONS RETURN TYPE) -> ()
                ty: Box::new(Type::ImplTrait(TypeImplTrait {
                    impl_token: Default::default(),
                    bounds: {
                        let mut bounds: Punctuated<_, _> = Punctuated::new();
                        bounds.push(TypeParamBound::Trait(TraitBound {
                            paren_token: None,
                            modifier: TraitBoundModifier::None,
                            lifetimes: None,
                            path: Path {
                                leading_colon: None,
                                segments: {
                                    let mut segments: Punctuated<_, _> = Punctuated::new();
                                    segments.push(PathSegment {
                                        ident: Ident::new("FnMut", Span::call_site()),
                                        arguments: PathArguments::Parenthesized(
                                            ParenthesizedGenericArguments {
                                                paren_token: Default::default(),
                                                inputs: {
                                                    let mut inputs: Punctuated<_, _> = Punctuated::new();
                                                    // If the methods return anything, FnMut should take it as sole argument
                                                    if let ReturnType::Type(_, bx) = output_type.clone() {
                                                        inputs.push(*bx);
                                                    }
                                                    inputs
                                                },
                                                output: ReturnType::Default,
                                            },
                                        ),
                                    });
                                    segments
                                },
                            },
                        }));
                        bounds
                    },
                })),
            }));
    }

    // By this point, supposing the methods have signatures like pub fn name<T: Trait>(arg: T) -> r
    // The invoke_all function has signature
    // pub fn invoke_all<T: Trait>(arg: T, mut consumer: FnMut(r) -> ()) -> ()

    // Set up body block for the invoke_all method:
    let mut invoke_block = Block {
        brace_token: Default::default(),
        stmts: vec![],
    };

    // Iterating over names, call consumer to consume a call of a given function:
    for &name in names {
        // Call function with forwarded parameters
        let inner_call = ExprCall {
            attrs: vec![],
            // Specify function to be called
            func: Box::new(Expr::Path(ExprPath {
                attrs: vec![],
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments: {
                        let mut function_path: Punctuated<_, _> = Punctuated::new();
                        // Add name of struct to path
                        function_path.push(struct_path.clone());
                        // Add name of function to path
                        function_path.push(PathSegment {
                            ident: name.sig.ident.clone(),
                            arguments: PathArguments::None,
                        });
                        function_path
                    },
                },
            })),
            paren_token: Default::default(),
            // forward invoke_all parameters as arguments to this call
            args: {
                let mut args: Punctuated<_, _> = Punctuated::new();
                for identifier in param_ids.iter().cloned() {
                    args.push(Expr::Path(ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: {
                                let mut p: Punctuated<_, _> = Punctuated::new();
                                p.push(PathSegment {
                                    ident: identifier,
                                    arguments: PathArguments::None,
                                });
                                p
                            },
                        },
                    }))
                }
                args
            },
        };

        if output_type != trailing_empty_return_type && output_type != ReturnType::Default {
                // Functions have return type, so the invoke_all function accepts a closure
                // Insert previous call into a call of consumer:
                let outer_call = ExprCall {
                    attrs: vec![],
                    func: Box::new(Expr::Path(ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: {
                                let mut function_path: Punctuated<_, _> = Punctuated::new();
                                function_path.push(PathSegment {
                                    ident: Ident::new(closure_name, Span::call_site()),
                                    arguments: PathArguments::None,
                                });
                                function_path
                            },
                        },
                    })),
                    paren_token: Default::default(),
                    args: {
                        let mut args: Punctuated<_, _> = Punctuated::new();
                        args.push(Expr::Call(inner_call));
                        args
                    },
                };

                // Insert combined call into statements
                invoke_block
                    .stmts
                    .push(Stmt::Semi(Expr::Call(outer_call), Default::default()));
        } else {
            invoke_block
                .stmts
                .push(Stmt::Semi(Expr::Call(inner_call), Default::default()));
        }
    }

    // Combine invoke_sig and invoke_block into an actual combined function
    ImplItem::Method(ImplItemMethod {
        attrs: vec![],
        vis: Visibility::Inherited,
        defaultness: None,
        sig: invoke_sig,
        block: invoke_block,
    })
}

/// Extract the identifier for the struct which the impl block belongs to. Necessary for type
/// qualification of function calls (e.g. X::f())
fn get_struct_identifier_as_path(input: &ItemImpl) -> PathSegment {
    // Get identifier of the struct type this impl block is on
    if let Type::Path(ref tp) = *input.self_ty {
        tp.path.segments[0].clone()
    } else {
        PathSegment {
            ident: Ident::new("Shouldn't be here!", Span::call_site()),
            arguments: PathArguments::None,
        }
    }
}
