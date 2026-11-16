{ config, lib, pkgs, ... }:

let
  cfg = config.flux.theme;
  dark = cfg.scheme == "dark";
  share = "${pkgs.flux-theme}/share/flux";
  palette = "${share}/palette-${cfg.scheme}.css";
  wallpaper = "/run/current-system/sw/share/backgrounds/flux/default.jpg";
  sessions = "${config.services.displayManager.sessionData.desktops}/share";
  normalUsers = lib.attrNames (lib.filterAttrs (_: u: u.isNormalUser) config.users.users);
  soleUser = if lib.length normalUsers == 1 then lib.head normalUsers else null;
  greeterUser = config.services.greetd.settings.default_session.user;

  mark = pkgs.runCommand "flux-greeter-mark.png" { nativeBuildInputs = [ pkgs.librsvg ]; } ''
    fg=$(sed -n 's/^@define-color fc_fg \(#[0-9a-f]*\);$/\1/p' ${palette})
    test -n "$fg"
    sed "s/currentColor/$fg/g" ${../site/static/flux-mark.svg} > mark.svg
    rsvg-convert -w 160 -h 160 mark.svg -o $out
  '';

  css = pkgs.runCommand "regreet.css" { } ''
    cat ${palette} > $out
    substitute ${./greeter.css} rules.css --subst-var-by mark ${mark}
    cat rules.css >> $out
  '';

  clock = lib.concatStrings [
    ''<span size="64pt" weight="300" letter_spacing="-1024">%H:%M</span>''
    "\n"
    ''<span size="13pt" weight="500">%A %-d %B</span>''
  ];
in
lib.mkIf (config.flux.profile == "desktop") {
  services.displayManager.regreet = {
    enable = true;
    theme.name = if dark then "Adwaita-dark" else "Adwaita";
    iconTheme = { package = pkgs.adwaita-icon-theme; name = "Adwaita"; };
    cursorTheme = { package = pkgs.adwaita-icon-theme; name = "Adwaita"; };
    font = { package = pkgs.inter; name = cfg.font; size = 11; };
    extraCss = css;
    settings = {
      skip_selection = soleUser != null;
      background = { path = wallpaper; fit = "Cover"; };
      GTK.application_prefer_dark_theme = dark;
      appearance.greeting_msg = "Welcome back";
      commands = {
        reboot = [ "systemctl" "reboot" ];
        poweroff = [ "systemctl" "poweroff" ];
      };
      widget.clock = {
        format = clock;
        resolution = "1s";
        label_width = 320;
      };
    };
  };

  services.greetd = {
    enable = true;
    settings.default_session.command = lib.concatStringsSep " " [
      "${pkgs.coreutils}/bin/env"
      "XDG_DATA_DIRS=${sessions}:/run/current-system/sw/share"
      "XCURSOR_THEME=Adwaita"
      "XCURSOR_SIZE=24"
      "XCURSOR_PATH=/run/current-system/sw/share/icons"
      "GTK_USE_PORTAL=0"
      "${pkgs.dbus}/bin/dbus-run-session"
      "${lib.getExe pkgs.cage}"
      (lib.escapeShellArgs config.services.displayManager.regreet.cageArgs)
      "--"
      (lib.getExe config.services.displayManager.regreet.package)
    ];
  };

  systemd.tmpfiles.settings."10-regreet" = lib.mkIf (soleUser != null) {
    "/var/lib/regreet/state.toml".f = {
      user = greeterUser;
      group = config.users.users.${greeterUser}.group;
      mode = "0644";
      argument = "last_user = \"${soleUser}\"\n\n[user_to_last_sess]\n${soleUser} = \"Niri\"\n";
    };
  };
}
