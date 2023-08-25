use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Result, Error, Data, Fields, DeriveInput, FnArg, ItemFn};

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

#[proc_macro_attribute]
pub fn time_consuming(_: TokenStream, item: TokenStream) -> TokenStream {
    let decoratee = parse_macro_input!(item as ItemFn);
    let vis = &decoratee.vis;
    let ident = &decoratee.sig.ident;
    let block = &decoratee.block;
    let inputs = &decoratee.sig.inputs;
    let output = &decoratee.sig.output;
    let arguments: Vec<_> = inputs
        .iter()
        .map(|input| match input {
            FnArg::Typed(val) => &val.pat,
            _ => unreachable!()
        })
        .collect();
    let gen = quote! {
        #vis fn #ident(#inputs) #output {
            let begin = chrono::Local::now().timestamp_millis();
            fn func(#inputs) #output #block ;
            let r = func(#(#arguments), *);
            let end = chrono::Local::now().timestamp_millis();
            log::info!("{} spent time: {} ms", std::stringify!(#ident), end - begin);
            return r;
        }
    };
    gen.into()
}