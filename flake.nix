{
  description = "Rust esp-32 flake";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };
  outputs = inputs:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux"];
      perSystem = {pkgs, ...}: let
        alias = pkgs.writeShellScriptBin;
      in {
        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = [
            (alias "pull-image" ''sudo podman pull docker.io/espressif/idf-rust:esp32_latest'')
            (alias "build" ''sudo podman rm esp; sudo podman run -v "./.:/project/:rw" -v "/home/alex/.cargo/registry/:/home/esp/.cargo/registry/:rw" --name "esp" -it espressif/idf-rust:esp32_latest sh -c "chmod +x ./export-esp.sh && ./export-esp.sh && cd /project && cargo build"'')
            (alias "build-release" ''sudo podman rm esp; sudo podman run -v "./.:/project/:rw" -v "/home/alex/.cargo/registry/:/home/esp/.cargo/registry/:rw" --name "esp" -it espressif/idf-rust:esp32_latest sh -c "chmod +x ./export-esp.sh && ./export-esp.sh && cd /project && cargo build --release"'')
            (alias "flash" ''espflash flash ./target/xtensa-esp32-espidf/debug/test'')
          ];
          buildInputs = with pkgs; [
            rustup
            cargo-generate
            cargo-espflash
            ldproxy
            llvm

            wget
            flex
            bison
            gperf
            python3
            python312Packages.pip
            cmake
            ninja
            ccache
            libffi
            openssl
            dfu-util
            libusb1
            zlib
          ];

          shellHook =
            # sh
            ''
              . /home/alex/export-esp.sh
              export PATH="/home/alex/.cargo/bin:$PATH"
              export LD_LIBRARY_PATH="${pkgs.stdenv.cc.cc.lib.outPath}/lib:$LD_LIBRARY_PATH"
              export LD_LIBRARY_PATH="$LIBCLANG_PATH:$LD_LIBRARY_PATH"
              export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath buildInputs}:$LD_LIBRARY_PATH"
            '';
        };
      };
      imports = [];
      flake = {};
    };
}
