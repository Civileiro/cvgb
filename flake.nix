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

        rpathLibs = with pkgs; [
          xorg.libX11
          # xorg.libXcursor
          # xorg.libXrandr
          xorg.libXi
          # xorg.libXinerama
          # xorg.libXext
          # xorg.libXxf86vm
          libxkbcommon
          libGL
          wayland
        ];

      in {
        packages.default = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cvgb";
          version = "0.1.0";
          src = ./.;

          cargoLock = { lockFile = ./Cargo.lock; };

          nativeBuildInputs = [ ];
          buildInputs = rpathLibs;

          postFixup = with pkgs; ''
            patchelf --add-rpath ${
              lib.makeLibraryPath rpathLibs
            } $out/bin/${pname}
          '';
        };

        devShells.default = pkgs.mkShell {
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath rpathLibs;
          buildInputs = [ ] ++ rpathLibs;

        };
      });
}
