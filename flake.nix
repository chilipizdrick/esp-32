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
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            (alias "pull-container" ''sudo docker pull espressif/idf-rust:all_latest'')
            (alias "build" ''sudo docker rm esp-32; sudo docker run -v "./.:/project/:rw" --name "esp-32" espressif/idf-rust:all_latest sh -c "chmod +x ./export-esp.sh && ./export-esp.sh && cd /project && cargo build"'')
            (alias "flash" ''espflash flash ./target/xtensa-esp32-espidf/debug/test'')
          ];
          buildInputs = with pkgs; [
            cargo
            rustup
            cargo-generate
            cargo-espflash
            llvm
            espup

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

            docker
            docker-compose
          ];
          shellHook = ''
            export PATH="/home/alex/.rustup/toolchains/esp/xtensa-esp-elf/esp-14.2.0_20240906/xtensa-esp-elf/bin:$PATH"
            export LIBCLANG_PATH="/home/alex/.rustup/toolchains/esp/xtensa-esp32-elf-clang/esp-18.1.2_20240912/esp-clang/lib"
            # export LIBCLANG_PATH="${pkgs.llvmPackages_18.libclang.lib}/lib"
          '';
        };
      };
      imports = [];
      flake = {};
    };
}
