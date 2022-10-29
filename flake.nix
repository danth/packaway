{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { self, nixpkgs, crane, utils, ... }:
    {
      nixosModules.packaway = import ./nixos.nix self;
      hydraJobs.packaway = self.packages.x86_64-linux.default;
    } //
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.lib.${system};

        commonArguments = rec {
          src = craneLib.cleanCargoSource ./.;

          cargoArtifacts = craneLib.buildDepsOnly { inherit src; };

          # Create a temporary Nix database to prepare queries against
          preConfigure = ''
            ${pkgs.nix}/bin/nix \
              --experimental-features nix-command \
              --store /tmp/nix \
              store ping
          '';
          DATABASE_URL = "sqlite:/tmp/nix/nix/var/nix/db/db.sqlite";
        };
      in {
        packages.default = craneLib.buildPackage commonArguments;
      }
    );
}
