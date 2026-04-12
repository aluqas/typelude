use proc_macro2::Span;
use syn::{
    Error, LitStr, Token,
    parse::{Parse, ParseStream},
};

pub struct WasmWatInput {
    pub module: LitStr,
}

impl Parse for WasmWatInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut module = None;

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "module" => {
                    if module.is_some() {
                        return Err(Error::new(key.span(), "`module` specified more than once"));
                    }
                    module = Some(input.parse()?);
                },
                _ => {
                    return Err(Error::new(key.span(), "expected only the `module` field"));
                },
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            module: module
                .ok_or_else(|| Error::new(Span::call_site(), "missing `module` field"))?,
        })
    }
}
