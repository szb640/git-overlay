{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/0e251e24a4f24e036a084b6b4b2d2491af4167f4";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }: let
    targetSystems = [
      "x86_64-linux"
      "aarch64-linux"
    ];
  in flake-utils.lib.eachSystem targetSystems (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          (import rust-overlay)
        ];
      };
      cargoConfig = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    in {
      packages = {
        git-overlay = pkgs.rustPlatform.buildRustPackage {
          pname = cargoConfig.package.name;
          version = cargoConfig.package.version;
          
          src = ./.;

          nativeBuildInputs = [ pkgs.git ];

          cargoHash = "sha256-l4icYo3ZPRbV/wisu5y67FVqQ0RxlctiNqwWP2xgzTg=";

          meta = {
            description = cargoConfig.package.description;
            mainProgram = "git-overlay";
            maintainers = [{ name = "szb640"; email = "szb640@gmail.com"; }];
          };
        };
      };

      overlays.default = final: prev: {
        git-overlay = self.packages.${final.stdenv.hostPlatform.system}.git-overlay;
      };

      devShells = {
        default = pkgs.mkShell {
          packages = with pkgs; [
            rustc
            cargo
            rustfmt
            clippy
            zip
            jq
            yq
          ];
        };

        crossCompile = let
          rustTargets = [
            "x86_64-pc-windows-gnu"
            "x86_64-unknown-linux-musl"
            "aarch64-unknown-linux-musl"
          ];
        in pkgs.mkShell {
          CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS =
            "-L native=${pkgs.pkgsCross.mingwW64.windows.pthreads}/lib";
          packages = [
            (pkgs.rust-bin.stable.latest.default.override { targets = rustTargets; })
            pkgs.pkgsCross.mingwW64.stdenv.cc
            pkgs.pkgsCross.aarch64-multiplatform-musl.stdenv.cc
            pkgs.pkgsMusl.stdenv.cc
            pkgs.dpkg
          ];
        };
      };
    }
  );
}
