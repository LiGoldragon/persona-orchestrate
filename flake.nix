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

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
      crane,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
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
        commonArgs = {
          inherit src;
          strictDeps = true;
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        mindConstraintCheck =
          name: script:
          pkgs.runCommand name { } ''
            set -euo pipefail

            export MIND_BIN=${self.packages.${system}.default}/bin/mind
            ${pkgs.bash}/bin/bash ${script}

            touch "$out"
          '';
      in
      {
        packages.default = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            meta.mainProgram = "mind";
          }
        );
        checks = {
          default = craneLib.cargoTest (commonArgs // { inherit cargoArtifacts; });
          build = craneLib.cargoBuild (commonArgs // { inherit cargoArtifacts; });
          test = craneLib.cargoTest (commonArgs // { inherit cargoArtifacts; });
          weird-actor-truth = craneLib.cargoTest (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoTestExtraArgs = "--test weird_actor_truth";
            }
          );
          daemon-wire = craneLib.cargoTest (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoTestExtraArgs = "--test daemon_wire";
            }
          );
          mind-typed-graph-uses-graph-actor-lane = craneLib.cargoTest (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoTestExtraArgs = "--test actor_topology typed_thought_runs_through_graph_actor_lane_and_store_mints_id";
            }
          );
          mind-typed-thought-graph-survives-process-restart = craneLib.cargoTest (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoTestExtraArgs = "--test daemon_wire mind_typed_thought_graph_survives_process_restart";
            }
          );
          mind-superseded-thought-excluded-from-current-query = craneLib.cargoTest (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoTestExtraArgs = "--test actor_topology superseded_thought_excluded_from_current_query";
            }
          );
          mind-supersedes-rejects-different-thought-kinds = craneLib.cargoTest (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoTestExtraArgs = "--test actor_topology supersedes_relation_rejects_different_thought_kinds";
            }
          );
          mind-typed-thought-subscription-registers-and-returns-initial-snapshot = craneLib.cargoTest (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoTestExtraArgs = "--test actor_topology typed_thought_subscription_registers_and_returns_initial_snapshot";
            }
          );
          mind-typed-relation-subscription-registers-and-returns-initial-snapshot = craneLib.cargoTest (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoTestExtraArgs = "--test actor_topology typed_relation_subscription_registers_and_returns_initial_snapshot";
            }
          );
          mind-thought-subscription-is-durable-table-data = craneLib.cargoTest (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoTestExtraArgs = "thought_subscription_is_durable_table_data";
            }
          );
          cli = craneLib.cargoTest (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoTestExtraArgs = "--test cli";
            }
          );
          mind-cli-accepts-full-signal-mind-request-for-typed-graph = craneLib.cargoTest (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoTestExtraArgs = "--test cli mind_cli_accepts_full_signal_mind_request_for_typed_graph";
            }
          );
          cli-binary = pkgs.runCommand "mind-cli-binary" { } ''
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
          mind-cli-accepts-one-nota-record-and-prints-one-nota-reply = mindConstraintCheck "mind-cli-accepts-one-nota-record-and-prints-one-nota-reply" ./scripts/mind-cli-accepts-one-nota-record-and-prints-one-nota-reply;
          mind-cli-sends-signal-frames-to-long-lived-daemon = mindConstraintCheck "mind-cli-sends-signal-frames-to-long-lived-daemon" ./scripts/mind-cli-sends-signal-frames-to-long-lived-daemon;
          mind-cli-opens-and-queries-work-item-through-daemon = mindConstraintCheck "mind-cli-opens-and-queries-work-item-through-daemon" ./scripts/mind-cli-opens-and-queries-work-item-through-daemon;
          mind-store-survives-process-restart = mindConstraintCheck "mind-store-survives-process-restart" ./scripts/mind-store-survives-process-restart;
          test-doc = craneLib.cargoTest (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoTestExtraArgs = "--doc";
            }
          );
          doc = craneLib.cargoDoc (
            commonArgs
            // {
              inherit cargoArtifacts;
              RUSTDOCFLAGS = "-D warnings";
            }
          );
          fmt = craneLib.cargoFmt { inherit src; };
          clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- -D warnings";
            }
          );
        };
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/mind";
        };
        devShells.default = pkgs.mkShell {
          name = "persona-mind";
          packages = [
            pkgs.jujutsu
            pkgs.pkg-config
            toolchain
          ];
        };
        formatter = pkgs.nixfmt;
      }
    );
}
