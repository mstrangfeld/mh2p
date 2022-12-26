{
  description = "Build a cargo project without extra checks";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        craneLib = crane.lib.${system};
        mh2p = craneLib.buildPackage rec {
          src = craneLib.cleanCargoSource ./.;

          nativeBuildInputs = with pkgs; [
            pkg-config
            llvmPackages.bintools # To use lld linker
          ];

          buildInputs = with pkgs; [
            udev
            alsaLib
            vulkan-loader
            xlibsWrapper
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi # To use x11 feature
            libxkbcommon
            wayland # To use wayland feature
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };
      in
      {
        checks = {
          inherit mh2p;
        };

        packages.default = mh2p;

        apps.default = flake-utils.lib.mkApp {
          drv = mh2p;
        };

        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = with pkgs; [
            cargo
            rustc
            pkg-config
            llvmPackages.bintools # To use lld linker
          ];
          buildInputs = with pkgs; [
            udev
            alsaLib
            vulkan-loader
            xlibsWrapper
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi # To use x11 feature
            libxkbcommon
            wayland # To use wayland feature
          ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };
      });
}
