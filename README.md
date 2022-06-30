# Description

invoke_impl is a Rust lib crate built around an attribute procedural macro that, when applied to a struct's impl block, inserts additional code before compilation that will produce functions that automate the calling of associated functions and methods inside the impl block provided they share the same signature. 

## How it works

Suppose we have the following Rust struct and associated impl block:

```rust
    struct Tester1;

    #[invoke_impl]
    impl Tester1 {
        pub fn fn1(i: i32) -> i32 {
            i
        }

        pub fn fn2(i: i32) -> i32 {
            i
        }

        pub fn fn3(i: i32) -> i32 {
            i
        }
    }
```

As it currently works, when the #[invoke_impl] attribute proc macro is applied to the impl block for Tester1, the following code is produced before compilation (and can be viewed with the cargo expand command):

```rust
    struct Tester1;
    impl Tester1 {
        pub fn fn1(i: i32) -> i32 {
            i
        }
        pub fn fn2(i: i32) -> i32 {
            i
        }
        pub fn fn3(i: i32) -> i32 {
            i
        }
        pub fn invoke_all(i: i32, mut consumer: impl FnMut(i32)) {
            consumer(Tester1::fn1(i));
            consumer(Tester1::fn2(i));
            consumer(Tester1::fn3(i));
        }
        pub fn invoke_subset(
            i: i32,
            mut consumer: impl FnMut(i32),
            mut invoke_impl_iter: impl Iterator<Item = usize>,
        ) {
            for invoke_impl_i in invoke_impl_iter {
                match invoke_impl_i {
                    0usize => consumer(Tester1::fn1(i)),
                    1usize => consumer(Tester1::fn2(i)),
                    2usize => consumer(Tester1::fn3(i)),
                    _ => ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                        &["Iter contains invalid function index!"],
                        &[],
                    )),
                }
            }
        }
        pub fn invoke_all_enumerated(i: i32, mut consumer: impl FnMut(usize, i32)) {
            consumer(0usize, Tester1::fn1(i));
            consumer(1usize, Tester1::fn2(i));
            consumer(2usize, Tester1::fn3(i));
        }
        pub fn invoke_all_enum(i: i32, mut consumer: impl FnMut(Tester1_invoke_impl_enum, i32)) {
            consumer(Tester1_invoke_impl_enum::fn1, Tester1::fn1(i));
            consumer(Tester1_invoke_impl_enum::fn2, Tester1::fn2(i));
            consumer(Tester1_invoke_impl_enum::fn3, Tester1::fn3(i));
        }
        pub fn invoke_enumerated(
            i: i32,
            mut consumer: impl FnMut(usize, i32),
            mut invoke_impl_iter: impl Iterator<Item = usize>,
        ) {
            for invoke_impl_i in invoke_impl_iter {
                match invoke_impl_i {
                    0usize => {
                        consumer(0usize, Tester1::fn1(i));
                    }
                    1usize => {
                        consumer(1usize, Tester1::fn2(i));
                    }
                    2usize => {
                        consumer(2usize, Tester1::fn3(i));
                    }
                    _ => ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                        &["Iter contains invalid function index!"],
                        &[],
                    )),
                }
            }
        }
        pub fn invoke_enum(
            i: i32,
            mut consumer: impl FnMut(Tester1_invoke_impl_enum, i32),
            mut invoke_impl_iter: impl Iterator<Item = Tester1_invoke_impl_enum>,
        ) {
            for invoke_impl_i in invoke_impl_iter {
                match invoke_impl_i {
                    Tester1_invoke_impl_enum::fn1 => {
                        consumer(Tester1_invoke_impl_enum::fn1, Tester1::fn1(i));
                    }
                    Tester1_invoke_impl_enum::fn2 => {
                        consumer(Tester1_invoke_impl_enum::fn2, Tester1::fn2(i));
                    }
                    Tester1_invoke_impl_enum::fn3 => {
                        consumer(Tester1_invoke_impl_enum::fn3, Tester1::fn3(i));
                    }
                }
            }
        }
        pub const METHOD_COUNT: usize = 3usize;
        pub const METHOD_LIST: [&'static str; 3usize] = ["fn1", "fn2", "fn3"];
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Tester1_invoke_impl_enum {
        fn1,
        fn2,
        fn3,
    }

    impl TryFrom<&str> for Tester1_invoke_impl_enum {
        type Error = &'static str;
        fn try_from(value: &str) -> Result<Self, Self::Error> {
            match value {
                "fn1" => Ok(Self::fn1),
                "fn2" => Ok(Self::fn2),
                "fn3" => Ok(Self::fn3),
                _ => Err("Input str does not match any enums in Self!"),
            }
        }
    }

    impl From<Tester1_invoke_impl_enum> for &str {
        fn from(en: Tester1_invoke_impl_enum) -> Self {
            use Tester1_invoke_impl_enum::*;
            match en {
                fn1 => "fn1",
                fn2 => "fn2",
                fn3 => "fn3",
            }
        }
    }
```

As is demonstrated, the invoke functions added to impl blocks process the output of the invoked associated functions via a FnMut(function return type) closure. In the event that the associated functions do not have a return type, invoke functions will either not have a closure parameter or have a closure that simply takes in a specifier type (either usize or the generated enum type) to indicate which function was called. Namely, if the functions being called have no return type, invoke_all and invoke_subset will not take any closures, invoke_all_enum and invoke_enum will take a closure taking an enum of the type of the enum generated by the macro, and invoke_all_enumerated and invoke_enumerated will take a closure taking usize. 

The invoke_impl attribute can also take two user-specified arguments. The name argument must be a string literal, provided as #[invoke_impl(name("MY_NAME"))]. When this is used, the name argument is appended to provide different identifiers for all the generated code: 

```rust
    struct Tester1;

    #[invoke_impl(name("MY_NAME"))]
    impl Tester1 {
        pub fn fn1(i: i32) -> i32 {
            i
        }

        pub fn fn2(i: i32) -> i32 {
            i
        }

        pub fn fn3(i: i32) -> i32 {
            i
        }
    }
```

becomes

```rust
    struct Tester1;
    impl Tester1 {
        pub fn fn1(i: i32) -> i32 {
            i
        }
        pub fn fn2(i: i32) -> i32 {
            i
        }
        pub fn fn3(i: i32) -> i32 {
            i
        }
        pub fn invoke_all_MY_NAME(i: i32, mut consumer: impl FnMut(i32)) {
            consumer(Tester1::fn1(i));
            consumer(Tester1::fn2(i));
            consumer(Tester1::fn3(i));
        }
        pub fn invoke_subset_MY_NAME(
            i: i32,
            mut consumer: impl FnMut(i32),
            mut invoke_impl_iter: impl Iterator<Item = usize>,
        ) {
            for invoke_impl_i in invoke_impl_iter {
                match invoke_impl_i {
                    0usize => consumer(Tester1::fn1(i)),
                    1usize => consumer(Tester1::fn2(i)),
                    2usize => consumer(Tester1::fn3(i)),
                    _ => ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                        &["Iter contains invalid function index!"],
                        &[],
                    )),
                }
            }
        }
        pub fn invoke_all_enumerated_MY_NAME(i: i32, mut consumer: impl FnMut(usize, i32)) {
            consumer(0usize, Tester1::fn1(i));
            consumer(1usize, Tester1::fn2(i));
            consumer(2usize, Tester1::fn3(i));
        }
        pub fn invoke_all_enum_MY_NAME(
            i: i32,
            mut consumer: impl FnMut(Tester1_invoke_impl_enum_MY_NAME, i32),
        ) {
            consumer(Tester1_invoke_impl_enum_MY_NAME::fn1, Tester1::fn1(i));
            consumer(Tester1_invoke_impl_enum_MY_NAME::fn2, Tester1::fn2(i));
            consumer(Tester1_invoke_impl_enum_MY_NAME::fn3, Tester1::fn3(i));
        }
        pub fn invoke_enumerated_MY_NAME(
            i: i32,
            mut consumer: impl FnMut(usize, i32),
            mut invoke_impl_iter: impl Iterator<Item = usize>,
        ) {
            for invoke_impl_i in invoke_impl_iter {
                match invoke_impl_i {
                    0usize => {
                        consumer(0usize, Tester1::fn1(i));
                    }
                    1usize => {
                        consumer(1usize, Tester1::fn2(i));
                    }
                    2usize => {
                        consumer(2usize, Tester1::fn3(i));
                    }
                    _ => ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                        &["Iter contains invalid function index!"],
                        &[],
                    )),
                }
            }
        }
        pub fn invoke_enum_MY_NAME(
            i: i32,
            mut consumer: impl FnMut(Tester1_invoke_impl_enum_MY_NAME, i32),
            mut invoke_impl_iter: impl Iterator<Item = Tester1_invoke_impl_enum_MY_NAME>,
        ) {
            for invoke_impl_i in invoke_impl_iter {
                match invoke_impl_i {
                    Tester1_invoke_impl_enum_MY_NAME::fn1 => {
                        consumer(Tester1_invoke_impl_enum_MY_NAME::fn1, Tester1::fn1(i));
                    }
                    Tester1_invoke_impl_enum_MY_NAME::fn2 => {
                        consumer(Tester1_invoke_impl_enum_MY_NAME::fn2, Tester1::fn2(i));
                    }
                    Tester1_invoke_impl_enum_MY_NAME::fn3 => {
                        consumer(Tester1_invoke_impl_enum_MY_NAME::fn3, Tester1::fn3(i));
                    }
                }
            }
        }
        pub const METHOD_COUNT_MY_NAME: usize = 3usize;
        pub const METHOD_LIST_MY_NAME: [&'static str; 3usize] = ["fn1", "fn2", "fn3"];
    }
    #[derive(Debug, Clone, Copy)]
    pub enum Tester1_invoke_impl_enum_MY_NAME {
        fn1,
        fn2,
        fn3,
    }
    impl TryFrom<&str> for Tester1_invoke_impl_enum_MY_NAME {
        type Error = &'static str;
        fn try_from(value: &str) -> Result<Self, Self::Error> {
            match value {
                "fn1" => Ok(Self::fn1),
                "fn2" => Ok(Self::fn2),
                "fn3" => Ok(Self::fn3),
                _ => Err("Input str does not match any enums in Self!"),
            }
        }
    }
    impl From<Tester1_invoke_impl_enum_MY_NAME> for &str {
        fn from(en: Tester1_invoke_impl_enum_MY_NAME) -> Self {
            use Tester1_invoke_impl_enum_MY_NAME::*;
            match en {
                fn1 => "fn1",
                fn2 => "fn2",
                fn3 => "fn3",
            }
        }
    }

```

The other argument that invoke_impl can take is the clone argument. Since procedural macros can more or less only work over tokens, the invoke_impl macro cannot tell when an argument that it forwards from an invoke function into an associated function or method call is a move-only type. Therefore, the parameter identifiers are simply copy-pasted into the associated calls. This works fine for types that are copy like usize, or can sometimes implicitly reborrow like &mut (something), but fails for something like String which is move-only. To handle this case, there are two primary options: either make the associated functions/methods in the impl block take their arguments as copy types (namely references), or clone the input for each call. The clone argument is the latter approach. The argument takes a comma-separated list of integer literals indicating which parameters (0-indexed) of the associated functions should be cloned before each call. 

```rust
    struct Tester1;

    #[invoke_impl(clone(1))]
    impl Tester1 {
        pub fn fn1(i: i32, s: String) -> i32 {
            i
        }

        pub fn fn2(i: i32, s: String) -> i32 {
            i
        }

        pub fn fn3(i: i32, s: String) -> i32 {
            i
        }
    }
```

becomes 

```rust
    struct Tester1;
    impl Tester1 {
        pub fn fn1(i: i32, s: String) -> i32 {
            i
        }
        pub fn fn2(i: i32, s: String) -> i32 {
            i
        }
        pub fn fn3(i: i32, s: String) -> i32 {
            i
        }
        pub fn invoke_all(i: i32, s: String, mut consumer: impl FnMut(i32)) {
            consumer(Tester1::fn1(i, s.clone()));
            consumer(Tester1::fn2(i, s.clone()));
            consumer(Tester1::fn3(i, s.clone()));
        }
        pub fn invoke_subset(
            i: i32,
            s: String,
            mut consumer: impl FnMut(i32),
            mut invoke_impl_iter: impl Iterator<Item = usize>,
        ) {
            for invoke_impl_i in invoke_impl_iter {
                match invoke_impl_i {
                    0usize => consumer(Tester1::fn1(i, s.clone())),
                    1usize => consumer(Tester1::fn2(i, s.clone())),
                    2usize => consumer(Tester1::fn3(i, s.clone())),
                    _ => ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                        &["Iter contains invalid function index!"],
                        &[],
                    )),
                }
            }
        }
        pub fn invoke_all_enumerated(i: i32, s: String, mut consumer: impl FnMut(usize, i32)) {
            consumer(0usize, Tester1::fn1(i, s.clone()));
            consumer(1usize, Tester1::fn2(i, s.clone()));
            consumer(2usize, Tester1::fn3(i, s.clone()));
        }
        pub fn invoke_all_enum(
            i: i32,
            s: String,
            mut consumer: impl FnMut(Tester1_invoke_impl_enum, i32),
        ) {
            consumer(Tester1_invoke_impl_enum::fn1, Tester1::fn1(i, s.clone()));
            consumer(Tester1_invoke_impl_enum::fn2, Tester1::fn2(i, s.clone()));
            consumer(Tester1_invoke_impl_enum::fn3, Tester1::fn3(i, s.clone()));
        }
        pub fn invoke_enumerated(
            i: i32,
            s: String,
            mut consumer: impl FnMut(usize, i32),
            mut invoke_impl_iter: impl Iterator<Item = usize>,
        ) {
            for invoke_impl_i in invoke_impl_iter {
                match invoke_impl_i {
                    0usize => {
                        consumer(0usize, Tester1::fn1(i, s.clone()));
                    }
                    1usize => {
                        consumer(1usize, Tester1::fn2(i, s.clone()));
                    }
                    2usize => {
                        consumer(2usize, Tester1::fn3(i, s.clone()));
                    }
                    _ => ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                        &["Iter contains invalid function index!"],
                        &[],
                    )),
                }
            }
        }
        pub fn invoke_enum(
            i: i32,
            s: String,
            mut consumer: impl FnMut(Tester1_invoke_impl_enum, i32),
            mut invoke_impl_iter: impl Iterator<Item = Tester1_invoke_impl_enum>,
        ) {
            for invoke_impl_i in invoke_impl_iter {
                match invoke_impl_i {
                    Tester1_invoke_impl_enum::fn1 => {
                        consumer(Tester1_invoke_impl_enum::fn1, Tester1::fn1(i, s.clone()));
                    }
                    Tester1_invoke_impl_enum::fn2 => {
                        consumer(Tester1_invoke_impl_enum::fn2, Tester1::fn2(i, s.clone()));
                    }
                    Tester1_invoke_impl_enum::fn3 => {
                        consumer(Tester1_invoke_impl_enum::fn3, Tester1::fn3(i, s.clone()));
                    }
                }
            }
        }
        pub const METHOD_COUNT: usize = 3usize;
        pub const METHOD_LIST: [&'static str; 3usize] = ["fn1", "fn2", "fn3"];
    }
    #[derive(Debug, Clone, Copy)]
    pub enum Tester1_invoke_impl_enum {
        fn1,
        fn2,
        fn3,
    }
    impl TryFrom<&str> for Tester1_invoke_impl_enum {
        type Error = &'static str;
        fn try_from(value: &str) -> Result<Self, Self::Error> {
            match value {
                "fn1" => Ok(Self::fn1),
                "fn2" => Ok(Self::fn2),
                "fn3" => Ok(Self::fn3),
                _ => Err("Input str does not match any enums in Self!"),
            }
        }
    }
    impl From<Tester1_invoke_impl_enum> for &str {
        fn from(en: Tester1_invoke_impl_enum) -> Self {
            use Tester1_invoke_impl_enum::*;
            match en {
                fn1 => "fn1",
                fn2 => "fn2",
                fn3 => "fn3",
            }
        }
    }
```

Note that to reduce the overall length of these already long examples, I've removed the code generated from the #[derive()] on the generated enum but it will be visible in practice when using cargo expand.

## Use cases

The main use case for this crate is obvious: when a user wishes to invoke a large number of functions with identical signatures, typically to do something with the results. This approach with procedural macros has several advantages over alternative ways to address this problem. To begin with, one way to perform a similar behavior is to store a Vec of function pointers, or perhaps of boxed closures. However, both of these approaches would require manually adding the items to the Vec, or using another procedural macro. Furthermore, both techniques do not permit storing of generic functions without specifically instantiating an instance with concrete types, which contributes to increasing the code the developer is responsible for maintaining. 

In contrast, when using this approach, functions are automatically added to invoke functions and associated consts when implemented in the impl block. Furthermore, if the functions in the impl block are generic, so to will be the invoke functions generated:

```rust
    struct Tester4;

    #[invoke_impl]
    impl Tester4 {
        pub fn fn1<T: Add + Copy>(i: T, j: T) -> <T as Add>::Output {
            i + j
        }

        pub fn fn2<T: Add + Copy>(i: T, j: T) -> <T as Add>::Output {
            i + j
        }

        pub fn fn3<T: Add + Copy>(i: T, j: T) -> <T as Add>::Output {
            i + j
        }
    }
```

becomes 

```rust
    struct Tester4;
    impl Tester4 {
        pub fn fn1<T: Add + Copy>(i: T, j: T) -> <T as Add>::Output {
            i + j
        }
        pub fn fn2<T: Add + Copy>(i: T, j: T) -> <T as Add>::Output {
            i + j
        }
        pub fn fn3<T: Add + Copy>(i: T, j: T) -> <T as Add>::Output {
            i + j
        }
        pub fn invoke_all<T: Add + Copy>(i: T, j: T, mut consumer: impl FnMut(<T as Add>::Output)) {
            consumer(Tester4::fn1::<T>(i, j));
            consumer(Tester4::fn2::<T>(i, j));
            consumer(Tester4::fn3::<T>(i, j));
        }
        pub const METHOD_COUNT: usize = 3usize;
        pub const METHOD_LIST: [&'static str; 3usize] = ["fn1", "fn2", "fn3"];
    }
```

Altogether, this approach seems to be much more easily maintained (when things go well...).

Note that when there are generic type parameters, the turbofish syntax is automatically applied. This is less important in examples like the one above where T is deducible by looking at the field passed to the function, but it is important in cases like the one below: 

```rust
    struct Tester5;

    #[invoke_impl]
    impl Tester5 {
        pub fn fn1<C: FromIterator<usize>>(i: &Vec<usize>) -> C {
            i.iter().copied().collect::<C>()
        }

        pub fn fn2<C: FromIterator<usize>>(i: &Vec<usize>) -> C {
            i.iter().copied().collect::<C>()
        }

        pub fn fn3<C: FromIterator<usize>>(i: &Vec<usize>) -> C {
            i.iter().copied().collect::<C>()
        }
    }
```

Since C cannot be inferred from the argument the function receives, the implementation of of an invoke function such as invoke_all must (and does) use the turbofish to specify the type of C for each call:

```rust
        pub fn invoke_all<C: FromIterator<usize>>(i: &Vec<usize>, mut consumer: impl FnMut(C)) {
            consumer(Tester5::fn1::<C>(i));
            consumer(Tester5::fn2::<C>(i));
            consumer(Tester5::fn3::<C>(i));
        }
```

Note that because of this [issue](https://github.com/rust-lang/rust/issues/83701), any function with impl trait usage currently will not work with this macro due the presence of the turbofish in the invoke function definitions. 

## Current status

Currently, the invoke functions inherit their visibility from the signature of the first method/function in the impl block. They now work for actual methods that take &self or &mut self as a parameter (how or even if methods that take self as a parameter should be handled is a different matter; I will likely eventually implement it via clone). Additionally, the error output is for the most part garbage as I've focused on trying to get a working macro for most cases as the expense of decent error messages; what error messages do arise will be through panics.

## Future improvements planned

I plan to extend this macro to add the ability for user-input in the macro to specify different names for the functions created, to generate multiple such functions for specified function signatures, etc. Additionally, I hope to write case-handling to deal with impl Trait parameters and return types. All coming soon enough, hopefully!
