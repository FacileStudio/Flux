//! A container that centres its child and caps its width, the GTK spelling of
//! muse's page column (`max-w-[720px] mx-auto`). GTK 4 alone has no such
//! widget, and libadwaita's `AdwClamp` would pull a whole library in for it.

use std::cell::{Cell, RefCell};

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Clamp {
        pub child: RefCell<Option<gtk4::Widget>>,
        pub maximum: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Clamp {
        const NAME: &'static str = "FxClamp";
        type Type = super::Clamp;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for Clamp {
        fn dispose(&self) {
            if let Some(child) = self.child.take() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for Clamp {
        fn request_mode(&self) -> gtk4::SizeRequestMode {
            gtk4::SizeRequestMode::HeightForWidth
        }

        fn measure(&self, orientation: gtk4::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            let Some(child) = self.child.borrow().clone() else {
                return (0, 0, -1, -1);
            };
            let maximum = self.maximum.get();
            if orientation == gtk4::Orientation::Horizontal {
                let (minimum, natural, _, _) = child.measure(orientation, for_size);
                return (minimum, natural.min(maximum).max(minimum), -1, -1);
            }
            let width = if for_size < 0 {
                for_size
            } else {
                for_size.min(maximum)
            };
            let (minimum, natural, _, _) = child.measure(orientation, width);
            (minimum, natural, -1, -1)
        }

        fn size_allocate(&self, width: i32, height: i32, _baseline: i32) {
            let Some(child) = self.child.borrow().clone() else {
                return;
            };
            let (minimum, _, _, _) = child.measure(gtk4::Orientation::Horizontal, -1);
            let inner = width.min(self.maximum.get()).max(minimum.min(width));
            let x = (width - inner) / 2;
            child.size_allocate(&gtk4::Allocation::new(x, 0, inner, height), -1);
        }
    }
}

glib::wrapper! {
    /// Centres one child and keeps it at most `maximum` pixels wide.
    pub struct Clamp(ObjectSubclass<imp::Clamp>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Clamp {
    /// A clamp that lets its child grow to `maximum` logical pixels.
    pub fn new(maximum: i32) -> Clamp {
        let clamp: Clamp = glib::Object::new();
        clamp.imp().maximum.set(maximum);
        clamp
    }

    /// Replace the child, or remove it with `None`.
    pub fn set_child(&self, child: Option<&impl IsA<gtk4::Widget>>) {
        if let Some(old) = self.imp().child.take() {
            old.unparent();
        }
        if let Some(child) = child {
            child.set_parent(self);
            self.imp().child.replace(Some(child.clone().upcast()));
        }
        self.queue_resize();
    }
}
