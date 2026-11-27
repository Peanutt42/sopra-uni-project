{
  description = "SphynxEngine";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # on my personal Github Repo, just a flake so that nix can run codebook in checkPhase without needing networking access to download the dictionaries
    codebook-nix.url = "github:Peanutt42/codebook-nix";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      treefmt-nix,
      codebook-nix,
    }:
    let
      inherit (nixpkgs) lib;

      cargoToml = lib.importTOML ./Cargo.toml;
      pname = cargoToml.package.name;

      rustBinFor = pkgs: (pkgs.extend rust-overlay.overlays.default).rust-bin;
      # minimal rust toolchain for build and testing the package on host target and wasm target as well as verifying rust formatting
      toolchainFor =
        pkgs:
        (rustBinFor pkgs).selectLatestNightlyWith (
          toolchain:
          toolchain.minimal.override {
            extensions = [
              "clippy"
              "rustfmt"
            ];
            targets = [ "wasm32-unknown-unknown" ];
          }
        );
      # bigger rust toolchain for actual development (used in devShell)
      devToolchainFor =
        pkgs:
        (rustBinFor pkgs).selectLatestNightlyWith (
          toolchain:
          toolchain.default.override {
            extensions = [
              "clippy"
              "rustfmt"
              "rust-src"
              "rust-analyzer"
              "rustc-codegen-cranelift-preview"
            ];
            targets = [ "wasm32-unknown-unknown" ];
          }
        );
      runtimeLibsFor =
        pkgs: with pkgs; [
          wayland
          libxkbcommon
          vulkan-loader

          # X11 fallback
          libx11
          libxcursor
          libxrandr
          libxi
        ];
      depsFor = pkgs: {
        nativeBuildInputs = with pkgs; [
          pkg-config
          cargo-hack # for running clippy and testing on all feature combinations
          codebook # for typo checking
          tombi # for toml formatting and linting
        ];
        buildInputs = runtimeLibsFor pkgs;
      };
    in
    {
      overlays.default = final: _prev: {
        ${pname} =
          let
            toolchain = toolchainFor final;
            rustPlatform = final.makeRustPlatform {
              cargo = toolchain;
              rustc = toolchain;
            };
          in
          rustPlatform.buildRustPackage (
            {
              inherit pname;
              inherit (cargoToml.package) version;

              src = lib.fileset.toSource {
                root = ./.;
                fileset = lib.fileset.unions [
                  ./Cargo.toml
                  ./Cargo.lock
                  ./src
                  ./game
                  ./crates
                  ./assets
                ];
              };
              cargoLock.lockFile = ./Cargo.lock;
              strictDeps = true;

              postFixup = lib.optionalString final.stdenv.hostPlatform.isLinux ''
                patchelf --add-rpath ${lib.makeLibraryPath (runtimeLibsFor final)} $out/bin/${pname}
              '';

              checkPhase = ''
                runHook preCheck
                # run clippy and tests on all feature combinations for linux
                cargo hack --feature-powerset --all clippy
                cargo hack --feature-powerset --all test
                # run clippy on all feature combinations for web (wasm)
                cargo hack --feature-powerset --all --exclude-features hot-reload clippy --target wasm32-unknown-unknown
                runHook postCheck
              '';

              meta = {
                description = cargoToml.package.description or pname;
                license = lib.licenses.mit;
                mainProgram = pname;
              };
            }
            // depsFor final
          );
      };
    }
    // flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ self.overlays.default ];
        };
        treefmtEval = treefmt-nix.lib.evalModule pkgs {
          projectRootFile = "flake.nix";
          programs = {
            nixfmt.enable = true;
            rustfmt = {
              enable = true;
              package = toolchainFor pkgs;
            };
            just = {
              enable = true;
              indentation = "\t";
            };
            wgslfmt = {
              enable = true;
              tabs = true;
              includes = [
                "*.wgsl"
                "*.wesl"
              ];
            };
            # for *.toml
            taplo = {
              enable = true;
              excludes = [ ".direnv/**" ];
              settings.formatting = {
                indent_string = "\t";
                array_trailing_comma = false;
              };
            };
            mdformat.enable = true;
            # mainly for ./.gitlab-ci.yml
            yamlfmt.enable = true;
            yamllint.enable = true;
          };
        };
      in
      {
        packages = {
          default = pkgs.${pname};
          ${pname} = pkgs.${pname};
        };

        apps.default = flake-utils.lib.mkApp { drv = pkgs.${pname}; } // {
          meta = pkgs.${pname}.meta;
        };

        devShells.default = pkgs.mkShell {
          inherit (depsFor pkgs) nativeBuildInputs buildInputs;
          packages = with pkgs; [
            (devToolchainFor pkgs)
            clang # linker used by .cargo/fast_dev_config.toml
            mold # for faster debug builds
            parallel # for running the game and rebuilding game lib in parallel
            wgsl-analyzer # for wgsl-analyzer, but also for wgslfmt
            sccache # for caching rust compiler results, useful for git worktrees
            just
            cargo-hack
            trunk
            binaryen
          ];
          env.RUST_BACKTRACE = "1";
          shellHook = ''
            export LD_LIBRARY_PATH="${lib.makeLibraryPath (runtimeLibsFor pkgs)}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
            export RUSTC_WRAPPER="${pkgs.sccache}/bin/sccache"
          '';
        };

        formatter = treefmtEval.config.build.wrapper;

        checks.${pname} = pkgs.${pname};

        checks.formatting = treefmtEval.config.build.check self;

        checks.codebook = codebook-nix.lib.mkCodebookCheck {
          inherit pkgs;
          src = self;
        };
      }
    );
}
