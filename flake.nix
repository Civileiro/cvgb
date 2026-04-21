{
  description = "CVGB Game Boy emulator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        nativeLibs = with pkgs; [ pkg-config ];
        rpathLibs = with pkgs; [
          libX11
          # libXcursor
          libXrandr
          libXi
          # libXinerama
          # libXext
          # libXxf86vm
          libxkbcommon
          libGL
          wayland
          wayland-protocols
          mesa
          vulkan-loader
          alsa-lib
          jack2
          pipewire
        ];

      in {
        packages.default = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cvgb";
          version = "0.1.0";
          src = ./.;

          cargoLock = { lockFile = ./Cargo.lock; };

          nativeBuildInputs = nativeLibs;
          buildInputs = rpathLibs;

          postFixup = with pkgs; ''
            patchelf --add-rpath ${
              lib.makeLibraryPath rpathLibs
            } $out/bin/${pname}
          '';
        };

        devShells.default = pkgs.mkShell {
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath rpathLibs;
          buildInputs = nativeLibs ++ rpathLibs;
        };
      });
}
