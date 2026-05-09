     1|{
     2|  description = "Nix packaging for cc-switch-tui";
     3|
     4|  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
     5|
     6|  outputs = { self, nixpkgs }:
     7|    let
     8|      cargoManifest = builtins.fromTOML (builtins.readFile ./src-tauri/Cargo.toml);
     9|      systems = [
    10|        "x86_64-linux"
    11|        "aarch64-linux"
    12|        "x86_64-darwin"
    13|        "aarch64-darwin"
    14|      ];
    15|      forAllSystems = f:
    16|        nixpkgs.lib.genAttrs systems (system:
    17|          f system (import nixpkgs { inherit system; }));
    18|    in
    19|    {
    20|      packages = forAllSystems (system: pkgs:
    21|        let
    22|          cc_switch_cli = pkgs.rustPlatform.buildRustPackage {
    23|            pname = cargoManifest.package.name;
    24|            version = cargoManifest.package.version;
    25|
    26|            src = pkgs.lib.cleanSource ./.;
    27|
    28|            cargoRoot = "src-tauri";
    29|            buildAndTestSubdir = "src-tauri";
    30|            cargoLock = {
    31|              lockFile = ./src-tauri/Cargo.lock;
    32|            };
    33|
    34|            # The upstream repository owns the Rust test suite. The flake package is
    35|            # intended to build and install the CLI on NixOS without depending on
    36|            # host-specific assistant CLIs or live config fixtures during checkPhase.
    37|            doCheck = false;
    38|
    39|            meta = with pkgs.lib; {
    40|              description = "TUI manager for Claude Code, Codex, Gemini, OpenCode, OpenClaw, and Hermes";
    41|              homepage = "https://github.com/handy-sun/cc-switch-tui";
    42|              license = licenses.mit;
    43|              mainProgram = "cc-switch-tui";
    44|              platforms = platforms.unix;
    45|            };
    46|          };
    47|        in
    48|        {
    49|          cc-switch-tui = cc_switch_cli;
    50|          # legacy alias kept for transition
          cc-switch = cc_switch_cli;
    51|          default = cc_switch_cli;
    52|        });
    53|    };
    54|}
    55|