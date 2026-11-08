//! `Family::Mono` when the mono slot holds no monospaced face: the app named none (the slot
//! defaults to the sans), or named one it didn't register, so the slot fell to the sans. Mono
//! text and mono spans then shape with `Family::SystemMono`'s families first, as CSS
//! `monospace` would, rather than in a proportional face.

use super::*;

impl TextSystem {
    /// Whether mono text puts `Family::SystemMono`'s families before the mono slot's. Asked
    /// once per slot: a font query, and on its first `true` the system mono fonts load.
    pub(super) fn mono_falls_back(&mut self) -> bool {
        if let Some(falls_back) = self.mono_fallback {
            return falls_back;
        }
        let falls_back = !self.slot_system_mono && !self.mono_slot_is_monospaced();
        self.mono_fallback = Some(falls_back);
        falls_back
    }

    fn mono_slot_is_monospaced(&mut self) -> bool {
        let names: Vec<String> = FontFamilyName::parse_css_list(&self.mono)
            .flatten()
            .filter_map(|f| match f { FontFamilyName::Named(name) => Some(name.into_owned()), _ => None })
            .collect();
        let mut query = self.font_cx.collection.query(&mut self.font_cx.source_cache);
        query.set_families(names.iter().map(|name| parley::fontique::QueryFamily::Named(name)));
        query.set_attributes(parley::fontique::Attributes::new(parley::fontique::FontWidth::NORMAL, FontStyle::Normal, FontWeight::NORMAL));
        query
            .first_available_font()
            .and_then(|f| FontRef::from_index(f.blob.as_ref(), f.index as usize).map(is_monospaced))
            .unwrap_or(false)
    }
}

/// A fixed-pitch face: its `post` table says so, or `i` and `M` advance alike.
pub(super) fn is_monospaced(font: FontRef) -> bool {
    if font.metrics(&[]).is_monospace {
        return true;
    }
    let (map, advances) = (font.charmap(), font.glyph_metrics(&[]));
    let (i, m) = (map.map('i'), map.map('M'));
    i != 0 && m != 0 && (advances.advance_width(i) - advances.advance_width(m)).abs() < 1e-3
}
