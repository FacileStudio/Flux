{ config, lib, pkgs, ... }:

let
  cfg = config.flux.theme;
  theme = pkgs.flux-theme;
  share = "${theme}/share/flux";
  dark = cfg.scheme == "dark";
  gtkTheme = if dark then "Flux-dark" else "Flux";
  monoFont = "JetBrains Mono";
in
{
  options.flux.theme = {
    font = lib.mkOption {
      type = lib.types.str;
      default = "Inter";
      description = "UI sans-serif family. Magnetik is licensed for web use only: set it here only with a desktop licence and its files installed through fonts.packages.";
    };
    scheme = lib.mkOption {
      type = lib.types.enum [ "light" "dark" ];
      default = "light";
      description = "muse color scheme for /etc/flux/palette.css, GTK, fuzzel and the GNOME color-scheme preference.";
    };
  };

  config = lib.mkIf (config.flux.profile == "desktop") {
    environment.systemPackages = [ theme ];
    environment.pathsToLink = [ "/share/themes" ];

    environment.etc = {
      "flux/palette.css".source = "${share}/palette-${cfg.scheme}.css";
      "flux/gtk-4.0/gtk.css".source = "${share}/gtk-4.0/gtk.css";
      "flux/ghostty/config".text = ''
        theme = light:${share}/ghostty/themes/flux-light,dark:${share}/ghostty/themes/flux-dark
      '';
      "xdg/fuzzel/fuzzel.ini".text = ''
        [main]
        include=${share}/fuzzel/${cfg.scheme}.ini
        font=${cfg.font}:size=12

        [border]
        width=0
        radius=12
      '';
      "xdg/gtk-3.0/settings.ini".text = ''
        [Settings]
        gtk-theme-name=${gtkTheme}
        gtk-font-name=${cfg.font} 11
        gtk-application-prefer-dark-theme=${if dark then "1" else "0"}
      '';
    };

    programs.dconf = {
      enable = true;
      profiles.user.databases = [{
        settings."org/gnome/desktop/interface" = {
          color-scheme = if dark then "prefer-dark" else "prefer-light";
          gtk-theme = gtkTheme;
          font-name = "${cfg.font} 11";
          document-font-name = "${cfg.font} 11";
          monospace-font-name = "${monoFont} 11";
        };
      }];
    };

    systemd.user.tmpfiles.rules = [
      "d %h/.config/gtk-4.0 0755 - - -"
      "L %h/.config/gtk-4.0/gtk.css - - - - /etc/flux/gtk-4.0/gtk.css"
      "d %h/.config/ghostty 0755 - - -"
      "L %h/.config/ghostty/config - - - - /etc/flux/ghostty/config"
    ];

    fonts.packages = with pkgs; [ inter jetbrains-mono ];
    fonts.fontconfig.defaultFonts = {
      sansSerif = [ cfg.font ];
      monospace = [ monoFont ];
    };
  };
}
