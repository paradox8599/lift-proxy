{ pkgs, ... }:
{
  # https://devenv.sh/basics/

  packages = [
    pkgs.git
    pkgs.openssl
    pkgs.sqlx-cli
  ];

  languages.rust.enable = true;

  # See full reference at https://devenv.sh/reference/options/
}
