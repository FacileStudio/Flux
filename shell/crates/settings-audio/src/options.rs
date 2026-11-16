//! Reading and writing `~/.config/flux/audio.json`. Blocking: call off the GTK thread.

use std::path::PathBuf;

use crate::model::Options;

fn path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("flux").join("audio.json"))
}

/// The saved options, or the defaults when there is no readable file.
pub fn load() -> Options {
    path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|text| Options::parse(&text))
        .unwrap_or_default()
}

/// Save the options, replacing the file in one rename.
pub fn save(options: Options) -> std::io::Result<()> {
    let path = path().ok_or_else(|| std::io::Error::other("no home directory"))?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let partial = path.with_extension("json.partial");
    std::fs::write(&partial, options.to_json())?;
    std::fs::rename(partial, path)
}
