//! The wallpaper choice: where the images live, which one is current, and how a
//! new choice reaches the `flux-wallpaper` service that draws it.
//!
//! The choice is one file, `~/.config/flux/wallpaper`, holding a path on one
//! line (a symlink to the image is read too). The service reads it on start, so
//! setting a wallpaper is writing that file and restarting the service. The
//! panel's picker and the `sidebar wallpaper` subcommand both go through here.

pub mod picker;

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::process::Command;

/// The wallpapers Flux ships, linked into the system profile.
const SYSTEM_DIR: &str = "/run/current-system/sw/share/backgrounds/flux";

/// The image the service falls back to when nothing is chosen.
const DEFAULT_NAME: &str = "default.jpg";

/// The systemd user unit that draws the wallpaper.
const SERVICE: &str = "flux-wallpaper.service";

/// The file extensions the picker offers, compared case-insensitively: the
/// formats the shell's pixbuf loaders and swaybg both decode.
pub const EXTENSIONS: [&str; 3] = ["jpg", "jpeg", "png"];

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// The user's own folder, `~/Pictures/Wallpapers`, where added images go.
pub fn user_dir() -> Option<PathBuf> {
    Some(home()?.join("Pictures/Wallpapers"))
}

/// The file that records the choice, `~/.config/flux/wallpaper`.
fn choice_file() -> Option<PathBuf> {
    Some(home()?.join(".config/flux/wallpaper"))
}

/// The folders the picker lists: the system set, then the user's own.
pub fn dirs() -> Vec<PathBuf> {
    let mut dirs = vec![PathBuf::from(SYSTEM_DIR)];
    dirs.extend(user_dir());
    dirs
}

/// Whether `path` names an image the picker offers, by extension.
fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str()))
}

/// Every image directly inside `dirs`, each folder sorted by file name, in the
/// order the folders are given. A missing folder lists nothing, and a symlink
/// to an image already listed (the shipped `default.jpg`) is left out.
pub fn list(dirs: &[PathBuf]) -> Vec<PathBuf> {
    let images: Vec<PathBuf> = dirs
        .iter()
        .flat_map(|dir| {
            let mut images: Vec<PathBuf> = std::fs::read_dir(dir)
                .into_iter()
                .flatten()
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path.is_file() && is_image(path))
                .collect();
            images.sort();
            images
        })
        .collect();
    let targets: Vec<PathBuf> = images
        .iter()
        .filter(|path| !path.is_symlink())
        .filter_map(|path| std::fs::canonicalize(path).ok())
        .collect();
    images
        .into_iter()
        .filter(|path| {
            !path.is_symlink()
                || std::fs::canonicalize(path).map_or(true, |target| !targets.contains(&target))
        })
        .collect()
}

/// The chosen image as recorded in `file`: a symlink's target, or the first
/// line of a regular file. `None` when the record is missing or empty.
fn read_choice(file: &Path) -> Option<PathBuf> {
    if file.is_symlink() {
        return std::fs::read_link(file).ok();
    }
    let text = std::fs::read_to_string(file).ok()?;
    let line = text.lines().next()?.trim();
    (!line.is_empty()).then(|| PathBuf::from(line))
}

/// The wallpaper on screen now: the recorded choice when it still exists, the
/// shipped default otherwise.
pub fn current() -> PathBuf {
    choice_file()
        .and_then(|file| read_choice(&file))
        .filter(|path| path.is_file())
        .unwrap_or_else(|| Path::new(SYSTEM_DIR).join(DEFAULT_NAME))
}

/// Record `image` in `file` as one line, replacing any symlink there, through a
/// temporary file so a reader never sees half a path.
fn write_choice(file: &Path, image: &Path) -> std::io::Result<()> {
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let staged = file.with_extension("tmp");
    std::fs::write(&staged, format!("{}\n", image.display()))?;
    std::fs::rename(&staged, file)
}

/// The path `set` records for `image`: absolute, but with its symlinks kept.
///
/// A shipped wallpaper is listed as `/run/current-system/sw/share/...`, a link
/// into the Nix store. Recording the store path would break after an update
/// and a garbage collection; the profile path keeps pointing at the image.
pub fn recorded(image: &Path) -> Result<PathBuf, String> {
    let image = std::path::absolute(image).map_err(|error| format!("{}: {error}", image.display()))?;
    if !image.is_file() || !is_image(&image) {
        return Err(format!("{}: not an image file", image.display()));
    }
    Ok(image)
}

/// Make `image` the wallpaper: record it and restart the service.
pub fn set(image: &Path) -> Result<(), String> {
    let image = recorded(image)?;
    let file = choice_file().ok_or("HOME is not set")?;
    write_choice(&file, &image).map_err(|error| format!("{}: {error}", file.display()))?;
    let status = Command::new("systemctl")
        .args(["--user", "restart", SERVICE])
        .status()
        .map_err(|error| format!("systemctl: {error}"))?;
    if !status.success() {
        return Err(format!("saved, but {SERVICE} did not restart"));
    }
    Ok(())
}

/// Copy `source` into the user's folder and return the copy's path, so an
/// image added from anywhere stays listed after the original moves.
pub fn import(source: &Path) -> Result<PathBuf, String> {
    let dir = user_dir().ok_or("HOME is not set")?;
    let name = source.file_name().ok_or("the file has no name")?;
    std::fs::create_dir_all(&dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    let target = dir.join(name);
    if std::fs::canonicalize(source).ok() != std::fs::canonicalize(&target).ok() {
        std::fs::copy(source, &target).map_err(|error| format!("{}: {error}", target.display()))?;
    }
    Ok(target)
}

/// `sidebar wallpaper <list|set FILE|current>`, the command-line side of the
/// picker. Returns the process exit code.
pub fn cli(args: &[String]) -> i32 {
    match args {
        [command] if command == "list" => {
            list(&dirs()).iter().for_each(|path| println!("{}", path.display()));
            0
        }
        [command] if command == "current" => {
            println!("{}", current().display());
            0
        }
        [command, file] if command == "set" => match set(Path::new(file)) {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("flux-wallpaper: {error}");
                1
            }
        },
        _ => {
            eprintln!("usage: flux-wallpaper <list|current|set FILE>");
            2
        }
    }
}
