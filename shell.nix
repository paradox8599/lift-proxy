{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell {
  name = "lfit-proxy";

  buildInputs = with pkgs; [
    pkg-config
    openssl.dev
  ];

  nativeBuildInputs = [
    pkgs.pkg-config
  ];

  shellHook = ''
  '';
}
