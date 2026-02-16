{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Use latest Rust from nixpkgs-unstable or system
    # (comment out if using rustup)
    # rustc
    # cargo
    
    # Build dependencies
    gnumake
    pkg-config
    openssl
    clang
    libclang
    gcc
    
    # Optional: for development
    # rust-analyzer
  ];
  
  # Set LIBCLANG_PATH for bindgen
  LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
  
  shellHook = ''
    echo "Moltis development environment"
    echo "Using system Rust: $(rustc --version 2>/dev/null || echo 'not found')"
    echo "LIBCLANG_PATH: $LIBCLANG_PATH"
    export PATH="${pkgs.gnumake}/bin:$PATH"
  '';
}
