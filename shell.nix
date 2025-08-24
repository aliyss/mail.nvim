{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  buildInputs = with pkgs; [openssl pkg-config sea-orm-cli];

  RUST_BACKTRACE = 1;
}
