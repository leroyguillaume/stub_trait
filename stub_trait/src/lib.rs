//! Macro to implement stub object for a trait.
//!
//! # Usage
//!
//! ```
//! use stub_trait::stub;
//!
//! #[stub]
//! trait Animal {
//!     fn feed(&self, quantity: usize) -> usize;
//!
//!     fn name(&self) -> &str;
//! }
//!
//! let mut animal = StubAnimal::default();
//! animal.stub_all_calls_of_name(|| "Ivana");
//! animal.register_stub_of_feed(|quantity| quantity - 1);
//! animal.register_stub_of_feed(|quantity| quantity + 1);
//! assert_eq!(animal.name(), "Ivana");
//! assert_eq!(animal.name(), "Ivana");
//! assert_eq!(animal.count_calls_of_name(), 2);
//! assert_eq!(animal.feed(10), 9);
//! assert_eq!(animal.feed(10), 11);
//! ```

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, FnArg, GenericArgument, ItemTrait, Lifetime, PathArguments, ReturnType,
    TraitItem, Type,
};

#[proc_macro_attribute]
pub fn stub(_: TokenStream, input: TokenStream) -> TokenStream {
    let item_trait = parse_macro_input!(input as ItemTrait);

    let trait_ident = &item_trait.ident;
    let trait_generic_params = &item_trait.generics.params;
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

            let mut method_output = item_method.sig.output.clone();
            let method_output = match method_output {
                ReturnType::Default => quote! {},
                ReturnType::Type(_, ref mut ty) => match ty.as_mut() {
                    Type::Path(ty) => {
                        let mut segments = ty.path.segments.clone();
                        let last_segment = segments.last_mut().unwrap();
                        if let PathArguments::AngleBracketed(ty) = &mut last_segment.arguments {
                            if let GenericArgument::Lifetime(lifetime) =
                                ty.args.first_mut().unwrap()
                            {
                                if lifetime.ident == format_ident!("_") {
                                    lifetime.ident = format_ident!("static");
                                }
                            }
                        }
                        quote! { -> #segments }
                    }
                    Type::Reference(ref mut ty) => {
                        if ty.lifetime.is_none() {
                            ty.lifetime = Some(Lifetime::new("'static", Span::call_site()));
                        }
                        quote! { -> #ty }
                    }
                    ty => quote! { -> #ty },
                },
            };
            let closure_type = quote! {
                Fn(#(#method_arg_types),*) #method_output + 'static
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
                fn #method_ident(#method_inputs) #method_output {
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
        pub struct #stub_struct_ident<#trait_generic_params> {
            #(#attrs),*
        }

        impl<#trait_generic_params> #stub_struct_ident<#trait_generic_params> {
            #(#impl_fns)*
        }

        impl<#trait_generic_params> #trait_ident<#trait_generic_params> for #stub_struct_ident<#trait_generic_params> {
            #(#stub_fns)*
        }
    };
    TokenStream::from(expanded)
}
