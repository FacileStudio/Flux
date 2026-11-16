//! The xkb registry (`rules/evdev.xml` from xkeyboard-config): layouts, their
//! variants and the layout-switch shortcuts, plus the system layout NixOS
//! writes for X.

use super::config::Layout;

/// One layout or variant: its xkb name and English description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub name: String,
    pub description: String,
}

/// A layout and the variants it offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutEntry {
    pub item: Item,
    pub variants: Vec<Item>,
}

/// What the pages need from the registry.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Registry {
    pub layouts: Vec<LayoutEntry>,
    pub switches: Vec<Item>,
}

/// The layout-switch shortcuts offered, in this order, when the registry
/// lists them.
pub const SWITCHES: [&str; 7] = [
    "grp:alt_shift_toggle",
    "grp:win_space_toggle",
    "grp:ctrl_shift_toggle",
    "grp:alt_space_toggle",
    "grp:caps_toggle",
    "grp:shifts_toggle",
    "grp:ctrl_alt_toggle",
];

fn child<'a, 'i>(node: roxmltree::Node<'a, 'i>, tag: &str) -> Option<roxmltree::Node<'a, 'i>> {
    node.children().find(|n| n.has_tag_name(tag))
}

fn config_item(node: roxmltree::Node) -> Option<Item> {
    let item = child(node, "configItem")?;
    let name = child(item, "name")?.text()?.trim().to_string();
    let description = child(item, "description").and_then(|d| d.text()).unwrap_or(&name).trim().to_string();
    (!name.is_empty()).then_some(Item { name, description })
}

fn layout_entry(node: roxmltree::Node) -> Option<LayoutEntry> {
    let item = config_item(node)?;
    let mut variants: Vec<Item> = child(node, "variantList")
        .map(|list| list.children().filter(|n| n.has_tag_name("variant")).filter_map(config_item).collect())
        .unwrap_or_default();
    variants.sort_by(|a, b| a.description.cmp(&b.description));
    Some(LayoutEntry { item, variants })
}

/// Parse `evdev.xml`. Layouts come sorted by description.
pub fn parse_registry(xml: &str) -> Result<Registry, String> {
    let options = roxmltree::ParsingOptions { allow_dtd: true, ..roxmltree::ParsingOptions::default() };
    let document = roxmltree::Document::parse_with_options(xml, options).map_err(|error| error.to_string())?;
    let root = document.root_element();
    let mut layouts: Vec<LayoutEntry> = child(root, "layoutList")
        .map(|list| list.children().filter(|n| n.has_tag_name("layout")).filter_map(layout_entry).collect())
        .unwrap_or_default();
    layouts.sort_by(|a, b| a.item.description.cmp(&b.item.description));
    let options: Vec<Item> = child(root, "optionList")
        .into_iter()
        .flat_map(|list| list.descendants().filter(|n| n.has_tag_name("option")))
        .filter_map(config_item)
        .collect();
    let switches = SWITCHES
        .iter()
        .filter_map(|name| options.iter().find(|option| option.name == *name).cloned())
        .collect();
    Ok(Registry { layouts, switches })
}

impl Registry {
    pub fn layout(&self, name: &str) -> Option<&LayoutEntry> {
        self.layouts.iter().find(|entry| entry.item.name == name)
    }
}

/// Split xkb's comma lists into layouts, padding missing variants.
pub fn zip_layouts(layouts: &str, variants: &str) -> Vec<Layout> {
    let variants: Vec<&str> = variants.split(',').map(str::trim).collect();
    layouts
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .enumerate()
        .map(|(index, name)| Layout {
            layout: name.to_string(),
            variant: variants.get(index).copied().unwrap_or_default().to_string(),
        })
        .collect()
}

fn xorg_option(text: &str, key: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|line| line.starts_with("Option"))
        .find_map(|line| {
            let mut parts = line.split('"').skip(1);
            let name = parts.next()?;
            parts.next();
            let value = parts.next()?;
            name.eq_ignore_ascii_case(key).then(|| value.to_string())
        })
        .unwrap_or_default()
}

/// The system layouts from `/etc/X11/xorg.conf.d/00-keyboard.conf`, which
/// NixOS writes from `services.xserver.xkb` and locale1 serves to niri.
pub fn parse_xorg_keyboard(text: &str) -> (Vec<Layout>, String) {
    let layouts = zip_layouts(&xorg_option(text, "XkbLayout"), &xorg_option(text, "XkbVariant"));
    (layouts, xorg_option(text, "XkbOptions"))
}

/// The `grp:` option inside an xkb options list.
pub fn switch_option(options: &str) -> String {
    options
        .split(',')
        .map(str::trim)
        .find(|option| option.starts_with("grp:"))
        .unwrap_or_default()
        .to_string()
}
