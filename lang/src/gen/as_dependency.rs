// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

//! Code generation for smart contracts when they are compiled as a dependency of another smart contract
//! using the generally available `ink-as-dependency` crate feature.
//!
//! The code generated by this generally conflicts with other generated code since an ink! contract
//! that is compiled as dependency no longer requires any infrastructure to dispatch calls or instantiations.
//! However, it requires special treatment for all public messages since their bodies are completely
//! replaced by direct forwards to the remote call infrastructure going through SRML contracts.

use crate::{
    ast,
    hir,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Mutability {
    Immutable,
    Mutable,
}

pub fn generate_code(tokens: &mut TokenStream2, contract: &hir::Contract) {
    let messages = generate_messages_as_dependency(contract);
    let call_enhancer_messages =
        generate_call_enhancer_messages(contract, Mutability::Immutable);
    let call_enhancer_mut_messages =
        generate_call_enhancer_messages(contract, Mutability::Mutable);
    let state = generate_state_as_dependency(contract);
    let contract_ident = &contract.name;

    tokens.extend(quote! {
        #[cfg(feature = "ink-as-dependency")]
        mod as_dependency {
            use super::*;

            #state

            const _: () = {
                impl<E> #contract_ident<E>
                where
                    E: ink_core::env::Env,
                    E::Balance: Default,
                    E::AccountId: Clone,
                {
                    #(#messages)*
                }

                impl<'a, E> CallEnhancer<'a, E>
                where
                    E: ink_core::env::Env,
                    E::Balance: Default,
                    E::AccountId: Clone,
                {
                    #(#call_enhancer_messages)*
                }

                impl<'a, E> CallEnhancerMut<'a, E>
                where
                    E: ink_core::env::Env,
                    E::Balance: Default,
                    E::AccountId: Clone,
                {
                    #(#call_enhancer_mut_messages)*
                }
            };
        }
    });
}

fn generate_state_as_dependency(contract: &hir::Contract) -> TokenStream2 {
    let name = &contract.name;
    let attrs = &contract.state.attrs;
    quote! {
        #( #attrs )*
        pub struct #name<E: ink_core::env::Env> {
            account_id: E::AccountId,
        }

        #( #attrs )*
        pub struct CallEnhancer<'a, E: ink_core::env::Env> {
            contract: &'a #name<E>,
        }

        #( #attrs )*
        pub struct CallEnhancerMut<'a, E: ink_core::env::Env> {
            contract: &'a mut #name<E>,
        }

        impl<E> #name<E>
        where
            E: ink_core::env::Env,
        {
            pub fn from_address(account_id: E::AccountId) -> Self {
                Self { account_id }
            }

            pub fn call(&self) -> CallEnhancer<E> {
                CallEnhancer { contract: self }
            }

            pub fn call_mut(&mut self) -> CallEnhancerMut<E> {
                CallEnhancerMut { contract: self }
            }
        }
    }
}

fn generate_messages_as_dependency<'a>(
    contract: &'a hir::Contract,
) -> impl Iterator<Item = TokenStream2> + 'a {
    let contract_ident = &contract.name;
    contract.messages.iter().map(move |message| {
        let ident = &message.sig.ident;
        let attrs = &message.attrs;
        let args = message.sig.decl.inputs.iter().skip(1);
        let (self_arg, call_fn) = if message.is_mut() {
            (quote! { &mut self }, quote! { call_mut() })
        } else {
            (quote! { &self }, quote! { call() })
        };
        let inputs = message
            .sig
            .decl
            .inputs
            .iter()
            .filter_map(ast::FnArg::ident)
            .map(|ident| quote! { #ident });
        let output = &message.sig.decl.output;
        let (_impl_generics, type_generics, where_clause) =
            message.sig.decl.generics.split_for_impl();
        match output {
            syn::ReturnType::Default => {
                quote! {
                    #(#attrs)*
                    pub fn #ident #type_generics (
                        #self_arg ,
                        #(#args ,)*
                    ) #where_clause {
                        self.#call_fn.#ident( #(#inputs ,)* )
                            .fire()
                            .expect(concat!(
                                "invocation of ",
                                stringify!(#contract_ident), "::", stringify!(#ident),
                                " message was invalid"))
                    }
                }
            }
            syn::ReturnType::Type(_, ty) => {
                quote! {
                    #(#attrs)*
                    pub fn #ident #type_generics (
                        #self_arg ,
                        #(#args ,)*
                    ) -> #ty #where_clause {
                        self.#call_fn.#ident( #(#inputs ,)* )
                            .fire()
                            .expect(concat!(
                                "evaluation of ",
                                stringify!(#contract_ident), "::", stringify!(#ident),
                                " message was invalid"))
                    }
                }
            }
        }
    })
}

fn generate_call_enhancer_messages<'a>(
    contract: &'a hir::Contract,
    mutability: Mutability,
) -> impl Iterator<Item = TokenStream2> + 'a {
    contract.messages
        .iter()
        .filter(move |message| {
            if mutability == Mutability::Mutable {
                message.is_mut()
            } else {
                !message.is_mut()
            }
        })
        .map(|message| {
            let ident = &message.sig.ident;
            let attrs = &message.attrs;
            let args = message.sig.decl.inputs.iter().skip(1);
            let inputs = message.sig.decl.inputs
                .iter()
                .filter_map(ast::FnArg::ident)
                .map(|ident| quote! { #ident });
            let output = &message.sig.decl.output;
            let (_impl_generics, type_generics, where_clause) = message.sig.decl.generics.split_for_impl();
            let selector = message.selector();
            match output {
                syn::ReturnType::Default => quote! {
                    #(#attrs)*
                    pub fn #ident #type_generics (
                        self,
                        #(#args ,)*
                    ) -> ink_core::env::CallBuilder<E, ()> #where_clause {
                        ink_core::env::CallBuilder::<E>::invoke(self.contract.account_id.clone(), #selector)
                            #(
                                .push_arg(#inputs)
                            )*
                    }
                },
                syn::ReturnType::Type(_, ty) => quote! {
                    #(#attrs)*
                    pub fn #ident #type_generics (
                        self,
                        #(#args ,)*
                    ) -> ink_core::env::CallBuilder<E, ink_core::env::ReturnType<#ty>> #where_clause {
                        ink_core::env::CallBuilder::eval(self.contract.account_id.clone(), #selector)
                            #(
                                .push_arg(&#inputs)
                            )*
                    }
                }
            }
        })
}
