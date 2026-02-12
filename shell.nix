{
  nixpkgs ? <nixpkgs>,
  system ? builtins.currentSystem,
  pkgs ? import nixpkgs { inherit system; },
  fenix ? import (fetchTarball "https://github.com/nix-community/fenix/archive/monthly.tar.gz") { },
  pimalaya ? import (fetchTarball "https://github.com/pimalaya/nix/archive/master.tar.gz"),
}:

pimalaya.mkShell {
  inherit
    nixpkgs
    system
    pkgs
    fenix
    ;

  buildInputs = with pkgs; [
    dbus
    openssl
  ];
}
