{
  description = "Entorno de desarrollo Rust ultra-rápido";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Usamos la última herramienta estable con componentes necesarios
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
          ];
        };

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
            openssl
            mold
            sccache
          ];

          # Optimizaciones clave en el entorno
          shellHook = ''
            # Activar sccache como wrapper del compilador
            export RUSTC_WRAPPER=${pkgs.sccache}/bin/sccache

            # Configurar mold para linkear más rápido (linux)
            export RUSTFLAGS="-C link-arg=-fuse-ld=mold $RUSTFLAGS"

            # Directorio de caché local para sccache (opcional, por defecto es ~/.cache/sccache)
            # export SCCACHE_DIR=$PWD/.sccache

            echo "🦀 Rust Environment Ready"
            echo "🚀 sccache & mold enabled"
          '';
        };
      }
    );
}
