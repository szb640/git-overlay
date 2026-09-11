pkgsFor: installPackage:  let
  system = pkgsFor.stdenv.hostPlatform.system;
  debianArchMap = {
    "x86_64-linux" = "amd64";
    "aarch64-linux" = "arm64";
    "i686-linux" = "i386";
    "armv7l-linux" = "armhf";
    "armv6l-linux" = "armel";
    "riscv64-linux" = "riscv64";
    "s390x-linux" = "s390x";
    "powerpc64le-linux" = "ppc64el";
  };
  debianArch = debianArchMap.${system};
in pkgsFor.stdenv.mkDerivation {
  pname = "git-overlay-debian";
  version = installPackage.version or "0.1.0";
  
  dontUnpack = true;
  dontConfigure = true;
  dontBuild = true;
  
  nativeBuildInputs = [ pkgsFor.dpkg ];

  installPhase = ''
    mkdir -p "$out"

    packageRoot="$TMPDIR/package"

    mkdir -p "$packageRoot/DEBIAN"
    mkdir -p "$packageRoot/usr/bin"

    cp ${installPackage}/bin/git-overlay "$packageRoot/usr/bin/git-overlay"
    chmod 0755 "$packageRoot/usr/bin/git-overlay"

    cat > "$packageRoot/DEBIAN/control" <<EOF
    Package: git-overlay
    Version: ${installPackage.version}
    Section: utils
    Priority: optional
    Architecture: ${debianArch}
    Maintainer: Bence Szikszai <szb640@gmail.com>
    Description: Git Overlay
      Software for overlaying personal files onto a git repository
    EOF

    dpkg-deb --build \
      --root-owner-group \
      "$packageRoot" \
      "$out/git-overlay_${installPackage.version}_${debianArch}.deb"
  '';
}
