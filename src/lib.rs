//! Macro to implement stub object for a trait.
//!
//! # Usage
//!
//! ```
//! use stub_trait::stub;
//!
//! #[stub]
//! trait Animal {
//!     fn feed(&self, quantity: usize) -> &str;
//! }
//!
//! let animal = StubAnimal::new().with_stub_of_feed(|i, quantity| {
//!     if i == 0 {
//!         assert_eq!(quantity, 10);
//!         "sad!"
//!     } else if i == 1 {
//!         assert_eq!(quantity, 20);
//!         "happy!"
//!     } else {
//!         panic!("too much invocations!")
//!     }
//! });
//! assert_eq!(animal.feed(10), "sad!");
//! assert_eq!(animal.feed(20), "happy!");
//! assert_eq!(animal.count_calls_of_feed(), 2);
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
    let mut attrs_init = vec![];
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
            let with_stub_of_fn_ident = format_ident!("with_stub_of_{}", method_ident);

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
                Fn(usize, #(#method_arg_types),*) #method_output + 'static
            };
            let attr = quote! {
                #attr_ident: Option<(Box<dyn #closure_type>, std::sync::atomic::AtomicUsize)>
            };
            let attr_init = quote! {
                #attr_ident: None
            };
            let fns = quote! {
                pub fn #count_calls_of_fn_ident(&self) -> usize {
                    self.#attr_ident.as_ref()
                        .map(|stub| stub.1.load(std::sync::atomic::Ordering::Relaxed))
                        .unwrap_or_default()
                }

                pub fn #with_stub_of_fn_ident<F: #closure_type>(mut self, f: F) -> Self {
                    self.#attr_ident = Some((Box::new(f), std::sync::atomic::AtomicUsize::new(0)));
                    self
                }
            };
            let stub_fn = quote! {
                fn #method_ident(#method_inputs) #method_output {
                    match &self.#attr_ident {
                        Some(stub) => {
                            let i = stub.1.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            stub.0(i, #(#method_arg_names),*)
                        }
                        None => panic!("unexpected invocation of {}", stringify!(#method_ident)),
                    }
                }
            };
            attrs.push(attr);
            attrs_init.push(attr_init);
            impl_fns.push(fns);
            stub_fns.push(stub_fn);
        }
    }

    let expanded = quote! {
        #item_trait

        pub struct #stub_struct_ident<#trait_generic_params> {
            #(#attrs),*
        }

        impl<#trait_generic_params> #stub_struct_ident<#trait_generic_params> {
            pub fn new() -> Self {
                Self {
                    #(#attrs_init),*
                }
            }

            #(#impl_fns)*
        }

        impl<#trait_generic_params> Default for #stub_struct_ident<#trait_generic_params> {
            fn default() -> Self {
                Self::new()
            }
        }

        impl<#trait_generic_params> #trait_ident<#trait_generic_params> for #stub_struct_ident<#trait_generic_params> {
            #(#stub_fns)*
        }
    };
    TokenStream::from(expanded)
}
