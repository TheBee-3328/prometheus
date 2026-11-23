{
  description = "PROMETHEUS — Hutter-style enwik9 compressor (Rust + judge harness)";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            rustc
            cargo
            rustfmt
            clippy
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
          RUSTFLAGS = "-C target-cpu=native";
          shellHook = ''
            echo "PROMETHEUS nix shell: $(rustc --version) | $(cargo --version)"
          '';
        };
      });
    };
}
