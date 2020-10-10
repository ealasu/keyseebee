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
    #bossa
    gcc-arm-embedded

    #(rustPlatform.buildRustPackage rec {
      #pname = "hf2-rs";
      #version = "master";

      #src = fetchFromGitHub {
        #owner = "jacobrosenthal";
        #repo = pname;
        #rev = version;
        #sha256 = "0zi1yvdl63gyzy2vp58pz8jr8wcrxnmcpp0x1jmshv9kqcvws8nj";
      #};

      #cargoSha256 = "17ldqr3asrdcsh4l29m3b5r37r5d0b3npq1lrgjmxb6vlx6a36qj";
      #verifyCargoDeps = true;
    #})

    #(stdenv.mkDerivation {
      #name = "bossa-1.6.1";

      #src = fetchgit {
        ##url = "https://github.com/shumatech/BOSSA";
        ##rev = "293b73818e113bf4544bc1aa636495f1b2f0832d";
        ##sha256 = "0cz49y07cqd9l66nsm7cbr19j9wxyv8pgm52wz108zhdiin4hy58";
        #url = "https://github.com/mistoll/BOSSA";
        #rev = "a920e3bc6f8c0e5fdabfa7a4622f06b46cce351a";
        #sha256 = "0vz9i22x3fwrz0pr3khz8xaq5zc67pfa5z92hxf4rnhji3qb57nc";
      #};

      ##patches = [ ./bossa-no-applet-build.patch ];

      ##nativeBuildInputs = [ bin2c ];
      #buildInputs = [
        ##wxGTK libX11
        #readline
      #];

      ## Explicitly specify targets so they don't get stripped.
      #makeFlags = [ "bin/bossac" ];
      #NIX_CFLAGS_COMPILE = "-Wno-error=deprecated-declarations -fpermissive";

      #installPhase = ''
        #mkdir -p $out/bin
        #cp bin/bossac $out/bin/
      #'';
    #})

  ];

  # Set Environment Variables
  #RUST_BACKTRACE = 1;
}
