# TODO: move this to nixpkgs
# This file aims to be a replacement for the nixpkgs derivation.

{
  lib,
  pkg-config,
  rustPlatform,
  fetchFromGitHub,
  stdenv,
  apple-sdk,
  installShellFiles,
  installShellCompletions ? stdenv.buildPlatform.canExecute stdenv.hostPlatform,
  installManPages ? stdenv.buildPlatform.canExecute stdenv.hostPlatform,
  withNoDefaultFeatures ? false,
  withFeatures ? [ ],
}:

let
  version = "0.1.0";
  hash = "";
  cargoHash = "";
in

rustPlatform.buildRustPackage rec {
  inherit cargoHash version;

  pname = "ortie";

  src = fetchFromGitHub {
    inherit hash;
    owner = "pimalaya";
    repo = "ortie";
    rev = "v${version}";
  };

  buildNoDefaultFeatures = withNoDefaultFeatures;
  buildFeatures = withFeatures;

  nativeBuildInputs = [
    pkg-config
  ] ++ lib.optional (installManPages || installShellCompletions) installShellFiles;

  buildInputs = lib.optional stdenv.hostPlatform.isDarwin apple-sdk;

  configureFlags = lib.optionals (!stdenv.buildPlatform.canExecute stdenv.hostPlatform) [
    "kyua_cv_getopt_plus=yes"
    "kyua_cv_attribute_noreturn=yes"
    "kyua_cv_getcwd_works=yes"
  ];

  # unit tests only
  doCheck = false;
  auditable = false;

  postInstall =
    ''
      mkdir -p $out/share/{completions,man}
    ''
    + lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
      "$out"/bin/ortie man "$out"/share/man
    ''
    + lib.optionalString installManPages ''
      installManPage "$out"/share/man/*
    ''
    + lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
      "$out"/bin/ortie completion bash > "$out"/share/completions/ortie.bash
      "$out"/bin/ortie completion elvish > "$out"/share/completions/ortie.elvish
      "$out"/bin/ortie completion fish > "$out"/share/completions/ortie.fish
      "$out"/bin/ortie completion powershell > "$out"/share/completions/ortie.powershell
      "$out"/bin/ortie completion zsh > "$out"/share/completions/ortie.zsh
    ''
    + lib.optionalString installShellCompletions ''
      installShellCompletion "$out"/share/completions/ortie.{bash,fish,zsh}
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
