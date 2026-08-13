# TODO: move this to nixpkgs
# This file aims to be a replacement for the nixpkgs derivation.

{
  buildFeatures ? [ ],
  buildNoDefaultFeatures ? false,
  buildPackages,
  dbus,
  fetchFromGitHub,
  installManPages ? stdenv.buildPlatform.canExecute stdenv.hostPlatform,
  installShellCompletions ? stdenv.buildPlatform.canExecute stdenv.hostPlatform,
  installShellFiles,
  lib,
  openssl,
  pkg-config,
  rustPlatform,
  stdenv,
}:

let
  dbus' =
    # undefined reference in _dbus_atomic_* functions
    if stdenv.hostPlatform.isLinux && stdenv.hostPlatform.isAarch64 then
      dbus.overrideAttrs (old: {
        env = (old.env or { }) // {
          NIX_CFLAGS_COMPILE = (old.env.NIX_CFLAGS_COMPILE or "") + " -mno-outline-atomics";
        };
      })
    else
      dbus;

in
rustPlatform.buildRustPackage (finalAttrs: {
  __structuredAttrs = true;

  inherit buildNoDefaultFeatures;

  pname = "ortie";
  version = "2.1.0";
  cargoHash = "sha256-ZAW+Coyv1DVmF2HWr7CmFMtQT9hNaS+1koWoZCJ+Wsc=";

  src = fetchFromGitHub {
    owner = "pimalaya";
    repo = finalAttrs.pname;
    tag = "v${finalAttrs.version}";
    hash = "sha256-5KRXPIDYBfF/2fNnrJur8WdfLYvgtJWlTWA7YD8yYWg=";
  };

  # OpenSSL should not be provided by vendors, not even on Windows
  env.OPENSSL_NO_VENDOR = 1;

  nativeBuildInputs = [
    pkg-config
    installShellFiles
  ];

  buildInputs =
    lib.optional (builtins.elem "native-tls" buildFeatures) openssl
    # On Windows, D-Bus is provided by vendors
    ++ lib.optional (builtins.elem "notify" buildFeatures && !stdenv.hostPlatform.isWindows) dbus';

  buildFeatures =
    buildFeatures
    # On Windows, D-Bus is provided by vendors
    ++ lib.optional (builtins.elem "notify" buildFeatures && stdenv.hostPlatform.isWindows) "vendored";

  postInstall =
    let
      exe =
        if stdenv.buildPlatform.canExecute stdenv.hostPlatform then
          "$out/bin/${finalAttrs.pname}"
        else
          lib.getExe buildPackages.${finalAttrs.pname};
    in
    ''
      mkdir -p $out/share/{completions,man}
      ${exe} manuals "$out"/share/man
      ${exe} completions -d "$out"/share/completions bash elvish fish powershell zsh
    ''
    + lib.optionalString installManPages ''
      installManPage "$out"/share/man/*
    ''
    + lib.optionalString installShellCompletions ''
      installShellCompletion --cmd ${finalAttrs.pname} \
        --bash "$out"/share/completions/${finalAttrs.pname}.bash \
        --fish "$out"/share/completions/${finalAttrs.pname}.fish \
        --zsh "$out"/share/completions/_${finalAttrs.pname}
    '';

  # Disable impure integration tests
  cargoTestFlags = [ "--bins" ];

  meta = {
    description = "CLI to manage OAuth 2.0 tokens";
    mainProgram = finalAttrs.pname;
    homepage = "https://github.com/pimalaya/${finalAttrs.pname}";
    changelog = "https://github.com/pimalaya/${finalAttrs.pname}/releases/${finalAttrs.src.tag}";
    license = with lib.licenses; [
      asl20
      mit
    ];
    maintainers = with lib.maintainers; [
      soywod
      bobberb
    ];
  };
})
