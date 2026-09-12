{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/0e251e24a4f24e036a084b6b4b2d2491af4167f4";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }: let
    currentSystem = "x86_64-linux";
    targetSystems = [
      "x86_64-linux"
      "x86_64-windows"
      "armv7l-linux"
      "aarch64-linux"
    ];
  in flake-utils.lib.eachSystem targetSystems (system:
    let
      pkgs = if system == "x86_64-windows" && currentSystem == "x86_64-linux" then
        nixpkgs.legacyPackages.${currentSystem}.pkgsCross.mingwW64
      else
        import nixpkgs {
          localSystem = currentSystem;
          crossSystem = system;
        };
      cargoConfig = builtins.fromTOML (builtins.readFile ./Cargo.toml);

      mkDebianPackage = import ./mkDebianPackage.nix;
    in {
      packages = {
        git-overlay = pkgs.rustPlatform.buildRustPackage {
          pname = cargoConfig.package.name;
          version = cargoConfig.package.version;
          
          src = ./.;

          nativeBuildInputs = [ pkgs.git ];

          cargoHash = "sha256-zHrbESsao05xeCCH+UUTr7QjPoq+9M8bnEoUF1bBSh0=";

          meta = {
            description = "Software for overlaying personal files onto a git repository";
            mainProgram = "git-overlay";
            maintainers = [{ name = "szb640"; email = "szb640@gmail.com"; }];
          };
        };
        
        git-overlay-debian = mkDebianPackage pkgs self.packages.${system}.git-overlay;
      };

      overlays.default = final: prev: {
        git-overlay = self.packages.${final.stdenv.hostPlatform.system}.git-overlay;
      };

      devShells.default = pkgs.mkShell {
        packages = with pkgs;[
          rustc
          cargo
          rustfmt
          clippy
          zip
          jq
        ];
      };
    }
  );
}
