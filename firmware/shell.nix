with import <nixpkgs> {
  overlays = [
    (import (fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/57c8084c7ef41366993909c20491e359bbb90f54.tar.gz))
  ];
};
pkgs.mkShell {
  buildInputs = with pkgs; [
    (latest.rustChannels.stable.rust.override {
      extensions = [
        "rust-src"
        "rls-preview"
        "clippy-preview"
        "rustfmt-preview"
      ];
      targets = [
        "thumbv6m-none-eabi"
      ];
    })
    pkgconfig
    cargo-edit
    cargo-generate
    openssl
    openocd
    libusb
    gcc-arm-embedded



(callPackage ({ stdenv, requireFile, autoPatchelfHook, substituteAll,
  qt4, fontconfig, freetype, libusb, libICE, libSM, ncurses5, udev,
  libX11, libXext, libXcursor, libXfixes, libXrender, libXrandr }:
let
  architecture = {
    x86_64-linux = "x86_64";
    i686-linux   = "i386";
    armv7l-linux = "arm";
  }.${stdenv.hostPlatform.system} or (throw "unsupported system ${stdenv.hostPlatform.system}");

  sha256 = {
    x86_64-linux = "7bca4caea63f076c53d1aeed4ee7561c66fd73605f3e1528ee65929a3348ce3a";
    i686-linux   = "01qm56jyac3mzjny1z5lynik8y4hqrfq93n8119mvj6d4xiknv8y";
    armv7l-linux = "03l2zkfjw7z6j6nsdw6j4nxxzh8mgby8qrc179qjcajbdr3hmbr7";
  }.${stdenv.hostPlatform.system} or (throw "unsupported system ${stdenv.hostPlatform.system}");
in
stdenv.mkDerivation rec {
  pname = "jlink";
  version = "V686e";

  src = requireFile {
    name = "JLink_Linux_${version}_${architecture}.tgz";
    url = "https://www.segger.com/downloads/jlink#J-LinkSoftwareAndDocumentationPack";
    sha256 = sha256;
  };

  dontConfigure = true;
  dontBuild = true;
  dontStrip = true;

  nativeBuildInputs = [ autoPatchelfHook ];
  buildInputs = [
    qt4 fontconfig freetype libusb libICE libSM ncurses5
    libX11 libXext libXcursor libXfixes libXrender libXrandr
  ];

  runtimeDependencies = [ udev ];

  installPhase = ''
    mkdir -p $out/{JLink,bin}
    cp -R * $out/JLink
    ln -s $out/JLink/J* $out/bin/
    rm -r $out/bin/JLinkDevices.xml $out/JLink/libQt*
    install -D -t $out/lib/udev/rules.d 99-jlink.rules
  '';

  preFixup = ''
    patchelf --add-needed libudev.so.1 $out/JLink/libjlinkarm.so
  '';

  meta = with stdenv.lib; {
    homepage = "https://www.segger.com/downloads/jlink";
    description = "SEGGER J-Link";
    license = licenses.unfree;
    platforms = platforms.linux;
    maintainers = with maintainers; [ reardencode ];
  };
}){})

  ];

  # Set Environment Variables
  #RUST_BACKTRACE = 1;
}
