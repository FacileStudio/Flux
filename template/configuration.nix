{ ... }:

{
  networking.networkmanager.enable = true;

  i18n.supportedLocales = map (l: "${l}/UTF-8") [
    "C.UTF-8"
    "en_US.UTF-8" "en_GB.UTF-8" "fr_FR.UTF-8" "fr_BE.UTF-8" "fr_CH.UTF-8" "fr_CA.UTF-8" "es_ES.UTF-8"
    "de_DE.UTF-8" "de_CH.UTF-8" "it_IT.UTF-8" "pt_PT.UTF-8" "pt_BR.UTF-8" "nl_NL.UTF-8"
  ];

  system.stateVersion = "25.11";
}
