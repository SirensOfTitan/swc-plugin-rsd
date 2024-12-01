{
  pkgs,
  inputs,
  ...
}:

let
  master = import inputs.nixpkgs-master { system = pkgs.stdenv.system; };
in
{
  languages.rust = {
    enable = true;
    channel = "nightly";
    targets = [ "wasm32-wasip1" ];
  };

  languages.javascript = {
    enable = true;
    bun.enable = true;
    bun.package = master.bun;
  };
}
