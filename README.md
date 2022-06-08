# Description

invoke_impl is a Rust lib crate built around attribute procedural macros that, when applied to a struct's impl block, inserts additional code before compilation that will automate the calling of associated functions and methods inside the impl block provided they share the same signature. 

## How it works

Suppose we have the following Rust struct and associated impl block:

```Rust
  struct Tester1;

    #[invoke_all]
    impl Tester1 {
        pub fn fn1() -> i32 {
            1
        }

        pub fn fn2() -> i32 {
            2
        }

        pub fn fn3() -> i32 {
            3
        }
    }
```

As it currently works, when the #[invoke_all] attribute proc macro is applied to the impl block for Tester1, the following code is produced before compilation (and can be found with the cargo expand command):

```Rust
    struct Tester1;
    impl Tester1 {
        pub fn fn1() -> i32 {
            1
        }
        pub fn fn2() -> i32 {
            2
        }
        pub fn fn3() -> i32 {
            3
        }
        fn invoke_all(mut consumer: impl FnMut(i32)) {
            consumer(Tester1::fn1());
            consumer(Tester1::fn2());
            consumer(Tester1::fn3());
        }
        pub const METHOD_COUNT: usize = 3usize;
        pub const METHOD_LIST: [&'static str; 3usize] = ["fn1", "fn2", "fn3"];
    }
```

As is demonstrated, invoke functions added to impl blocks process the output of the invoked associated functions via a FnMut(function return type) closure. In the event that the assocaited functions do not have a return type, invoke functions will not a closure argument since presumably there is no output from the functions to process. 

## Use cases

The main use case for this crate is obvious: when a user wishes to invoke a large number of functions with identical signatures, typically to do something with the results. This approach with procedural macros has several advantages over alternative ways to address this problem. To begin with, one way to perform a similar behavior is to store a Vec of function pointers, or perhaps of boxed closures. However, both of these approaches would require manually adding the items to the Vec, or using another procedural macro. Furthermore, both techniques do not permit storing of generic functions without specifically instantiating an instance with concrete types, which contributes increasing the code the developer is responsible for maintaining in the event that we have functions that are generic. 

In contrast, when using this approach, functions are automatically added to invoke functions and associated consts when implemented in the impl block. Furthermore, if the functions in the impl block are generic, so to will be the invoke functions generated:

```Rust
    struct Tester4;

    #[invoke_all]
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

```Rust
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
        fn invoke_all<T: Add + Copy>(i: T, j: T, mut consumer: impl FnMut(<T as Add>::Output)) {
            consumer(Tester4::fn1(i, j));
            consumer(Tester4::fn2(i, j));
            consumer(Tester4::fn3(i, j));
        }
        pub const METHOD_COUNT: usize = 3usize;
        pub const METHOD_LIST: [&'static str; 3usize] = ["fn1", "fn2", "fn3"];
    }
```

Altogether, this approach seems to be much more easily maintained (when things go well...)

## Current status

As it stands, the only invoke function that the enum adds is the invoke_all function and the two associated consts. It doesn't work yet for actual methods that take self as a parameter. Additionally, the error output is for the most part garbage as I've focused on trying to get a working macro for most cases as the expense of decent error messages. 

## Future improvements planned

I plan to extend this macro to work for methods that take &self or &mut self, as well as possibly add the ability for user-input in the macro to specify different names for the functions created, to generate multiple such functions for specified function signatures, etc. I also intend to implement enumerated and named versions of invoke_all, so that the closure can more easily know which function/method it is evaluating a return value from; as well as invoke functions/methods that only invoke lists of specified (by index in the associated const array or by functio/method identifier) functions. Also, I need to make sure the invoke functions have the same visibility as the functions they're invoking. All coming soon enough, hopefully!
