# List all commands available
default:
    just --list

# Runs clippy on the entire project using the fast dev cargo configuration (mold linker + cranelift codegen-backend)
check-dev *extra_cargo_args:
	cargo clippy --all --features egui --features hot-reload --config ./.cargo/fast_dev_config.toml {{extra_cargo_args}}

# Runs clippy on the entire project with every possible feature combination using the fast dev cargo configuration (mold linker + cranelift codegen-backend). requires cargo-hack
check-dev-all *extra_cargo_args:
	cargo hack --feature-powerset --all clippy --config ./.cargo/fast_dev_config.toml {{extra_cargo_args}}

# Runs the game using the fast dev cargo configuration (mold linker + cranelift codegen-backend)
run-dev extra_cargo_args="":
	cargo run --features egui --features hot-reload --config ./.cargo/fast_dev_config.toml {{extra_cargo_args}}

# Runs the game/engine with hot-reload enabled, will rebuild the game lib when change is detected
run-dev-hotreloadable cargo_args="" engine_only_cargo_args="":
    parallel --line-buffer --halt now,done=1 ::: \
      "CLICOLOR_FORCE=1 cargo run --config ./.cargo/fast_dev_config.toml --features hot-reload --color always {{cargo_args}} {{engine_only_cargo_args}}" \
      "cargo watch -w game -x 'build --config ./.cargo/fast_dev_config.toml -p game --features hot-reload --color always {{cargo_args}}'"

# Build the web crate bundle into web/dist
build-web *extra_args:
    trunk build ./crates/web/index.html {{extra_args}}

# Serve web on http://localhost:8080
serve-web *extra_args:
    trunk serve ./crates/web/index.html {{extra_args}}

# Run clippy on the entire project using the wasm target
check-web *extra_args:
    cargo clippy --all --target wasm32-unknown-unknown {{extra_args}}

# Run clippy on the entire project with every possible feature combination (execpt for hot-reload which is unsupported) using the wasm target. requires cargo-hack
check-web-all *extra_args:
	cargo hack --feature-powerset --all --exclude-features hot-reload clippy --target wasm32-unknown-unknown {{extra_args}}
