{
  description = "Dao44 Rust layout generator, live preview, and native ZMK toolchain";

  inputs.nixpkgs.url = "https://github.com/NixOS/nixpkgs/archive/refs/heads/nixos-25.05.tar.gz";

  outputs = { self, nixpkgs }:
    let
      systems = [ "aarch64-darwin" "x86_64-darwin" "aarch64-linux" "x86_64-linux" ];
      forAllSystems = function:
        nixpkgs.lib.genAttrs systems (system: function nixpkgs.legacyPackages.${system});
    in {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            bun
            cargo
            clippy
            cmake
            dtc
            gcc-arm-embedded
            gperf
            just
            ninja
            rustc
            rustfmt
            uv
          ];
          GNUARMEMB_TOOLCHAIN_PATH = "${pkgs.gcc-arm-embedded}";
          ZEPHYR_TOOLCHAIN_VARIANT = "gnuarmemb";
        };
      });
    };
}
