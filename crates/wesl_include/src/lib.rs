use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::path::PathBuf;
use syn::{
	LitStr, Token,
	parse::{Parse, ParseStream},
	parse_macro_input,
};

/// `include_wesl!("../../../assets/shaders", "shader_module")` -> &'static str of the WGSL
/// generated from wesl code
#[proc_macro]
pub fn include_wesl(input: TokenStream) -> TokenStream {
	let input2: TokenStream2 = input.clone().into();
	let args = parse_macro_input!(input as Args);
	let shader_dir =
		PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join(args.dir.value());

	let compiled = match sphynx_wesl::compile_module(&shader_dir, &args.module.value()) {
		Ok(compiled) => compiled,
		Err(e) => {
			return syn::Error::new_spanned(
				&input2,
				format!(
					"WESL compile error in {}: {e}",
					shader_dir.to_string_lossy()
				),
			)
			.to_compile_error()
			.into();
		}
	};

	let wgsl_code = compiled.wgsl_code;
	let linked_files = compiled
		.files
		.iter()
		.chain(compiled.config_files.iter())
		.map(|p| p.to_string_lossy().to_string())
		.collect::<Vec<_>>();

	quote! {
		{
			// rebuild tracking: depend on every shader file via include_bytes!
			// (stable-compatible alternative to nightly `tracked_path`)
			#( const _: &[u8] = include_bytes!(#linked_files); )*
			#wgsl_code
		}
	}
	.into()
}

struct Args {
	dir: LitStr,
	module: LitStr,
}

impl Parse for Args {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let dir = input.parse()?;
		input.parse::<Token![,]>()?;
		let module = input.parse()?;
		let _ = input.parse::<Option<Token![,]>>()?;
		Ok(Args { dir, module })
	}
}
