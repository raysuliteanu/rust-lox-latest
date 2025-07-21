use proc_macro::TokenStream;

#[proc_macro]
pub fn func(input: TokenStream) -> TokenStream {
    let _ = input;

    unimplemented!()
}
