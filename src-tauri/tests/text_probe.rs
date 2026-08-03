// Temporary diagnostic — verifies the px_scale fix in text.rs.

use ab_glyph::Font as _;
use audiowave_studio_lib::gpu2d::text;

#[test]
fn measure_text_run() {
  let font = text::select_font_for_text_style("monospace", 700.0, false, "PARITY STRESS").unwrap();
  println!("upm={:?}", font.arc.units_per_em());
  for ch in ['M', 'A', ' ', 'I'] {
    let w = text::measure(font, &ch.to_string(), 55.0, 0.0);
    println!("measure '{ch}' @55 spacing0 -> {w:.2}px  (em {:.4})", w / 55.0);
  }
  for (fs, spacing) in [(44.0f32, 3.0f32), (55.0, 3.0), (55.0, 0.0), (44.0, 0.0)] {
    let w = text::measure(font, "PARITY STRESS", fs, spacing);
    println!("measure 'PARITY STRESS' fs={fs} spacing={spacing} -> {w:.2}px");
  }
  println!("ascent @55 = {:.2}", text::ascent(font, 55.0));
}
