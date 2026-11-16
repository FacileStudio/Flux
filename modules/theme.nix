{ config, lib, pkgs, ... }:

let
  cfg = config.flux.theme;
  theme = pkgs.flux-theme;
  share = "${theme}/share/flux";
  dark = cfg.scheme == "dark";
  gtkTheme = if dark then "Flux-dark" else "Flux";
  monoFont = "JetBrains Mono";
  fluxScheme = pkgs.writeShellApplication {
    name = "flux-scheme";
    runtimeInputs = [ pkgs.dconf pkgs.coreutils ];
    text = ''
      current=$(dconf read /org/gnome/desktop/interface/color-scheme || true)
      case "''${1:-toggle}" in
        light) want=light ;;
        dark) want=dark ;;
        toggle) if [ "$current" = "'prefer-dark'" ]; then want=light; else want=dark; fi ;;
        get) if [ "$current" = "'prefer-dark'" ]; then echo dark; else echo light; fi; exit 0 ;;
        *) echo "usage: flux-scheme [light|dark|toggle|get]" >&2; exit 2 ;;
      esac
      if [ "$want" = dark ]; then gtk=Flux-dark; else gtk=Flux; fi
      dconf write /org/gnome/desktop/interface/color-scheme "'prefer-$want'"
      dconf write /org/gnome/desktop/interface/gtk-theme "'$gtk'"
      mkdir -p "$HOME/.config/fuzzel"
      printf '[main]\ninclude=${share}/fuzzel/%s.ini\nfont=${cfg.font}:size=12\n\n[border]\nwidth=0\nradius=12\n' "$want" > "$HOME/.config/fuzzel/fuzzel.ini"
      echo "$want"
    '';
  };
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
      description = "Initial muse color scheme. Users switch at runtime with `flux-scheme light|dark|toggle` or the shell toggle; the shell, libadwaita apps and Ghostty follow the GNOME color-scheme preference live.";
    };
  };

  config = lib.mkIf (config.flux.profile == "desktop") {
    environment.systemPackages = [ theme fluxScheme ];
    environment.pathsToLink = [ "/share/themes" ];

    environment.etc = {
      "flux/palette.css".source = "${share}/palette-${cfg.scheme}.css";
      "flux/palette-light.css".source = "${share}/palette-light.css";
      "flux/palette-dark.css".source = "${share}/palette-dark.css";
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
