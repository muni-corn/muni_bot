{
  description = "muni_bot";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix.url = "github:nix-community/fenix";
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nmattia/naersk/master";
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
    utils,
    naersk,
  }: let
    appName = "muni_bot";

    out =
      utils.lib.eachDefaultSystem
      (system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [fenix.overlays.default]; # for rust-analyzer-nightly
        };
        rust = with fenix.packages.${system}; combine [
          default.rustc
          default.cargo
          complete.rust-src
          targets.wasm32-unknown-unknown.latest.rust-std
        ];

        naersk-lib = naersk.lib.${system}.override {
          inherit (rust) cargo rustc;
        };

        nativeBuildInputs =
          [rust]
          ++ (with pkgs; [
            clang
            diesel-cli
            glibc
            leptosfmt
            pkg-config
            trunk
          ]);
        buildInputs = with pkgs; [libressl];
      in {
        # `nix build`
        packages.default = naersk-lib.buildPackage {
          pname = appName;
          root = builtins.path {
            path = ./.;
            name = "${appName}-src";
          };
          inherit nativeBuildInputs buildInputs;
        };

        # `nix run`
        apps.default = utils.lib.mkApp {
          name = appName;
          drv = self.packages."${system}".default;
          exePath = "/bin/${appName}";
        };

        # `nix develop`
        devShells.default = pkgs.mkShell {
          packages =
            nativeBuildInputs
            ++ buildInputs
            ++ (with pkgs; [cargo-watch cargo-outdated rust-analyzer-nightly]);
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          RUST_LOG = "info";
        };
      });
  in
    out
    // {
      overlays.default = final: prev: {
        ${appName} = self.packages.${prev.system}.default;
      };

      nixosModules.default = import ./nix/nixos.nix self;
    };
}
