{
  pimalaya ? import (fetchTarball "https://github.com/pimalaya/nix/archive/master.tar.gz"),
  ...
}@args:

let
  args' = removeAttrs args [ "pimalaya" ];
  default = {
    src = ./.;
    version = "0.1.0";
    mkPackage = (
      {
        lib,
        pkgs,
        rustPlatform,
        defaultFeatures,
        features,
      }:

      pkgs.callPackage ./package.nix {
        inherit lib rustPlatform;
        apple-sdk = pkgs.apple-sdk_15;
        installShellCompletions = false;
        installManPages = false;
        withNoDefaultFeatures = !defaultFeatures;
        withFeatures = lib.splitString "," features;
      }
    );
  };
in

pimalaya.mkDefault (default // args')
