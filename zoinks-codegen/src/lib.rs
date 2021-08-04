use proc_macro::TokenStream;
use syn::LitStr;
use syn::parse_macro_input;
use syn::parse::{Parse, ParseStream, Result};

use zoinks_lib::{Schema, genimpl};

struct Args {
    input_fn: String,
}
impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let input_fn = input.parse::<LitStr>()?.value();
        Ok(Self { input_fn })
    }
}

#[proc_macro]
pub fn schema2struct(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Args);

    let schema_string = std::fs::read_to_string(args.input_fn).unwrap();
    let schema : Schema = serde_json::from_str(&schema_string).unwrap();

    let imp = genimpl(&schema);
    imp.into()
}
