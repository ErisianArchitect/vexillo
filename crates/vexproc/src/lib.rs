use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;


#[proc_macro]
pub fn prototype(input: TokenStream) -> TokenStream {
    quote!().into()
}