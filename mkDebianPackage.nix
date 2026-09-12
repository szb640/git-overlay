pkgsFor: installPackage:  let
  system = pkgsFor.stdenv.hostPlatform.system;
  maintainer = builtins.elemAt installPackage.meta.maintainers 0;
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
  pname = "${installPackage.name}-debian";
  version = installPackage.version;
  
  dontUnpack = true;
  dontConfigure = true;
  dontBuild = true;
  
  nativeBuildInputs = [ pkgsFor.dpkg ];

  installPhase = ''
    mkdir -p "$out"

    packageRoot="$TMPDIR/package"

    mkdir -p "$packageRoot/DEBIAN"
    mkdir -p "$packageRoot/usr/bin"

    cp -r ${installPackage}/ "$packageRoot/usr/"

    cat > "$packageRoot/DEBIAN/control" <<EOF
    Package: ${installPackage.pname}
    Version: ${installPackage.version}
    Section: utils
    Priority: optional
    Architecture: ${debianArch}
    Maintainer: ${maintainer.name} <${maintainer.email}>
    Description: ${installPackage.meta.mainProgram}
      ${installPackage.meta.description}
    EOF

    dpkg-deb --build \
      --root-owner-group \
      "$packageRoot" \
      "$out/${installPackage.pname}-${installPackage.version}-${system}.deb"
  '';
}
