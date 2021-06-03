# Run this first: nix-store --add-fixed sha256 ~/Downloads/JLink_Linux_V686e_x86_64.tgz
{callPackage}:

(callPackage ({ stdenv, requireFile, autoPatchelfHook, substituteAll,
  qt4, fontconfig, freetype, libusb, libICE, libSM, ncurses5, udev,
  libX11, libXext, libXcursor, libXfixes, libXrender, libXrandr }:
let
  architecture = "x86_64";
  sha256 = "7bca4caea63f076c53d1aeed4ee7561c66fd73605f3e1528ee65929a3348ce3a";
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
}){})
