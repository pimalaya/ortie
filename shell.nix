{
  pimalaya ? import (fetchTarball "https://github.com/pimalaya/nix/archive/master.tar.gz"),
  ...
}@args:

let
  args' = removeAttrs args [ "pimalaya" ];
  extraBuildInputs = "nixd,nixfmt-rfc-style,openssl,dbus";
  shell = { inherit extraBuildInputs; };

in
pimalaya.mkShell (shell // args')
