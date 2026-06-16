{ self
, lib
, flake-parts-lib
, inputs
, config
, makeCranePkgs
, getSystem
, ...
}:

let
  inherit (lib)
    mkOption
    types
    ;
  inherit (flake-parts-lib)
    mkPerSystemOption
    ;

  # Re-evaluate perSystem with cross nixpkgs
  # HACK before https://github.com/hercules-ci/flake-parts/issues/95 is solved
  evalCross = { system, pkgs }: config.allSystems.${system}.debug.extendModules {
    modules = [
      ({ config, lib, ... }: {
        _module.args.pkgs = pkgs;
        _module.args.self' = lib.mkForce config;
      })
    ];
  };
in
{
  options = {
    perSystem = mkPerSystemOption {
      options.attic = {
        toolchain = mkOption {
          type = types.nullOr types.package;
          default = null;
        };
        extraPackageArgs = mkOption {
          type = types.attrsOf types.anything;
          default = {};
        };
      };
    };
  };

  config = {
    _module.args.makeCranePkgs = lib.mkDefault (pkgs: let
      perSystemConfig = getSystem pkgs.system;
      craneLib = builtins.foldl' (acc: f: f acc) pkgs [
        inputs.crane.mkLib
        (craneLib:
          if perSystemConfig.attic.toolchain == null then craneLib
          else craneLib.overrideToolchain config.attic.toolchain
        )
      ];
    in pkgs.callPackage ../crane.nix {
      inherit craneLib;
      inherit (perSystemConfig.attic) extraPackageArgs;
    });

    perSystem = {
      self',
      pkgs,
      config,
      cranePkgs,
      cranePkgsStatic,
      ...
    }: (lib.mkMerge [
      {
        _module.args = {
          cranePkgs = makeCranePkgs pkgs;
          cranePkgsStatic = makeCranePkgs pkgs.pkgsStatic;
        };

        packages = {
          default = self'.packages.attic;

          attic-web-deps = pkgs.stdenvNoCC.mkDerivation {
            pname = "attic-web-deps";
            version = "0.1.0";
            src = ../web;
            nativeBuildInputs = [ pkgs.bun ];

            outputHashAlgo = "sha256";
            outputHashMode = "recursive";
            outputHash = "sha256-oIDDNQX8PW73ZfdUyGiOjK8NIQP9QUyw541GNO9+iHI=";

            dontConfigure = true;
            dontBuild = true;
            dontFixup = true;

            installPhase = ''
              runHook preInstall
              export HOME="$TMPDIR"
              export BUN_INSTALL_CACHE_DIR="$TMPDIR/bun-cache"
              bun install --frozen-lockfile --no-progress
              mkdir -p $out
              cp -R node_modules $out/
              runHook postInstall
            '';
          };

          attic-web = pkgs.stdenvNoCC.mkDerivation {
            pname = "attic-web";
            version = "0.1.0";
            src = ../web;
            nativeBuildInputs = [ pkgs.bun ];

            dontConfigure = true;
            dontFixup = true;

            buildPhase = ''
              runHook preBuild
              cp -R ${self'.packages.attic-web-deps}/node_modules ./node_modules
              chmod -R u+w ./node_modules
              export HOME="$TMPDIR"
              export BUN_INSTALL_CACHE_DIR="$TMPDIR/bun-cache"
              bun ./node_modules/vite/bin/vite.js build
              runHook postBuild
            '';

            installPhase = ''
              runHook preInstall
              mkdir -p $out
              cp -R build/. $out/
              runHook postInstall
            '';
          };

          inherit (cranePkgs)
            attic
            attic-client
            attic-server
          ;

          attic-static = cranePkgsStatic.attic;
          attic-client-static = cranePkgsStatic.attic-client;
          attic-server-static = cranePkgsStatic.attic-server;

          attic-ci-installer = pkgs.callPackage ../ci-installer.nix {
            inherit self;
          };

          book = pkgs.callPackage ../book {
            attic = self'.packages.attic;
          };
        };
      }

      (lib.mkIf pkgs.stdenv.isLinux {
        packages = let
          lazycatConfig = pkgs.writeText "attic-lazycat-server.toml" (builtins.readFile ../server/lazycat-server.toml);
          version = "0.1.0";
        in {
          attic-server-image = pkgs.dockerTools.buildImage {
            name = "attic-server";
            tag = "main";
            copyToRoot = [
              self'.packages.attic-server

              # Debugging utilities for `fly ssh console`
              pkgs.busybox

              # Now required by the fly.io sshd
              pkgs.dockerTools.fakeNss
            ];
            config = {
              Entrypoint = [ "/bin/atticd" ];
              Env = [
                "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
              ];
            };
          };

          attic-lazycat-rootfs = let
            runtimeClosure = pkgs.closureInfo {
              rootPaths = [
                self'.packages.attic-server
                pkgs.busybox
                pkgs.cacert
                pkgs.dockerTools.fakeNss
              ];
            };
          in pkgs.runCommand "attic-lazycat-rootfs-${version}" {} ''
            mkdir -p $out/bin $out/app/web $out/etc/attic $out/etc/ssl/certs $out/lzcapp/var/attic/storage
            xargs -I path cp -a --parents path $out < ${runtimeClosure}/store-paths
            ln -s ${self'.packages.attic-server}/bin/atticd $out/bin/atticd
            ln -s ${self'.packages.attic-server}/bin/atticadm $out/bin/atticadm
            ln -s ${pkgs.busybox}/bin/busybox $out/bin/busybox
            ln -s ${pkgs.busybox}/bin/sh $out/bin/sh
            ln -s ${pkgs.busybox}/bin/mkdir $out/bin/mkdir
            cp -R ${self'.packages.attic-web}/. $out/app/web/
            cp ${lazycatConfig} $out/etc/attic/server.toml
            cp ${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt $out/etc/ssl/certs/ca-bundle.crt
          '';

          attic-lazycat-image = pkgs.dockerTools.buildImage {
            name = "attic-lazycat";
            tag = "${version}-nix";
            copyToRoot = null;
            extraCommands = ''
              cp -a ${self'.packages.attic-lazycat-rootfs}/. .
            '';
            config = {
              Cmd = [
                "/bin/atticd"
                "-f"
                "/etc/attic/server.toml"
              ];
              Env = [
                "ATTIC_WEB_DIR=/app/web"
                "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
              ];
              ExposedPorts = {
                "8080/tcp" = {};
              };
            };
          };

          lpk = pkgs.stdenvNoCC.mkDerivation {
            pname = "cloud.lazycat.app.attic";
            inherit version;
            src = lib.cleanSourceWith {
              src = ./..;
              filter = name: type: let
                base = baseNameOf name;
              in !(type == "directory" && builtins.elem base [
                ".git"
                "target"
                "node_modules"
                "result"
                "build"
              ]);
            };

            nativeBuildInputs = [
              pkgs.python3
            ];

            dontConfigure = true;
            dontBuild = true;
            dontFixup = true;

            installPhase = ''
              runHook preInstall
              mkdir -p $out
              lpk="$out/cloud.lazycat.app.attic-v${version}-nix.lpk"
              python3 scripts/docker_archive_to_lpk.py \
                --src-root . \
                --docker-archive ${self'.packages.attic-lazycat-image} \
                --image-ref attic \
                --output "$lpk"
              test -s "$lpk"
              runHook postInstall
            '';
          };
        };
      })

      (lib.mkIf (pkgs.system == "x86_64-linux") {
        packages = {
          attic-server-image-aarch64 = let
            eval = evalCross {
              system = "aarch64-linux";
              pkgs = pkgs.pkgsCross.aarch64-multiplatform;
            };

          in eval.config.packages.attic-server-image;
        };
      })
    ]);
  };
}
