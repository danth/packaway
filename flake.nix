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
    } //
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.lib.${system};

        commonArguments = rec {
          src = ./.;
          cargoArtifacts = craneLib.buildDepsOnly { inherit src; };
        };

      in {
        packages.default = craneLib.buildPackage commonArguments;

        checks.clippy = craneLib.cargoClippy (commonArguments // {
          cargoClippyExtraArgs = "-- --deny warnings";
        });

        devShells.default = with pkgs; mkShell {
          nativeBuildInputs = [ cargo ];
        };
      }
    );
}
