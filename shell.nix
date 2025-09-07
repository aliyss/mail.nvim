{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  buildInputs = with pkgs; [openssl pkg-config luajit];

  RUST_BACKTRACE = 1;
}
