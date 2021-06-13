with import <nixpkgs> {
  overlays = [
    (import (fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/ad227c55c34124cde17b30c9bb28be6cd3c70815.tar.gz))
  ];
};
pkgs.mkShell {
  buildInputs = with pkgs; [
    #(rustChannelOf { rustToolchain = ./rust-toolchain.toml; }).rust

    (latest.rustChannels.stable.rust.override {
      extensions = [
        "rust-src"
        "rls-preview"
        "clippy-preview"
        "rustfmt-preview"
        "llvm-tools-preview"
      ];
      targets = [
        "thumbv6m-none-eabi"
      ];
    })
    pkgconfig
    rustup
    cargo-edit
    cargo-generate
    openssl
    openocd
    libusb
    gcc-arm-embedded
    gcc
    ##(import ./jlink.nix)


    (callPackage ({ stdenv, requireFile, autoPatchelfHook, substituteAll,
      qt4, fontconfig, freetype, libusb, libICE, libSM, ncurses5, udev,
      libX11, libXext, libXcursor, libXfixes, libXrender, libXrandr }:
    let
      architecture = "x86_64";
      sha256 = "686c0a7698f5c993288f770cec4945c9c158396c612eb57ba26a1a12798f2ada";
    in
    stdenv.mkDerivation rec {
      pname = "jlink";
      version = "V722";

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
        rm $out/JLink/JLinkSTM32Exe
        ln -s $out/JLink/J* $out/bin/
        rm -r $out/bin/JLinkDevices.xml $out/JLink/libQt*
        install -D -t $out/lib/udev/rules.d 99-jlink.rules
      '';

      preFixup = ''
        patchelf --add-needed libudev.so.1 $out/JLink/libjlinkarm.so
      '';
    }){})

    (callPackage ({ libusb, pkgconfig }:
    rustPlatform.buildRustPackage rec {
      pname = "hf2-cli";
      version = "0.3.1";

      src = fetchCrate {
        inherit pname version;
        sha256 = "0xqrmcjz2xdkddvf6rr5hmcdjbgc5zivyxpgqnxiigyvn18rwd1l";
      };

      cargoHash = "sha256:1m367b9w7vd87454vgpzwgqa1n9h8g54mccgxxg9d1rxlqiisi2l";
      cargoDepsName = pname;

      buildInputs = [ libusb ];
      nativeBuildInputs = [ pkgconfig ];
    }){})

  ];

  # Set Environment Variables
  #RUST_BACKTRACE = 1;
}
