{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/0e251e24a4f24e036a084b6b4b2d2491af4167f4";
  };

  outputs = { self, nixpkgs }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
    cargoConfig = builtins.fromTOML (builtins.readFile ./Cargo.toml);
  in {
    packages.${system}.git-overlay = pkgs.rustPlatform.buildRustPackage {
      pname = cargoConfig.package.name;
      version = cargoConfig.package.version;

      src = ./.;

      # E2E tests (`tests/`) invoke the real `git` CLI to create test repos.
      nativeBuildInputs = [ pkgs.git ];

      cargoHash = "sha256-L3cgFz5fWTDyWjxA3Tr9XWIQ5h0BieCX4Pw4FOjubEw=";

      meta = {
        description = "Software for overlaying personal files onto a git repository";
        mainProgram = "git-overlay";
        maintainers = [{ name = "szb640"; }];
      };
    };
    
    overlays.default = final: prev: {
      flake-helper = self.packages.${final.system}.flake-helper;
    };
    
    devShells.${system}.default = pkgs.mkShell {
      packages = with pkgs;[
        rustc
        cargo
        rustfmt
        clippy
      ];
    };
  };
}
