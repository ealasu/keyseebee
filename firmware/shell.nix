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
  ];

  # Set Environment Variables
  #RUST_BACKTRACE = 1;
}
