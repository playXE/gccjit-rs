/*#[macro_use]
extern crate synstructure;
#[macro_use]
extern crate quote;
extern crate proc_macro2;
extern crate proc_macro;

use synstructure::Structure;
use syn::export::TokenStream;


#[proc_macro_derive(Typeable)]
pub fn derive_typeable(s: TokenStream) -> proc_macro::TokenStream {
    let derive_input: syn::DeriveInput = s.into();
    let s = Structure::new(&derive_input);
    let name = s.ast().ident.to_string();
    let body = s.each(|bi| quote!{ctx.new_field(None,#bi::get_type(),stringify!(#bi)),});
    let implementation = s.bound_impl(quote!(gccjit_rs::ty::Typeable),quote! {
        fn get_type<'a,'ctx>(ctx: &'a gccjit_rs::ctx::Context<'ctx>) -> gccjit_rs::ty::Type<'a> {
            let fields = vec![#body];
            ctx.new_struct_type(None,&#name,&fields).as_type()
        }
    });

    quote!(#implementation)
}
*/
//decl_derive!([Typeable] => derive_typeable);

#[macro_use]
extern crate synstructure;
#[macro_use]
extern crate quote;
extern crate proc_macro2;




fn derive_typeable(s: synstructure::Structure) -> proc_macro2::TokenStream {
    let name = s.ast().ident.to_string();

    let body = s.each(|bi| {let fname = bi.ast().ident.clone().unwrap(); quote!{ctx.new_field(None,#bi::get_type(),&#fname),}});
    s.bound_impl(quote!(gccjit_rs::ty::Typeable),quote! {
        fn get_type<'a,'ctx>(ctx: &'a gccjit_rs::ctx::Context<'ctx>) -> gccjit_rs::ty::Type<'a> {
            let fields = vec![#body];
            ctx.new_struct_type(None,&#name,&fields).as_type()
        }
    })
}

decl_derive!([Typeable] => derive_typeable);