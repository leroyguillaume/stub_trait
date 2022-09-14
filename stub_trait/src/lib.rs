//! Macro to implement stub object for a trait.
//!
//! # Usage
//!
//! ```
//! use stub_trait::stub;
//!
//! #[stub]
//! trait Animal {
//!     fn name(&self) -> &str;
//! }
//!
//! let mut animal1 = StubAnimal::default();
//! animal1.stub_all_calls_of_name(|| "Ivana");
//! assert_eq!(animal1.name(), "Ivana");
//! assert_eq!(animal1.name(), "Ivana");
//! assert_eq!(animal1.count_calls_of_name(), 2);
//!
//! let mut animal2 = StubAnimal::default();
//! animal2.register_stub_of_name(|| "Ivana");
//! animal2.register_stub_of_name(|| "Truffle");
//! assert_eq!(animal2.name(), "Ivana");
//! assert_eq!(animal2.name(), "Truffle");
//! assert_eq!(animal2.count_calls_of_name(), 2);
//! ```

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, FnArg, ItemTrait, Lifetime, ReturnType, TraitItem, Type, TypeReference,
};

#[proc_macro_attribute]
pub fn stub(_: TokenStream, input: TokenStream) -> TokenStream {
    let item_trait = parse_macro_input!(input as ItemTrait);

    let trait_ident = &item_trait.ident;
    let stub_struct_ident = format_ident!("Stub{}", trait_ident);

    let mut attrs = vec![];
    let mut impl_fns = vec![];
    let mut stub_fns = vec![];

    for trait_item in &item_trait.items {
        if let TraitItem::Method(item_method) = trait_item {
            let method_inputs = &item_method.sig.inputs;
            let mut method_inputs_iter = method_inputs.iter();
            match method_inputs_iter.next() {
                Some(FnArg::Receiver(_)) => {}
                _ => panic!("The trait must be can made into an object"),
            }
            let mut method_arg_names = vec![];
            let mut method_arg_types = vec![];
            for arg in method_inputs_iter {
                if let FnArg::Typed(arg) = arg {
                    method_arg_names.push(arg.pat.as_ref());
                    method_arg_types.push(arg.ty.as_ref());
                }
            }

            let method_ident = &item_method.sig.ident;
            let attr_ident = format_ident!("{}_stub", method_ident);
            let count_calls_of_fn_ident = format_ident!("count_calls_of_{}", method_ident);
            let stub_all_calls_of_fn_ident = format_ident!("stub_all_calls_of_{}", method_ident);
            let register_stub_of_fn_ident = format_ident!("register_stub_of_{}", method_ident);

            let method_type = match &item_method.sig.output {
                ReturnType::Default => quote! { () },
                ReturnType::Type(_, ty) => match ty.as_ref() {
                    Type::Reference(ty) => {
                        let ty = TypeReference {
                            lifetime: Some(Lifetime::new("'static", Span::call_site())),
                            ..ty.clone()
                        };
                        quote! { #ty }
                    }
                    ty => quote! { #ty },
                },
            };
            let closure_type = quote! {
                Fn(#(#method_arg_types),*) -> #method_type + 'static
            };
            let attr = quote! {
                #attr_ident: Option<stub_trait_core::StubFn<Box<dyn #closure_type>>>
            };
            let fns = quote! {
                pub fn #count_calls_of_fn_ident(&self) -> usize {
                    self.#attr_ident.as_ref()
                        .map(|stub| *stub.count.lock().unwrap())
                        .unwrap_or_default()
                }

                pub fn #register_stub_of_fn_ident<F: #closure_type>(&mut self, f: F) {
                    let f: Box<dyn #closure_type> = Box::new(f);
                    if let Some(ref mut stub) = &mut self.#attr_ident {
                        match &mut stub.kind {
                            stub_trait_core::StubFnKind::AllCalls(_) => {
                                panic!("All calls of {} are already stubbed", stringify!(#method_ident));
                            }
                            stub_trait_core::StubFnKind::CallByCall(ref mut fns) => {
                                fns.push(f);
                            }
                        }
                    } else {
                        let kind = stub_trait_core::StubFnKind::CallByCall(vec![f]);
                        let stub = stub_trait_core::StubFn {
                            count: std::sync::Mutex::new(0),
                            kind,
                        };
                        self.#attr_ident = Some(stub);
                    }
                }

                pub fn #stub_all_calls_of_fn_ident<F: #closure_type>(&mut self, f: F) {
                    if self.#attr_ident.is_some() {
                        panic!("At least one call of {} is already stubbed", stringify!(#method_ident));
                    }
                    let f: Box<dyn #closure_type> = Box::new(f);
                    let stub = stub_trait_core::StubFn {
                        count: std::sync::Mutex::new(0),
                        kind: stub_trait_core::StubFnKind::AllCalls(f),
                    };
                    self.#attr_ident = Some(stub);
                }
            };
            let stub_fn = quote! {
                fn #method_ident(#method_inputs) -> #method_type {
                    if let Some(stub) = &self.#attr_ident {
                        let mut count = stub.count.lock().unwrap();
                        *count += 1;
                        match &stub.kind {
                            stub_trait_core::StubFnKind::AllCalls(f) => f(#(#method_arg_names),*),
                            stub_trait_core::StubFnKind::CallByCall(ref fns) => {
                                if fns.len() < *count {
                                    unimplemented!("{} (too much invocations)", stringify!(#method_ident));
                                }
                                let f = &fns[*count - 1];
                                f(#(#method_arg_names),*)
                            }
                        }
                    } else {
                        unimplemented!(stringify!(#method_ident));
                    }
                }
            };
            attrs.push(attr);
            impl_fns.push(fns);
            stub_fns.push(stub_fn);
        }
    }

    let expanded = quote! {
        #item_trait

        #[derive(Default)]
        pub struct #stub_struct_ident {
            #(#attrs),*
        }

        impl #stub_struct_ident {
            #(#impl_fns)*
        }

        impl #trait_ident for #stub_struct_ident {
            #(#stub_fns)*
        }
    };
    TokenStream::from(expanded)
}
