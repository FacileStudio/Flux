//! The wallpaper picker at the foot of the panel: a grid of thumbnails over
//! [`super::dirs`], the current one ringed, plus an "Add from file" button.
//!
//! Each thumbnail decodes on GIO's blocking pool and lands on the main loop as
//! a texture, so a folder of multi-megabyte photos never stalls the panel.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use gtk4::{gdk, gio, glib};
use gtk4::{
    ApplicationWindow, Box, Button, ContentFit, FileDialog, FileFilter, FlowBox, Label, Picture,
    PolicyType, ScrolledWindow, SelectionMode,
};

/// A thumbnail's size on screen; three fit across the 360px panel.
const THUMB_WIDTH: i32 = 96;
const THUMB_HEIGHT: i32 = 60;

/// The picker's widgets and the thumbnail buttons it last drew.
pub struct Picker {
    root: Box,
    grid: FlowBox,
    status: Label,
    window: ApplicationWindow,
    tiles: RefCell<Vec<(PathBuf, Button)>>,
}

impl Picker {
    /// Build the picker for the panel `window` and fill it from disk.
    pub fn build(window: &ApplicationWindow) -> Rc<Picker> {
        let root = Box::new(gtk4::Orientation::Vertical, 8);
        root.add_css_class("wallpapers");
        let (header, add) = header();
        let (scroll, grid) = grid();
        let status = Label::new(None);
        status.add_css_class("dim");
        status.set_halign(gtk4::Align::Start);
        status.set_wrap(true);
        status.set_visible(false);

        root.append(&header);
        root.append(&scroll);
        root.append(&status);

        let picker = Rc::new(Picker {
            root,
            grid,
            status,
            window: window.clone(),
            tiles: RefCell::new(Vec::new()),
        });
        let add_picker = picker.clone();
        add.connect_clicked(move |_| Picker::add_from_file(&add_picker));
        Picker::reload(&picker);
        picker
    }

    /// The picker's root, for the panel to place.
    pub fn widget(&self) -> &Box {
        &self.root
    }

    /// Redraw the grid from the folders as they are now.
    pub fn reload(picker: &Rc<Picker>) {
        while let Some(child) = picker.grid.first_child() {
            picker.grid.remove(&child);
        }
        let tiles: Vec<(PathBuf, Button)> = super::list(&super::dirs())
            .into_iter()
            .map(|path| {
                let button = tile(picker, &path);
                picker.grid.insert(&button, -1);
                (path, button)
            })
            .collect();
        *picker.tiles.borrow_mut() = tiles;
        picker.mark_current();
    }

    /// Ring the tile of the wallpaper on screen and only that one.
    fn mark_current(&self) {
        let current = std::fs::canonicalize(super::current()).ok();
        for (path, button) in self.tiles.borrow().iter() {
            if std::fs::canonicalize(path).ok() == current {
                button.add_css_class("current");
            } else {
                button.remove_css_class("current");
            }
        }
    }

    /// Show `message` under the grid, or hide the line when it is `None`.
    fn report(&self, message: Option<String>) {
        self.status.set_visible(message.is_some());
        self.status.set_text(message.as_deref().unwrap_or_default());
    }

    /// Apply `path` off the main loop, then re-mark the grid.
    fn choose(picker: &Rc<Picker>, path: PathBuf) {
        let picker = picker.clone();
        glib::spawn_future_local(async move {
            let outcome = gio::spawn_blocking(move || super::set(&path))
                .await
                .unwrap_or_else(|_| Err("the wallpaper worker stopped".into()));
            picker.report(outcome.err());
            picker.mark_current();
        });
    }

    /// Ask for an image, copy it into `~/Pictures/Wallpapers` and apply it.
    ///
    /// The panel is an overlay layer surface that would sit on top of the
    /// dialog, so it steps aside while the dialog is open.
    fn add_from_file(picker: &Rc<Picker>) {
        let filter = FileFilter::new();
        filter.set_name(Some("Images"));
        filter.add_pixbuf_formats();
        let filters = gio::ListStore::new::<FileFilter>();
        filters.append(&filter);
        let dialog = FileDialog::builder()
            .title("Add a wallpaper")
            .modal(true)
            .filters(&filters)
            .build();

        picker.window.set_visible(false);
        let picker = picker.clone();
        dialog.open(None::<&gtk4::Window>, gio::Cancellable::NONE, move |result| {
            picker.window.present();
            let Some(source) = result.ok().and_then(|file| file.path()) else {
                return;
            };
            match super::import(&source) {
                Ok(copy) => {
                    Picker::reload(&picker);
                    Picker::choose(&picker, copy);
                }
                Err(error) => picker.report(Some(error)),
            }
        });
    }
}

/// The title row: the heading and the "Add from file" button.
fn header() -> (Box, Button) {
    let header = Box::new(gtk4::Orientation::Horizontal, 8);
    let title = Label::new(Some("Wallpaper"));
    title.add_css_class("title");
    title.set_halign(gtk4::Align::Start);
    title.set_hexpand(true);
    let add = Button::with_label("Add from file");
    add.add_css_class("clear");
    header.append(&title);
    header.append(&add);
    (header, add)
}

/// The three-across thumbnail grid, scrolling past a few rows.
fn grid() -> (ScrolledWindow, FlowBox) {
    let grid = FlowBox::new();
    grid.set_selection_mode(SelectionMode::None);
    grid.set_homogeneous(true);
    grid.set_min_children_per_line(3);
    grid.set_max_children_per_line(3);
    grid.set_column_spacing(8);
    grid.set_row_spacing(8);

    let scroll = ScrolledWindow::new();
    scroll.set_child(Some(&grid));
    scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
    scroll.set_propagate_natural_height(true);
    scroll.set_max_content_height(220);
    (scroll, grid)
}

/// One thumbnail button for `path`, its image decoded in the background.
fn tile(picker: &Rc<Picker>, path: &Path) -> Button {
    let picture = Picture::new();
    picture.set_content_fit(ContentFit::Cover);
    picture.set_can_shrink(true);
    picture.set_size_request(THUMB_WIDTH, THUMB_HEIGHT);
    load_thumbnail(&picture, path.to_path_buf());

    let button = Button::new();
    button.set_child(Some(&picture));
    button.add_css_class("wallpaper");
    button.set_tooltip_text(path.file_name().and_then(|name| name.to_str()));
    let picker = picker.clone();
    let chosen = path.to_path_buf();
    button.connect_clicked(move |_| Picker::choose(&picker, chosen.clone()));
    button
}

/// Decode `path` at twice the tile size on the blocking pool, then hand the
/// texture to `picture`. A pixbuf cannot cross threads, so the worker returns
/// its raw pixels. A file that does not decode keeps an empty tile.
fn load_thumbnail(picture: &Picture, path: PathBuf) {
    let picture = picture.clone();
    glib::spawn_future_local(async move {
        let decoded = gio::spawn_blocking(move || {
            let pixbuf =
                Pixbuf::from_file_at_scale(&path, THUMB_WIDTH * 2, THUMB_HEIGHT * 2, true).ok()?;
            let format = if pixbuf.has_alpha() {
                gdk::MemoryFormat::R8g8b8a8
            } else {
                gdk::MemoryFormat::R8g8b8
            };
            let stride = usize::try_from(pixbuf.rowstride()).ok()?;
            Some((pixbuf.read_pixel_bytes(), pixbuf.width(), pixbuf.height(), stride, format))
        })
        .await;
        if let Ok(Some((bytes, width, height, stride, format))) = decoded {
            let texture = gdk::MemoryTexture::new(width, height, format, &bytes, stride);
            picture.set_paintable(Some(&texture));
        }
    });
}
