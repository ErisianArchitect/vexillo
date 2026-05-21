use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input};
use vexcore::flags_input::FlagsInput;

#[proc_macro]
pub fn flags(input: TokenStream) -> TokenStream {
    let flags = parse_macro_input!(input as FlagsInput);
    quote!(#flags).into()
}
