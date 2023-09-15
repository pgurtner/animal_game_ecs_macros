use proc_macro2::TokenStream;
use syn::ItemStruct;
use quote::quote;

pub fn to_bevy_component (input: ItemStruct) -> TokenStream {
    let expanded = quote! {
        #[derive(bevy_ecs::component::Component)]
        #input
    };

    return expanded;
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use proc_macro2::TokenStream;
    use quote::ToTokens;
    use super::*;
    #[test]
    fn test_me () {
        let ts = TokenStream::from_str(
            "struct StructName {\
            a: i32\
            }"
        ).unwrap();

        let struct_item = syn::parse2::<syn::ItemStruct>(ts).unwrap();

        let result = to_bevy_component(struct_item);
        let a = result.into_token_stream().to_string();
        println!("{}", a);
    }
}