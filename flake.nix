{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils }:
  let
    name = "homers";
    version = "1.0.0";
  in
  flake-utils.lib.eachDefaultSystem (system:
    with nixpkgs.legacyPackages.${system}; {
      packages.homers = rustPlatform.buildRustPackage {
        name = "${name}";
        version = "${version}";

        src = lib.cleanSource ./.;


        cargoLock = {
          lockFile = ./Cargo.lock;
          allowBuiltinFetchGit = true;
        };

        buildInputs = [
          pkg-config
          openssl.dev
        ];
        nativeBuildInputs = [
          rustc
          cargo
          pkg-config
          openssl.dev
        ];
      };
      packages.docker = dockerTools.buildImage {
        name = "mcth/${name}";
        tag = "${system}-${version}";
        created = "now";
        copyToRoot = pkgs.buildEnv {
          name = "homers";
          paths = [
            (pkgs.writeTextDir "/etc/homers/config.toml" (builtins.readFile ./config.toml))
            pkgs.cacert
          ];
          pathsToLink = [ "/etc" ];
        };
        config = {
          Cmd = [
            "${self.packages.${system}.homers}/bin/${name}"
            "--config"
            "/etc/homers/config.toml"
          ];
          ExposedPorts = {
            "8000/tcp" = { };
          };
        };
      };
      packages.default = self.packages.${system}.homers;
      devShells.default = mkShell {
        inputsFrom = builtins.attrValues self.packages.${system};

        buildInputs = [
          pkg-config
          openssl.dev
          rust-analyzer
          clippy
          rustc
          cargo
          rustfmt
        ];
        #++ lib.optionals stdenv.isDarwin [ darwin.apple_sdk.frameworks.SystemConfiguration ];
      };
      nixosModules.homers = { config, lib, pkgs, ...}:
        with lib;
        let
          cfg = config.services.homers;
          format = pkgs.formats.toml {};
          configFile = format.generate "config.toml" cfg.settings;
        in {
          options.services.homers = {
            enable = mkOption {
              type = types.bool;
              default = false;
              description = "Enable Homers service";
            };
            package = mkOption {
              type = types.package;
              default = self.packages.${system}.homers;
              description = "Homers package to use";
            };
            settings = mkOption {
              type = types.submodule {
                freeformType = format.type;
              };
              default = { };
              description = ''
                Homers configuration settings (non-secret values).
                These are rendered to a TOML config file in the Nix store.
                Do NOT put API keys or tokens here - use environmentFile instead.
              '';
            };
            environmentFile = mkOption {
              type = types.nullOr types.path;
              default = null;
              description = ''
                Path to an environment file containing secret overrides.
                Homers uses Figment with HOMERS_ prefix and _ separator.
                Environment variables override TOML config values.

                Example file contents:
                  HOMERS_SONARR_MAIN_APIKEY=your-sonarr-key
                  HOMERS_TAUTULLI_APIKEY=your-tautulli-key
                  HOMERS_RADARR_MAIN_APIKEY=your-radarr-key
                  HOMERS_OVERSEERR_APIKEY=your-overseerr-key
                  HOMERS_PLEX_KEVIN_TOKEN=your-plex-token
              '';
            };
          };
          config = mkIf cfg.enable {
            systemd.services.homers = {
              description = "Homers Service";
              after = [ "network.target" ];
              wantedBy = [ "multi-user.target" ];
              serviceConfig = {
                ExecStart = "${cfg.package}/bin/${name} --config ${configFile}";
                Restart = "on-failure";
                ExecReload = "${pkgs.coreutils}/bin/kill -SIGHUP $MAINPID";
                KillSignal = "SIGINT";
                TimeoutStopSec = "30s";
                User = "homers";
                Group = "homers";
                AmbientCapabilities = "CAP_NET_BIND_SERVICE";
              } // optionalAttrs (cfg.environmentFile != null) {
                EnvironmentFile = cfg.environmentFile;
              };
            };
            users.users.homers = {
              isSystemUser = true;
              description = "Homers Service User";
              group = "homers";
            };
            users.groups.homers = {};
          };
        };
        });
}
