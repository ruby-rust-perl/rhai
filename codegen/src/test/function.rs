#[cfg(test)]
mod function_tests {
    use crate::function::ExportedFn;

    use proc_macro2::TokenStream;
    use quote::quote;

    #[test]
    fn minimal_fn() {
        let input_tokens: TokenStream = quote! {
            pub fn do_nothing() { }
        };

        let item_fn = syn::parse2::<ExportedFn>(input_tokens).unwrap();
        assert_eq!(&item_fn.name().to_string(), "do_nothing");
        assert!(!item_fn.mutable_receiver());
        assert!(item_fn.is_public());
        assert!(item_fn.return_type().is_none());
        assert_eq!(item_fn.arg_list().count(), 0);
    }

    #[test]
    fn one_arg_fn() {
        let input_tokens: TokenStream = quote! {
            pub fn do_something(x: usize) { }
        };

        let item_fn = syn::parse2::<ExportedFn>(input_tokens).unwrap();
        assert_eq!(&item_fn.name().to_string(), "do_something");
        assert_eq!(item_fn.arg_list().count(), 1);
        assert!(!item_fn.mutable_receiver());
        assert!(item_fn.is_public());
        assert!(item_fn.return_type().is_none());

        assert_eq!(
            item_fn.arg_list().next().unwrap(),
            &syn::parse2::<syn::FnArg>(quote! { x: usize }).unwrap()
        );
    }

    #[test]
    fn two_arg_fn() {
        let input_tokens: TokenStream = quote! {
            pub fn do_something(x: usize, y: f32) { }
        };

        let item_fn = syn::parse2::<ExportedFn>(input_tokens).unwrap();
        assert_eq!(&item_fn.name().to_string(), "do_something");
        assert_eq!(item_fn.arg_list().count(), 2);
        assert!(!item_fn.mutable_receiver());
        assert!(item_fn.is_public());
        assert!(item_fn.return_type().is_none());

        assert_eq!(
            item_fn.arg_list().next().unwrap(),
            &syn::parse2::<syn::FnArg>(quote! { x: usize }).unwrap()
        );
        assert_eq!(
            item_fn.arg_list().nth(1).unwrap(),
            &syn::parse2::<syn::FnArg>(quote! { y: f32 }).unwrap()
        );
    }

    #[test]
    fn usize_returning_fn() {
        let input_tokens: TokenStream = quote! {
            pub fn get_magic_number() -> usize { 42 }
        };

        let item_fn = syn::parse2::<ExportedFn>(input_tokens).unwrap();
        assert_eq!(&item_fn.name().to_string(), "get_magic_number");
        assert!(!item_fn.mutable_receiver());
        assert!(item_fn.is_public());
        assert_eq!(item_fn.arg_list().count(), 0);
        assert_eq!(
            item_fn.return_type().unwrap(),
            &syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::parse2::<syn::Path>(quote! { usize }).unwrap()
            })
        );
    }

    #[test]
    fn ref_returning_fn() {
        let input_tokens: TokenStream = quote! {
            pub fn get_magic_phrase() -> &'static str { "open sesame" }
        };

        let err = syn::parse2::<ExportedFn>(input_tokens).unwrap_err();
        assert_eq!(
            format!("{}", err),
            "Rhai functions cannot return references"
        );
    }

    #[test]
    fn ptr_returning_fn() {
        let input_tokens: TokenStream = quote! {
            pub fn get_magic_phrase() -> *const str { "open sesame" }
        };

        let err = syn::parse2::<ExportedFn>(input_tokens).unwrap_err();
        assert_eq!(format!("{}", err), "Rhai functions cannot return pointers");
    }

    #[test]
    fn ref_arg_fn() {
        let input_tokens: TokenStream = quote! {
            pub fn greet(who: &Person) { }
        };

        let err = syn::parse2::<ExportedFn>(input_tokens).unwrap_err();
        assert_eq!(
            format!("{}", err),
            "references from Rhai in this position must be mutable"
        );
    }

    #[test]
    fn ref_second_arg_fn() {
        let input_tokens: TokenStream = quote! {
            pub fn greet(count: usize, who: &Person) { }
        };

        let err = syn::parse2::<ExportedFn>(input_tokens).unwrap_err();
        assert_eq!(
            format!("{}", err),
            "this type in this position passes from Rhai by value"
        );
    }

    #[test]
    fn mut_ref_second_arg_fn() {
        let input_tokens: TokenStream = quote! {
            pub fn give(item_name: &str, who: &mut Person) { }
        };

        let err = syn::parse2::<ExportedFn>(input_tokens).unwrap_err();
        assert_eq!(
            format!("{}", err),
            "this type in this position passes from Rhai by value"
        );
    }

    #[test]
    fn str_arg_fn() {
        let input_tokens: TokenStream = quote! {
            pub fn log(message: &str) { }
        };

        let item_fn = syn::parse2::<ExportedFn>(input_tokens).unwrap();
        assert_eq!(&item_fn.name().to_string(), "log");
        assert_eq!(item_fn.arg_list().count(), 1);
        assert!(!item_fn.mutable_receiver());
        assert!(item_fn.is_public());
        assert!(item_fn.return_type().is_none());

        assert_eq!(
            item_fn.arg_list().next().unwrap(),
            &syn::parse2::<syn::FnArg>(quote! { message: &str }).unwrap()
        );
    }

    #[test]
    fn str_second_arg_fn() {
        let input_tokens: TokenStream = quote! {
            pub fn log(level: usize, message: &str) { }
        };

        let item_fn = syn::parse2::<ExportedFn>(input_tokens).unwrap();
        assert_eq!(&item_fn.name().to_string(), "log");
        assert_eq!(item_fn.arg_list().count(), 2);
        assert!(!item_fn.mutable_receiver());
        assert!(item_fn.is_public());
        assert!(item_fn.return_type().is_none());

        assert_eq!(
            item_fn.arg_list().next().unwrap(),
            &syn::parse2::<syn::FnArg>(quote! { level: usize }).unwrap()
        );
        assert_eq!(
            item_fn.arg_list().nth(1).unwrap(),
            &syn::parse2::<syn::FnArg>(quote! { message: &str }).unwrap()
        );
    }

    #[test]
    fn private_fn() {
        let input_tokens: TokenStream = quote! {
            fn do_nothing() { }
        };

        let item_fn = syn::parse2::<ExportedFn>(input_tokens).unwrap();
        assert_eq!(&item_fn.name().to_string(), "do_nothing");
        assert!(!item_fn.mutable_receiver());
        assert!(!item_fn.is_public());
        assert!(item_fn.return_type().is_none());
        assert_eq!(item_fn.arg_list().count(), 0);
    }

    #[test]
    fn receiver_fn() {
        let input_tokens: TokenStream = quote! {
            pub fn act_upon(&mut self) { }
        };

        let item_fn = syn::parse2::<ExportedFn>(input_tokens).unwrap();
        assert_eq!(&item_fn.name().to_string(), "act_upon");
        assert!(item_fn.mutable_receiver());
        assert!(item_fn.is_public());
        assert!(item_fn.return_type().is_none());
        assert_eq!(item_fn.arg_list().count(), 1);
    }

    #[test]
    fn immutable_receiver_fn() {
        let input_tokens: TokenStream = quote! {
            pub fn act_upon(&self) { }
        };

        let item_fn = syn::parse2::<ExportedFn>(input_tokens).unwrap();
        assert_eq!(&item_fn.name().to_string(), "act_upon");
        assert!(item_fn.mutable_receiver());
        assert!(item_fn.is_public());
        assert!(item_fn.return_type().is_none());
        assert_eq!(item_fn.arg_list().count(), 1);
    }
}

#[cfg(test)]
mod generate_tests {
    use crate::function::ExportedFn;

    use proc_macro2::TokenStream;
    use quote::quote;

    fn assert_streams_eq(actual: TokenStream, expected: TokenStream) {
        let actual = actual.to_string();
        let expected = expected.to_string();
        if &actual != &expected {
            let mut counter = 0;
            let _iter = actual.chars().zip(expected.chars()).skip_while(|(a, e)| {
                if *a == *e {
                    counter += 1;
                    true
                } else {
                    false
                }
            });
            let (_actual_diff, _expected_diff) = {
                let mut actual_diff = String::new();
                let mut expected_diff = String::new();
                for (a, e) in _iter.take(50) {
                    actual_diff.push(a);
                    expected_diff.push(e);
                }
                (actual_diff, expected_diff)
            };
            eprintln!("actual != expected, diverge at char {}", counter);
            // eprintln!("  actual: {}", _actual_diff);
            // eprintln!("expected: {}", _expected_diff);
            // assert!(false);
        }
        assert_eq!(actual, expected);
    }

    #[test]
    fn minimal_fn() {
        let input_tokens: TokenStream = quote! {
            pub fn do_nothing() { }
        };

        let expected_tokens = quote! {
            #[automatically_derived]
            pub mod rhai_fn_do_nothing {
                use super::*;
                struct Token();
                impl PluginFunction for Token {
                    #[inline(always)] fn call(&self, context: NativeCallContext, args: &mut [&mut Dynamic]) -> RhaiResult {
                        debug_assert_eq!(args.len(), 0usize,
                                         "wrong arg count: {} != {}", args.len(), 0usize);
                        Ok(Dynamic::from(do_nothing()))
                    }

                    #[inline(always)] fn is_method_call(&self) -> bool { false }
                    #[inline(always)] fn is_variadic(&self) -> bool { false }
                    #[inline(always)] fn clone_boxed(&self) -> Box<dyn PluginFunction> { Box::new(Token()) }
                    #[inline(always)] fn param_names(&self) -> Box<[&'static str]> {
                        new_vec!["()"].into_boxed_slice()
                    }
                    #[inline(always)] fn input_types(&self) -> Box<[TypeId]> {
                        new_vec![].into_boxed_slice()
                    }
                }
                #[inline(always)] pub fn token_callable() -> CallableFunction {
                    Token().into()
                }
                #[inline(always)] pub fn token_param_names() -> Box<[&'static str]> {
                    Token().param_names()
                }
                #[inline(always)] pub fn token_input_types() -> Box<[TypeId]> {
                    Token().input_types()
                }
                #[allow(unused)]
                #[inline(always)] pub fn dynamic_result_fn() -> Result<Dynamic, Box<EvalAltResult> > {
                    Ok(Dynamic::from(do_nothing()))
                }
            }
        };

        let item_fn = syn::parse2::<ExportedFn>(input_tokens).unwrap();
        assert_streams_eq(item_fn.generate(), expected_tokens);
    }

    #[test]
    fn one_arg_usize_fn() {
        let input_tokens: TokenStream = quote! {
            pub fn do_something(x: usize) { }
        };

        let expected_tokens = quote! {
            #[automatically_derived]
            pub mod rhai_fn_do_something {
                use super::*;
                struct Token();
                impl PluginFunction for Token {
                    #[inline(always)]
                    fn call(&self, context: NativeCallContext, args: &mut [&mut Dynamic]) -> RhaiResult {
                        debug_assert_eq!(args.len(), 1usize,
                                    "wrong arg count: {} != {}", args.len(), 1usize);
                        let arg0 = mem::take(args[0usize]).cast::<usize>();
                        Ok(Dynamic::from(do_something(arg0)))
                    }

                    #[inline(always)] fn is_method_call(&self) -> bool { false }
                    #[inline(always)] fn is_variadic(&self) -> bool { false }
                    #[inline(always)] fn clone_boxed(&self) -> Box<dyn PluginFunction> { Box::new(Token()) }
                    #[inline(always)] fn param_names(&self) -> Box<[&'static str]> {
                        new_vec!["x: usize", "()"].into_boxed_slice()
                    }
                    #[inline(always)] fn input_types(&self) -> Box<[TypeId]> {
                        new_vec![TypeId::of::<usize>()].into_boxed_slice()
                    }
                }
                #[inline(always)] pub fn token_callable() -> CallableFunction {
                    Token().into()
                }
                #[inline(always)] pub fn token_param_names() -> Box<[&'static str]> {
                    Token().param_names()
                }
                #[inline(always)] pub fn token_input_types() -> Box<[TypeId]> {
                    Token().input_types()
                }
                #[allow(unused)]
                #[inline(always)] pub fn dynamic_result_fn(x: usize) -> Result<Dynamic, Box<EvalAltResult> > {
                    Ok(Dynamic::from(do_something(x)))
                }
            }
        };

        let item_fn = syn::parse2::<ExportedFn>(input_tokens).unwrap();
        assert_streams_eq(item_fn.generate(), expected_tokens);
    }

    #[test]
    fn one_arg_fn_with_context() {
        let input_tokens: TokenStream = quote! {
            pub fn do_something(context: NativeCallContext, x: usize) {}
        };

        let expected_tokens = quote! {
            #[automatically_derived]
            pub mod rhai_fn_do_something {
                use super::*;
                struct Token();
                impl PluginFunction for Token {
                    #[inline(always)]
                    fn call(&self, context: NativeCallContext, args: &mut [&mut Dynamic]) -> RhaiResult {
                        debug_assert_eq!(args.len(), 1usize,
                                    "wrong arg count: {} != {}", args.len(), 1usize);
                        let arg0 = mem::take(args[0usize]).cast::<usize>();
                        Ok(Dynamic::from(do_something(context, arg0)))
                    }

                    #[inline(always)] fn is_method_call(&self) -> bool { false }
                    #[inline(always)] fn is_variadic(&self) -> bool { false }
                    #[inline(always)] fn clone_boxed(&self) -> Box<dyn PluginFunction> { Box::new(Token()) }
                    #[inline(always)] fn param_names(&self) -> Box<[&'static str]> {
                        new_vec!["x: usize", "()"].into_boxed_slice()
                    }
                    #[inline(always)] fn input_types(&self) -> Box<[TypeId]> {
                        new_vec![TypeId::of::<usize>()].into_boxed_slice()
                    }
                }
                #[inline(always)] pub fn token_callable() -> CallableFunction {
                    Token().into()
                }
                #[inline(always)] pub fn token_param_names() -> Box<[&'static str]> {
                    Token().param_names()
                }
                #[inline(always)] pub fn token_input_types() -> Box<[TypeId]> {
                    Token().input_types()
                }
                #[allow(unused)]
                #[inline(always)] pub fn dynamic_result_fn(context: NativeCallContext, x: usize) -> Result<Dynamic, Box<EvalAltResult> > {
                    Ok(Dynamic::from(do_something(context, x)))
                }
            }
        };

        let item_fn = syn::parse2::<ExportedFn>(input_tokens).unwrap();
        assert!(item_fn.pass_context());
        assert_streams_eq(item_fn.generate(), expected_tokens);
    }

    #[test]
    fn return_dynamic() {
        let input_tokens: TokenStream = quote! {
            pub fn return_dynamic() -> (((rhai::Dynamic))) {
                ().into()
            }
        };

        let expected_tokens = quote! {
            #[automatically_derived]
            pub mod rhai_fn_return_dynamic {
                use super::*;
                struct Token();
                impl PluginFunction for Token {
                    #[inline(always)]
                    fn call(&self, context: NativeCallContext, args: &mut [&mut Dynamic]) -> RhaiResult {
                        debug_assert_eq!(args.len(), 0usize,
                                         "wrong arg count: {} != {}", args.len(), 0usize);
                        Ok(Dynamic::from(return_dynamic()))
                    }

                    #[inline(always)] fn is_method_call(&self) -> bool { false }
                    #[inline(always)] fn is_variadic(&self) -> bool { false }
                    #[inline(always)] fn clone_boxed(&self) -> Box<dyn PluginFunction> { Box::new(Token()) }
                    #[inline(always)] fn param_names(&self) -> Box<[&'static str]> {
                        new_vec!["rhai::Dynamic"].into_boxed_slice()
                    }
                    #[inline(always)] fn input_types(&self) -> Box<[TypeId]> {
                        new_vec![].into_boxed_slice()
                    }
                }
                #[inline(always)] pub fn token_callable() -> CallableFunction {
                    Token().into()
                }
                #[inline(always)] pub fn token_param_names() -> Box<[&'static str]> {
                    Token().param_names()
                }
                #[inline(always)] pub fn token_input_types() -> Box<[TypeId]> {
                    Token().input_types()
                }
                #[allow(unused)]
                #[inline(always)] pub fn dynamic_result_fn() -> Result<Dynamic, Box<EvalAltResult> > {
                    Ok(Dynamic::from(return_dynamic()))
                }
            }
        };

        let item_fn = syn::parse2::<ExportedFn>(input_tokens).unwrap();
        assert_streams_eq(item_fn.generate(), expected_tokens);
    }

    #[test]
    fn one_arg_usize_fn_impl() {
        let input_tokens: TokenStream = quote! {
            pub fn do_something(x: usize) { }
        };

        let expected_tokens = quote! {
            impl PluginFunction for TestStruct {
                #[inline(always)]
                fn call(&self, context: NativeCallContext, args: &mut [&mut Dynamic]) -> RhaiResult {
                    debug_assert_eq!(args.len(), 1usize,
                                "wrong arg count: {} != {}", args.len(), 1usize);
                    let arg0 = mem::take(args[0usize]).cast::<usize>();
                    Ok(Dynamic::from(do_something(arg0)))
                }

                #[inline(always)] fn is_method_call(&self) -> bool { false }
                #[inline(always)] fn is_variadic(&self) -> bool { false }
                #[inline(always)] fn clone_boxed(&self) -> Box<dyn PluginFunction> { Box::new(TestStruct()) }
                #[inline(always)] fn param_names(&self) -> Box<[&'static str]> {
                    new_vec!["x: usize", "()"].into_boxed_slice()
                }
                #[inline(always)] fn input_types(&self) -> Box<[TypeId]> {
                    new_vec![TypeId::of::<usize>()].into_boxed_slice()
                }
        }
        };

        let item_fn = syn::parse2::<ExportedFn>(input_tokens).unwrap();
        assert_streams_eq(item_fn.generate_impl("TestStruct"), expected_tokens);
    }

    #[test]
    fn two_arg_returning_usize_fn() {
        let input_tokens: TokenStream = quote! {
            pub fn add_together(x: usize, y: usize) -> usize { x + y }
        };

        let expected_tokens = quote! {
            #[automatically_derived]
            pub mod rhai_fn_add_together {
                use super::*;
                struct Token();
                impl PluginFunction for Token {
                    #[inline(always)]
                    fn call(&self, context: NativeCallContext, args: &mut [&mut Dynamic]) -> RhaiResult {
                        debug_assert_eq!(args.len(), 2usize,
                                    "wrong arg count: {} != {}", args.len(), 2usize);
                        let arg0 = mem::take(args[0usize]).cast::<usize>();
                        let arg1 = mem::take(args[1usize]).cast::<usize>();
                        Ok(Dynamic::from(add_together(arg0, arg1)))
                    }

                    #[inline(always)] fn is_method_call(&self) -> bool { false }
                    #[inline(always)] fn is_variadic(&self) -> bool { false }
                    #[inline(always)] fn clone_boxed(&self) -> Box<dyn PluginFunction> { Box::new(Token()) }
                    #[inline(always)] fn param_names(&self) -> Box<[&'static str]> {
                        new_vec!["x: usize", "y: usize", "usize"].into_boxed_slice()
                    }
                    #[inline(always)] fn input_types(&self) -> Box<[TypeId]> {
                        new_vec![TypeId::of::<usize>(),
                             TypeId::of::<usize>()].into_boxed_slice()
                    }
                }
                #[inline(always)] pub fn token_callable() -> CallableFunction {
                    Token().into()
                }
                #[inline(always)] pub fn token_param_names() -> Box<[&'static str]> {
                    Token().param_names()
                }
                #[inline(always)] pub fn token_input_types() -> Box<[TypeId]> {
                    Token().input_types()
                }
                #[allow(unused)]
                #[inline(always)] pub fn dynamic_result_fn(x: usize, y: usize) -> Result<Dynamic, Box<EvalAltResult> > {
                    Ok(Dynamic::from(add_together(x, y)))
                }
            }
        };

        let item_fn = syn::parse2::<ExportedFn>(input_tokens).unwrap();
        assert_streams_eq(item_fn.generate(), expected_tokens);
    }

    #[test]
    fn mut_arg_usize_fn() {
        let input_tokens: TokenStream = quote! {
            pub fn increment(x: &mut usize, y: usize) { *x += y; }
        };

        let expected_tokens = quote! {
            #[automatically_derived]
            pub mod rhai_fn_increment {
                use super::*;
                struct Token();
                impl PluginFunction for Token {
                    #[inline(always)]
                    fn call(&self, context: NativeCallContext, args: &mut [&mut Dynamic]) -> RhaiResult {
                        debug_assert_eq!(args.len(), 2usize,
                                    "wrong arg count: {} != {}", args.len(), 2usize);
                        if args[0usize].is_read_only() {
                            return Err(Box::new(
                                EvalAltResult::ErrorAssignmentToConstant("x".to_string(), Position::NONE)
                            ));
                        }
                        let arg1 = mem::take(args[1usize]).cast::<usize>();
                        let arg0 = &mut args[0usize].write_lock::<usize>().unwrap();
                        Ok(Dynamic::from(increment(arg0, arg1)))
                    }

                    #[inline(always)] fn is_method_call(&self) -> bool { true }
                    #[inline(always)] fn is_variadic(&self) -> bool { false }
                    #[inline(always)] fn clone_boxed(&self) -> Box<dyn PluginFunction> { Box::new(Token()) }
                    #[inline(always)] fn param_names(&self) -> Box<[&'static str]> {
                        new_vec!["x: &mut usize", "y: usize", "()"].into_boxed_slice()
                    }
                    #[inline(always)] fn input_types(&self) -> Box<[TypeId]> {
                        new_vec![TypeId::of::<usize>(),
                             TypeId::of::<usize>()].into_boxed_slice()
                    }
                }
                #[inline(always)] pub fn token_callable() -> CallableFunction {
                    Token().into()
                }
                #[inline(always)] pub fn token_param_names() -> Box<[&'static str]> {
                    Token().param_names()
                }
                #[inline(always)] pub fn token_input_types() -> Box<[TypeId]> {
                    Token().input_types()
                }
                #[allow(unused)]
                #[inline(always)] pub fn dynamic_result_fn(x: &mut usize, y: usize) -> Result<Dynamic, Box<EvalAltResult> > {
                    Ok(Dynamic::from(increment(x, y)))
                }
            }
        };

        let item_fn = syn::parse2::<ExportedFn>(input_tokens).unwrap();
        assert!(item_fn.mutable_receiver());
        assert_streams_eq(item_fn.generate(), expected_tokens);
    }

    #[test]
    fn str_arg_fn() {
        let input_tokens: TokenStream = quote! {
            pub fn special_print(message: &str) { eprintln!("----{}----", message); }
        };

        let expected_tokens = quote! {
            #[automatically_derived]
            pub mod rhai_fn_special_print {
                use super::*;
                struct Token();
                impl PluginFunction for Token {
                    #[inline(always)]
                    fn call(&self, context: NativeCallContext, args: &mut [&mut Dynamic]) -> RhaiResult {
                        debug_assert_eq!(args.len(), 1usize,
                                    "wrong arg count: {} != {}", args.len(), 1usize);
                        let arg0 = mem::take(args[0usize]).take_immutable_string().unwrap();
                        Ok(Dynamic::from(special_print(&arg0)))
                    }

                    #[inline(always)] fn is_method_call(&self) -> bool { false }
                    #[inline(always)] fn is_variadic(&self) -> bool { false }
                    #[inline(always)] fn clone_boxed(&self) -> Box<dyn PluginFunction> { Box::new(Token()) }
                    #[inline(always)] fn param_names(&self) -> Box<[&'static str]> {
                        new_vec!["message: &str", "()"].into_boxed_slice()
                    }
                    #[inline(always)] fn input_types(&self) -> Box<[TypeId]> {
                        new_vec![TypeId::of::<ImmutableString>()].into_boxed_slice()
                    }
                }
                #[inline(always)] pub fn token_callable() -> CallableFunction {
                    Token().into()
                }
                #[inline(always)] pub fn token_param_names() -> Box<[&'static str]> {
                    Token().param_names()
                }
                #[inline(always)] pub fn token_input_types() -> Box<[TypeId]> {
                    Token().input_types()
                }
                #[allow(unused)]
                #[inline(always)] pub fn dynamic_result_fn(message: &str) -> Result<Dynamic, Box<EvalAltResult> > {
                    Ok(Dynamic::from(special_print(message)))
                }
            }
        };

        let item_fn = syn::parse2::<ExportedFn>(input_tokens).unwrap();
        assert!(!item_fn.mutable_receiver());
        assert_streams_eq(item_fn.generate(), expected_tokens);
    }
}
