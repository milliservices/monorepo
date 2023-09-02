{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    # ghc-wasm-meta.url = "https://gitlab.haskell.org/ghc/ghc-wasm-meta";
    # javy-cli-source = {
    #   url = "https://github.com/bytecodealliance/javy/releases/download/v1.1.2/javy-x86_64-linux-v1.1.2.gz";
    #   flake = false;
    # };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    let
      shell = { pkgs }:
        with pkgs;
        let
          rust = rust-bin.nightly.latest.default.override {
            extensions = [ "rust-src" ];
            targets = [
              "wasm32-wasi"
              "x86_64-unknown-linux-gnu"
            ];
          };
          # javy-cli-package = pkgs.stdenv.mkDerivation {
          #   name = "javy-cli";
          #   src = "${javy-cli-source}";
          #   phases = [ "unpackPhase" "installPhase" ];
          #   buildInputs = [ gzip autoPatchelfHook stdenv.cc.cc.lib ];
          #   unpackPhase = ''
          #     mkdir -p $out/bin;
          #     cp $src $out/bin/javy-cli.gz;
          #     cd $out/bin;
          #     gunzip javy-cli.gz;
          #     chmod +x javy-cli && chmod 777 javy-cli;
          #     autoPatchelf javy-cli;
          #   '';
          #   installPhase = '' '';
          # };
        in
        mkShell rec {
          buildInputs = [
            rust
            rust-analyzer-unwrapped
            just
            cargo-watch
            wabt
            assemblyscript
          ];
          nativeBuildInputs = [ clang ];

          # RUST_BACKTRACE = 1;
          LIBCLANG_PATH = "${libclang.lib}/lib";
          LD_LIBRARY_PATH = "${lib.makeLibraryPath (buildInputs ++ nativeBuildInputs)}";
        };
    in
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        in
        {
          devShells.default = shell { inherit pkgs; };
          packages.default = pkgs.rustPlatform.buildRustPackage {
            inherit (cargoToml.package) name version;
            src = ./.;
            cargoLock = { lockFile = ./Cargo.lock; };
          };
        });
}
