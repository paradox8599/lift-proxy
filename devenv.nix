{ pkgs, ... }:
{
  # https://devenv.sh/basics/

  packages = [
    pkgs.git
    pkgs.openssl
    pkgs.cargo-shuttle
  ];

  languages.rust = {
    enable = true;
    channel = "nightly";
  };

  scripts.shuttle.exec = "cargo shuttle $@";

  # See full reference at https://devenv.sh/reference/options/
}
