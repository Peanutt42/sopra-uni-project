# List all commands available
default:
    just --list

# Runs clippy on the entire project using the fast dev cargo configuration (mold linker + cranelift codegen-backend)
check-dev *extra_cargo_args:
	cargo clippy --all --features hot-reload --config ./.cargo/fast_dev_config.toml {{extra_cargo_args}}

# Runs clippy on the entire project with every possible feature combination using the fast dev cargo configuration (mold linker + cranelift codegen-backend). requires cargo-hack
check-dev-all *extra_cargo_args:
	cargo hack --feature-powerset --all clippy --config ./.cargo/fast_dev_config.toml {{extra_cargo_args}}

# Runs the game using the fast dev cargo configuration (mold linker + cranelift codegen-backend)
run-dev extra_cargo_args="":
	cargo run --features hot-reload --config ./.cargo/fast_dev_config.toml {{extra_cargo_args}}

# Runs the game/engine with hot-reload enabled, will rebuild the game lib when change is detected
run-dev-hotreloadable *extra_cargo_args:
	@# the game engine loads the libgame.so at start, but cargo does not compile the libgame.so if not explicitely requested
	cargo build -p game --config ./.cargo/fast_dev_config.toml --features hot-reload --color always {{extra_cargo_args}}

	@# runs both the game and the file watcher to rebuild libgame.so for hot-reloading. cargo watch with --postpone simply only starts recompiling after a file changed,
	@# since we already compiled the game code right before. CLICOLOR_FORCE=1 is for the game for colored logging output, cargo uses "--color always"
	parallel --line-buffer --halt now,done=1 ::: \
		"CLICOLOR_FORCE=1 cargo run --features hot-reload --config ./.cargo/fast_dev_config.toml --color always {{extra_cargo_args}}" \
		"cargo watch --postpone -w game -x 'build -p game --features hot-reload --config ./.cargo/fast_dev_config.toml --color always {{extra_cargo_args}}'"

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
