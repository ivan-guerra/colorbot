{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain
    cargo
    rustc
    rustfmt
    clippy
    
    # Required system libraries for dependencies
    xorg.libX11
    xorg.libXi
    xorg.libXtst
    xorg.libXrandr
    xorg.libxcb
    xdotool
    
    # Optional but useful
    pkg-config
  ];

  # Set library path for runtime
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    pkgs.xorg.libX11
    pkgs.xorg.libXi
    pkgs.xorg.libXtst
    pkgs.xorg.libXrandr
    pkgs.xorg.libxcb
  ];

  # Set PKG_CONFIG_PATH to help find the libraries
  PKG_CONFIG_PATH = "${pkgs.xorg.libxcb}/lib/pkgconfig";

  shellHook = ''
    echo "colorbot development environment loaded"
  '';
}
