{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/0e251e24a4f24e036a084b6b4b2d2491af4167f4";
  };

  outputs = { self, nixpkgs }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
    cargoConfig = builtins.fromTOML (builtins.readFile ./Cargo.toml);

    mkPackage = pkgsFor: pkgsFor.rustPlatform.buildRustPackage {
      pname = cargoConfig.package.name;
      version = cargoConfig.package.version;

      src = ./.;

      nativeBuildInputs = [ pkgsFor.git ];

      cargoHash = "sha256-zHrbESsao05xeCCH+UUTr7QjPoq+9M8bnEoUF1bBSh0=";

      meta = {
        description = "Software for overlaying personal files onto a git repository";
        mainProgram = "git-overlay";
        maintainers = [{ name = "szb640"; }];
      };
    };

    mkDebianPackage = import ./mkDebianPackage.nix;
  in {
    packages.${system} = {
      git-overlay = mkPackage pkgs;

      git-overlay-windows-x64 = mkPackage pkgs.pkgsCross.mingwW64;

      git-overlay-debian-x64 = mkDebianPackage pkgs self.packages.${system}.git-overlay;
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
