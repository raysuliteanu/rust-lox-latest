use proc_macro::TokenStream;

/*
* Some possible usage:
*   let func = lox_func!(self.eval_func_decl(...));
*
*
*/

#[proc_macro]
pub fn lox_func(input: TokenStream) -> TokenStream {
    let _ = input;

    unimplemented!()
}
