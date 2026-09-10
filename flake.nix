{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/0e251e24a4f24e036a084b6b4b2d2491af4167f4";
  };

  outputs = { self, nixpkgs }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
    cargoConfig = builtins.fromTOML (builtins.readFile ./Cargo.toml);

    # Builds the package with the given rustPlatform (i.e. cross-compilation
    # target), reusing the same Cargo.lock/vendored-dependency hash.
    mkPackage = rustPlatform: pkgsFor: rustPlatform.buildRustPackage {
      pname = cargoConfig.package.name;
      version = cargoConfig.package.version;

      src = ./.;

      # E2E tests (`tests/`) invoke the real `git` CLI to create test repos.
      nativeBuildInputs = [ pkgsFor.git ];

      cargoHash = "sha256-zHrbESsao05xeCCH+UUTr7QjPoq+9M8bnEoUF1bBSh0=";

      meta = {
        description = "Software for overlaying personal files onto a git repository";
        mainProgram = "git-overlay";
        maintainers = [{ name = "szb640"; }];
      };
    };
  in {
    packages.${system} = {
      git-overlay = mkPackage pkgs.rustPlatform pkgs;

      # Windows (mingw-w64) cross-compiled binary.
      git-overlay-windows = mkPackage
        pkgs.pkgsCross.mingwW64.rustPlatform
        pkgs.pkgsCross.mingwW64;
    };

    overlays.default = final: prev: {
      git-overlay = self.packages.${final.stdenv.hostPlatform.system}.git-overlay;
    };

    devShells.${system}.default = pkgs.mkShell {
      packages = with pkgs;[
        rustc
        cargo
        rustfmt
        clippy
        zip
      ];
    };
  };
}
