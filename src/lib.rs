use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::FnArg::Typed;
use syn::__private::Default;
use syn::__private::Span;
use syn::{parse_macro_input, Block, Expr, ExprCall, FnArg, Ident, ImplItem, ImplItemMethod, ItemImpl, Pat, ReturnType, Signature, Stmt, Type, GenericParam};

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
    let invoke_all = create_invoke_all(
        methods[0],
        &methods,
        get_struct_identifier_as_path(&input).unwrap(),
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

    input.into_token_stream().into()
}

/// Creates a function that invokes all functions in the impl block (all methods in the
/// annotated block must share the same signature). Note that rather than returning, the
/// invoke_all function will share the same parameter signature as the impl block functions but
/// also has a "consumer" FnMut(Original Return Type) -> () parameter added.
fn create_invoke_all(
    base_method: &ImplItemMethod,
    methods: &Vec<&ImplItemMethod>,
    struct_ident: Ident,
) -> ImplItem {
    // Get output type:
    let output_type = base_method.sig.output.clone();

    // Set up the signature for the invoke_all function.
    let mut invoke_sig = Signature {
        // Set function name to invoke_all
        ident: Ident::new("invoke_all", Span::call_site()),
        // Set return type to ()
        output: ReturnType::Default,
        ..base_method.sig.clone()
    };

    let mut is_method = false;

    // Grab parameter identifiers to invoke_all before appending consumer closure parameter
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
            GenericParam::Type(tp) => {Some(tp.ident)}
            _ => {None}
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
                    Ok(syn::parse(quote!(mut #closure_ident: impl FnMut(#bxtype)).into()).unwrap())
                } else {
                    Err("Shouldn't detect an empty return after the if statement!")
                }
            }
            .unwrap(),
        );
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
    for &method in methods {
        // Call function with forwarded parameters
        let method_name = method.sig.ident.clone();
        let inner_call: Expr = if is_method {
            Expr::MethodCall(
                syn::parse(quote!(self.#method_name::<#(#generic_params),*>(#(#param_ids),*)).into()).unwrap()
            )
        } else {
            Expr::Call(
                syn::parse(quote!(#struct_ident::#method_name::<#(#generic_params),*>(#(#param_ids),*)).into())
                    .unwrap(),
            )
        };

        if output_type != trailing_empty_return_type && output_type != ReturnType::Default {
            // Functions have return type, so the invoke_all function accepts a closure

            // Insert previous call into a call of consumer:
            let outer_call: ExprCall =
                syn::parse(quote!(#closure_ident(#inner_call)).into()).unwrap();

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
