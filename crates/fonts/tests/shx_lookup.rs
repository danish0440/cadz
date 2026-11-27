//! How SHX fonts are found: drawing folders, the search path, the font map, an app's font
//! provider and FONTALT. Process-wide settings, so one test in its own binary. The fonts are
//! compiled here from shape source written for the test; nothing is read from a CAD install.

use std::sync::Arc;

use cadz_fonts::shx::{self, FontProvider};
use cadz_fonts::{TextFont, ttf};

const FONT: &str = "*0,4,lookup test\n6,2,0,0\n*65,4,UCA\n1,064,0\n";

struct Provider;

impl FontProvider for Provider {
    fn load(&self, file_name: &str) -> Option<Vec<u8>> {
        (file_name == "cadz-provided.shx").then(|| shx::compile(FONT).ok()).flatten()
    }
}

#[test]
fn fonts_are_found_where_the_drawing_and_the_app_keep_them() {
    let bytes = shx::compile(FONT).unwrap();
    let dir = std::env::temp_dir().join(format!("cadz-shx-lookup-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("cadz-beside.shx"), &bytes).unwrap();

    // Beside the drawing: found once the drawing's folder is known; a later change of folders
    // starts a new generation (viewers regenerate).
    assert!(shx::find("cadz-beside.shx").is_none());
    let g = shx::generation();
    shx::add_drawing_dir(&dir);
    assert!(shx::generation() != g);
    assert_eq!(shx::drawing_dirs().first(), Some(&dir));
    assert!(shx::find("CADZ-BESIDE").is_some_and(|f| f.has_shape(65)));
    assert!(TextFont::resolve("cadz-beside.shx").is_shx());
    // A file whose name has another case than the drawing's (found on case-sensitive systems
    // too, by looking through the folder).
    std::fs::write(dir.join("CadZ-Mixed.SHX"), &bytes).unwrap();
    assert!(shx::find("cadz-mixed.shx").is_some());

    // The search path, and the font map renaming a font to one that is there.
    let other = dir.join("lib");
    std::fs::create_dir_all(&other).unwrap();
    std::fs::write(other.join("cadz-onpath.shx"), &bytes).unwrap();
    assert!(shx::find("cadz-onpath.shx").is_none());
    shx::set_search_path(&other.to_string_lossy());
    assert!(shx::find("cadz-onpath.shx").is_some());
    shx::set_font_map("cadz-old.shx;cadz-onpath.shx\nnonsense");
    assert_eq!(shx::font_map(), "cadz-old.shx;cadz-onpath.shx");
    assert!(shx::find("cadz-old").is_some());
    shx::set_font_map("");

    // An app's provider (the web build's file store).
    assert!(shx::find("cadz-provided.shx").is_none());
    shx::set_provider(Some(Arc::new(Provider)));
    assert!(shx::find("cadz-provided.shx").is_some());
    shx::set_provider(None);
    assert!(shx::find("cadz-provided.shx").is_none());

    // FONTALT replaces a missing SHX font (and may name a big font after a comma).
    assert!(!TextFont::resolve("cadz-missing.shx").is_shx());
    ttf::set_font_alt("cadz-beside.shx");
    assert!(TextFont::resolve("cadz-missing.shx").is_shx());
    assert!(!TextFont::resolve("").is_shx(), "the built-in font is not replaced");
    ttf::set_font_alt("");

    // Registered bytes (files opened alongside a drawing on the web) and clearing them.
    shx::register("cadz-registered.shx", &bytes).unwrap();
    assert!(shx::find("cadz-registered.shx").is_some());
    shx::clear_registered();
    assert!(shx::find("cadz-registered.shx").is_none());

    shx::set_search_path("");
    assert_eq!(shx::search_path(), "");
    let _ = std::fs::remove_dir_all(&dir);
}
