use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Result, Error, Data, Fields, DeriveInput};

type StructFields = syn::punctuated::Punctuated<syn::Field, syn::Token!(,)>;

fn get_fields_from_derive_input(input: &DeriveInput) -> Result<&StructFields> {
    if let Data::Struct(
        syn::DataStruct {
            fields: Fields::Named(syn::FieldsNamed { ref named, .. }),
            ..
        }) = input.data {
        return Ok(named);
    }
    Err(Error::new_spanned(input, "Must define on a Struct, not Enum".to_string()))
}

fn expand_derive_updater(input: &DeriveInput) -> Result<proc_macro2::TokenStream> {
    let fields = get_fields_from_derive_input(input)?;
    let struct_name_ident = &input.ident;
    let mut stream = proc_macro2::TokenStream::new();
    for field in fields {
        let field_name_ident = field.ident.as_ref().unwrap();
        stream.extend(quote!(
            if let Some(x) = rhs.#field_name_ident {
                self.#field_name_ident = Some(x);
            }
        ));
    }
    let ret = quote!(
        impl #struct_name_ident {
            fn update(&mut self, rhs: Self) {
                #stream
            }
        }
    );
    Ok(ret)
}

#[proc_macro_derive(Updater)]
pub fn derive_updater(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_derive_updater(&input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}