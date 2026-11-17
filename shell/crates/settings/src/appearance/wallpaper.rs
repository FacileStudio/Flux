//! The wallpaper grid. The list, the current choice and every change go through
//! the `flux-wallpaper list|current|set` command the sidebar ships, so both
//! pickers agree on where images live and how a choice reaches the screen.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gio, glib, Orientation};
use shell_common::ui;

use crate::{i18n, preview};

mod thumbnail;

use thumbnail::{load_thumbnail, sample_texture, THUMB_HEIGHT, THUMB_WIDTH};

const PROGRAM: &str = "flux-wallpaper";

struct Picker {
    holder: gtk4::Box,
    status: gtk4::Label,
    tiles: RefCell<Vec<(PathBuf, gtk4::Button)>>,
}

/// The wallpapers `flux-wallpaper list` printed and the one `current` printed. Blocking.
fn read() -> Result<(Vec<PathBuf>, Option<PathBuf>), String> {
    let list = super::run(PROGRAM, &["list"])?;
    let current = super::run(PROGRAM, &["current"]).ok();
    Ok(parse(&list, current.as_deref()))
}

/// The paths in `list`, one per line, and the trimmed `current` line.
pub fn parse(list: &str, current: Option<&str>) -> (Vec<PathBuf>, Option<PathBuf>) {
    let paths = list
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect();
    let current = current
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from);
    (paths, current)
}

/// Whether `target` already is `source`: the same file, or a copy with the same bytes.
fn same_image(source: &Path, target: &Path) -> bool {
    let resolved = |path: &Path| std::fs::canonicalize(path).ok();
    let same_file = resolved(source).is_some() && resolved(source) == resolved(target);
    same_file || std::fs::read(source).ok().zip(std::fs::read(target).ok()).is_some_and(|(new, old)| new == old)
}

/// Where in `dir` a copy of `source` called `name` goes without replacing a different image:
/// `name` itself when it is free or already that image, otherwise `name-2`, `name-3` and so on.
pub fn free_target(dir: &Path, name: &Path, source: &Path) -> PathBuf {
    let stem = name.file_stem().unwrap_or_default().to_string_lossy().into_owned();
    let extension = name.extension().map(|extension| format!(".{}", extension.to_string_lossy())).unwrap_or_default();
    std::iter::once(dir.join(name))
        .chain((2..).map(|count| dir.join(format!("{stem}-{count}{extension}"))))
        .find(|target| !target.exists() || same_image(source, target))
        .unwrap_or_else(|| dir.join(name))
}

/// Copy `source` into `~/Pictures/Wallpapers` and make it the wallpaper. Blocking.
fn import(source: &Path) -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME").ok_or("HOME is not set")?;
    let dir = PathBuf::from(home).join("Pictures/Wallpapers");
    let name = source.file_name().ok_or("the file has no name")?;
    std::fs::create_dir_all(&dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    let target = free_target(&dir, Path::new(name), source);
    if !same_image(source, &target) {
        std::fs::copy(source, &target).map_err(|error| format!("{}: {error}", target.display()))?;
    }
    super::run(PROGRAM, &["set", &target.to_string_lossy()])?;
    Ok(target)
}

impl Picker {
    fn report(&self, message: Option<String>) {
        self.status.set_visible(message.is_some());
        self.status.set_text(message.as_deref().unwrap_or_default());
    }

    fn mark(&self, current: Option<&Path>) {
        let current = current.and_then(|path| {
            std::fs::canonicalize(path)
                .ok()
                .or_else(|| Some(path.to_path_buf()))
        });
        for (path, button) in self.tiles.borrow().iter() {
            let resolved = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
            if Some(resolved) == current {
                button.add_css_class("current");
            } else {
                button.remove_css_class("current");
            }
        }
    }

    fn choose(picker: &Rc<Picker>, path: PathBuf) {
        picker.mark(Some(&path));
        if preview::active() {
            return;
        }
        let picker = picker.clone();
        glib::spawn_future_local(async move {
            let target = path.to_string_lossy().into_owned();
            let outcome = gio::spawn_blocking(move || super::run(PROGRAM, &["set", &target]))
                .await
                .unwrap_or_else(|_| Err("the wallpaper worker stopped".into()));
            let failed = outcome
                .err()
                .map(|error| i18n::text().wallpaper_failed.replace("{error}", &error));
            let reread = failed.is_some();
            picker.report(failed);
            if reread {
                Picker::reload(&picker);
            }
        });
    }

    fn tile(picker: &Rc<Picker>, path: &Path, index: usize) -> gtk4::Button {
        let picture = gtk4::Picture::new();
        picture.set_content_fit(gtk4::ContentFit::Cover);
        picture.set_can_shrink(true);
        picture.set_size_request(THUMB_WIDTH, THUMB_HEIGHT);
        if preview::active() {
            picture.set_paintable(Some(&sample_texture(index)));
        } else {
            load_thumbnail(&picture, path.to_path_buf());
        }
        let button = gtk4::Button::new();
        button.add_css_class("fx-wallpaper");
        button.set_child(Some(&picture));
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        button.set_tooltip_text(Some(name));
        button.update_property(&[gtk4::accessible::Property::Label(name)]);
        let picker = picker.clone();
        let chosen = path.to_path_buf();
        button.connect_clicked(move |_| Picker::choose(&picker, chosen.clone()));
        button
    }

    fn fill(picker: &Rc<Picker>, paths: Vec<PathBuf>, current: Option<PathBuf>) {
        while let Some(child) = picker.holder.first_child() {
            picker.holder.remove(&child);
        }
        let text = i18n::text();
        if paths.is_empty() {
            picker.holder.append(&ui::empty_state(
                "image-x-generic-symbolic",
                text.wallpaper_none,
                text.wallpaper_none_body,
            ));
            picker.tiles.borrow_mut().clear();
            return;
        }
        let grid = gtk4::FlowBox::new();
        grid.add_css_class("fx-wallpapers");
        grid.set_selection_mode(gtk4::SelectionMode::None);
        grid.set_homogeneous(true);
        grid.set_min_children_per_line(2);
        grid.set_max_children_per_line(4);
        grid.set_column_spacing(12);
        grid.set_row_spacing(12);
        let tiles = paths
            .iter()
            .enumerate()
            .map(|(index, path)| {
                let button = Picker::tile(picker, path, index);
                grid.insert(&button, -1);
                (path.clone(), button)
            })
            .collect();
        *picker.tiles.borrow_mut() = tiles;
        picker.holder.append(&grid);
        picker.mark(current.as_deref());
    }

    fn reload(picker: &Rc<Picker>) {
        if preview::active() {
            let paths = (1..=6)
                .map(|index| PathBuf::from(format!("/preview/wallpaper-{index}.jpg")))
                .collect();
            Picker::fill(
                picker,
                paths,
                Some(PathBuf::from("/preview/wallpaper-2.jpg")),
            );
            return;
        }
        let picker = picker.clone();
        glib::spawn_future_local(async move {
            match gio::spawn_blocking(read).await {
                Ok(Ok((paths, current))) => Picker::fill(&picker, paths, current),
                Ok(Err(error)) => {
                    Picker::fill(&picker, Vec::new(), None);
                    picker.report(Some(error));
                }
                Err(_) => Picker::fill(&picker, Vec::new(), None),
            }
        });
    }

    fn add_from_file(picker: &Rc<Picker>) {
        let text = i18n::text();
        let filter = gtk4::FileFilter::new();
        filter.set_name(Some(text.images));
        filter.add_pixbuf_formats();
        let filters = gio::ListStore::new::<gtk4::FileFilter>();
        filters.append(&filter);
        let dialog = gtk4::FileDialog::builder()
            .title(text.add_wallpaper_title)
            .modal(true)
            .filters(&filters)
            .build();
        let window = picker.holder.root().and_downcast::<gtk4::Window>();
        let picker = picker.clone();
        dialog.open(window.as_ref(), gio::Cancellable::NONE, move |result| {
            let Some(source) = result.ok().and_then(|file| file.path()) else {
                return;
            };
            glib::spawn_future_local(async move {
                let outcome = gio::spawn_blocking(move || import(&source))
                    .await
                    .unwrap_or_else(|_| Err("the wallpaper worker stopped".into()));
                let failed = outcome
                    .err()
                    .map(|error| i18n::text().wallpaper_failed.replace("{error}", &error));
                picker.report(failed);
                Picker::reload(&picker);
            });
        });
    }
}

/// The Wallpaper section: the grid, then a line with any error and the add button.
pub fn section() -> gtk4::Box {
    let text = i18n::text();
    let (root, card) = ui::section(text.wallpaper);
    card.set_spacing(16);
    let holder = gtk4::Box::new(Orientation::Vertical, 0);
    holder.append(&ui::spinner_row(text.loading));
    let footer = gtk4::Box::new(Orientation::Horizontal, 12);
    let status = gtk4::Label::new(None);
    status.add_css_class("fx-error");
    status.set_xalign(0.0);
    status.set_wrap(true);
    status.set_hexpand(true);
    status.set_visible(false);
    let spacer = gtk4::Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    let add = ui::secondary_button(text.add_wallpaper);
    footer.append(&status);
    footer.append(&spacer);
    footer.append(&add);
    card.append(&holder);
    card.append(&footer);
    let picker = Rc::new(Picker {
        holder,
        status,
        tiles: RefCell::new(Vec::new()),
    });
    let adder = picker.clone();
    add.connect_clicked(move |_| Picker::add_from_file(&adder));
    Picker::reload(&picker);
    root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cli_output_becomes_paths() {
        let (paths, current) = parse("/a/one.jpg\n\n/b/two.png\n", Some("/b/two.png\n"));
        assert_eq!(
            paths,
            vec![PathBuf::from("/a/one.jpg"), PathBuf::from("/b/two.png")]
        );
        assert_eq!(current, Some(PathBuf::from("/b/two.png")));
        assert_eq!(parse("", Some("  ")), (Vec::new(), None));
    }

    #[test]
    fn an_import_never_replaces_a_different_image() {
        let dir = std::env::temp_dir().join(format!("flux-wallpaper-test-{}", std::process::id()));
        let library = dir.join("library");
        std::fs::create_dir_all(&library).expect("test directory");
        let source = dir.join("sea.jpg");
        std::fs::write(&source, b"new").expect("source");
        let name = Path::new("sea.jpg");
        assert_eq!(free_target(&library, name, &source), library.join("sea.jpg"));
        std::fs::write(library.join("sea.jpg"), b"old").expect("existing");
        assert_eq!(free_target(&library, name, &source), library.join("sea-2.jpg"));
        std::fs::write(library.join("sea-2.jpg"), b"new").expect("copy");
        assert_eq!(free_target(&library, name, &source), library.join("sea-2.jpg"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
