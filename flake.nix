{
  description = "PROMETHEUS — Hutter-style enwik9 compressor (Rust + judge harness)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system);
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          # Nightly + rust-src for -Z build-std / panic = "immediate-abort".
          rust = fenix.packages.${system}.complete.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rustc"
            "rustfmt"
          ];
          # judge.sh expects target/release/*, but build-std needs --target which
          # places bins under target/<triple>/release/. Mirror after each cargo.
          cargoWrap = pkgs.writeShellScriptBin "cargo" ''
            set -uo pipefail
            "${rust}/bin/cargo" "$@"
            status=$?
            triple_dir="target/x86_64-unknown-linux-gnu/release"
            if [[ "$status" -eq 0 && -d "$triple_dir" ]]; then
              mkdir -p target/release
              for b in prometheus-compress prometheus-decompress; do
                if [[ -f "$triple_dir/$b" ]]; then
                  cp -f "$triple_dir/$b" "target/release/$b"
                fi
              done
            fi
            exit "$status"
          '';
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargoWrap
              rust
              binutils          # strip
              coreutils
              diffutils        # cmp
              time             # /usr/bin/time equivalent: command time
              gawk
              gnused
              gnugrep
              findutils
              bash
              git
            ];
            RUST_SRC_PATH = "${fenix.packages.${system}.complete.rust-src}/lib/rustlib/src/rust/library";
            # Env RUSTFLAGS replaces config rustflags; keep size + CPU flags together.
            RUSTFLAGS = "-Zunstable-options -Cpanic=immediate-abort -Zlocation-detail=none -Cforce-unwind-tables=no -Ctarget-cpu=native";
            shellHook = ''
              # Prefer wrapped cargo (copies bins to target/release for judge.sh).
              export PATH="${cargoWrap}/bin:$PATH"
              # mkShell may inject: NIX_LDFLAGS=-rpath $PWD/outputs/out/lib
              # Unquoted spaces in $PWD break the linker (ld sees multiple args).
              if [[ "''${NIX_LDFLAGS:-}" == *" "* ]]; then
                export NIX_LDFLAGS=""
              fi
              echo "PROMETHEUS nix shell: $(rustc --version) | $(cargo --version)"
            '';
          };
        });
    };
}
