use font_kit::family_name::FamilyName;
use font_kit::handle::Handle;
use font_kit::properties::{Properties, Style, Weight};
use font_kit::source::SystemSource;
use fontdue::{Font, FontSettings};
use std::collections::HashMap;

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct GlyphKey {
    pub c: char,
    pub px: u32,
    pub bold: bool,
}

pub struct GlyphCache {
    font: Font,
    bold_font: Font,
    cache: HashMap<GlyphKey, (Vec<u8>, u32, u32)>,
}

impl GlyphCache {
    pub fn new(family: &str) -> Self {
        let font = load_system_font(family, false)
            .unwrap_or_else(|| load_fallback(false));
        let bold_font = load_system_font(family, true)
            .unwrap_or_else(|| load_fallback(true));
        Self { font, bold_font, cache: HashMap::new() }
    }

    /// Returns (bitmap, width, height) for a glyph.
    pub fn rasterize(&mut self, c: char, px: f32, bold: bool) -> (&[u8], u32, u32) {
        let key = GlyphKey { c, px: px as u32, bold };
        if !self.cache.contains_key(&key) {
            let font = if bold { &self.bold_font } else { &self.font };
            let (metrics, bitmap) = font.rasterize(c, px);
            self.cache.insert(key.clone(), (bitmap, metrics.width as u32, metrics.height as u32));
        }
        let (bmp, w, h) = self.cache.get(&key).unwrap();
        (bmp.as_slice(), *w, *h)
    }
}

fn load_system_font(family: &str, bold: bool) -> Option<Font> {
    let source = SystemSource::new();
    let family_name = FamilyName::Title(family.to_string());

    let mut props = Properties::new();
    if bold {
        props.weight = Weight::BOLD;
    } else {
        props.weight = Weight::NORMAL;
    }
    props.style = Style::Normal;

    let handle = source.select_best_match(&[family_name, FamilyName::Monospace], &props).ok()?;
    let bytes = font_bytes(handle)?;
    let font = Font::from_bytes(bytes.as_slice(), FontSettings::default()).ok()?;
    log::info!("Loaded {} font: {}", if bold { "bold" } else { "regular" }, family);
    Some(font)
}

fn font_bytes(handle: Handle) -> Option<Vec<u8>> {
    match handle {
        Handle::Path { path, .. } => std::fs::read(&path).ok().or_else(|| {
            log::warn!("Could not read font file: {}", path.display());
            None
        }),
        Handle::Memory { bytes, .. } => Some(bytes.to_vec()),
    }
}

fn load_fallback(bold: bool) -> Font {
    let data: &[u8] = if bold {
        include_bytes!("../../assets/JetBrainsMono-Bold.ttf")
    } else {
        include_bytes!("../../assets/JetBrainsMono-Regular.ttf")
    };
    Font::from_bytes(data, FontSettings::default()).expect("embedded fallback font failed")
}
