# Sopra versenken - Ship happens (hennig-peter)

All third-party licenses are listed in [assets/LICENSES.md](./assets/LICENSES.md).

## SphynxEngine

My own custom game engine written in rust using `wgpu` Graphics API and a ECS (Entity-Component-System) Architecture.

### How to compile / what dependencies and tools are needed

#### Using Nix (install nix the package manager at <https://nixos.org/download/>)

The easiest and fastest way to compiling on linux is using the provided `flake.nix`:
```bash
nix build .
```

For development, the `flake.nix` provides a devShell:
```bash
nix develop # or run direnv allow if you have direnv installed
```

#### Manual compiling

Dependencies for compiling:
- cargo and rustc: 1.97.1 or newer, latest stable or nightly works  (nightly is required for faster dev builds using `./.cargo/fast_dev_config.toml`, i test on `rustc 1.101.0-nightly`)
- pkgconfig

Runtime dependencies:
- wayland
- libxkbcommon
- vulkan-loader
- libx11
- libxcursor
- libxrandr
- libxi

Optional dev dependencies (not at all needed if you dont intend on using them):
- mold
- parallel
- wgsl-analyzer
- sccache

### Available commands for fast development (justfile)

```bash
$ just --list
Available recipes:
    build-web *extra_args           # Build the web crate bundle into web/dist
    check-dev *extra_cargo_args     # Runs clippy on the entire project using the fast dev cargo configuration (mold linker + cranelift codegen-backend)
    check-dev-all *extra_cargo_args # Runs clippy on the entire project with every possible feature combination using the fast dev cargo configuration (mold linker + cranelift codegen-backend). requires cargo-hack
    check-web *extra_args           # Run clippy on the entire project using the wasm target
    check-web-all *extra_args       # Run clippy on the entire project with every possible feature combination (execpt for hot-reload which is unsupported) using the wasm target. requires cargo-hack
    default                         # List all commands available
    run-dev extra_cargo_args=""     # Runs the game using the fast dev cargo configuration (mold linker + cranelift codegen-backend)
    # Runs the game/engine with hot-reload enabled, will rebuild the game lib when change is detected
    run-dev-hotreloadable cargo_args="" engine_only_cargo_args=""
    serve-web *extra_args           # Serve web on http://localhost:8080
```

> [!NOTE]
> When running the game manually without the justfile with the `hot-reload` feature enabled, make sure you have ran `cargo b -p game --features hot-reload` before
> (the `libgame.so` needed by the `hot-reload` feature is only build when explicitly requesting it. the justfile commands to automatically request it)

