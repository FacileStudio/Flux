//! The pages' strings in English and French. Other languages read English.

pub struct Text {
    pub code: &'static str,
    pub months: [&'static str; 12],
    pub apply: &'static str,
    pub cancel: &'static str,
    pub change: &'static str,
    pub save: &'static str,
    pub unavailable: &'static str,

    pub system_title: &'static str,
    pub system_subtitle: &'static str,
    pub device_section: &'static str,
    pub device_name: &'static str,
    pub device_name_hint: &'static str,
    pub rename: &'static str,
    pub hostname_invalid: &'static str,
    pub datetime_section: &'static str,
    pub auto_time: &'static str,
    pub auto_time_hint: &'static str,
    pub timezone: &'static str,
    pub timezone_auto: &'static str,
    pub zone_search: &'static str,
    pub zone_automatic: &'static str,
    pub zone_none: &'static str,
    pub clock_24h: &'static str,
    pub clock_24h_hint: &'static str,
    pub language_section: &'static str,
    pub language: &'static str,
    pub formats: &'static str,
    pub formats_hint: &'static str,
    pub formats_same: &'static str,
    pub next_login: &'static str,
    pub login_keyboard: &'static str,
    pub login_keyboard_hint: &'static str,
    pub software_section: &'static str,
    pub software_facile: &'static str,
    pub software_facile_body: &'static str,
    pub software_apps: &'static str,
    pub software_apps_body: &'static str,
    pub software_gaming: &'static str,
    pub software_gaming_body: &'static str,
    pub software_nvidia: &'static str,
    pub software_nvidia_body: &'static str,
    pub software_note: &'static str,
    pub problem_title: &'static str,
    pub problem_body: &'static str,

    pub users_title: &'static str,
    pub users_subtitle: &'static str,
    pub users_section: &'static str,
    pub add_user: &'static str,
    pub administrator: &'static str,
    pub standard: &'static str,
    pub you: &'static str,
    pub admin_hint: &'static str,
    pub full_name: &'static str,
    pub username: &'static str,
    pub username_hint: &'static str,
    pub password: &'static str,
    pub password_again: &'static str,
    pub name_invalid: &'static str,
    pub username_invalid: &'static str,
    pub username_taken: &'static str,
    pub password_empty: &'static str,
    pub password_mismatch: &'static str,
    pub change_password: &'static str,
    pub remove: &'static str,
    pub remove_title: &'static str,
    pub remove_body: &'static str,
    pub last_admin: &'static str,
    pub self_remove: &'static str,
    pub no_users_title: &'static str,
    pub no_users_body: &'static str,

    pub updates_title: &'static str,
    pub updates_subtitle: &'static str,
    pub installed: &'static str,
    pub check: &'static str,
    pub checking: &'static str,
    pub up_to_date: &'static str,
    pub available: &'static str,
    pub check_failed: &'static str,
    pub check_hint: &'static str,
    pub update_now: &'static str,
    pub restart: &'static str,
    pub restart_hint: &'static str,
    pub auto_updates: &'static str,
    pub auto_updates_hint: &'static str,
    pub auto_updates_off: &'static str,
    pub previous: &'static str,
    pub previous_hint: &'static str,
    pub current: &'static str,
    pub go_back: &'static str,
    pub go_back_title: &'static str,
    pub go_back_body: &'static str,
    pub clean: &'static str,
    pub clean_hint: &'static str,
    pub clean_button: &'static str,
    pub no_generations: &'static str,
    pub kernel: &'static str,

    pub step_start: &'static str,
    pub step_fetch: &'static str,
    pub step_evaluate: &'static str,
    pub step_build: &'static str,
    pub step_activate: &'static str,
    pub step_bootloader: &'static str,
    pub step_clean: &'static str,
    pub step_password: &'static str,
    pub step_done: &'static str,
    pub details: &'static str,
    pub failed: &'static str,
    pub failed_hint: &'static str,
    pub failed_unknown: &'static str,
    pub failed_auth: &'static str,
    pub failed_admin: &'static str,
    pub failed_unsupported: &'static str,
    pub failed_busy: &'static str,
    pub failed_settings: &'static str,
    pub failed_space: &'static str,
    pub failed_network: &'static str,
    pub dismiss: &'static str,
    pub keep_going: &'static str,
}

pub const EN: Text = Text {
    code: "en",
    months: ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"],
    apply: "Apply",
    cancel: "Cancel",
    change: "Change",
    save: "Save",
    unavailable: "Not available on this system",

    system_title: "System",
    system_subtitle: "Device name, date and time, and language.",
    device_section: "Device",
    device_name: "Device name",
    device_name_hint: "Other devices on your network see this name",
    rename: "Rename",
    hostname_invalid: "Use up to 63 letters, digits and hyphens, not starting or ending with a hyphen",
    datetime_section: "Date and time",
    auto_time: "Set the time automatically",
    auto_time_hint: "Uses network time servers",
    timezone: "Time zone",
    timezone_auto: "Set automatically from your location",
    zone_search: "Search a city or region",
    zone_automatic: "Set automatically",
    zone_none: "No time zone matches",
    clock_24h: "24-hour clock",
    clock_24h_hint: "13:00 rather than 1:00 PM",
    language_section: "Language and formats",
    language: "Language",
    formats: "Formats",
    formats_hint: "Dates, numbers and currency",
    formats_same: "Same as language",
    next_login: "These apply the next time you sign in.",
    login_keyboard: "Keyboard at the login screen",
    login_keyboard_hint: "Used to type your password. The layouts of your session are in Keyboard and mouse.",
    software_section: "Software",
    software_facile: "Facile tools",
    software_facile_body: "kit, the Facile package manager, with Sablier, Nuage, Casier, Bulle and the other Facile tools.",
    software_apps: "Everyday apps",
    software_apps_body: "Firefox, Files, a text editor, a calculator and viewers for images, PDFs and videos. Turn off for a minimal desktop.",
    software_gaming: "Gaming",
    software_gaming_body: "Steam with Proton, GameMode, Gamescope and MangoHud. Some online games block Linux with their anti-cheat.",
    software_nvidia: "NVIDIA driver",
    software_nvidia_body: "This computer has an NVIDIA graphics card. Its own driver is much faster than the free one.",
    software_note: "Gaming and the NVIDIA driver download from the internet. This can take a while.",
    problem_title: "The settings file has an error",
    problem_body: "Flux cannot read /etc/flux/settings.json. Changes made here will fail until the file is fixed.",

    users_title: "Users",
    users_subtitle: "Who can sign in to this device.",
    users_section: "Accounts",
    add_user: "Add user",
    administrator: "Administrator",
    standard: "Standard",
    you: "You",
    admin_hint: "Can change system settings and add users",
    full_name: "Full name",
    username: "Username",
    username_hint: "Used to sign in; it cannot be changed later",
    password: "Password",
    password_again: "Repeat the password",
    name_invalid: "Enter a name of up to 64 characters, without : or ,",
    username_invalid: "Start with a lowercase letter, then lowercase letters, digits, - or _",
    username_taken: "This username is already taken",
    password_empty: "Enter a password",
    password_mismatch: "The passwords do not match",
    change_password: "Change password",
    remove: "Remove",
    remove_title: "Remove {name}?",
    remove_body: "{name} will no longer be able to sign in. Their files in /home/{user} stay on the disk.",
    last_admin: "The only administrator cannot be removed",
    self_remove: "You are signed in with this account",
    no_users_title: "No accounts yet",
    no_users_body: "Accounts Flux manages appear here.",

    updates_title: "Updates",
    updates_subtitle: "Keep Flux current, or go back to an earlier version.",
    installed: "Released {date} · revision {rev}",
    check: "Check for updates",
    checking: "Checking for updates…",
    up_to_date: "Flux is up to date",
    available: "A new version of Flux is ready to install",
    check_failed: "Could not check for updates. Check your connection and try again.",
    check_hint: "Look for a newer version of Flux",
    update_now: "Update now",
    restart: "Restart to finish",
    restart_hint: "The new version is installed. Restart to start using it.",
    auto_updates: "Update automatically",
    auto_updates_hint: "Once a week, while plugged in",
    auto_updates_off: "Off. Flux only updates when you choose Update now on this page.",
    previous: "Previous versions",
    previous_hint: "Each update keeps the version before it, so you can always go back.",
    current: "In use",
    go_back: "Go back",
    go_back_title: "Go back to {version}?",
    go_back_body: "Your files and settings stay as they are. You can return to the newer version from this list.",
    clean: "Clean up old versions",
    clean_hint: "Keeps the three newest and frees disk space",
    clean_button: "Clean up",
    no_generations: "Previous versions appear here after the first update.",
    kernel: "kernel {kernel}",

    step_start: "Starting…",
    step_fetch: "Looking for the new version",
    step_evaluate: "Preparing the new version",
    step_build: "Downloading and building",
    step_activate: "Switching to the new version",
    step_bootloader: "Updating the boot menu",
    step_clean: "Freeing disk space",
    step_password: "Setting the password",
    step_done: "Done",
    details: "Details",
    failed: "That did not work",
    failed_hint: "Your system was not changed.",
    failed_unknown: "Your system was not changed. Details shows the technical message.",
    failed_auth: "The password was wrong, or this account may not change system settings.",
    failed_admin: "Only an administrator can make this change.",
    failed_unsupported: "This version of Flux cannot make this change. Update Flux, then try again.",
    failed_busy: "Another change is still running. Wait for it to finish, then try again.",
    failed_settings: "The settings file /etc/flux/settings.json has an error, so Flux cannot save changes.",
    failed_space: "The disk is full. Free some space, for example with Clean up in Updates, then try again.",
    failed_network: "Flux could not reach the internet. Check your connection, then try again.",
    dismiss: "Close",
    keep_going: "You can close Settings. The change keeps going and shows here when you come back.",
};

pub const FR: Text = Text {
    code: "fr",
    months: ["janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.", "déc."],
    apply: "Appliquer",
    cancel: "Annuler",
    change: "Modifier",
    save: "Enregistrer",
    unavailable: "Indisponible sur ce système",

    system_title: "Système",
    system_subtitle: "Nom de l'appareil, date et heure, et langue.",
    device_section: "Appareil",
    device_name: "Nom de l'appareil",
    device_name_hint: "Les autres appareils du réseau voient ce nom",
    rename: "Renommer",
    hostname_invalid: "Jusqu'à 63 lettres, chiffres et tirets, sans tiret au début ni à la fin",
    datetime_section: "Date et heure",
    auto_time: "Régler l'heure automatiquement",
    auto_time_hint: "Utilise les serveurs de temps du réseau",
    timezone: "Fuseau horaire",
    timezone_auto: "Réglé automatiquement selon votre position",
    zone_search: "Rechercher une ville ou une région",
    zone_automatic: "Régler automatiquement",
    zone_none: "Aucun fuseau ne correspond",
    clock_24h: "Horloge sur 24 heures",
    clock_24h_hint: "13:00 plutôt que 1:00 PM",
    language_section: "Langue et formats",
    language: "Langue",
    formats: "Formats",
    formats_hint: "Dates, nombres et devises",
    formats_same: "Comme la langue",
    next_login: "Ces réglages s'appliquent à la prochaine connexion.",
    login_keyboard: "Clavier de l'écran de connexion",
    login_keyboard_hint: "Sert à saisir votre mot de passe. Les dispositions de votre session sont dans Clavier et souris.",
    software_section: "Logiciels",
    software_facile: "Outils Facile",
    software_facile_body: "kit, le gestionnaire de paquets Facile, avec Sablier, Nuage, Casier, Bulle et les autres outils Facile.",
    software_apps: "Apps du quotidien",
    software_apps_body: "Firefox, Fichiers, un éditeur de texte, une calculatrice et des visionneuses d'images, de PDF et de vidéos. Désactivez pour un bureau minimal.",
    software_gaming: "Jeux",
    software_gaming_body: "Steam avec Proton, GameMode, Gamescope et MangoHud. Certains jeux en ligne bloquent Linux avec leur anti-triche.",
    software_nvidia: "Pilote NVIDIA",
    software_nvidia_body: "Cet ordinateur a une carte graphique NVIDIA. Son propre pilote est bien plus rapide que le pilote libre.",
    software_note: "Les jeux et le pilote NVIDIA se téléchargent depuis internet. Cela peut prendre un moment.",
    problem_title: "Le fichier de réglages contient une erreur",
    problem_body: "Flux ne peut pas lire /etc/flux/settings.json. Les changements faits ici échoueront tant que le fichier n'est pas corrigé.",

    users_title: "Utilisateurs",
    users_subtitle: "Qui peut se connecter à cet appareil.",
    users_section: "Comptes",
    add_user: "Ajouter un utilisateur",
    administrator: "Administrateur",
    standard: "Standard",
    you: "Vous",
    admin_hint: "Peut modifier les réglages du système et ajouter des utilisateurs",
    full_name: "Nom complet",
    username: "Nom d'utilisateur",
    username_hint: "Sert à se connecter ; il ne pourra plus être changé",
    password: "Mot de passe",
    password_again: "Répétez le mot de passe",
    name_invalid: "Saisissez un nom de 64 caractères au plus, sans : ni ,",
    username_invalid: "Une minuscule d'abord, puis minuscules, chiffres, - ou _",
    username_taken: "Ce nom d'utilisateur est déjà pris",
    password_empty: "Saisissez un mot de passe",
    password_mismatch: "Les mots de passe ne correspondent pas",
    change_password: "Changer le mot de passe",
    remove: "Supprimer",
    remove_title: "Supprimer {name} ?",
    remove_body: "{name} ne pourra plus se connecter. Ses fichiers dans /home/{user} restent sur le disque.",
    last_admin: "Le seul administrateur ne peut pas être supprimé",
    self_remove: "Vous êtes connecté avec ce compte",
    no_users_title: "Aucun compte pour l'instant",
    no_users_body: "Les comptes gérés par Flux apparaissent ici.",

    updates_title: "Mises à jour",
    updates_subtitle: "Gardez Flux à jour, ou revenez à une version précédente.",
    installed: "Publiée le {date} · révision {rev}",
    check: "Rechercher des mises à jour",
    checking: "Recherche de mises à jour…",
    up_to_date: "Flux est à jour",
    available: "Une nouvelle version de Flux est prête à être installée",
    check_failed: "Impossible de rechercher les mises à jour. Vérifiez la connexion et réessayez.",
    check_hint: "Chercher une version plus récente de Flux",
    update_now: "Mettre à jour",
    restart: "Redémarrer pour terminer",
    restart_hint: "La nouvelle version est installée. Redémarrez pour l'utiliser.",
    auto_updates: "Mettre à jour automatiquement",
    auto_updates_hint: "Une fois par semaine, sur secteur",
    auto_updates_off: "Désactivé. Flux ne se met à jour que si vous choisissez Mettre à jour sur cette page.",
    previous: "Versions précédentes",
    previous_hint: "Chaque mise à jour garde la version d'avant : vous pouvez toujours y revenir.",
    current: "En cours",
    go_back: "Revenir",
    go_back_title: "Revenir à {version} ?",
    go_back_body: "Vos fichiers et réglages restent tels quels. Vous pourrez revenir à la version plus récente depuis cette liste.",
    clean: "Nettoyer les anciennes versions",
    clean_hint: "Garde les trois plus récentes et libère de l'espace disque",
    clean_button: "Nettoyer",
    no_generations: "Les versions précédentes apparaissent ici après la première mise à jour.",
    kernel: "noyau {kernel}",

    step_start: "Démarrage…",
    step_fetch: "Recherche de la nouvelle version",
    step_evaluate: "Préparation de la nouvelle version",
    step_build: "Téléchargement et construction",
    step_activate: "Passage à la nouvelle version",
    step_bootloader: "Mise à jour du menu de démarrage",
    step_clean: "Libération de l'espace disque",
    step_password: "Définition du mot de passe",
    step_done: "Terminé",
    details: "Détails",
    failed: "Cela n'a pas fonctionné",
    failed_hint: "Votre système n'a pas été modifié.",
    failed_unknown: "Votre système n'a pas été modifié. Détails affiche le message technique.",
    failed_auth: "Le mot de passe est incorrect, ou ce compte ne peut pas modifier les réglages du système.",
    failed_admin: "Seul un administrateur peut faire ce changement.",
    failed_unsupported: "Cette version de Flux ne peut pas faire ce changement. Mettez Flux à jour, puis réessayez.",
    failed_busy: "Un autre changement est encore en cours. Attendez qu'il se termine, puis réessayez.",
    failed_settings: "Le fichier de réglages /etc/flux/settings.json contient une erreur. Flux ne peut pas enregistrer les changements.",
    failed_space: "Le disque est plein. Libérez de l'espace, par exemple avec Nettoyer dans Mises à jour, puis réessayez.",
    failed_network: "Flux n'a pas pu joindre internet. Vérifiez la connexion, puis réessayez.",
    dismiss: "Fermer",
    keep_going: "Vous pouvez fermer les Réglages. Le changement continue et s'affiche ici à votre retour.",
};

/// The language code a locale value such as `fr_FR.UTF-8` names.
pub fn language_of(value: &str) -> &str {
    value.split(['_', '.', '@']).next().unwrap_or("")
}

/// The strings for the session language: `LC_ALL`, `LC_MESSAGES`, then `LANG`.
pub fn text() -> &'static Text {
    let value = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|key| std::env::var(key).ok())
        .find(|value| !value.is_empty())
        .unwrap_or_default();
    if language_of(&value) == "fr" {
        &FR
    } else {
        &EN
    }
}

/// The name a locale goes by, in its own language.
pub fn locale_name(locale: &str) -> &'static str {
    match locale {
        "en_US.UTF-8" => "English (United States)",
        "en_GB.UTF-8" => "English (United Kingdom)",
        "fr_FR.UTF-8" => "Français (France)",
        "fr_BE.UTF-8" => "Français (Belgique)",
        "fr_CH.UTF-8" => "Français (Suisse)",
        "fr_CA.UTF-8" => "Français (Canada)",
        "es_ES.UTF-8" => "Español (España)",
        "de_DE.UTF-8" => "Deutsch (Deutschland)",
        "de_CH.UTF-8" => "Deutsch (Schweiz)",
        "it_IT.UTF-8" => "Italiano (Italia)",
        "pt_PT.UTF-8" => "Português (Portugal)",
        "pt_BR.UTF-8" => "Português (Brasil)",
        "nl_NL.UTF-8" => "Nederlands (Nederland)",
        _ => "",
    }
}

/// The name a login keyboard layout goes by, in its own language.
pub fn keymap_name(keymap: &str) -> &'static str {
    match keymap {
        "us" => "English (US)",
        "gb" => "English (UK)",
        "fr" => "Français (AZERTY)",
        "de" => "Deutsch",
        "es" => "Español",
        "it" => "Italiano",
        "pt" => "Português",
        "br" => "Português (Brasil)",
        "be" => "Belge",
        "ch" => "Schweiz, Suisse",
        "ca" => "Canadien",
        "nl" => "Nederlands",
        "se" => "Svenska",
        "no" => "Norsk",
        "dk" => "Dansk",
        "fi" => "Suomi",
        "pl" => "Polski",
        "cz" => "Čeština",
        "latam" => "Español (Latinoamérica)",
        _ => "",
    }
}

/// `template` with each `{key}` replaced.
pub fn fill(template: &str, pairs: &[(&str, &str)]) -> String {
    pairs.iter().fold(template.to_string(), |text, (key, value)| text.replace(&format!("{{{key}}}"), value))
}

/// A day such as `16 Nov 2026` from seconds since the epoch.
pub fn date(text: &Text, seconds: u64) -> String {
    let days = i64::try_from(seconds / 86_400).unwrap_or(0);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    let name = text.months[usize::try_from(month - 1).unwrap_or(0)];
    format!("{day} {name} {year}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates() {
        assert_eq!(date(&EN, 0), "1 Jan 1970");
        assert_eq!(date(&EN, 1_794_787_200), "16 Nov 2026");
        assert_eq!(date(&FR, 951_782_400), "29 févr. 2000");
    }

    #[test]
    fn helpers() {
        assert_eq!(language_of("fr_FR.UTF-8"), "fr");
        assert_eq!(language_of("en"), "en");
        assert_eq!(fill("Remove {name}? /home/{user}", &[("name", "Bob"), ("user", "bob")]), "Remove Bob? /home/bob");
        for locale in flux_config::validate::LOCALES {
            assert!(!locale_name(locale).is_empty(), "{locale}");
        }
        for keymap in flux_config::validate::KEYMAPS {
            assert!(!keymap_name(keymap).is_empty(), "{keymap}");
        }
    }
}
