{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

{
  # https://devenv.sh/basics/

  packages = [
    pkgs.git
    pkgs.openssl

  ];

  languages.rust = {
    enable = true;
    channel = "nightly";
  };

  # See full reference at https://devenv.sh/reference/options/
}
