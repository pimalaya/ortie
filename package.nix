# TODO: move this to nixpkgs
# This file aims to be a replacement for the nixpkgs derivation.

{
  apple-sdk,
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
  version = "0.1.0";
  hash = "";
  cargoHash = "";

  inherit (stdenv.hostPlatform)
    isLinux
    isWindows
    isx86_64
    isAarch64
    isDarwin
    ;

  emulator = stdenv.hostPlatform.emulator buildPackages;
  exe = stdenv.hostPlatform.extensions.executable;

  # notify feature is part of default cargo features
  hasNotifyFeature = !buildNoDefaultFeatures || builtins.elem "notify" buildFeatures;
  #hasNativeTlsFeature = builtins.elem "native-tls" buildFeatures;
  hasNativeTlsFeature = !buildNoDefaultFeatures || builtins.elem "native-tls" buildFeatures;

  # statically link dbus via cargo (vendored)
  dbusFromCargo = hasNotifyFeature && isWindows && isx86_64;
  # statically link dbus via nixpkgs
  dbusFromNix = hasNotifyFeature && !(isWindows && isx86_64);

  # needed for building dbus on aarch64-linux
  dbus' = dbus.overrideAttrs (old: {
    env = (old.env or { }) // {
      NIX_CFLAGS_COMPILE =
        (old.env.NIX_CFLAGS_COMPILE or "")
        + lib.optionalString (isLinux && isAarch64) " -mno-outline-atomics";
    };
  });

in
rustPlatform.buildRustPackage {
  inherit cargoHash version buildNoDefaultFeatures;

  pname = "ortie";

  src = fetchFromGitHub {
    inherit hash;
    owner = "pimalaya";
    repo = "ortie";
    rev = "v${version}";
  };

  env = lib.optionalAttrs (isLinux && isAarch64) {
    NIX_CFLAGS_COMPILE = "-mno-outline-atomics";
  };

  nativeBuildInputs =
    [ ]
    ++ lib.optional (hasNotifyFeature || hasNativeTlsFeature) pkg-config
    ++ lib.optional (installManPages || installShellCompletions) installShellFiles;

  buildInputs =
    [ ]
    ++ lib.optional isDarwin apple-sdk
    ++ lib.optional dbusFromNix dbus'
    ++ lib.optional hasNativeTlsFeature openssl;

  buildFeatures = buildFeatures ++ lib.optional dbusFromCargo "vendored";

  doCheck = false;

  postInstall =
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
      installShellCompletion --cmd ortie \
        --bash "$out"/share/completions/ortie.bash \
        --fish "$out"/share/completions/ortie.fish \
        --zsh "$out"/share/completions/_ortie
    '';

  meta = {
    description = "CLI to manage OAuth tokens";
    mainProgram = "ortie";
    homepage = "https://github.com/pimalaya/ortie";
    changelog = "https://github.com/pimalaya/ortie/blob/v${version}/CHANGELOG.md";
    license = lib.licenses.agpl3Plus;
    maintainers = with lib.maintainers; [ soywod ];
  };
}
