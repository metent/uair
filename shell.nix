with import <nixpkgs> { };
stdenv.mkDerivation {
  name = "uair";
  buildInputs = [
    cargo
    cargo-watch
    clippy
    rustc
    rustfmt
  ];
}
