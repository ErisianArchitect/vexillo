use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use vexcore::flags_input::FlagsInput;

#[proc_macro]
pub fn prototype(input: TokenStream) -> TokenStream {
    quote!().into()
}

#[proc_macro]
pub fn flags(_input: TokenStream) -> TokenStream {
    let _flags = parse_macro_input!(_input as FlagsInput);
    quote!().into()
}