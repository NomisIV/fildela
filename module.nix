{ lib, config, pkgs, ... }: let
  cfg = config.services.fildela;
in {
  options.services.fildela = with lib; {
    enable = mkEnableOption "fildela HTTP file server";

    openFirewall = mkEnableOption "open the port for fildela in the firewall";

    port = mkOption {
      type = types.port;
      default = 8000;
      description = "The port to serve at";
    };

    dataDir = mkOption {
      type = types.str;
      default = "/var/lib/fildela";
      description = "The location of the file root";
    };

    user = mkOption {
      type = types.str;
      default = "fildela";
      description = "The user to run fildela as";
    };

    group = mkOption {
      type = types.str;
      default = "fildela";
      description = "The group to run fildela as";
    };
  };

  config = lib.mkIf cfg.enable {
    # Main fildela service
    systemd.services.fildela = {
      description = "fildela HTTP file server";
      after = [ "network.target" ];
      requires = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        ExecStart = "${pkgs.fildela}/bin/fildela ${toString cfg.port} ${cfg.dataDir}";
        User = cfg.user;
        Group = cfg.group;
      };
    };

    # Open firewall
    networking.firewall = lib.mkIf cfg.openFirewall {
      allowedTCPPorts = [ cfg.port ];
    };

    # Add user and group
    users.users = lib.mkIf (cfg.user == "fildela") {
      fildela = {
        group = cfg.group;
        home = cfg.dataDir;
        uid = 351;
      };
    };

    users.groups = lib.mkIf (cfg.group == "fildela") {
      fildela.gid = 351;
    };
  };
}
