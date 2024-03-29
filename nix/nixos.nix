self: {
  config,
  lib,
  pkgs,
  ...
}: let
  inherit (lib) mkEnableOption mkOption types;
in {
  options.services.muni_bot = {
    enable = mkEnableOption "muni_bot";
    databaseUrl = mkOption {
      type = types.str;
      description = "URL for the surrealdb instance to connect to.";
      default = "127.0.0.1:7654";
    };
    databaseUser = mkOption {
      type = types.str;
      description = "Username to login with for the surrealdb instance.";
      default = "muni_bot";
    };
    environmentFile = mkOption {
      type = types.str;
      description = ''
        Path to the environment file for muni_bot containing secrets for database, Discord, and Twitch authentication.

        muni_bot requires the following variables to be set: DATABASE_PASS, DISCORD_APPLICATION_ID, DISCORD_CLIENT_SECRET, DISCORD_PUBLIC_KEY, DISCORD_TOKEN, TWITCH_CLIENT_ID, TWITCH_CLIENT_SECRET, TWITCH_TOKEN, and VENTR_ALLOWLIST.
      '';
    };
  };

  config = let
    cfg = config.services.muni_bot;
  in
    lib.mkIf cfg.enable {
      nixpkgs.overlays = [
        self.overlays.default
      ];

      systemd.services.muni_bot = {
        enable = true;

        description = "muni_bot";
        environment = {
          DATABASE_URL = cfg.databaseUrl;
          DATABASE_USER = cfg.databaseUser;
        };
        serviceConfig = {
          EnvironmentFile = cfg.environmentFile;
          ExecStart = "${pkgs.muni_bot}/bin/muni_bot";
          PassEnvironment = [
            "DATABASE_URL"
            "DATABASE_USER"
            "DATABASE_PASS"
            "DISCORD_APPLICATION_ID"
            "DISCORD_CLIENT_SECRET"
            "DISCORD_PUBLIC_KEY"
            "DISCORD_TOKEN"
            "TWITCH_CLIENT_ID"
            "TWITCH_CLIENT_SECRET"
            "TWITCH_TOKEN"
            "VENTR_ALLOWLIST"
          ];
          Restart = "always";
          RestartSec = 10;
          RestartSteps = 5;
          Type = "exec";
          User = "muni_bot";
          Group = "muni_bot";
        };
        wantedBy = ["multi-user.target"];
      };

      users = {
        groups.muni_bot = {};
        users.muni_bot = {
          isSystemUser = true;
          name = "muni_bot";
          group = "muni_bot";
        };
      };
    };
}
