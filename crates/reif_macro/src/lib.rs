mod create_process;

use proc_macro::TokenStream;
use create_process::_create_process;

#[proc_macro]
pub fn create_process(tokens: TokenStream) -> TokenStream {
    _create_process(tokens.into()).into()
}
