use syn::{parse_macro_input, ItemStruct, ItemFn};
use proc_macro::TokenStream;

mod bevy_system;
mod bevy_component;

/// proc_macro adaptor for bevy_ecs components.
/// Behaves like a no op. It just places a derive statement for bevy's component macro above the struct.
#[proc_macro_attribute]
pub fn to_bevy_component(
    _args: TokenStream,
    input: TokenStream,
) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);

    let result = bevy_component::to_bevy_component(input);

    proc_macro::TokenStream::from(result)
}

/// proc_macro adapter for bevy_ecs systems
///
/// turns
/// `
/// #[system]
/// [pub] fn fn_name (a: A, b: B, etc: Etc) {
///     ...
/// }
/// `
/// into
/// `
/// [pub] fn fn_name (query: Query<(A, B, Etc)>) {
///     for (a, b, etc) in &[mut ]query {
///         ...
///     }
/// }
///`
///
/// currently the loop always borrows query.
#[proc_macro_attribute]
pub fn to_bevy_system(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    let resulting_ts = bevy_system::to_bevy_system_fn(input).unwrap_or_else(|ts| ts);

    proc_macro::TokenStream::from(resulting_ts)
}