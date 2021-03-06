//! This crate revolves around the single attribute procedural macro invoke_impl, which when applied
//! to a struct impl block where all the methods or associated functions share identical signatures
//! will generate functions that help automate the calling process for invoking these functions.
//!
//! Six functions are generated: invoke_all, invoke_subset, invoke_all_enumerated, invoke_all_enum,
//! invoke_enumerated, and invoke_enum. When the associated functions or methods of the impl block
//! have a return type, these functions all take closure parameters that receive this return type.
//! Additionally, the closures for invoke_all_enumerated and invoke_enumerated take a usize and
//! the closures for invoke_all_enum and invoke_enum take an enum of the type of the macro generated
//! enum, which can be used within the closure to determine specifically which function is being
//! invoked at a given point in time.
//!
//! When there is no return type, invoke_all and invoke_subset do not take a closure, while
//! invoke_all_enumerated and invoke_enumerated take a closure that takes usize and invoke_all_enum
//! and invoke_enum take a closure that takes the type of the macro-generated enum.
//!
//! invoke_impl takes two arguments, name (expecting a string literal) and clone (expecting a list
//! of int literals). Name specifiers an optional name to be appended to the identifiers of
//! generated code, while clone indicates which 0-indexed parameters of the functions or methods
//! in the impl block are to be cloned instead of directly forwarded.
//!
//! Additionally, invoke_impl adds two const fields to the impl block it is on: a list of &str
//! copies of the identifiers of the invocable functions contained in the impl block, and a usize
//! of the total count of invocable functions.
//!
//! For example:
//!
//!```
//!    struct Tester1;
//!
//!    #[invoke_impl]
//!     impl Tester1 {
//!         pub fn fn1(i: i32) -> i32 {
//!             i
//!         }
//!
//!         pub fn fn2(i: i32) -> i32 {
//!            i
//!         }
//!
//!         pub fn fn3(i: i32) -> i32 {
//!             i
//!         }
//!     }
//!```
//! is expanded into the following code:
//!
//! ```
//!     struct Tester1;
//!     impl Tester1 {
//!       pub fn fn1(i: i32) -> i32 {
//!           i
//!       }
//!       pub fn fn2(i: i32) -> i32 {
//!           i
//!       }
//!       pub fn fn3(i: i32) -> i32 {
//!           i
//!       }
//!       pub fn invoke_all(i: i32, mut consumer: impl FnMut(i32)) {
//!           consumer(Tester1::fn1(i));
//!           consumer(Tester1::fn2(i));
//!           consumer(Tester1::fn3(i));
//!       }
//!       pub fn invoke_subset(
//!           i: i32,
//!           mut consumer: impl FnMut(i32),
//!           mut invoke_impl_iter: impl Iterator<Item = usize>,
//!       ) {
//!           for invoke_impl_i in invoke_impl_iter {
//!               match invoke_impl_i {
//!                   0usize => consumer(Tester1::fn1(i)),
//!                   1usize => consumer(Tester1::fn2(i)),
//!                   2usize => consumer(Tester1::fn3(i)),
//!                   _ => ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
//!                       &["Iter contains invalid function index!"],
//!                       &[],
//!                   )),
//!               }
//!           }
//!       }
//!       pub fn invoke_all_enumerated(i: i32, mut consumer: impl FnMut(usize, i32)) {
//!           consumer(0usize, Tester1::fn1(i));
//!           consumer(1usize, Tester1::fn2(i));
//!           consumer(2usize, Tester1::fn3(i));
//!       }
//!       pub fn invoke_all_enum(i: i32, mut consumer: impl FnMut(Tester1_invoke_impl_enum, i32)) {
//!           consumer(Tester1_invoke_impl_enum::fn1, Tester1::fn1(i));
//!           consumer(Tester1_invoke_impl_enum::fn2, Tester1::fn2(i));
//!           consumer(Tester1_invoke_impl_enum::fn3, Tester1::fn3(i));
//!       }
//!       pub fn invoke_enumerated(
//!           i: i32,
//!           mut consumer: impl FnMut(usize, i32),
//!           mut invoke_impl_iter: impl Iterator<Item = usize>,
//!       ) {
//!           for invoke_impl_i in invoke_impl_iter {
//!               match invoke_impl_i {
//!                   0usize => {
//!                       consumer(0usize, Tester1::fn1(i));
//!                   }
//!                   1usize => {
//!                       consumer(1usize, Tester1::fn2(i));
//!                   }
//!                   2usize => {
//!                       consumer(2usize, Tester1::fn3(i));
//!                   }
//!                   _ => ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
//!                       &["Iter contains invalid function index!"],
//!                       &[],
//!                   )),
//!               }
//!           }
//!       }
//!       pub fn invoke_enum(
//!           i: i32,
//!           mut consumer: impl FnMut(Tester1_invoke_impl_enum, i32),
//!           mut invoke_impl_iter: impl Iterator<Item = Tester1_invoke_impl_enum>,
//!       ) {
//!           for invoke_impl_i in invoke_impl_iter {
//!               match invoke_impl_i {
//!                   Tester1_invoke_impl_enum::fn1 => {
//!                       consumer(Tester1_invoke_impl_enum::fn1, Tester1::fn1(i));
//!                   }
//!                   Tester1_invoke_impl_enum::fn2 => {
//!                       consumer(Tester1_invoke_impl_enum::fn2, Tester1::fn2(i));
//!                   }
//!                   Tester1_invoke_impl_enum::fn3 => {
//!                       consumer(Tester1_invoke_impl_enum::fn3, Tester1::fn3(i));
//!                   }
//!               }
//!           }
//!       }
//!       pub const METHOD_COUNT: usize = 3usize;
//!       pub const METHOD_LIST: [&'static str; 3usize] = ["fn1", "fn2", "fn3"];
//!   }
//!   pub enum Tester1_invoke_impl_enum {
//!       fn1,
//!       fn2,
//!       fn3,
//!   }
//!   #[automatically_derived]
//!   #[allow(unused_qualifications)]
//!   impl ::core::fmt::Debug for Tester1_invoke_impl_enum {
//!       fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
//!           match (&*self,) {
//!               (&Tester1_invoke_impl_enum::fn1,) => ::core::fmt::Formatter::write_str(f, "fn1"),
//!               (&Tester1_invoke_impl_enum::fn2,) => ::core::fmt::Formatter::write_str(f, "fn2"),
//!               (&Tester1_invoke_impl_enum::fn3,) => ::core::fmt::Formatter::write_str(f, "fn3"),
//!           }
//!       }
//!   }
//!   #[automatically_derived]
//!   #[allow(unused_qualifications)]
//!   impl ::core::clone::Clone for Tester1_invoke_impl_enum {
//!       #[inline]
//!       fn clone(&self) -> Tester1_invoke_impl_enum {
//!           {
//!               *self
//!           }
//!       }
//!   }
//!   #[automatically_derived]
//!   #[allow(unused_qualifications)]
//!   impl ::core::marker::Copy for Tester1_invoke_impl_enum {}
//!   impl Tester1_invoke_impl_enum {
//!       pub fn iter() -> impl Iterator<Item = &'static Tester1_invoke_impl_enum> {
//!           use Tester1_invoke_impl_enum::*;
//!           static members: [Tester1_invoke_impl_enum; 3usize] = [fn1, fn2, fn3];
//!           members.iter()
//!       }
//!   }
//!   impl TryFrom<&str> for Tester1_invoke_impl_enum {
//!       type Error = &'static str;
//!       fn try_from(value: &str) -> Result<Self, Self::Error> {
//!           match value {
//!               "fn1" => Ok(Self::fn1),
//!               "fn2" => Ok(Self::fn2),
//!               "fn3" => Ok(Self::fn3),
//!               _ => Err("Input str does not match any enums in Self!"),
//!           }
//!       }
//!   }
//!   impl From<Tester1_invoke_impl_enum> for &str {
//!       fn from(en: Tester1_invoke_impl_enum) -> Self {
//!           use Tester1_invoke_impl_enum::*;
//!           match en {
//!               fn1 => "fn1",
//!               fn2 => "fn2",
//!               fn3 => "fn3",
//!           }
//!       }
//!   }
///```

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::FnArg::Typed;
use syn::__private::Span;
use syn::__private::{str, Default};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{
    parse_macro_input, Block, Expr, ExprCall, ExprForLoop, ExprMatch, FnArg, GenericParam, Ident,
    ImplItem, ImplItemMethod, ItemEnum, ItemImpl, Lit, MetaList, NestedMeta, Pat, ReturnType,
    Signature, Stmt, Type,
};

use std::collections::HashSet;

/// Proc macro which appends six different functions to a struct impl block that each represent
/// different ways of invoking functions or methods implemented in that impl block, as well as
/// two associated constants.
#[proc_macro_attribute]
pub fn invoke_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemImpl);
    let (name, clones) = parse_args(args);

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
    let enum_tokenstream = create_enum(&methods, &struct_ident, &name);

    // Generate invoke_all function to impl block:
    let invoke_all = create_invoke_function(
        methods[0],
        &methods,
        &struct_ident,
        InvokeType::All,
        &name,
        &clones,
    );

    // Generate invoke_subset function to impl block:
    let invoke_subset = create_invoke_function(
        methods[0],
        &methods,
        &struct_ident,
        InvokeType::Subset,
        &name,
        &clones,
    );

    // Generate invoke_all_enumerated function to impl block:
    let invoke_all_enumerated = create_invoke_function(
        methods[0],
        &methods,
        &struct_ident,
        InvokeType::SpecifiedAll(SpecificationType::Enumerated),
        &name,
        &clones,
    );

    // Generate invoke_all_enum function to impl block:
    let invoke_all_enum = create_invoke_function(
        methods[0],
        &methods,
        &struct_ident,
        InvokeType::SpecifiedAll(SpecificationType::Enum),
        &name,
        &clones,
    );

    // Generate invoke_enumerated function to impl block:
    let invoke_enumerated = create_invoke_function(
        methods[0],
        &methods,
        &struct_ident,
        InvokeType::Specified(SpecificationType::Enumerated),
        &name,
        &clones,
    );

    // Generate invoke_enum function to impl block:
    let invoke_enum = create_invoke_function(
        methods[0],
        &methods,
        &struct_ident,
        InvokeType::Specified(SpecificationType::Enum),
        &name,
        &clones,
    );

    input.items.push(invoke_all);
    input.items.push(invoke_subset);
    input.items.push(invoke_all_enumerated);
    input.items.push(invoke_all_enum);
    input.items.push(invoke_enumerated);
    input.items.push(invoke_enum);

    // Append the number of functions (excluding those added by macro) to the impl block:
    let mc_ident = if let Some(ref s) = name {
        format_ident!("METHOD_COUNT_{}", s)
    } else {
        format_ident!("METHOD_COUNT")
    };
    input
        .items
        .push(syn::parse(quote!(pub const #mc_ident: usize = #count;).into()).unwrap());

    // Append an array containing all function identifiers into the tokenstream
    let ml_ident = if let Some(ref s) = name {
        format_ident!("METHOD_LIST_{}", s)
    } else {
        format_ident!("METHOD_LIST")
    };
    input.items.push(
        syn::parse(quote!(pub const #ml_ident: [&'static str; #count] = [#(#names),*];).into())
            .unwrap(),
    );

    let mut revised_impl: TokenStream = input.into_token_stream().into();
    revised_impl.extend(enum_tokenstream);
    revised_impl
}

/// Helper enum to specify which kind of specification an invoke function uses: enumerated (usize)
/// or enum (the associated enum constructed).
#[derive(Copy, Clone)]
enum SpecificationType {
    Enum,
    Enumerated,
}

/// Helper enum to indicate which type of invoke function is being built
#[derive(Copy, Clone)]
enum InvokeType {
    /// invoke function takes in specified intoiter over either usize or enum with matching closure
    /// and only invokes functions designed by intoiter
    Specified(SpecificationType),
    /// invoke function has closure taking in either usize or enum plus returntype, invoked over
    /// all functions in marked impl block
    SpecifiedAll(SpecificationType),
    /// invoke function has closure only taking returntype, invoked over intoiter of usize to
    /// indicate which functions get called
    Subset,
    /// invoke function has a closure only taking returntype, invoked over all functions in impl
    /// block
    All,
}

/// Creates a function that generates an invoke in the impl block (all methods to be invoked must
/// share the same signature, excepting details like names, comments, etc).
/// Note that rather than returning, the
/// invoke  functions will share the same parameter signature as the impl block functions but
/// also has a "consumer" of one of:
///     FnMut(Original Return Type)
///     FnMut(Enum Variant, Original Return Type)
///     FnMut(usize, Original Return Type)
/// In the event the return type is (), either implicitly or explicitly, then these are replaced
/// by:
///     (No closure in this instance)
///     FnMut(Enum)
///     FnMut(usize)
/// Additionally, an invoke function which is specified (meaning it takes a specified list of
/// which functions to invoke) will further take a parameter of IntoIterator
fn create_invoke_function(
    base_method: &ImplItemMethod,
    methods: &Vec<&ImplItemMethod>,
    struct_ident: &Ident,
    invoke_type: InvokeType,
    name: &Option<String>,
    clone: &Option<HashSet<usize>>,
) -> ImplItem {
    // Get output type:
    let output_type = base_method.sig.output.clone();

    // Generate Ident for the name of the function
    let invoke_name = generate_invoke_name(name, invoke_type);

    // Generate Ident corresponding to enum name, in case this exists:
    let enum_name = generate_enum_name(struct_ident, name);

    // Set up the signature for the invoke function being constructed.
    let mut invoke_sig = Signature {
        // Set function name to invoke_all
        ident: invoke_name,
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
        .enumerate()
        .filter_map(|(index, fnarg)| match fnarg {
            FnArg::Receiver(receiver) => {
                if receiver.reference.is_some() {
                    is_method = true;
                } else {
                    panic!("invoke_impl cannot be used with methods taking self as move!");
                }
                None
            }
            Typed(pattype) => Some((index, pattype)),
        })
        .filter_map(|(index, pat)| match *pat.pat {
            Pat::Ident(patident) => Some({
                let id = patident.ident;
                if let Some(hs) = clone {
                    if hs.contains(&index) {
                        // Clone this parameter
                        Expr::MethodCall(syn::parse(quote!(#id.clone()).into()).unwrap())
                    } else {
                        // Do not clone this parameter
                        Expr::Path(syn::parse(quote!(#id).into()).unwrap())
                    }
                } else {
                    // All parameters are non-clone
                    Expr::Path(syn::parse(quote!(#id).into()).unwrap())
                }
            }),
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

    // Append correct closure parameter, if necessary
    if output_type != generate_trailing_return_type() && output_type != ReturnType::Default {
        // Use method return type to create an impl trait definition for consumer closures
        let arg = if let ReturnType::Type(_, bx) = output_type.clone() {
            let bxtype = *bx;
            match invoke_type {
                InvokeType::Specified(st) | InvokeType::SpecifiedAll(st) => match st {
                    SpecificationType::Enum => syn::parse(
                        quote!(mut #closure_ident: impl FnMut(#enum_name, #bxtype)).into(),
                    )
                    .unwrap(),
                    SpecificationType::Enumerated => {
                        syn::parse(quote!(mut #closure_ident: impl FnMut(usize, #bxtype)).into())
                            .unwrap()
                    }
                },
                InvokeType::All | InvokeType::Subset => {
                    syn::parse(quote!(mut #closure_ident: impl FnMut(#bxtype)).into()).unwrap()
                }
            }
        } else {
            panic!("Shouldn't detect an empty return after the if statement!")
        };
        invoke_sig.inputs.push(arg);
    } else {
        // Closure doesn't have to take in returntype
        let arg = match invoke_type {
            InvokeType::Specified(st) | InvokeType::SpecifiedAll(st) => match st {
                SpecificationType::Enum => Some(
                    syn::parse(quote!(mut #closure_ident: impl FnMut(#enum_name)).into()).unwrap(),
                ),
                SpecificationType::Enumerated => {
                    Some(syn::parse(quote!(mut #closure_ident: impl FnMut(usize)).into()).unwrap())
                }
            },
            InvokeType::Subset | InvokeType::All => None,
        };
        if let Some(fnarg) = arg {
            invoke_sig.inputs.push(fnarg);
        }
    }

    // If relevant, append parameter specifying which functions to call:
    let specifier = match invoke_type {
        InvokeType::Specified(st) => match st {
            SpecificationType::Enum => Some(
                syn::parse(quote!(mut invoke_impl_iter: impl Iterator<Item=#enum_name>).into())
                    .unwrap(),
            ),
            SpecificationType::Enumerated => Some(
                syn::parse(quote!(mut invoke_impl_iter: impl Iterator<Item=usize>).into()).unwrap(),
            ),
        },
        InvokeType::Subset => Some(
            syn::parse(quote!(mut invoke_impl_iter: impl Iterator<Item=usize>).into()).unwrap(),
        ),
        InvokeType::All | InvokeType::SpecifiedAll(_) => None,
    };
    if let Some(fnarg) = specifier {
        invoke_sig.inputs.push(fnarg);
    }

    // By this point, supposing the methods have signatures like pub fn name<T: Trait>(arg: T) -> r
    // The invoke function has signature like
    // pub fn invoke<T: Trait>(arg: T, mut consumer: FnMut(r) -> ()) -> ()

    // Attach correct body block to correct function signature:
    let invoke_block = match invoke_type {
        InvokeType::Specified(st) => invoke_enum_block(
            is_method,
            st,
            &output_type,
            methods,
            &closure_ident,
            &struct_ident,
            name,
            &generic_params,
            &param_ids,
        ),
        InvokeType::SpecifiedAll(st) => invoke_all_enum_block(
            is_method,
            st,
            &output_type,
            methods,
            &closure_ident,
            &struct_ident,
            name,
            &generic_params,
            &param_ids,
        ),
        InvokeType::Subset => invoke_some_block(
            is_method,
            &output_type,
            methods,
            &closure_ident,
            &struct_ident,
            &generic_params,
            &param_ids,
        ),
        InvokeType::All => invoke_all_block(
            is_method,
            &output_type,
            methods,
            &closure_ident,
            &struct_ident,
            &generic_params,
            &param_ids,
        ),
    };

    // Combine invoke_sig and invoke_block into an actual combined function
    ImplItem::Method(ImplItemMethod {
        sig: invoke_sig,
        block: invoke_block,
        ..base_method.clone()
    })
}

/// Generates a body block for an invoke_all function
fn invoke_all_block(
    is_method: bool,
    output_type: &ReturnType,
    methods: &Vec<&ImplItemMethod>,
    closure_ident: &Ident,
    struct_ident: &Ident,
    generic_params: &Vec<Ident>,
    param_ids: &Vec<Expr>,
) -> Block {
    // Set up body block for the invoke  method:
    let mut invoke_block = Block {
        brace_token: Default::default(),
        stmts: vec![],
    };

    // Iterating over names, call consumer to consume a call of a given function:
    for &method in methods {
        // Call function with forwarded parameters
        let inner_call =
            get_inner_call_expr(is_method, method, struct_ident, generic_params, param_ids);

        if output_type != &generate_trailing_return_type() && output_type != &ReturnType::Default {
            // Functions have return type, so the invoke_all function accepts a closure

            // Insert previous call into a call of consumer:
            let outer_call: ExprCall =
                syn::parse(quote!(#closure_ident(#inner_call)).into()).unwrap();

            // Insert combined call into statements
            invoke_block
                .stmts
                .push(Stmt::Semi(Expr::Call(outer_call), Default::default()));
        } else {
            // Only need to insert inner call
            invoke_block
                .stmts
                .push(Stmt::Semi(inner_call, Default::default()));
        }
    }
    invoke_block
}

/// Generates a body block for the invoke_subset function
fn invoke_some_block(
    is_method: bool,
    output_type: &ReturnType,
    methods: &Vec<&ImplItemMethod>,
    closure_ident: &Ident,
    struct_ident: &Ident,
    generic_params: &Vec<Ident>,
    param_ids: &Vec<Expr>,
) -> Block {
    // Set up body block for the invoke  method:
    let mut invoke_block = Block {
        brace_token: Default::default(),
        stmts: vec![],
    };

    // Set up inner match statement
    let mut match_statement: ExprMatch = syn::parse(quote!(match invoke_impl_i {}).into()).unwrap();

    // Iterate over methods, generating match arms:
    for (index, &method) in methods.into_iter().enumerate() {
        // Get inner call
        let inner_call =
            get_inner_call_expr(is_method, method, struct_ident, generic_params, param_ids);

        // Convert/merge to outer call
        let outer_call = if output_type != &generate_trailing_return_type()
            && output_type != &ReturnType::Default
        {
            // Functions have return type, so the invoke_subset function accepts a closure
            // Insert previous call into a call of consumer:
            syn::parse(quote!(#closure_ident(#inner_call)).into()).unwrap()
        } else {
            // Only want to call the inner function in this case
            inner_call
        };

        // Parse to match arm
        match_statement
            .arms
            .push(syn::parse(quote!(#index => #outer_call,).into()).unwrap());
    }

    // Add default case to match statement
    match_statement.arms.push(
        syn::parse(quote!(_ => panic!("Iter contains invalid function index!")).into()).unwrap(),
    );

    // Wrap match in loop
    let loopexpr: ExprForLoop = syn::parse(
        quote!(for invoke_impl_i in invoke_impl_iter {
            #match_statement
        })
        .into(),
    )
    .unwrap();

    // Add loop to block
    invoke_block.stmts.push(Stmt::Expr(Expr::ForLoop(loopexpr)));

    invoke_block
}

/// Generates bodies for invoke_all_enum and invoke_all_enumerated
fn invoke_all_enum_block(
    is_method: bool,
    specification_type: SpecificationType,
    output_type: &ReturnType,
    methods: &Vec<&ImplItemMethod>,
    closure_ident: &Ident,
    struct_ident: &Ident,
    name: &Option<String>,
    generic_params: &Vec<Ident>,
    param_ids: &Vec<Expr>,
) -> Block {
    // Set up body block for the invoke  method:
    let mut invoke_block = Block {
        brace_token: Default::default(),
        stmts: vec![],
    };

    // Generate enum name
    let enum_name = generate_enum_name(struct_ident, name);

    // Generate list of idents that enum has:
    let identifiers = methods
        .into_iter()
        .map(|im| im.sig.ident.clone())
        .collect::<Vec<_>>();

    for (index, (enum_ident, &method)) in
        identifiers.into_iter().zip(methods.into_iter()).enumerate()
    {
        // Get inner call
        let inner_call =
            get_inner_call_expr(is_method, method, struct_ident, generic_params, param_ids);

        // Convert/merge to outer call
        let outer_call = if output_type != &generate_trailing_return_type()
            && output_type != &ReturnType::Default
        {
            // Functions have return type, so the invoke function accepts a closure with returntype
            // Insert previous call into a call of consumer:
            match specification_type {
                SpecificationType::Enum => {
                    syn::parse(quote!(#closure_ident(#enum_name::#enum_ident, #inner_call)).into())
                        .unwrap()
                }
                SpecificationType::Enumerated => {
                    syn::parse(quote!(#closure_ident(#index, #inner_call)).into()).unwrap()
                }
            }
        } else {
            // Closure takes iteration type; so call inner first and then call closure:
            invoke_block
                .stmts
                .push(Stmt::Semi(inner_call, Default::default()));
            match specification_type {
                SpecificationType::Enum => {
                    syn::parse(quote!(#closure_ident(#enum_name::#enum_ident)).into()).unwrap()
                }
                SpecificationType::Enumerated => {
                    syn::parse(quote!(#closure_ident(#index)).into()).unwrap()
                }
            }
        };

        // Add outer call to block
        invoke_block
            .stmts
            .push(Stmt::Semi(Expr::Call(outer_call), Default::default()));
    }

    invoke_block
}

/// Generates bodies for invoke_enum and invoke_enumerated
fn invoke_enum_block(
    is_method: bool,
    specification_type: SpecificationType,
    output_type: &ReturnType,
    methods: &Vec<&ImplItemMethod>,
    closure_ident: &Ident,
    struct_ident: &Ident,
    name: &Option<String>,
    generic_params: &Vec<Ident>,
    param_ids: &Vec<Expr>,
) -> Block {
    // Set up body block for the invoke  method:
    let mut invoke_block = Block {
        brace_token: Default::default(),
        stmts: vec![],
    };

    // Generate enum name
    let enum_name = generate_enum_name(struct_ident, name);

    // Generate list of idents that enum has:
    let identifiers = methods
        .into_iter()
        .map(|im| im.sig.ident.clone())
        .collect::<Vec<_>>();

    // Set up inner match statement
    let mut match_statement: ExprMatch = syn::parse(quote!(match invoke_impl_i {}).into()).unwrap();

    // Iterate over methods, generating match arms:
    for (index, (enum_ident, &method)) in
        identifiers.into_iter().zip(methods.into_iter()).enumerate()
    {
        // Get inner call
        let inner_call =
            get_inner_call_expr(is_method, method, struct_ident, generic_params, param_ids);

        // Convert/merge to outer call
        let outer_call: Expr = if output_type != &generate_trailing_return_type()
            && output_type != &ReturnType::Default
        {
            // Functions have return type, so the invoke function accepts a closure
            // Insert previous call into a call of consumer with appropriate specification type:
            match specification_type {
                SpecificationType::Enum => syn::parse(
                    quote!({#closure_ident(#enum_name::#enum_ident, #inner_call);}).into(),
                )
                .unwrap(),
                SpecificationType::Enumerated => {
                    syn::parse(quote!({#closure_ident(#index, #inner_call);}).into()).unwrap()
                }
            }
        } else {
            // Need to pass in only specifier to closure, so call inner_call first and then closure
            match specification_type {
                SpecificationType::Enum => syn::parse(
                    quote!({
                        #inner_call;
                        #closure_ident(#enum_name::#enum_ident);
                    })
                    .into(),
                )
                .unwrap(),
                SpecificationType::Enumerated => syn::parse(
                    quote!({
                        #inner_call;
                        #closure_ident(#index);
                    })
                    .into(),
                )
                .unwrap(),
            }
        };

        // Parse to match arm
        match specification_type {
            SpecificationType::Enum => {
                match_statement.arms.push(
                    syn::parse(quote!(#enum_name::#enum_ident => #outer_call,).into()).unwrap(),
                );
            }
            SpecificationType::Enumerated => {
                match_statement
                    .arms
                    .push(syn::parse(quote!(#index => #outer_call,).into()).unwrap());
            }
        }
    }

    // Add default case to match statement if enumerated by usize
    match specification_type {
        SpecificationType::Enum => {}
        SpecificationType::Enumerated => {
            match_statement.arms.push(
                syn::parse(quote!(_ => panic!("Iter contains invalid function index!")).into())
                    .unwrap(),
            );
        }
    }

    // Wrap match in loop
    let loopexpr: ExprForLoop = syn::parse(
        quote!(for invoke_impl_i in invoke_impl_iter {
            #match_statement
        })
        .into(),
    )
    .unwrap();

    // Add loop to block
    invoke_block.stmts.push(Stmt::Expr(Expr::ForLoop(loopexpr)));

    invoke_block
}

/// Helper function to generate inner function calls
fn get_inner_call_expr(
    is_method: bool,
    method: &ImplItemMethod,
    struct_ident: &Ident,
    generic_params: &Vec<Ident>,
    param_ids: &Vec<Expr>,
) -> Expr {
    // Generate inner call
    let method_name = method.sig.ident.clone();
    if is_method {
        Expr::MethodCall(
            syn::parse(quote!(self.#method_name::<#(#generic_params),*>(#(#param_ids),*)).into())
                .unwrap(),
        )
    } else {
        Expr::Call(
            syn::parse(
                quote!(#struct_ident::#method_name::<#(#generic_params),*>(#(#param_ids),*)).into(),
            )
            .unwrap(),
        )
    }
}

/// Given a list of methods bound together by some invoke function, generate an enum to
/// represent them. Namely, if methods = [fn1, fn2, fn3, ... fnm] and struct_ident = struct_name,
/// then this will create an enum with members fn1, fn2, fn3, ... fnm. The created enum will
/// implement Debug, Clone, Copy, and TryFrom<&str>. &str will implement From<enum_name>.
fn create_enum(methods: &Vec<&ImplItemMethod>, struct_ident: &Ident, name: &Option<String>) -> TokenStream {
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
    let enum_name = generate_enum_name(struct_ident, name);

    let enum_declaration: ItemEnum = syn::parse(
        quote!(
            #[allow(non_camel_case_types)]
            #[derive(Debug, Clone, Copy)]
            pub enum #enum_name {
            #(#identifiers),*
        })
        .into(),
    )
    .unwrap();

    let enum_impl: ItemImpl = syn::parse(
        quote!(impl #enum_name {
            pub fn iter() -> impl Iterator<Item=&'static #enum_name> {
                use #enum_name::*;
                static members: [#enum_name; #num_members] = [#(#identifiers),*];
                members.iter()
            }
        })
        .into(),
    )
    .unwrap();

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
        )
        .into(),
    )
    .unwrap();

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
        )
        .into(),
    )
    .unwrap();

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

/// Helper function to parse the args passed into the attribute. Currently, the format parsed will
/// be akin to #[invoke_impl(name("some_string"); clone(2, 3))] where the name field denotes what
/// name (if any) the user wants to give the invoke_functions and enum, and copy indicates which
/// fields of the functions or methods being invoked need to be passed via cloning due to otherwise
/// being moves.
fn parse_args(args: TokenStream) -> (Option<String>, Option<HashSet<usize>>) {
    let punctuated_args = Punctuated::<MetaList, syn::Token![;]>::parse_terminated
        .parse(args)
        .unwrap();
    let mut result = (None, None);
    if punctuated_args.is_empty() {
        // No args, go with defaults
        result
    } else if punctuated_args.len() == 1 || punctuated_args.len() == 2 {
        // Need to parse at least one argument!
        for arg in punctuated_args {
            match arg
                .path
                .get_ident()
                .cloned()
                .unwrap()
                .to_string()
                .to_lowercase()
                .as_str()
            {
                "name" => {
                    if result.0.is_some() {
                        panic!("Argument name passed to invoke_impl twice!")
                    }
                    if arg.nested.len() != 1 {
                        panic!("There can only be a single literal str argument to name!")
                    } else {
                        match &arg.nested[0] {
                            NestedMeta::Meta(_) => {
                                panic!("There can only be a single literal str argument to name!")
                            }
                            NestedMeta::Lit(lit) => {
                                match lit {
                                    Lit::Str(litstr) => result.0 = Some(litstr.value()),
                                    _ => {
                                        panic!("There can only be a single literal str argument to name!")
                                    }
                                }
                            }
                        }
                    }
                }
                "clone" => {
                    if result.1.is_some() {
                        panic!("Argument clone passed to invoke_impl twice!")
                    }
                    let mut indices = HashSet::new();
                    for nm in &arg.nested {
                        match nm {
                            NestedMeta::Meta(_) => {
                                panic!("Arguments to clone must be literal ints!")
                            }
                            NestedMeta::Lit(lit) => match lit {
                                Lit::Int(litint) => {
                                    indices
                                        .insert(litint.base10_digits().parse::<usize>().unwrap());
                                }
                                _ => {
                                    panic!("Arguments to clone must be literal ints!")
                                }
                            },
                        }
                    }
                    result.1 = Some(indices);
                }
                _ => {
                    panic!("The only valid arguments to invoke_impl are name and clone!")
                }
            }
        }
        result
    } else {
        panic!(
            "invoke_impl currently only supports args name and clone in the format \
        #[invoke-impl(name(\"name\"); clone(2, 3, 4)], and more than two args were passed in!"
        );
    }
}

/// Helper function to generate the correct Ident for an invoke function signature
fn generate_invoke_name(name: &Option<String>, invoke_type: InvokeType) -> Ident {
    let base_string = match invoke_type {
        InvokeType::Specified(specifier) => match specifier {
            SpecificationType::Enum => "invoke_enum",
            SpecificationType::Enumerated => "invoke_enumerated",
        },
        InvokeType::SpecifiedAll(specifier) => match specifier {
            SpecificationType::Enum => "invoke_all_enum",
            SpecificationType::Enumerated => "invoke_all_enumerated",
        },
        InvokeType::All => "invoke_all",
        InvokeType::Subset => "invoke_subset",
    };
    if let Some(name_s) = name {
        format_ident!("{}_{}", base_string, name_s)
    } else {
        format_ident!("{}", base_string)
    }
}

/// Helper function to generate the name of the associated enum
fn generate_enum_name(struct_ident: &Ident, name: &Option<String>) -> Ident {
    if let Some(n) = name {
        format_ident!("{}_invoke_impl_enum_{}", struct_ident, n)
    } else {
        format_ident!("{}_invoke_impl_enum", struct_ident)
    }
}

/// Helper function to generate return type -> (), since this parses differently than having no
/// return type at all
fn generate_trailing_return_type() -> ReturnType {
    let trailing_empty_return_type: ReturnType = syn::parse(quote!(-> ()).into()).unwrap();
    trailing_empty_return_type
}
