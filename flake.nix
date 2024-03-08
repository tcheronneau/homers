{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils }:
  let 
    name = "homers";
    version = "0.1.0";
  in 
  flake-utils.lib.eachDefaultSystem (system:
    with nixpkgs.legacyPackages.${system}; {
      packages.homers = rustPlatform.buildRustPackage {
        name = "${name}";
        version = "${version}";

        src = lib.cleanSource ./.;

        cargoSha256 =
          "sha256-SNGYOUpycIegRvELolBk3PpbeMe/GfW2lFVksu8YLJo=";
        nativeBuildInputs = [ 
          rustc
          cargo
          pkg-config
          openssl.dev
        ];
      };
      packages.docker = dockerTools.buildLayeredImage {
        name = "mcth/${name}";
        contents = [ cacert ];
        tag = "${system}-${version}";
        created = "now";
        config = {
          Cmd = [
            "${self.packages.${system}.homers}/bin/${name}"
          ];
          ExposedPorts = {
            "8080/tcp" = { };
          };
        };
      };
      defaultPackage = self.packages.${system}.homers;
      devShell = mkShell {
        inputsFrom = builtins.attrValues self.packages.${system};

        buildInputs = [ 
          pkg-config
          openssl.dev
          rust-analyzer
          rustc
          cargo
        ];
      };
    });
}
