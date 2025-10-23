{
  description = "CLI to manage OAuth 2.0 access tokens";

  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/staging-next";
    };
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pimalaya = {
      flake = false;
      url = "github:pimalaya/nix";
    };
  };

  outputs =
    inputs:
    (import inputs.pimalaya).mkFlakeOutputs inputs {
      shell = ./shell.nix;
      default = ./default.nix;
    };
}
