{
  pkgs,
  naersk,
}: let
  manix = naersk.buildPackage {
    pname = "manix";
    root = ./.;
    overrideMain = _: {
      postInstall = ''
        APPLICATION_DIR="$out/share/applications/manix"
        mkdir -p $APPLICATION_DIR
        cp ${pkgs.home-manager-options}/share/doc/home-manager/options.json $APPLICATION_DIR/hm-options.json
        cp ${pkgs.nix-options}/share/doc/nixos/options.json $APPLICATION_DIR/nixos-options.json
      '';
    };
  };
in
  pkgs.symlinkJoin {
    name = "manix";
    buildInputs = [pkgs.makeWrapper];
    paths = [manix];
    postBuild = ''
      wrapProgram $out/bin/manix \
        --prefix PATH : ${pkgs.nix}/bin \
        --set NIXOS_JSON_OPTIONS_PATH ${manix}/share/applications/manix/nixos-options.json \
        --set HOME_MANAGER_JSON_OPTIONS_PATH ${manix}/share/applications/manix/hm-options.json
    '';
  }

#   {
#   pkgs,
#   naersk,
# }: let
#   manix = naersk.buildPackage {
#     pname = "manix";
#     root = ./.;
#     # overrideMain = _: {
#     #   postInstall = ''
#     #   '';
#     # };
#   };
# in
#   pkgs.symlinkJoin {
#     name = "manix";
#     buildInputs = [pkgs.makeWrapper pkgs.jq];
#     paths = [manix];
#     postBuild = ''
#       APPLICATION_DIR="$out/share/applications/manix"
#       mkdir -p $APPLICATION_DIR
#       cat ${pkgs.home-manager-options}/share/doc/home-manager/options.json \
#         | jq -c 'del(."_module.args")' > $APPLICATION_DIR/hm-options.json
#       cp -r ${pkgs.nix-options}/share/doc/nixos/options.json $APPLICATION_DIR/nixos-options.json
#
#       wrapProgram $out/bin/manix \
#         --prefix PATH : ${pkgs.nix}/bin \
#         --set NIXOS_JSON_OPTIONS_PATH ${manix}/share/applications/manix/nixos-options.json \
#         --set HOME_MANAGER_JSON_OPTIONS_PATH ${manix}/share/applications/manix/hm-options.json
#     '';
#   }
#
