{
  description = "DNS Updater Service";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }: let
    supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
    forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    pkgsFor = system: nixpkgs.legacyPackages.${system};
  in {
    # 1. The Rust Package
    packages = forAllSystems (system: let
      pkgs = pkgsFor system;
    in {
      default = pkgs.rustPlatform.buildRustPackage {
        pname = "dns-updater";
        version = "0.1.0";
        src = ./.;
        cargoLock = {
          lockFile = ./Cargo.lock;
        };
      };
    });

    # 2. The Home Manager Module
    homeManagerModules.default = { config, lib, pkgs, ... }:
      let
        cfg = config.services.dns-updater;
      in {
        options.services.dns-updater = {
          enable = lib.mkEnableOption "dns-updater service";

          package = lib.mkOption {
            type = lib.types.package;
            default = self.packages.${pkgs.system}.default;
            description = "The dns-updater package to use.";
          };

          interface = lib.mkOption {
            type = lib.types.str;
            description = "The network interface to monitor (e.g., eth0, wlan0).";
            example = "eth0";
          };

          dnsTuples = lib.mkOption {
            type = lib.types.path;
            description = "Path to a file, where each line represents a DNS tuple (e.g., 'DD;token;ipv4;interval;domains' or '(DD;token;ipv4;interval;domains)').";
            example = "./dns_tuples.txt";
          };
        };

        config = lib.mkIf cfg.enable {
          systemd.user.services.dns-updater = {
            Unit = {
              Description = "DNS Updater Service";
              After = [ "network.target" ];
            };

            Service = {
              ExecStart = "${cfg.package}/bin/dns_updater";
              Restart = "always";
              
              # Map Nix options to the Environment Variables your Rust code expects
              Environment = [
                "RUST_LOG=debug"
                "INTERFACE=${cfg.interface}"
                "DNS_TUPLES=${builtins.readFile cfg.dnsTuples}"
              ];
            };

            Install = {
              WantedBy = [ "default.target" ];
            };
          };
        };
      };
  };
}
