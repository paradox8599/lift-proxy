{ pkgs, ... }:
{
  # https://devenv.sh/basics/

  packages = [
    pkgs.git
    pkgs.openssl
    pkgs.cargo-shuttle
    pkgs.sqlx-cli
  ];

  languages.rust.enable = true;

  scripts.shuttle.exec = "cargo shuttle $@";

  # See full reference at https://devenv.sh/reference/options/
}
