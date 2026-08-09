//! Arabic / RTL text handling.
//!
//! NOTE: this module deliberately does NOT do bespoke Arabic joining. Unicode
//! shaping is delegated to the native engines already in the dependency tree:
//!   * GPU/CPU visualizer renderers → HarfBuzz via `rustybuzz`
//!     (`src/gpu2d/text.rs::shape_run`, which feeds `rustybuzz::shape` and
//!     rasterizes the returned glyph ids).
//!   * Slint UI labels → Slint's own swash-based text stack.
//! Both shape Arabic correctly (contextual joining, Lam-Alef ligatures, RTL
//! ordering) from BASE Unicode codepoints. Feeding either engine pre-joined
//! Arabic Presentation Forms (U+FB50–U+FEFF) double-shapes the run: HarfBuzz
//! applies joining features to an already-joined form, so glyphs render
//! disconnected or with phantom joins. The old hand-rolled joining table did
//! exactly that (and only covered a subset of the Arabic/Persian/Urdu blocks),
//! so it has been removed in favour of the native shapers.

/// Does `input` contain any Arabic-script codepoint (base block, Arabic
/// supplement, extended-A, or presentation forms)? Used to route text runs to
/// the shaped rasterization path in `src/gpu2d/text.rs`.
pub fn contains_arabic(input: &str) -> bool {
    input.chars().any(is_arabic_char)
}

fn is_arabic_char(ch: char) -> bool {
    matches!(ch,
        '\u{0600}'..='\u{06FF}' | '\u{0750}'..='\u{077F}' |
        '\u{08A0}'..='\u{08FF}' | '\u{FB50}'..='\u{FDFF}' |
        '\u{FE70}'..='\u{FEFF}'
    )
}

/// Prepare a display string WITHOUT shaping it.
///
/// The returned string keeps base (unjoined) codepoints so the native shaping
/// engines produce correctly joined, ligated and RTL-ordered glyphs. The only
/// transformation is cosmetic: a trailing audio-file extension is stripped so
/// the title bar shows "على النبي" instead of "على النبي.mp3".
pub fn shape_text(input: &str) -> String {
    if let Some(dot_pos) = input.rfind('.') {
        let ext = &input[dot_pos..];
        if matches!(ext.to_lowercase().as_str(), ".mp3" | ".wav" | ".flac" | ".ogg" | ".aac" | ".m4a") {
            return input[..dot_pos].to_string();
        }
    }
    input.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_audio_extension_only() {
        assert_eq!(shape_text("على النبي.wav"), "على النبي");
        assert_eq!(shape_text("على النبي.MP3"), "على النبي");
        // A dotted (non-audio) filename is left intact.
        assert_eq!(shape_text("playlist.v2.txt"), "playlist.v2.txt");
        // Non-Arabic text passes through untouched.
        assert_eq!(shape_text("My Track.mp3"), "My Track");
    }

    #[test]
    fn preserves_base_codepoints_for_native_shaping() {
        // The output must keep the base characters — never pre-joined
        // presentation forms (that would double-shape in the HarfBuzz path).
        let shaped = shape_text("على النبي");
        assert!(contains_arabic(&shaped));
        assert!(shaped.contains('\u{0644}'));
        assert!(!shaped.contains('\u{FEDD}'));
    }
}
