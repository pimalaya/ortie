# TODO: move this to nixpkgs
# This file aims to be a replacement for the nixpkgs derivation.

{
  lib,
  pkg-config,
  buildPackages,
  rustPlatform,
  fetchFromGitHub,
  stdenv,
  apple-sdk,
  installShellFiles,
  installShellCompletions ? stdenv.buildPlatform.canExecute stdenv.hostPlatform,
  installManPages ? stdenv.buildPlatform.canExecute stdenv.hostPlatform,
  buildNoDefaultFeatures ? false,
  buildFeatures ? [ ],
}:

let
  version = "0.1.0";
  hash = "";
  cargoHash = "";
in

rustPlatform.buildRustPackage rec {
  inherit
    cargoHash
    version
    buildNoDefaultFeatures
    buildFeatures
    ;

  pname = "ortie";

  src = fetchFromGitHub {
    inherit hash;
    owner = "pimalaya";
    repo = "ortie";
    rev = "v${version}";
  };

  nativeBuildInputs = [
    pkg-config
  ]
  ++ lib.optional (installManPages || installShellCompletions) installShellFiles;

  buildInputs = lib.optional stdenv.hostPlatform.isDarwin apple-sdk;

  # configureFlags = lib.optionals (!stdenv.buildPlatform.canExecute stdenv.hostPlatform) [
  #   "kyua_cv_getopt_plus=yes"
  #   "kyua_cv_attribute_noreturn=yes"
  #   "kyua_cv_getcwd_works=yes"
  # ];

  doCheck = false;

  postInstall =
    let
      emulator = stdenv.hostPlatform.emulator buildPackages;
      exe = stdenv.hostPlatform.extensions.executable;
    in
    lib.optionalString (lib.hasInfix "wine" emulator) ''
      export WINEPREFIX="''${WINEPREFIX:-$(mktemp -d)}"
      mkdir -p $WINEPREFIX
    ''
    + ''
      mkdir -p $out/share/{completions,man}
      ${emulator} "$out"/bin/ortie${exe} manuals "$out"/share/man
      ${emulator} "$out"/bin/ortie${exe} completions -d "$out"/share/completions bash elvish fish powershell zsh
    ''
    + lib.optionalString installManPages ''
      installManPage "$out"/share/man/*
    ''
    + lib.optionalString installShellCompletions ''
      installShellCompletion --bash "$out"/share/completions/ortie.bash
      installShellCompletion --fish "$out"/share/completions/ortie.fish
      installShellCompletion --zsh "$out"/share/completions/_ortie
    '';

  meta = {
    description = "CLI to manage OAuth access tokens";
    mainProgram = "ortie";
    homepage = "https://github.com/pimalaya/ortie";
    changelog = "https://github.com/pimalaya/ortie/blob/v${version}/CHANGELOG.md";
    license = lib.licenses.agpl3Only;
    maintainers = with lib.maintainers; [ soywod ];
  };
}
