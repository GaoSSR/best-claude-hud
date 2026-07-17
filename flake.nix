{
  description = "Nix flake for best-claude-hud";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      nixpkgs,
      flake-parts,
      rust-overlay,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      perSystem =
        {
          self',
          system,
          lib,
          ...
        }:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };

          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

          source = lib.cleanSourceWith {
            src = ./.;
            filter =
              path: type:
              let
                base = baseNameOf path;
              in
              !(base == "target"
                || base == "npm-publish"
                || base == "npm-tarballs"
                || base == "release-artifacts"
                || base == "result"
                || lib.hasPrefix "result-" base);
          };

          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          };

          rustPlatform = pkgs.makeRustPlatform {
            cargo = pkgs.rust-bin.stable.latest.minimal;
            rustc = pkgs.rust-bin.stable.latest.minimal;
          };
        in
        {
          packages.default = rustPlatform.buildRustPackage {
            pname = cargoToml.package.name;
            version = cargoToml.package.version;
            src = source;

            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };

            nativeCheckInputs = [ pkgs.gitMinimal ];

            # Disable cargo-auditable until https://github.com/rust-secure-code/cargo-auditable/issues/124 is fixed.
            auditable = false;

            meta = {
              homepage = "https://github.com/GaoSSR/best-claude-hud";
              description = cargoToml.package.description;
              license = lib.licenses.asl20;
              mainProgram = cargoToml.package.name;
              platforms = lib.platforms.unix;
            };
          };

          devShells.default = pkgs.mkShell {
            name = "best-claude-hud-dev-shell";
            inputsFrom = [ self'.packages.default ];
            packages = [ rustToolchain ];
            env = {
              RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
            };
            shellHook = ''
              echo "best-claude-hud Rust development shell ready: $(rustc --version)"
            '';
          };
        };
    };
}
