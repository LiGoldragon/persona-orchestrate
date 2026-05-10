{
  description = "Typed mind state for Persona agents.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, flake-utils, fenix, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        toolchain = fenix.packages.${system}.stable.withComponents [
          "cargo"
          "rustc"
          "rustfmt"
          "clippy"
          "rust-analyzer"
          "rust-src"
        ];
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
        src = craneLib.cleanCargoSource ./.;
        commonArgs = { inherit src; strictDeps = true; };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      in
      {
        packages.default = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          meta.mainProgram = "mind";
        });
        checks = {
          default = craneLib.cargoTest (commonArgs // { inherit cargoArtifacts; });
          build = craneLib.cargoBuild (commonArgs // { inherit cargoArtifacts; });
          test = craneLib.cargoTest (commonArgs // { inherit cargoArtifacts; });
          weird-actor-truth = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            cargoTestExtraArgs = "--test weird_actor_truth";
          });
          daemon-wire = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            cargoTestExtraArgs = "--test daemon_wire";
          });
          cli = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            cargoTestExtraArgs = "--test cli";
          });
          cli-binary = pkgs.runCommand "mind-cli-binary" {} ''
            set -euo pipefail

            workspace="$(mktemp -d)"
            socket="$workspace/mind.sock"
            store="$workspace/mind.redb"

            ${self.packages.${system}.default}/bin/mind daemon \
              --socket "$socket" \
              --store "$store" &
            daemon="$!"
            trap 'kill "$daemon" 2>/dev/null || true' EXIT

            for attempt in $(seq 1 100); do
              if [ -S "$socket" ]; then
                break
              fi
              sleep 0.05
            done
            test -S "$socket"

            ${self.packages.${system}.default}/bin/mind \
              --socket "$socket" \
              --actor operator \
              '(RoleClaim Operator [(Path "/git/github.com/LiGoldragon/persona-mind")] "claim from binary check")' \
              > "$workspace/claim.out"
            grep -F '(ClaimAcceptance Operator [(Path "/git/github.com/LiGoldragon/persona-mind")])' \
              "$workspace/claim.out"

            ${self.packages.${system}.default}/bin/mind \
              --socket "$socket" \
              --actor operator \
              '(RoleObservation)' \
              > "$workspace/observe.out"
            grep -F '(RoleStatus Operator [(ClaimEntry (Path "/git/github.com/LiGoldragon/persona-mind") "claim from binary check")]' \
              "$workspace/observe.out"

            touch "$out"
          '';
          test-doc = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            cargoTestExtraArgs = "--doc";
          });
          doc = craneLib.cargoDoc (commonArgs // {
            inherit cargoArtifacts;
            RUSTDOCFLAGS = "-D warnings";
          });
          fmt = craneLib.cargoFmt { inherit src; };
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });
        };
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/mind";
        };
        devShells.default = pkgs.mkShell {
          name = "persona-mind";
          packages = [ pkgs.jujutsu pkgs.pkg-config toolchain ];
        };
        formatter = pkgs.nixfmt;
      });
}
