use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(Hello)]
pub fn hello_derive(input: TokenStream) -> TokenStream {
    // Parse the input TokenStream into a syntax tree
    let ast = syn::parse(input).unwrap();
    impl_hello(&ast)
}

fn impl_hello(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident; // struct name
    let gen = quote! {
        impl Hello for #name {
            fn hello() {
                println!("Hello, I am {}", stringify!(#name));
            }
        }
    };
    gen.into()
}
