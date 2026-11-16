{ config, lib, pkgs, ... }:

let
  browser = "firefox.desktop";
  files = "org.gnome.Nautilus.desktop";
  images = "org.gnome.Loupe.desktop";
  documents = "org.gnome.Papers.desktop";
  player = "org.gnome.Showtime.desktop";
  editor = "org.gnome.TextEditor.desktop";
  archives = "org.gnome.FileRoller.desktop";
  terminal = "com.mitchellh.ghostty.desktop";

  types = {
    ${browser} = [
      "text/html"
      "application/xhtml+xml"
      "x-scheme-handler/http"
      "x-scheme-handler/https"
      "x-scheme-handler/about"
      "x-scheme-handler/unknown"
    ];
    ${files} = [ "inode/directory" ];
    ${images} = map (t: "image/${t}") [
      "png" "jpeg" "gif" "webp" "bmp" "tiff" "avif" "heic" "heif" "jxl" "svg+xml" "x-icon" "vnd.microsoft.icon" "x-portable-pixmap"
    ];
    ${documents} = [ "application/pdf" "application/x-pdf" "image/vnd.djvu" "application/vnd.comicbook+zip" "application/x-cbz" ];
    ${player} = map (t: "video/${t}") [
      "mp4" "webm" "x-matroska" "quicktime" "x-msvideo" "mpeg" "ogg" "x-flv" "3gpp" "x-ms-wmv" "mp2t" "x-m4v"
    ] ++ map (t: "audio/${t}") [
      "mpeg" "mp4" "x-m4a" "aac" "flac" "x-flac" "ogg" "x-vorbis+ogg" "opus" "x-opus+ogg" "wav" "x-wav" "webm" "x-matroska"
    ];
    ${editor} = [
      "text/plain"
      "text/markdown"
      "text/x-log"
      "text/csv"
      "text/x-csrc"
      "text/x-chdr"
      "text/x-python"
      "text/x-shellscript"
      "application/x-shellscript"
      "application/json"
      "application/xml"
      "application/toml"
      "application/x-yaml"
    ];
    ${archives} = map (t: "application/${t}") [
      "zip" "x-tar" "gzip" "x-gzip" "x-compressed-tar" "x-bzip2" "x-bzip2-compressed-tar" "x-xz" "x-xz-compressed-tar"
      "zstd" "x-zstd-compressed-tar" "x-7z-compressed" "vnd.rar" "x-rar" "x-rar-compressed"
    ];
  };

  defaults = lib.foldlAttrs
    (acc: app: mimes: acc // lib.genAttrs mimes (_: [ app ]))
    { }
    types;

  hidden = [ "gvim.desktop" "vim.desktop" "vis.desktop" ];
  hiddenEntries = pkgs.runCommand "flux-hidden-entries" { } (''
    mkdir -p $out/share/applications
  '' + lib.concatMapStrings (name: ''
    printf '[Desktop Entry]\nType=Application\nName=${name}\nNoDisplay=true\nHidden=true\n' > $out/share/applications/${name}
  '') hidden);
in
lib.mkIf (config.flux.profile == "desktop") {
  programs.firefox = {
    enable = true;
    preferencesStatus = "default";
    preferences = {
      "widget.use-xdg-desktop-portal.file-picker" = 1;
      "widget.use-xdg-desktop-portal.mime-handler" = 1;
      "widget.use-xdg-desktop-portal.settings" = 1;
      "layout.css.prefers-color-scheme.content-override" = 2;
      "media.ffmpeg.vaapi.enabled" = true;
    };
    policies = {
      DisableTelemetry = true;
      DisableFirefoxStudies = true;
      DontCheckDefaultBrowser = true;
      OverrideFirstRunPage = "";
    };
  };

  environment.systemPackages = (with pkgs; [
    nautilus
    loupe
    papers
    showtime
    gnome-text-editor
    gnome-calculator
    file-roller
    gnome-system-monitor
    baobab
    cliphist
    ffmpegthumbnailer
  ]) ++ [ (lib.hiPrio hiddenEntries) ];

  xdg.mime = {
    enable = true;
    defaultApplications = defaults;
  };

  xdg.terminal-exec = {
    enable = true;
    settings.default = [ terminal ];
  };
}
