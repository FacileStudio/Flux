//! The shared sheet checks.
//!
//! [`CSS`] is the one definition of every Flux control, so these read it, and
//! the Settings sheets that sit on top of it, as text: a rule GTK cannot match
//! or a second definition of a shared control fails here instead of showing up
//! as an unstyled widget.

use std::cell::RefCell;
use std::rc::Rc;

use super::CSS;

/// The page sheets that load above the shared one.
const SHEETS: [&str; 6] = ["settings", "settings-audio", "settings-bluetooth", "settings-display", "settings-network", "settings-system"];

fn sheet(name: &str) -> String {
    let path = format!("{}/../{name}/style.css", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("cannot read {path}: {error}"))
}

/// Every selector of `css`, one per comma-separated entry, without the declarations.
fn selectors(css: &str) -> Vec<String> {
    css.split('}')
        .filter_map(|rule| rule.split_once('{').map(|(head, _)| head))
        .filter(|head| !head.trim_start().starts_with('@'))
        .flat_map(|head| head.split(','))
        .map(|selector| selector.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|selector| !selector.is_empty())
        .collect()
}

/// The node names a selector uses: each compound's leading word, when it has one.
fn nodes(selector: &str) -> Vec<&str> {
    selector
        .split([' ', '>', '+', '~'])
        .map(|compound| compound.split(['.', ':', '[']).next().unwrap_or(""))
        .filter(|node| !node.is_empty() && *node != "*")
        .collect()
}

/// What GTK refuses to parse in `document`. Built through `glib::Object::new` so no display is needed.
fn parse_errors(document: &str) -> Vec<String> {
    let errors = Rc::new(RefCell::new(Vec::new()));
    let sink = errors.clone();
    let provider: gtk4::CssProvider = gtk4::glib::Object::new();
    provider.connect_parsing_error(move |_, _, error| sink.borrow_mut().push(error.to_string()));
    provider.load_from_data(document);
    let collected = errors.borrow().clone();
    collected
}

#[test]
fn selectors_split_rules_and_lists() {
    let found = selectors("a,\n b > c { x: y; }\n@keyframes k { from { o: 0; } }\n.d:hover { z: 1; }");
    assert!(found.contains(&"a".to_string()));
    assert!(found.contains(&"b > c".to_string()));
    assert!(found.contains(&".d:hover".to_string()));
    assert_eq!(nodes("expander-widget > box > title:hover"), ["expander-widget", "box", "title"]);
    assert_eq!(nodes(".fx-sys entry.password"), ["entry"]);
}

/// `passwordentry`, `searchentry` and `expander title` match nothing in GTK 4: the nodes are `entry.password`, `entry.search` and `expander-widget > box > title`.
#[test]
fn no_sheet_names_a_node_gtk_does_not_have() {
    let mut sheets = vec![("shell-common ui".to_string(), CSS.to_string())];
    sheets.extend(SHEETS.iter().map(|name| (name.to_string(), sheet(name))));
    for (name, css) in &sheets {
        for selector in selectors(css) {
            let names = nodes(&selector);
            assert!(!names.contains(&"passwordentry") && !names.contains(&"searchentry"), "{name}: `{selector}` never matches");
            let arrow = names.iter().position(|node| *node == "expander");
            assert!(arrow.is_none_or(|index| index > 0 && names[index - 1] == "title"), "{name}: `{selector}` takes the arrow for the expander");
        }
    }
}

/// The shared sheet defines button, entry, switch, list row, tag, banner and card once; a page sheet that restates one under its root class drifts from it.
#[test]
fn page_sheets_do_not_redefine_shared_controls() {
    let shared = ["switch", "switch slider", "entry", "button.fx-button", "dropdown > button", ".fx-tag", ".fx-banner", ".fx-card", ".fx-row", "list.fx-list > row"];
    let roots = [".fx-sys ", ".fx-sound ", ".fx-network ", ".fx-display ", ".bt-page "];
    for name in SHEETS {
        for selector in selectors(&sheet(name)) {
            let own = roots.iter().find_map(|root| selector.strip_prefix(root)).unwrap_or(selector.as_str());
            assert!(!shared.contains(&own), "{name}: `{selector}` restates a shared control");
        }
    }
}

/// The type scale has no 13 px, and nothing draws a border or dims with opacity instead of the muted colour.
#[test]
fn sheets_keep_to_the_spec() {
    let mut sheets = vec![("shell-common ui".to_string(), CSS.to_string())];
    sheets.extend(SHEETS.iter().map(|name| (name.to_string(), sheet(name))));
    for (name, css) in &sheets {
        for size in ["13px", "15px", "20px"] {
            assert!(!css.contains(&format!("font-size: {size}")), "{name}: {size} is not on the type scale");
        }
        assert!(!css.contains("border:") && !css.contains("border-color"), "{name}: borders are not allowed");
        assert!(!css.contains("opacity: 0.5") && !css.contains("opacity: 0.6"), "{name}: disabled is a colour, not an opacity");
        assert!(!css.contains('#'), "{name}: colours come from the palette");
    }
}

#[test]
fn sheets_parse_on_top_of_the_base() {
    let _gtk = crate::theme::tests::gtk();
    let base = format!("{}\n{}\n{CSS}", crate::palette::FALLBACK, crate::theme::BASE_CSS);
    assert_eq!(parse_errors(&base), Vec::<String>::new(), "the shared sheet must parse");
    for name in SHEETS {
        let document = format!("{base}\n{}", sheet(name));
        assert_eq!(parse_errors(&document), Vec::<String>::new(), "{name} style.css does not parse on top of the shared sheet");
    }
}
