{
  description = "A Nix binary cache server";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixpkgs-stable.url = "github:NixOS/nixpkgs/nixos-25.05";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    crane.url = "github:ipetkov/crane";

    nix-github-actions = {
      url = "github:nix-community/nix-github-actions";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = inputs @ { self, flake-parts, ... }: let
    supportedSystems = [
      "x86_64-linux"
      "aarch64-linux"
      "riscv64-linux"
      "aarch64-darwin"
      "x86_64-darwin"
    ];

    inherit (inputs.nixpkgs) lib;

    modules = builtins.foldl' (acc: f: f acc) ./flake [
      builtins.readDir
      (lib.filterAttrs (name: type:
        type == "regular" && lib.hasSuffix ".nix" name
      ))
      (lib.mapAttrsToList (name: _:
        lib.path.append ./flake name
      ))
    ];

  in flake-parts.lib.mkFlake { inherit inputs; } {
    imports = modules;
    systems = supportedSystems;

    perSystem = { system, ... }: {
      _module.args.pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [
          (final: prev: {
            remarshal = prev.writeScriptBin "remarshal" ''
              #!${prev.python3Minimal}/bin/python3
              import argparse
              import json
              import re

              bare_key = re.compile(r"^[A-Za-z0-9_-]+$")

              def key(value):
                  return value if bare_key.match(value) else json.dumps(value)

              def toml_value(value):
                  if isinstance(value, str):
                      return json.dumps(value)
                  if isinstance(value, bool):
                      return "true" if value else "false"
                  if isinstance(value, (int, float)):
                      return str(value)
                  if isinstance(value, list):
                      return "[" + ", ".join(toml_value(item) for item in value) + "]"
                  if isinstance(value, dict):
                      return "{ " + ", ".join(f"{key(k)} = {toml_value(v)}" for k, v in value.items()) + " }"
                  if value is None:
                      raise TypeError("TOML has no null value")
                  raise TypeError(f"unsupported TOML value: {type(value).__name__}")

              def write_table(lines, name, table):
                  lines.append(f"[{'.'.join(key(part) for part in name)}]")
                  nested = []
                  for k, v in table.items():
                      if isinstance(v, dict) and any(isinstance(item, dict) for item in v.values()):
                          nested.append((k, v))
                      else:
                          lines.append(f"{key(k)} = {toml_value(v)}")
                  lines.append("")
                  for k, v in nested:
                      write_table(lines, [*name, k], v)

              parser = argparse.ArgumentParser()
              parser.add_argument("-i", "--input", required=True)
              parser.add_argument("-o", "--output", required=True)
              parser.add_argument("-if", "--input-format", dest="input_format")
              parser.add_argument("-of", "--output-format", dest="output_format")
              args = parser.parse_args()

              if args.input_format not in (None, "json") or args.output_format not in (None, "toml"):
                  raise SystemExit("this lightweight remarshal only supports json -> toml")

              with open(args.input) as f:
                  data = json.load(f)

              lines = []
              for k, v in data.items():
                  if isinstance(v, dict):
                      write_table(lines, [k], v)
                  else:
                      lines.append(f"{key(k)} = {toml_value(v)}")

              with open(args.output, "w") as f:
                  f.write("\n".join(lines))
                  f.write("\n")
            '';

            fetchFromSavannah = args:
              if (args.repo or null) == "config"
                && (args.rev or null) == "576c839acca0e082e536fd27568b90a446ce5b96"
              then prev.fetchgit {
                url = "https://git.savannah.gnu.org/git/config.git";
                inherit (args) rev sha256;
              }
              else prev.fetchFromSavannah args;
          })
        ];
      };
    };

    debug = true;
  };
}
