self:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  inherit (lib) mkEnableOption mkOption types;

  toml = pkgs.formats.toml { };

  defaultSettings = {
    db = {
      url = "127.0.0.1:7654";
      user = "muni_bot";
    };
  };
in
{
  options.services.munibot = {
    enable = mkEnableOption "munibot";
    settings = mkOption {
      type = toml.type;
      description = "Settings for munibot.";
      default = defaultSettings;
    };
    environmentFile = mkOption {
      type = types.str;
      description = ''
        Path to the environment file for munibot containing secrets for database, Discord, and Twitch authentication.

        munibot requires the following variables to be set: DATABASE_PASS, DISCORD_APPLICATION_ID, DISCORD_CLIENT_SECRET, DISCORD_PUBLIC_KEY, DISCORD_TOKEN, TWITCH_CLIENT_ID, TWITCH_CLIENT_SECRET, and TWITCH_TOKEN.
      '';
    };
  };

  config =
    let
      cfg = config.services.munibot;
    in
    lib.mkIf cfg.enable {
      nixpkgs.overlays = [
        self.overlays.default
      ];

      systemd.services.munibot =
        let
          configFile = toml.generate "munibot.toml" (lib.recursiveUpdate defaultSettings cfg.settings);
        in
        {
          enable = true;

          description = "munibot";
          serviceConfig = {
            EnvironmentFile = cfg.environmentFile;
            ExecStart = "${pkgs.munibot}/bin/munibot --config-file ${configFile}";
            Environment = "RUST_LOG=error,munibot=info";
            PassEnvironment = [
              "DATABASE_PASS"
              "DISCORD_APPLICATION_ID"
              "DISCORD_CLIENT_SECRET"
              "DISCORD_PUBLIC_KEY"
              "DISCORD_TOKEN"
              "TWITCH_CLIENT_ID"
              "TWITCH_CLIENT_SECRET"
              "TWITCH_TOKEN"
            ];
            Restart = "always";
            RestartSec = 10;
            RestartSteps = 5;
            Type = "exec";
            User = "munibot";
            Group = "munibot";
          };
          wantedBy = [ "multi-user.target" ];
        };

      users = {
        groups.munibot = { };
        users.munibot = {
          isSystemUser = true;
          name = "munibot";
          group = "munibot";
        };
      };
    };
}
