{
  description = "A minimalist utility for caching small results of deterministic calculations.
.";

  inputs.nixpkgs.url = "nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

        rustPkg = pkgs.rustPlatform.buildRustPackage {
          pname = cargoToml.package.name;
          src = self;
          version = cargoToml.package.version;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };

        dev-deps = with pkgs; [
          cargo-watch
          nix
          nixfmt-rfc-style
          rustup
        ];

      in
      {
        apps.default.program = "${rustPkg}/bin/${cargoToml.package.name}";
        apps.default.type = "app";

        packages.default = rustPkg;
        packages.detcache = rustPkg;

        devShells.default = pkgs.mkShell {
          packages = dev-deps;
        };
      }
    );
}
