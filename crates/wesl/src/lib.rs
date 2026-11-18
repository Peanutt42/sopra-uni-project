use std::path::{Path, PathBuf};
use std::str::FromStr;
use wesl::CompileOptions;
use wesl::resolver::{FileResolver, Router, StandardResolver};
use wesl::syntax::{ModulePath, PathOrigin};
use wesl::toml_cfg::{DependenciesConfig, DependencySpec, WeslToml};

/// wesl shader code is compiled to wgsl code, tracking the referenced files for automatic hot-reloading
#[derive(Debug, Clone)]
pub struct CompiledShader {
	/// wgsl code generated from wesl code
	pub wgsl_code: String,
	/// all referenced shader files by this shader
	pub files: Vec<PathBuf>,
	/// all `wesl.toml` filepaths referenced
	pub config_files: Vec<PathBuf>,
	/// package roots of all referenced modules/dependencies
	pub package_dirs: Vec<PathBuf>,
}

/// `module_name` must be relative to the wesl package described in the `shader_package_root_dir`'s
/// wesl.toml (gets "package::" added as a prefix)
pub fn compile_module(
	shader_package_root_dir: &Path,
	module_name: &str,
) -> Result<CompiledShader, String> {
	let shader_module_name_str = format!("package::{module_name}");

	let shader_module_name =
		ModulePath::from_str(&shader_module_name_str).map_err(|e| e.to_string())?;

	let (shader_config, _shader_package_root) =
		load_wesl_package(shader_package_root_dir).map_err(|e| e.to_string())?;

	let mut package_dirs = vec![shader_package_root_dir.to_path_buf()];
	let mut config_files = vec![shader_package_root_dir.join("wesl.toml")];

	let mut router = Router::new();
	mount_deps(
		&mut router,
		None,
		shader_package_root_dir,
		&shader_config,
		&mut package_dirs,
		&mut config_files,
	)?;
	// local `package::` files + the virtual `constants` package
	router.mount_fallback_resolver(StandardResolver::new(shader_package_root_dir));

	let compiler_options = CompileOptions {
		// visibility is a very early and not stable feature of wesl
		// for example, my neovim lsp does not know about the public keyword etc.
		// TODO: turn on once the ecosystem has stabilized it enough
		visibility: false,
		..Default::default()
	};

	let compiler = wesl::Compiler::new_with_resolver(compiler_options, router);

	let compiler_result = compiler
		.compile_module(&shader_module_name)
		.map_err(|e| e.to_string())?;

	let linked_wesl = compiler_result.syntax.to_string();

	let Some(sourcemap) = compiler_result.sourcemap else {
		return Err("no sourcemap was included by the wesl compiler/linker".to_string());
	};

	// collect every source file that was used by the linker. This includes both
	// the root package and dependency packages. It is used for build script /
	// proc macro tracking and for the runtime file watcher.
	let mut files = compiler_result
		.used_items
		.iter()
		.filter_map(|(module_path, _)| sourcemap.file(module_path)?.path.clone())
		.collect::<Vec<_>>();
	dedup(&mut files);
	dedup(&mut package_dirs);
	dedup(&mut config_files);

	Ok(CompiledShader {
		wgsl_code: linked_wesl,
		files,
		config_files,
		package_dirs,
	})
}

fn dedup(paths: &mut Vec<PathBuf>) {
	paths.sort();
	paths.dedup();
}

/// Returns (config, absolute_shader_root)
fn load_wesl_package(dir: &Path) -> Result<(WeslToml, PathBuf), wesl::error::TomlError> {
	let toml_path = dir.join("wesl.toml");
	let config = WeslToml::from_file(toml_path)?;
	let absolute_shader_root = dir.join(&config.package.root);
	Ok((config, absolute_shader_root))
}

/// Mount every `path` dependency of `config` (recursively) onto the router.
fn mount_deps(
	router: &mut Router,
	prefix: Option<&str>,
	shader_package_dir: &Path,
	config: &WeslToml,
	package_dirs: &mut Vec<PathBuf>,
	config_files: &mut Vec<PathBuf>,
) -> Result<(), String> {
	let deps = match &config.dependencies {
		DependenciesConfig::None => return Ok(()),
		DependenciesConfig::Auto => {
			return Err("`dependencies = \"auto\"` is not supported by this macro".into());
		}
		DependenciesConfig::Manual(m) => m,
	};

	for (name, spec) in deps {
		let DependencySpec::Path(rel) = spec else {
			return Err(format!(
				"dependency `{name}`: only `path = \"...\"` dependencies are supported by this macro"
			));
		};
		let dep_dir = shader_package_dir.join(rel);
		let (dep_cfg, dep_root) = load_wesl_package(&dep_dir).map_err(|e| e.to_string())?;

		package_dirs.push(dep_dir.clone());
		config_files.push(dep_dir.join("wesl.toml"));

		// sub-dependencies are named `parent/child` by wesl's import resolution
		let full = match prefix {
			Some(p) => format!("{p}/{name}"),
			None => name.clone(),
		};

		router.mount_resolver(
			ModulePath::new(PathOrigin::Package(full.clone()), vec![]),
			FileResolver::new(&dep_root),
		);

		mount_deps(
			router,
			Some(&full),
			&dep_dir,
			&dep_cfg,
			package_dirs,
			config_files,
		)?;
	}

	Ok(())
}
