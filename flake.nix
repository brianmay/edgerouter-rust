{
  description = "Read/write EdgeRouter VyOS/Vyatta files";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        osxlibs = pkgs.lib.lists.optional pkgs.stdenv.isDarwin
          pkgs.darwin.apple_sdk.frameworks.Security;

        src = ./.;

        rustPlatform = pkgs.rust-bin.stable.latest.default;

        craneLib = (crane.mkLib pkgs).overrideToolchain rustPlatform;

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly { inherit src; };

        # Run clippy (and deny all warnings) on the crate source.
        clippy = craneLib.cargoClippy {
          inherit cargoArtifacts src;
          cargoClippyExtraArgs = "-- --deny warnings";
        };

        # Next, we want to run the tests and collect code-coverage, _but only if
        # the clippy checks pass_ so we do not waste any extra cycles.
        coverage = craneLib.cargoTarpaulin {
          inherit src;
          cargoArtifacts = clippy;
        };

        # create the workspace & dependencies package set
        pkg = craneLib.buildPackage {
          inherit src;
          inherit cargoArtifacts;
          buildInputs = osxlibs;

          # Add extra inputs here or any other derivation settings
          doCheck = true;
        };

        # The workspace defines a development shell with all of the dependencies
        # and environment settings necessary for a regular `cargo build`
        rustSrcPlatform =
          rustPlatform.override { extensions = [ "rust-src" ]; };
        workspaceShell = pkgs.mkShell {
          buildInputs = [ pkgs.rust-analyzer rustSrcPlatform ] ++ osxlibs;
        };

      in rec {
        checks = { inherit clippy coverage pkg; };
        packages = { default = pkg; };
        devShells.default = workspaceShell;
      });
}
