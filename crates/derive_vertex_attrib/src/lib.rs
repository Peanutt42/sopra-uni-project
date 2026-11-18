use proc_macro::TokenStream;

mod instance;
mod vertex;

/// derive proc macro to auto-implement the Vertex trait
#[proc_macro_derive(Vertex)]
pub fn vertex_derive(input: TokenStream) -> TokenStream {
	vertex::vertex_derive_impl(input)
}

/// derive proc macro to auto-implement the InstanceData trait
#[proc_macro_derive(InstanceData)]
pub fn instance_data_derive(input: TokenStream) -> TokenStream {
	instance::instance_data_derive_impl(input)
}
