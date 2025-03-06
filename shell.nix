{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell {
  name = "rust project";

  buildInputs = with pkgs; [
    pkg-config
    openssl.dev
  ];

  nativeBuildInputs = [
    pkgs.pkg-config
  ];

  shellHook = ''
    export SHELL=zsh
    exec zsh
  '';
}
