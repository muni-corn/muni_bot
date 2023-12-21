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
        fenix' = fenix.packages.${system};

        rust = fenix'.default;
        naersk-lib = naersk.lib.${system}.override {
          inherit (rust) cargo rustc;
        };

        nativeBuildInputs = with pkgs; [
          rust.toolchain
          pkg-config
          clang
          diesel-cli
          glibc
        ];
        buildInputs = with pkgs; [libressl_3_6];
      in {
        # `nix build`
        defaultPackage = naersk-lib.buildPackage {
          pname = appName;
          root = builtins.path {
            path = ./.;
            name = "${appName}-src";
          };
          inherit nativeBuildInputs buildInputs;
        };

        # `nix run`
        defaultApp = utils.lib.mkApp {
          name = appName;
          drv = self.defaultPackage."${system}";
          exePath = "/bin/${appName}";
        };

        # `nix develop`
        devShell = pkgs.mkShell {
          packages =
            nativeBuildInputs
            ++ buildInputs
            ++ (with pkgs; [cargo-watch rust-analyzer-nightly]);
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          RUST_SRC_PATH = "${fenix'.complete.rust-src}/lib/rustlib/src/rust/library";
        };
      });
  in
    out
    // {
      overlay = final: prev: {
        ${appName} = self.defaultPackage.${prev.system};
      };
    };
}
