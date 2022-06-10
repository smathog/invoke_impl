use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::FnArg::Typed;
use syn::__private::{Default, str};
use syn::__private::Span;
use syn::{parse_macro_input, Block, Expr, ExprCall, FnArg, GenericParam, Ident, ImplItem, ImplItemMethod, ItemImpl, Pat, ReturnType, Signature, Stmt, Type, ItemEnum};

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

    // Validate all methods share identical structure
    validate_signatures(methods[0], &methods);

    let struct_ident = get_struct_identifier_as_path(&input).unwrap();

    // Generate enum
    let enum_tokenstream = create_enum(&methods, struct_ident.clone());

    // Add invoke_all function to impl block:
    let invoke_all = create_invoke_all(
        methods[0],
        &methods,
        struct_ident.clone(),
        InvokeType::All
    );
    input.items.push(invoke_all);

    // Append the number of functions (excluding those added by macro) to the impl block:
    input
        .items
        .push(syn::parse(quote!(pub const METHOD_COUNT: usize = #count;).into()).unwrap());

    // Append an array containing all function identifiers into the tokenstream
    input.items.push(
        syn::parse(quote!(pub const METHOD_LIST: [&'static str; #count] = [#(#names),*];).into())
            .unwrap(),
    );

    let mut revised_impl: TokenStream = input.into_token_stream().into();
    revised_impl.extend(enum_tokenstream);
    revised_impl
}

#[derive(Copy, Clone)]
enum SpecificationType {
    Enum,
    Enumerated,
}
#[derive(Copy, Clone)]
enum InvokeType {
    Specified(SpecificationType),
    SpecifiedAll(SpecificationType),
    Subset,
    All,
}

/// Creates a function that generates an invoke in the impl block (all methods to be invoked must
/// share the same signature, excepting details like names, comments, etc).
/// Note that rather than returning, the
/// invoke  functions will share the same parameter signature as the impl block functions but
/// also has a "consumer" of either
///     FnMut(Original Return Type)
///     FnMut(Enum Variant, Original Return Type)
///     FnMut(usize, Original Return Type)
/// Additionally, an invoke function which is specified (meaning it takes a specified list of
/// which functions to invoke) will further take a parameter of IntoIterator
fn create_invoke_all(
    base_method: &ImplItemMethod,
    methods: &Vec<&ImplItemMethod>,
    struct_ident: Ident,
    invoke_type: InvokeType,
) -> ImplItem {
    // Get output type:
    let output_type = base_method.sig.output.clone();

    let method_name = match invoke_type {
        InvokeType::Specified(specifier) => match specifier {
            SpecificationType::Enum => "invoke_enum",
            SpecificationType::Enumerated => "invoke_enumerated",
        },
        InvokeType::SpecifiedAll(specifier) => match specifier {
            SpecificationType::Enum => "invoke_all_enum",
            SpecificationType::Enumerated => "invoke_all_enumerated",
        },
        InvokeType::All => "invoke_all",
        InvokeType::Subset => "invoke"
    };

    // Set up the signature for the invoke function being constructed.
    let mut invoke_sig = Signature {
        // Set function name to invoke_all
        ident: Ident::new(method_name, Span::call_site()),
        // Set return type to ()
        output: ReturnType::Default,
        ..base_method.sig.clone()
    };

    let mut is_method = false;

    // Grab parameter identifiers to invoke function before appending consumer closure parameter
    let param_ids = invoke_sig
        .inputs
        .iter()
        .cloned()
        .filter_map(|fnarg| match fnarg {
            FnArg::Receiver(receiver) => {
                if receiver.reference.is_some() {
                    is_method = true;
                } else {
                    panic!("invoke_impl cannot be used with methods taking self as move!");
                }
                None
            }
            Typed(pattype) => Some(pattype),
        })
        .filter_map(|pat| match *pat.pat {
            Pat::Ident(patident) => Some(patident.ident),
            _ => None,
        })
        .collect::<Vec<_>>();

    // Get generic parameters
    let generic_params = invoke_sig
        .generics
        .params
        .iter()
        .cloned()
        .filter_map(|gp| match gp {
            GenericParam::Type(tp) => Some(tp.ident),
            _ => None,
        })
        .collect::<Vec<_>>();

    // Specify name of closure parameter, if one will be provided:
    let closure_ident = Ident::new("consumer", Span::call_site());

    // If return type is (), don't bother adding closure; otherwise do so
    let trailing_empty_return_type: ReturnType = syn::parse(quote!(-> ()).into()).unwrap();
    if output_type != trailing_empty_return_type && output_type != ReturnType::Default {
        // Use method return type to create an impl trait definition for consumer closures
        invoke_sig.inputs.push(
            {
                if let ReturnType::Type(_, bx) = output_type.clone() {
                    let bxtype = *bx;
                    match invoke_type {
                        InvokeType::Specified(specifier) => match specifier {
                            SpecificationType::Enum => {
                                todo!()
                            }
                            SpecificationType::Enumerated => Ok(syn::parse(
                                quote!(mut #closure_ident: impl FnMut(usize, #bxtype)).into(),
                            )
                            .unwrap()),
                        },
                        InvokeType::SpecifiedAll(specifier) => match specifier {
                            SpecificationType::Enum => {
                                todo!()
                            }
                            SpecificationType::Enumerated => Ok(syn::parse(
                                quote!(mut #closure_ident: impl FnMut(usize, #bxtype)).into(),
                            )
                            .unwrap()),
                        },
                        InvokeType::All | InvokeType::Subset => Ok(syn::parse(
                            quote!(mut #closure_ident: impl FnMut(#bxtype)).into(),
                        )
                        .unwrap()),
                    }
                } else {
                    Err("Shouldn't detect an empty return after the if statement!")
                }
            }
            .unwrap(),
        );
    }

    // By this point, supposing the methods have signatures like pub fn name<T: Trait>(arg: T) -> r
    // The invoke function has signature like
    // pub fn invoke<T: Trait>(arg: T, mut consumer: FnMut(r) -> ()) -> ()

    // Set up body block for the invoke  method:
    let mut invoke_block = Block {
        brace_token: Default::default(),
        stmts: vec![],
    };

    // Iterating over names, call consumer to consume a call of a given function:
    for (index, &method) in methods.into_iter().enumerate() {
        // Call function with forwarded parameters
        let method_name = method.sig.ident.clone();
        let inner_call: Expr = if is_method {
            Expr::MethodCall(
                syn::parse(
                    quote!(self.#method_name::<#(#generic_params),*>(#(#param_ids),*)).into(),
                )
                .unwrap(),
            )
        } else {
            Expr::Call(
                syn::parse(
                    quote!(#struct_ident::#method_name::<#(#generic_params),*>(#(#param_ids),*))
                        .into(),
                )
                .unwrap(),
            )
        };

        if output_type != trailing_empty_return_type && output_type != ReturnType::Default {
            // Functions have return type, so the invoke_all function accepts a closure

            // Insert previous call into a call of consumer:
            let outer_call: ExprCall = match invoke_type {
                InvokeType::Specified(specifier) => match specifier {
                    SpecificationType::Enum => {
                        todo!()
                    }
                    SpecificationType::Enumerated => {
                        syn::parse(quote!(#closure_ident(#index, #inner_call)).into()).unwrap()
                    }
                },
                InvokeType::SpecifiedAll(specifier) => match specifier {
                    SpecificationType::Enum => {
                        todo!()
                    }
                    SpecificationType::Enumerated => {
                        syn::parse(quote!(#closure_ident(#index, #inner_call)).into()).unwrap()
                    }
                },
                InvokeType::All | InvokeType::Subset => syn::parse(quote!(#closure_ident(#inner_call)).into()).unwrap(),
            };

            // Insert combined call into statements
            invoke_block
                .stmts
                .push(Stmt::Semi(Expr::Call(outer_call), Default::default()));
        } else {
            invoke_block
                .stmts
                .push(Stmt::Semi(inner_call, Default::default()));
        }
    }

    // Combine invoke_sig and invoke_block into an actual combined function
    ImplItem::Method(ImplItemMethod {
        sig: invoke_sig,
        block: invoke_block,
        ..base_method.clone()
    })
}

/// Given a list of methods bound together by some invoke function, generate an enum to
/// represent them. Namely, if methods = [fn1, fn2, fn3, ... fnm] and struct_ident = struct_name,
/// then this will create an enum with members fn1, fn2, fn3, ... fnm
fn create_enum(methods: &Vec<&ImplItemMethod>, struct_ident: Ident) -> TokenStream {
    // Get list of identifiers from methods
    let identifiers = methods
        .into_iter()
        .map(|im| im.sig.ident.clone())
        .collect::<Vec<_>>();

    // Get list of identifiers as strings
    let names = identifiers
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>();

    let num_members = identifiers.len();

    // Generate enum name
    let enum_name = format_ident!("{}_invoke_impl_enum", struct_ident);

    let enum_declaration: ItemEnum = syn::parse(
        quote!(#[derive(Debug, Clone, Copy)]
                pub enum #enum_name {
                #(#identifiers),*
            }).into()
    ).unwrap();

    let enum_impl: ItemImpl = syn::parse(
        quote!(impl #enum_name {
            pub fn iter() -> impl Iterator<Item=&'static #enum_name> {
                use #enum_name::*;
                static members: [#enum_name; #num_members] = [#(#identifiers),*];
                members.iter()
            }
        }).into()
    ).unwrap();

    let try_from_str: ItemImpl = syn::parse(
        quote!(
            impl TryFrom<&str> for #enum_name {
                type Error = &'static str;
                fn try_from(value: &str) -> Result<Self, Self::Error> {
                    match value {
                        #(#names => Ok(Self::#identifiers),)*
                        _ => Err("Input str does not match any enums in Self!")
                    }
                }
            }
        ).into()
    ).unwrap();

    let from_num: ItemImpl = syn::parse(
        quote!(
            impl From<#enum_name> for &str {
                fn from(en: #enum_name) -> Self {
                    use #enum_name::*;
                    match en {
                        #(#identifiers => #names,)*
                    }
                }
            }
        ).into()
    ).unwrap();

    let mut enum_tokenstream: TokenStream = enum_declaration.into_token_stream().into();
    enum_tokenstream.extend::<TokenStream>(enum_impl.into_token_stream().into());
    enum_tokenstream.extend::<TokenStream>(try_from_str.into_token_stream().into());
    enum_tokenstream.extend::<TokenStream>(from_num.into_token_stream().into());
    enum_tokenstream
}

/// Safety function to check that base_method and all other methods share identical signatures
/// except for identity (names). Panics if not true.
fn validate_signatures(base_method: &ImplItemMethod, methods: &Vec<&ImplItemMethod>) {
    let base_signature = Signature {
        ident: Ident::new("name", Span::call_site()),
        ..base_method.sig.clone()
    };

    // Create standard ImplItemMethod to compare against
    let method_comparison = ImplItemMethod {
        sig: base_signature,
        // Discard attrs to get rid of doc comment differences
        attrs: vec![],
        block: Block {
            brace_token: Default::default(),
            stmts: vec![],
        },
        ..base_method.clone()
    };

    // Compare against each method:
    for &method in methods {
        let signature = Signature {
            ident: Ident::new("name", Span::call_site()),
            ..method.sig.clone()
        };

        // Create standard ImplItemMethod to compare against
        let methodimpl = ImplItemMethod {
            sig: signature,
            attrs: vec![],
            block: Block {
                brace_token: Default::default(),
                stmts: vec![],
            },
            ..method.clone()
        };

        if method_comparison != methodimpl {
            panic!(
                "ImplItemMethods different! \
            Base Method: {:?} \
            Method: {:?}",
                method_comparison.to_token_stream().to_string(),
                methodimpl.to_token_stream().to_string()
            );
        }
    }
}

/// Extract the identifier for the struct which the impl block belongs to. Necessary for type
/// qualification of function calls (e.g. X::f())
fn get_struct_identifier_as_path(input: &ItemImpl) -> Result<Ident, &str> {
    // Get identifier of the struct type this impl block is on
    if let Type::Path(ref tp) = *input.self_ty {
        Ok(tp.path.segments[0].ident.clone())
    } else {
        Err("No struct name detected!")
    }
}
