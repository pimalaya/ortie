{
  description = "CLI to manage contacts";

  inputs = {
    # https://nixpk.gs/pr-tracker.html?pr=407444
    # nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    nixpkgs.url = "github:nixos/nixpkgs/staging-next";
    fenix = {
      url = "github:nix-community/fenix";
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
