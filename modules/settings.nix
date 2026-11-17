{ config, lib, pkgs, ... }:

let
  rules = lib.importJSON ../shell/crates/flux-config/rules.json;
  file = config.flux.settingsFile;
  present = file != null && builtins.pathExists file;
  raw = if present then lib.importJSON file else { };
  set = lib.mkOverride 90;

  matches = re: s: builtins.isString s && builtins.match re s != null;
  isBool = builtins.isBool;
  unknownKeys = allowed: value:
    if builtins.isAttrs value then lib.subtractLists allowed (builtins.attrNames value) else [ ];

  hostnameRe = "[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?";
  usernameRe = "[a-z_][a-z0-9_-]{0,31}";
  zoneRe = "[A-Za-z0-9_+-]+(/[A-Za-z0-9_+-]+)*";
  fullNameRe = "[^[:space:]:,[:cntrl:]]([^:,[:cntrl:]]*[^[:space:]:,[:cntrl:]])?";

  topKeys = [ "version" "hostname" "timezone" "locale" "formats" "keymap" "users" "updates" ];
  userKeys = [ "name" "fullName" "admin" "groups" ];

  get = name: default: raw.${name} or default;
  users = get "users" [ ];
  usersList = if builtins.isList users then users else [ ];
  user = u: {
    name = u.name or "";
    fullName = u.fullName or "";
    admin = u.admin or false;
    groups = u.groups or [ "networkmanager" "video" "audio" ];
  };
  people = map user (builtins.filter builtins.isAttrs usersList);
  names = map (u: u.name) people;

  where = "settings.json (flux.settingsFile)";
  check = ok: message: { assertion = ok; message = "${where}: ${message}"; };

  validUser = u:
    matches usernameRe u.name
    && !(lib.elem u.name rules.reservedUsers)
    && !(lib.hasPrefix "systemd-" u.name)
    && !(lib.hasPrefix "nixbld" u.name)
    && matches fullNameRe u.fullName
    && builtins.stringLength u.fullName <= 256
    && isBool u.admin
    && builtins.isList u.groups
    && lib.all (g: lib.elem g rules.groups) u.groups;

  userChecks = u: [
    (check (matches usernameRe u.name)
      "user name ${builtins.toJSON u.name} must match ^${usernameRe}$")
    (check (!(lib.elem u.name rules.reservedUsers) && !(lib.hasPrefix "systemd-" u.name) && !(lib.hasPrefix "nixbld" u.name))
      "user name ${builtins.toJSON u.name} is reserved by the system")
    (check (matches fullNameRe u.fullName && builtins.stringLength u.fullName <= 256)
      "fullName of ${builtins.toJSON u.name} must be 1 to 64 characters with no colon, comma, control character or outer space")
    (check (isBool u.admin) "admin of ${builtins.toJSON u.name} must be true or false")
    (check (builtins.isList u.groups && lib.all (g: lib.elem g rules.groups) u.groups)
      "groups of ${builtins.toJSON u.name} must be a list drawn from: ${lib.concatStringsSep " " rules.groups}")
  ];

  hostname = get "hostname" null;
  timezone = get "timezone" null;
  locale = get "locale" null;
  formats = get "formats" null;
  keymap = get "keymap" null;
  updates = get "updates" { };

  validHostname = matches hostnameRe hostname;
  validTimezone = timezone == null || (matches zoneRe timezone && builtins.stringLength timezone <= 64);
  validLocale = lib.elem locale rules.locales;
  validFormats = formats == null || lib.elem formats rules.locales;
  validKeymap = lib.elem keymap rules.keymaps;

  assertions = [
    (check (builtins.isAttrs raw) "the file must hold one JSON object")
    (check (unknownKeys topKeys raw == [ ])
      "unknown keys ${builtins.toJSON (unknownKeys topKeys raw)}; allowed: ${lib.concatStringsSep " " topKeys}")
    (check ((get "version" null) == 1) "version must be 1")
    (check validHostname
      "hostname ${builtins.toJSON hostname} is not a valid device name: 1 to 63 letters, digits or hyphens, not starting or ending with a hyphen")
    (check validTimezone "timezone ${builtins.toJSON timezone} must be null (automatic) or a zone such as Europe/Paris")
    (check validLocale "locale ${builtins.toJSON locale} must be one of: ${lib.concatStringsSep " " rules.locales}")
    (check validFormats "formats ${builtins.toJSON formats} must be null or one of: ${lib.concatStringsSep " " rules.locales}")
    (check validKeymap "keymap ${builtins.toJSON keymap} must be one of: ${lib.concatStringsSep " " rules.keymaps}")
    (check (builtins.isList users && lib.all builtins.isAttrs usersList) "users must be a list of objects")
    (check (lib.all (u: unknownKeys userKeys u == [ ]) usersList)
      "a user has unknown keys; allowed: ${lib.concatStringsSep " " userKeys}")
    (check (lib.length (lib.unique names) == lib.length names) "a user name is listed twice")
    (check (people == [ ] || lib.any (u: u.admin == true) people) "at least one user must be an administrator (admin: true)")
    (check (builtins.isAttrs updates && unknownKeys [ "automatic" ] updates == [ ] && isBool (updates.automatic or false))
      "updates must be { \"automatic\": true|false }")
    (check (people == [ ] || config.users.mutableUsers)
      "users.mutableUsers must stay true: passwords of settings.json users live in /etc/shadow")
  ] ++ lib.concatMap userChecks people;

  managed = builtins.filter validUser people;
in
{
  options.flux.settingsFile = lib.mkOption {
    type = lib.types.nullOr lib.types.path;
    default = null;
    example = lib.literalExpression "./settings.json";
    description = ''
      The settings.json that Flux Settings and flux-config edit, given as a path relative to the
      machine flake (/etc/flux), such as `./settings.json`. A flake evaluates purely and cannot read
      /etc directly, so the machine flake passes its own copy. When the file does not exist yet,
      nothing is applied. Its values win over plain definitions in configuration.nix (priority 90).
    '';
  };

  config = lib.mkMerge [
    {
      environment.systemPackages = [ pkgs.flux-config ];
      users.mutableUsers = lib.mkDefault true;
      security.polkit.enable = lib.mkDefault true;
      security.polkit.extraConfig = ''
        polkit.addRule(function (action, subject) {
          if (action.id == "studio.facile.flux.config" && subject.local && subject.active
              && subject.isInGroup("wheel")) {
            return polkit.Result.AUTH_SELF_KEEP;
          }
        });
      '';
      systemd.tmpfiles.rules = [ "d /run/flux-config 0755 root root -" ];

      systemd.services.flux-auto-update = {
        description = "Flux automatic update";
        unitConfig.ConditionACPower = true;
        after = [ "network-online.target" ];
        wants = [ "network-online.target" ];
        serviceConfig = {
          Type = "oneshot";
          ExecStart = "${pkgs.flux-config}/bin/flux-config update --scheduled";
          Nice = 10;
          IOSchedulingClass = "idle";
        };
      };
      systemd.timers.flux-auto-update = {
        description = "Weekly Flux update, when switched on in Settings";
        wantedBy = [ "timers.target" ];
        timerConfig = {
          OnCalendar = "weekly";
          Persistent = true;
          RandomizedDelaySec = "1h";
        };
      };
    }

    (lib.mkIf present {
      inherit assertions;

      networking.hostName = lib.mkIf validHostname (set hostname);
      time.timeZone = lib.mkIf validTimezone (set timezone);
      services.automatic-timezoned.enable = lib.mkIf (timezone == null) (lib.mkDefault true);

      i18n.defaultLocale = lib.mkIf validLocale (set locale);
      i18n.extraLocaleSettings = lib.mkIf (formats != null && validFormats) (set (lib.genAttrs [
        "LC_ADDRESS" "LC_MEASUREMENT" "LC_MONETARY" "LC_NUMERIC" "LC_PAPER" "LC_TELEPHONE" "LC_TIME"
      ] (_: formats)));

      services.xserver.xkb.layout = lib.mkIf validKeymap (set keymap);
      console.useXkbConfig = lib.mkIf validKeymap (set true);
      environment.sessionVariables.XKB_DEFAULT_LAYOUT = lib.mkIf validKeymap (set keymap);

      programs.zsh.enable = lib.mkIf (managed != [ ]) (lib.mkDefault true);
      users.users = lib.listToAttrs (map
        (u: lib.nameValuePair u.name {
          isNormalUser = set true;
          description = set u.fullName;
          extraGroups = set (u.groups ++ lib.optional u.admin "wheel");
          shell = set pkgs.zsh;
          hashedPassword = set null;
        })
        managed);
    })
  ];
}
