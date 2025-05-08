{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forEachSupportedSystem = f: inputs.nixpkgs.lib.genAttrs supportedSystems (system: f {
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            inputs.rust-overlay.overlays.default
            inputs.self.overlays.default
          ];
        };
      });
    in
    {
      overlays.default = final: prev: {
        rustToolchain =
          let
            rust = prev.rust-bin;
          in
          if builtins.pathExists ./rust-toolchain.toml then
            rust.fromRustupToolchainFile ./rust-toolchain.toml
          else if builtins.pathExists ./rust-toolchain then
            rust.fromRustupToolchainFile ./rust-toolchain
          else
            rust.stable.latest.default.override {
              extensions = [ "rust-src" "rustfmt" ];
            };
      };

      devShells = forEachSupportedSystem ({ pkgs }: let
        myInputs = with pkgs; [
          rustToolchain
          openssl
          pkg-config
          cargo-deny
          cargo-edit
          cargo-watch
          rust-analyzer

          # necessary for building wgpu in 3rd party packages (in most cases)
          libxkbcommon
          wayland xorg.libX11 xorg.libXcursor xorg.libXrandr xorg.libXi
          alsa-lib
          fontconfig freetype
          shaderc directx-shader-compiler
          pkg-config cmake
          mold # could use any linker, needed for rustix (but mold is fast)

          libGL
          vulkan-headers vulkan-loader
          vulkan-tools vulkan-tools-lunarg
          vulkan-extension-layer
          vulkan-validation-layers # don't need them *strictly* but immensely helpful

          # necessary for developing (all of) wgpu itself
          cargo-nextest cargo-fuzz

          # nice for developing wgpu itself
          typos 

          yq # for tomlq below

          # nice tools
          gdb rr
          evcxr
          valgrind
          renderdoc
        ];

        libPaths = builtins.toString (pkgs.lib.makeLibraryPath myInputs);

      in {
        default = pkgs.mkShell {
          buildInputs = myInputs;

          LD_LIBRARY_PATH = libPaths;

          env = {
            RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
          };

          shellHook = ''
            export RUSTC_VERSION="$(tomlq -r .toolchain.channel rust-toolchain.toml)"
            export PATH="$PATH:''${CARGO_HOME:-~/.cargo}/bin"
            export PATH="$PATH:''${RUSTUP_HOME:-~/.rustup/toolchains/$RUSTC_VERSION-x86_64-unknown-linux/bin}"
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${libPaths}";

            rustup default $RUSTC_VERSION
            rustup component add rust-src rust-analyzer

            alias ga='git add'
            alias gs='git status'
            alias gc='git commit'
            alias gp='git push'
          '';
        };
      });
    };
}
