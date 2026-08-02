//! GpuCanvas — a Canvas2D-style retained-mode drawing builder.
//! Every drawing call expands immediately into triangles (CPU mesh).
//! Colors/gradients are computed in local space, positions transformed by the
//! current transform, and the final mesh is rendered by GpuRenderer.

#[derive(Clone, Copy, Debug)]
pub struct Color {
  pub r: f32,
  pub g: f32,
  pub b: f32,
  pub a: f32,
}

impl Color {
  pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
  pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
  pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

  pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color { r, g, b, a }
  }

  pub fn with_alpha(self, a: f32) -> Color {
    Color { r: self.r, g: self.g, b: self.b, a: self.a * a }
  }

  pub fn hex(hex: &str) -> Color {
    let h = hex.trim_start_matches('#');
    let parse = |i: usize| -> f32 {
      let s = &h[i * 2..(i * 2 + 2).min(h.len())];
      u8::from_str_radix(s, 16).unwrap_or(0) as f32 / 255.0
    };
    match h.len() {
      3 => {
        let c = |i: usize| -> f32 {
          let ch = h.chars().nth(i).unwrap_or('0');
          (u8::from_str_radix(&format!("{}{}", ch, ch), 16).unwrap_or(0) as f32) / 255.0
        };
        Color { r: c(0), g: c(1), b: c(2), a: 1.0 }
      }
      8 => Color {
        r: parse(0),
        g: parse(1),
        b: parse(2),
        a: parse(3),
      },
      _ => Color { r: parse(0), g: parse(1), b: parse(2), a: 1.0 },
    }
  }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Transform {
  pub a: f32,
  pub b: f32,
  pub c: f32,
  pub d: f32,
  pub e: f32,
  pub f: f32,
}

impl Transform {
  pub fn identity() -> Transform {
    Transform { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 0.0, f: 0.0 }
  }

  pub fn apply(&self, x: f32, y: f32) -> (f32, f32) {
    (self.a * x + self.c * y + self.e, self.b * x + self.d * y + self.f)
  }

  pub fn translated(tx: f32, ty: f32) -> Transform {
    Transform { a: 1.0, c: 0.0, e: tx, b: 0.0, d: 1.0, f: ty }
  }

  pub fn rotated(rad: f32) -> Transform {
    let (s, c) = rad.sin_cos();
    Transform { a: c, c: -s, e: 0.0, b: s, d: c, f: 0.0 }
  }

  pub fn scaled(sx: f32, sy: f32) -> Transform {
    Transform { a: sx, c: 0.0, e: 0.0, b: 0.0, d: sy, f: 0.0 }
  }

  /// Compose: first `self`, then `other` (matches Canvas order: p' = self(other(p))).
  pub fn then(&self, other: &Transform) -> Transform {
    let m1 = self;
    let m2 = other;
    Transform {
      a: m1.a * m2.a + m1.c * m2.b,
      b: m1.b * m2.a + m1.d * m2.b,
      c: m1.a * m2.c + m1.c * m2.d,
      d: m1.b * m2.c + m1.d * m2.d,
      e: m1.a * m2.e + m1.c * m2.f + m1.e,
      f: m1.b * m2.e + m1.d * m2.f + m1.f,
    }
  }
}

#[derive(Clone, Debug)]
pub struct ColorStop {
  pub t: f32,
  pub color: Color,
}

#[derive(Clone, Debug)]
pub enum Gradient {
  Linear {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    stops: Vec<ColorStop>,
  },
  Radial {
    x0: f32,
    y0: f32,
    r0: f32,
    x1: f32,
    y1: f32,
    r1: f32,
    stops: Vec<ColorStop>,
  },
}

impl Gradient {
  fn sample_stops(stops: &[ColorStop], t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    if stops.is_empty() {
      return Color::BLACK;
    }
    if stops.len() == 1 {
      return stops[0].color;
    }
    if t <= stops[0].t {
      return stops[0].color;
    }
    for pair in stops.windows(2) {
      let (a, b) = (&pair[0], &pair[1]);
      if t <= b.t {
        let span = (b.t - a.t).max(1e-6);
        let f = (t - a.t) / span;
        return Color {
          r: a.color.r + (b.color.r - a.color.r) * f,
          g: a.color.g + (b.color.g - a.color.g) * f,
          b: a.color.b + (b.color.b - a.color.b) * f,
          a: a.color.a + (b.color.a - a.color.a) * f,
        };
      }
    }
    stops.last().unwrap().color
  }

  pub fn sample(&self, x: f32, y: f32) -> Color {
    match self {
      Gradient::Linear { x0, y0, x1, y1, stops } => {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len2 = dx * dx + dy * dy;
        let t = if len2 < 1e-12 {
          0.0
        } else {
          ((x - x0) * dx + (y - y0) * dy) / len2
        };
        Self::sample_stops(stops, t)
      }
      Gradient::Radial { x0, y0, r0, x1, y1, r1, stops } => {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let denom = dx * dx + dy * dy;
        let t = if denom < 1e-12 {
          let d = (x - x0).hypot(y - y0);
          (d - r0) / (r1 - r0).max(1e-6)
        } else {
          let proj = ((x - x0) * dx + (y - y0) * dy) / denom;
          (proj - r0) / (r1 - r0).max(1e-6)
        };
        Self::sample_stops(stops, t)
      }
    }
  }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
  pub position: [f32; 2],
  pub color: [f32; 4],
  pub uv: [f32; 2],
  pub tex_id: f32,
}

impl Vertex {
  fn flat(position: (f32, f32), color: Color) -> Vertex {
    Vertex {
      position: [position.0, position.1],
      color: [color.r, color.g, color.b, color.a],
      uv: [0.0, 0.0],
      tex_id: 0.0,
    }
  }

  fn textured(position: (f32, f32), color: Color, uv: [f32; 2], layer: u32) -> Vertex {
    Vertex {
      position: [position.0, position.1],
      color: [color.r, color.g, color.b, color.a],
      uv,
      tex_id: (layer + 1) as f32,
    }
  }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LineCap {
  Butt,
  Round,
}

#[derive(Clone, Debug)]
pub struct CanvasState {
  pub transform: Transform,
  pub global_alpha: f32,
  pub fill: Fill,
  pub stroke: Fill,
  pub stroke_width: f32,
  pub line_cap: LineCap,
  pub shadow_color: Color,
  pub shadow_blur: f32,
}

#[derive(Clone, Debug)]
pub enum Fill {
  Solid(Color),
  Gradient(Gradient),
}

impl Fill {
  pub fn linear_gradient(x0: f32, y0: f32, x1: f32, y1: f32, stops: &[(f32, Color)]) -> Fill {
    Fill::Gradient(Gradient::Linear {
      x0,
      y0,
      x1,
      y1,
      stops: stops.iter().map(|(t, c)| ColorStop { t: *t, color: *c }).collect(),
    })
  }

  pub fn radial_gradient(x0: f32, y0: f32, r0: f32, x1: f32, y1: f32, r1: f32, stops: &[(f32, Color)]) -> Fill {
    Fill::Gradient(Gradient::Radial {
      x0,
      y0,
      r0,
      x1,
      y1,
      r1,
      stops: stops.iter().map(|(t, c)| ColorStop { t: *t, color: *c }).collect(),
    })
  }
}

impl Default for CanvasState {
  fn default() -> Self {
    CanvasState {
      transform: Transform::identity(),
      global_alpha: 1.0,
      fill: Fill::Solid(Color::BLACK),
      stroke: Fill::Solid(Color::BLACK),
      stroke_width: 1.0,
      line_cap: LineCap::Butt,
      shadow_color: Color::TRANSPARENT,
      shadow_blur: 0.0,
    }
  }
}

pub struct AtlasUpload {
  pub layer: u32,
  pub rgba: Vec<u8>,
  pub width: u32,
  pub height: u32,
}

pub struct GpuCanvas {
  pub width: f32,
  pub height: f32,
  pub clear: Color,
  verts: Vec<Vertex>,
  idx: Vec<u32>,
  stack: Vec<CanvasState>,
  pub state: CanvasState,
  pub segments: u32,
  atlases: Vec<AtlasUpload>,
  next_text_layer: u32,
}

impl GpuCanvas {
  pub fn new(width: u32, height: u32) -> GpuCanvas {
    GpuCanvas {
      width: width as f32,
      height: height as f32,
      clear: Color::BLACK,
      verts: Vec::new(),
      idx: Vec::new(),
      stack: Vec::new(),
      state: CanvasState::default(),
      segments: 64,
      atlases: Vec::new(),
      next_text_layer: 0,
    }
  }

  fn push_tri(&mut self, a: Vertex, b: Vertex, c: Vertex) {
    // Pixel space -> NDC: shader expects positions in [-1, 1] with +y up.
    let to_ndc = |p: [f32; 2]| -> [f32; 2] {
      [p[0] / self.width * 2.0 - 1.0, 1.0 - p[1] / self.height * 2.0]
    };
    let a = Vertex { position: to_ndc(a.position), ..a };
    let b = Vertex { position: to_ndc(b.position), ..b };
    let c = Vertex { position: to_ndc(c.position), ..c };
    let base = self.verts.len() as u32;
    self.verts.push(a);
    self.verts.push(b);
    self.verts.push(c);
    self.idx.push(base);
    self.idx.push(base + 1);
    self.idx.push(base + 2);
  }

  fn push_quad(&mut self, p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), p3: (f32, f32), fill: &Fill, xform: &Transform) {
    let c0 = self.vertex_color(fill, p0.0, p0.1, xform);
    let c1 = self.vertex_color(fill, p1.0, p1.1, xform);
    let c2 = self.vertex_color(fill, p2.0, p2.1, xform);
    let c3 = self.vertex_color(fill, p3.0, p3.1, xform);
    let t0 = xform.apply(p0.0, p0.1);
    let t1 = xform.apply(p1.0, p1.1);
    let t2 = xform.apply(p2.0, p2.1);
    let t3 = xform.apply(p3.0, p3.1);
    self.push_tri(
      Vertex::flat(t0, c0),
      Vertex::flat(t1, c1),
      Vertex::flat(t2, c2),
    );
    self.push_tri(
      Vertex::flat(t0, c0),
      Vertex::flat(t2, c2),
      Vertex::flat(t3, c3),
    );
  }

  fn vertex_color(&self, fill: &Fill, x: f32, y: f32, xform: &Transform) -> Color {
    let base = match fill {
      Fill::Solid(c) => *c,
      Fill::Gradient(g) => g.sample(x, y),
    };
    let local = xform.apply(x, y);
    let _ = local;
    base.with_alpha(self.state.global_alpha)
  }

  // --- state ---

  pub fn save(&mut self) {
    self.stack.push(self.state.clone());
  }

  pub fn restore(&mut self) {
    if let Some(s) = self.stack.pop() {
      self.state = s;
    }
  }

  pub fn translate(&mut self, tx: f32, ty: f32) {
    self.state.transform = self.state.transform.then(&Transform::translated(tx, ty));
  }

  pub fn rotate(&mut self, rad: f32) {
    self.state.transform = self.state.transform.then(&Transform::rotated(rad));
  }

  pub fn scale(&mut self, sx: f32, sy: f32) {
    self.state.transform = self.state.transform.then(&Transform::scaled(sx, sy));
  }

  pub fn set_fill(&mut self, fill: Fill) {
    self.state.fill = fill;
  }

  pub fn set_stroke(&mut self, fill: Fill) {
    self.state.stroke = fill;
  }

  pub fn set_line_width(&mut self, w: f32) {
    self.state.stroke_width = w.max(0.05);
  }

  pub fn set_line_cap(&mut self, cap: LineCap) {
    self.state.line_cap = cap;
  }

  pub fn set_global_alpha(&mut self, a: f32) {
    self.state.global_alpha = a.clamp(0.0, 1.0);
  }

  pub fn set_shadow(&mut self, color: Color, blur: f32) {
    self.state.shadow_color = color;
    self.state.shadow_blur = blur;
  }

  // --- shapes ---

  pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
    if w <= 0.0 || h <= 0.0 {
      return;
    }
    self.draw_shadow(x, y, w, h);
    let fill = self.state.fill.clone();
    let xform = self.state.transform;
    self.push_quad(
      (x, y),
      (x + w, y),
      (x + w, y + h),
      (x, y + h),
      &fill,
      &xform,
    );
  }

  pub fn stroke_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
    let hw = self.state.stroke_width / 2.0;
    let outline = Fill::Solid(self.stroke_sample(0.0, 0.0));
    let prev_fill = self.state.fill.clone();
    self.state.fill = outline;
    self.fill_rect(x - hw, y - hw, w + 2.0 * hw, hw);
    self.fill_rect(x - hw, y + h, w + 2.0 * hw, hw);
    self.fill_rect(x - hw, y, hw, h);
    self.fill_rect(x + w, y, hw, h);
    self.state.fill = prev_fill;
  }

  pub fn fill_circle(&mut self, cx: f32, cy: f32, r: f32) {
    if self.state.shadow_blur > 0.0 && self.state.shadow_color.a > 0.0 {
      self.draw_circle_glow(cx, cy, r);
    }
    self.fill_arc(cx, cy, r, 0.0, std::f32::consts::TAU);
  }

  /// Soft radial glow behind a circle when a shadow is set (Canvas shadowBlur).
  fn draw_circle_glow(&mut self, cx: f32, cy: f32, r: f32) {
    let blur = self.state.shadow_blur;
    let sc = self.state.shadow_color.with_alpha(self.state.shadow_color.a * 0.35);
    let gr = (r + blur * 1.2).max(1.0);
    let g = Gradient::Radial {
      x0: cx,
      y0: cy,
      r0: (r * 0.5).max(1.0),
      x1: cx,
      y1: cy,
      r1: gr,
      stops: vec![
        ColorStop { t: 0.0, color: sc },
        ColorStop { t: 1.0, color: Color::TRANSPARENT },
      ],
    };
    self.save();
    self.state.fill = Fill::Gradient(g);
    self.set_shadow(Color::TRANSPARENT, 0.0);
    self.fill_ellipse(cx, cy, gr, gr);
    self.restore();
  }

  pub fn stroke_circle(&mut self, cx: f32, cy: f32, r: f32) {
    self.stroke_arc(cx, cy, r, 0.0, std::f32::consts::TAU);
  }

  pub fn fill_arc(&mut self, cx: f32, cy: f32, r: f32, a0: f32, a1: f32) {
    if r <= 0.0 {
      return;
    }
    let n = self.segments.max(8);
    let mut a1 = a1;
    while a1 < a0 {
      a1 += std::f32::consts::TAU;
    }
    let steps = ((n as f32 * (a1 - a0) / std::f32::consts::TAU).ceil() as u32).clamp(2, 512);
    let c = self.vertex_color(&self.state.fill, cx, cy, &self.state.transform);
    let center = self.state.transform.apply(cx, cy);
    for i in 0..steps {
      let t0 = a0 + (a1 - a0) * (i as f32 / steps as f32);
      let t1 = a0 + (a1 - a0) * ((i + 1) as f32 / steps as f32);
      let (px, py) = (cx + r * t0.cos(), cy + r * t0.sin());
      let (qx, qy) = (cx + r * t1.cos(), cy + r * t1.sin());
      let cp = self.vertex_color(&self.state.fill, px, py, &self.state.transform);
      let cq = self.vertex_color(&self.state.fill, qx, qy, &self.state.transform);
      let tp = self.state.transform.apply(px, py);
      let tq = self.state.transform.apply(qx, qy);
      self.push_tri(
        Vertex::flat(center, c),
        Vertex::flat(tp, cp),
        Vertex::flat(tq, cq),
      );
    }
  }

  pub fn stroke_arc(&mut self, cx: f32, cy: f32, r: f32, a0: f32, a1: f32) {
    if self.state.shadow_blur > 0.0 && self.state.shadow_color.a > 0.0 {
      let blur = self.state.shadow_blur;
      let sc = self.state.shadow_color;
      let saved_stroke = self.state.stroke.clone();
      let saved_width = self.state.stroke_width;
      let saved_cap = self.state.line_cap;
      let saved_shadow = (self.state.shadow_color, self.state.shadow_blur);
      self.state.line_cap = LineCap::Butt;
      for i in 0..2 {
        self.state.stroke = Fill::Solid(sc.with_alpha(sc.a * (0.12 - 0.04 * i as f32)));
        self.state.stroke_width = saved_width + blur * (1.0 - 0.35 * i as f32);
        self.set_shadow(Color::TRANSPARENT, 0.0);
        self.stroke_arc_impl(cx, cy, r, a0, a1);
      }
      self.state.stroke = saved_stroke;
      self.state.stroke_width = saved_width;
      self.state.line_cap = saved_cap;
      self.set_shadow(saved_shadow.0, saved_shadow.1);
    }
    self.stroke_arc_impl(cx, cy, r, a0, a1);
  }

  fn stroke_arc_impl(&mut self, cx: f32, cy: f32, r: f32, a0: f32, a1: f32) {
    if r <= 0.0 {
      return;
    }
    let n = self.segments.max(8);
    let mut a1 = a1;
    while a1 < a0 {
      a1 += std::f32::consts::TAU;
    }
    let span = a1 - a0;
    let steps = ((n as f32 * span / std::f32::consts::TAU).ceil() as u32).clamp(2, 512);
    let w = self.state.stroke_width;
    let inner = (r - w / 2.0).max(0.0);
    let outer = r + w / 2.0;
    for i in 0..steps {
      let t0 = a0 + span * (i as f32 / steps as f32);
      let t1 = a0 + span * ((i + 1) as f32 / steps as f32);
      let (i0x, i0y) = (cx + inner * t0.cos(), cy + inner * t0.sin());
      let (i1x, i1y) = (cx + inner * t1.cos(), cy + inner * t1.sin());
      let (o0x, o0y) = (cx + outer * t0.cos(), cy + outer * t0.sin());
      let (o1x, o1y) = (cx + outer * t1.cos(), cy + outer * t1.sin());
      let ci0 = self.vertex_color(&self.state.stroke, i0x, i0y, &self.state.transform);
      let ci1 = self.vertex_color(&self.state.stroke, i1x, i1y, &self.state.transform);
      let co0 = self.vertex_color(&self.state.stroke, o0x, o0y, &self.state.transform);
      let co1 = self.vertex_color(&self.state.stroke, o1x, o1y, &self.state.transform);
      let ti0 = self.state.transform.apply(i0x, i0y);
      let ti1 = self.state.transform.apply(i1x, i1y);
      let to0 = self.state.transform.apply(o0x, o0y);
      let to1 = self.state.transform.apply(o1x, o1y);
      self.push_tri(
        Vertex::flat(ti0, ci0),
        Vertex::flat(ti1, ci1),
        Vertex::flat(to0, co0),
      );
      self.push_tri(
        Vertex::flat(ti1, ci1),
        Vertex::flat(to1, co1),
        Vertex::flat(to0, co0),
      );
    }
    if self.state.line_cap == LineCap::Round && span < std::f32::consts::TAU - 1e-4 {
      let hw = w / 2.0;
      let (ex0, ey0) = (cx + r * a0.cos(), cy + r * a0.sin());
      let (ex1, ey1) = (cx + r * a1.cos(), cy + r * a1.sin());
      self.cap_round((ex0, ey0), hw);
      self.cap_round((ex1, ey1), hw);
    }
  }

  pub fn stroke_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
    self.stroke_polyline(&[(x1, y1), (x2, y2)]);
  }

  pub fn stroke_polyline(&mut self, pts: &[(f32, f32)]) {
    if pts.len() < 2 {
      return;
    }
    if self.state.shadow_blur > 0.0 && self.state.shadow_color.a > 0.0 {
      let blur = self.state.shadow_blur;
      let sc = self.state.shadow_color;
      let saved_stroke = self.state.stroke.clone();
      let saved_width = self.state.stroke_width;
      let saved_cap = self.state.line_cap;
      let saved_shadow = (self.state.shadow_color, self.state.shadow_blur);
      self.state.line_cap = LineCap::Butt;
      for i in 0..2 {
        self.state.stroke = Fill::Solid(sc.with_alpha(sc.a * (0.12 - 0.04 * i as f32)));
        self.state.stroke_width = saved_width + blur * (1.0 - 0.35 * i as f32);
        self.set_shadow(Color::TRANSPARENT, 0.0);
        self.stroke_polyline_impl(pts);
      }
      self.state.stroke = saved_stroke;
      self.state.stroke_width = saved_width;
      self.state.line_cap = saved_cap;
      self.set_shadow(saved_shadow.0, saved_shadow.1);
    }
    self.stroke_polyline_impl(pts);
  }

  fn stroke_polyline_impl(&mut self, pts: &[(f32, f32)]) {
    for seg in pts.windows(2) {
      let (p0, p1) = (seg[0], seg[1]);
      let dx = p1.0 - p0.0;
      let dy = p1.1 - p0.1;
      let len = dx.hypot(dy);
      if len < 1e-6 {
        continue;
      }
      let nx = -dy / len;
      let ny = dx / len;
      let hw = self.state.stroke_width / 2.0;
      let a = (p0.0 + nx * hw, p0.1 + ny * hw);
      let b = (p0.0 - nx * hw, p0.1 - ny * hw);
      let c = (p1.0 + nx * hw, p1.1 + ny * hw);
      let d = (p1.0 - nx * hw, p1.1 - ny * hw);
      let stroke = self.state.stroke.clone();
      let xform = self.state.transform;
      self.push_quad(a, b, d, c, &stroke, &xform);
      if self.state.line_cap == LineCap::Round {
        self.cap_round(p0, hw);
        self.cap_round(p1, hw);
      }
    }
  }

  fn cap_round(&mut self, p: (f32, f32), radius: f32) {
    if radius <= 0.0 {
      return;
    }
    let n = 12u32;
    let center = self.state.transform.apply(p.0, p.1);
    let c = self.vertex_color(&self.state.stroke, p.0, p.1, &self.state.transform);
    for i in 0..n {
      let t0 = std::f32::consts::TAU * (i as f32 / n as f32);
      let t1 = std::f32::consts::TAU * ((i + 1) as f32 / n as f32);
      let (ax, ay) = (p.0 + radius * t0.cos(), p.1 + radius * t0.sin());
      let (bx, by) = (p.0 + radius * t1.cos(), p.1 + radius * t1.sin());
      let ca = self.vertex_color(&self.state.stroke, ax, ay, &self.state.transform);
      let cb = self.vertex_color(&self.state.stroke, bx, by, &self.state.transform);
      let ta = self.state.transform.apply(ax, ay);
      let tb = self.state.transform.apply(bx, by);
      self.push_tri(
        Vertex::flat(center, c),
        Vertex::flat(ta, ca),
        Vertex::flat(tb, cb),
      );
    }
  }

  pub fn fill_rounded_rect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32) {
    let r = r.clamp(0.0, w.min(h) / 2.0);
    if r <= 0.0 {
      self.fill_rect(x, y, w, h);
      return;
    }
    self.fill_rect(x + r, y, w - 2.0 * r, h);
    self.fill_rect(x, y + r, w, h - 2.0 * r);
    let corners = [
      (x + r, y + r),
      (x + w - r, y + r),
      (x + w - r, y + h - r),
      (x + r, y + h - r),
    ];
    for (cx, cy) in corners {
      self.fill_arc(cx, cy, r, 0.0, std::f32::consts::TAU);
    }
  }

  /// Canvas roundRect with only the top corners rounded (bars).
  pub fn fill_rounded_rect_top(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32) {
    let r = r.clamp(0.0, w.min(h) / 2.0);
    if r <= 0.0 {
      self.fill_rect(x, y, w, h);
      return;
    }
    let fill = self.state.fill.clone();
    let xform = self.state.transform;
    self.fill_rect(x + r, y, w - 2.0 * r, h);
    self.fill_rect(x, y + r, w, h - r);
    self.fill_arc(x + r, y + r, r, std::f32::consts::PI, std::f32::consts::PI * 1.5);
    self.fill_arc(x + w - r, y + r, r, std::f32::consts::PI * 1.5, std::f32::consts::TAU);
    let _ = (fill, xform);
  }

  /// Fill a simple (roughly convex/star-shaped) polygon via fan from pts[0].
  pub fn fill_polygon(&mut self, pts: &[(f32, f32)]) {
    if pts.len() < 3 {
      return;
    }
    let (fx, fy) = pts[0];
    let c0 = self.vertex_color(&self.state.fill, fx, fy, &self.state.transform);
    let t0 = self.state.transform.apply(fx, fy);
    for i in 1..pts.len() - 1 {
      let (ax, ay) = pts[i];
      let (bx, by) = pts[i + 1];
      let ca = self.vertex_color(&self.state.fill, ax, ay, &self.state.transform);
      let cb = self.vertex_color(&self.state.fill, bx, by, &self.state.transform);
      let ta = self.state.transform.apply(ax, ay);
      let tb = self.state.transform.apply(bx, by);
      self.push_tri(Vertex::flat(t0, c0), Vertex::flat(ta, ca), Vertex::flat(tb, cb));
    }
  }

  /// Sample a quadratic bezier p0 -> ctrl -> p1 into `steps` + 1 points.
  pub fn sample_quadratic(p0: (f32, f32), ctrl: (f32, f32), p1: (f32, f32), steps: u32) -> Vec<(f32, f32)> {
    let n = steps.max(2);
    let mut out = Vec::with_capacity(n as usize + 1);
    for i in 0..=n {
      let t = i as f32 / n as f32;
      let it = 1.0 - t;
      let x = it * it * p0.0 + 2.0 * it * t * ctrl.0 + t * t * p1.0;
      let y = it * it * p0.1 + 2.0 * it * t * ctrl.1 + t * t * p1.1;
      out.push((x, y));
    }
    out
  }

  pub fn fill_ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32) {
    if self.state.shadow_blur > 0.0 && self.state.shadow_color.a > 0.0 {
      self.draw_ellipse_glow(cx, cy, rx, ry);
    }
    let n = self.segments.max(8);
    let center = self.state.transform.apply(cx, cy);
    let c = self.vertex_color(&self.state.fill, cx, cy, &self.state.transform);
    for i in 0..n {
      let t0 = std::f32::consts::TAU * (i as f32 / n as f32);
      let t1 = std::f32::consts::TAU * ((i + 1) as f32 / n as f32);
      let (px, py) = (cx + rx * t0.cos(), cy + ry * t0.sin());
      let (qx, qy) = (cx + rx * t1.cos(), cy + ry * t1.sin());
      let cp = self.vertex_color(&self.state.fill, px, py, &self.state.transform);
      let cq = self.vertex_color(&self.state.fill, qx, qy, &self.state.transform);
      let tp = self.state.transform.apply(px, py);
      let tq = self.state.transform.apply(qx, qy);
      self.push_tri(
        Vertex::flat(center, c),
        Vertex::flat(tp, cp),
        Vertex::flat(tq, cq),
      );
    }
  }

  /// Soft radial glow behind an ellipse when a shadow is set.
  fn draw_ellipse_glow(&mut self, cx: f32, cy: f32, rx: f32, ry: f32) {
    let blur = self.state.shadow_blur;
    let sc = self.state.shadow_color.with_alpha(self.state.shadow_color.a * 0.35);
    let grx = (rx + blur * 1.2).max(1.0);
    let gry = (ry + blur * 1.2).max(1.0);
    let g = Gradient::Radial {
      x0: cx,
      y0: cy,
      r0: rx.max(ry) * 0.5,
      x1: cx,
      y1: cy,
      r1: grx.max(gry),
      stops: vec![
        ColorStop { t: 0.0, color: sc },
        ColorStop { t: 1.0, color: Color::TRANSPARENT },
      ],
    };
    self.save();
    self.state.fill = Fill::Gradient(g);
    self.set_shadow(Color::TRANSPARENT, 0.0);
    self.fill_ellipse(cx, cy, grx, gry);
    self.restore();
  }

  /// Fill the annular sector between two arcs (Canvas two-arc crescent).
  /// Spans [a0, a1] with `a1 > a0`; the radial edges close the shape.
  pub fn fill_ring_arc(
    &mut self,
    cx: f32,
    cy: f32,
    r_outer: f32,
    r_inner: f32,
    a0: f32,
    a1: f32,
  ) {
    if r_outer <= 0.0 || r_inner >= r_outer {
      return;
    }
    let mut a1 = a1;
    while a1 < a0 {
      a1 += std::f32::consts::TAU;
    }
    let n = self.segments.max(8);
    let steps = ((n as f32 * (a1 - a0) / std::f32::consts::TAU).ceil() as u32).clamp(2, 512);
    let r_inner = r_inner.max(0.0);
    for i in 0..steps {
      let t0 = a0 + (a1 - a0) * (i as f32 / steps as f32);
      let t1 = a0 + (a1 - a0) * ((i + 1) as f32 / steps as f32);
      let (o0x, o0y) = (cx + r_outer * t0.cos(), cy + r_outer * t0.sin());
      let (o1x, o1y) = (cx + r_outer * t1.cos(), cy + r_outer * t1.sin());
      let (i0x, i0y) = (cx + r_inner * t0.cos(), cy + r_inner * t0.sin());
      let (i1x, i1y) = (cx + r_inner * t1.cos(), cy + r_inner * t1.sin());
      let co0 = self.vertex_color(&self.state.fill, o0x, o0y, &self.state.transform);
      let co1 = self.vertex_color(&self.state.fill, o1x, o1y, &self.state.transform);
      let ci0 = self.vertex_color(&self.state.fill, i0x, i0y, &self.state.transform);
      let ci1 = self.vertex_color(&self.state.fill, i1x, i1y, &self.state.transform);
      let to0 = self.state.transform.apply(o0x, o0y);
      let to1 = self.state.transform.apply(o1x, o1y);
      let ti0 = self.state.transform.apply(i0x, i0y);
      let ti1 = self.state.transform.apply(i1x, i1y);
      self.push_tri(
        Vertex::flat(to0, co0),
        Vertex::flat(ti0, ci0),
        Vertex::flat(ti1, ci1),
      );
      self.push_tri(
        Vertex::flat(to0, co0),
        Vertex::flat(ti1, ci1),
        Vertex::flat(to1, co1),
      );
    }
  }

  /// Fill an annulus between `r_inner` and `r_outer` (Canvas two-arc ring).
  pub fn fill_ring(&mut self, cx: f32, cy: f32, r_outer: f32, r_inner: f32) {
    if r_outer <= 0.0 || r_inner >= r_outer {
      return;
    }
    let n = self.segments.max(8);
    let r_inner = r_inner.max(0.0);
    for i in 0..n {
      let t0 = std::f32::consts::TAU * (i as f32 / n as f32);
      let t1 = std::f32::consts::TAU * ((i + 1) as f32 / n as f32);
      let (io0x, io0y) = (cx + r_outer * t0.cos(), cy + r_outer * t0.sin());
      let (io1x, io1y) = (cx + r_outer * t1.cos(), cy + r_outer * t1.sin());
      let (ii0x, ii0y) = (cx + r_inner * t0.cos(), cy + r_inner * t0.sin());
      let (ii1x, ii1y) = (cx + r_inner * t1.cos(), cy + r_inner * t1.sin());
      let cio0 = self.vertex_color(&self.state.fill, io0x, io0y, &self.state.transform);
      let cio1 = self.vertex_color(&self.state.fill, io1x, io1y, &self.state.transform);
      let cii0 = self.vertex_color(&self.state.fill, ii0x, ii0y, &self.state.transform);
      let cii1 = self.vertex_color(&self.state.fill, ii1x, ii1y, &self.state.transform);
      let tio0 = self.state.transform.apply(io0x, io0y);
      let tio1 = self.state.transform.apply(io1x, io1y);
      let tii0 = self.state.transform.apply(ii0x, ii0y);
      let tii1 = self.state.transform.apply(ii1x, ii1y);
      self.push_tri(
        Vertex::flat(tio0, cio0),
        Vertex::flat(tii0, cii0),
        Vertex::flat(tii1, cii1),
      );
      self.push_tri(
        Vertex::flat(tio0, cio0),
        Vertex::flat(tii1, cii1),
        Vertex::flat(tio1, cio1),
      );
    }
  }

  /// Best-effort shadow approximation: a soft radial glow behind the rect.
  fn draw_shadow(&mut self, x: f32, y: f32, w: f32, h: f32) {
    if self.state.shadow_blur <= 0.0 || self.state.shadow_color.a <= 0.0 {
      return;
    }
    let blur = self.state.shadow_blur;
    let sc = self.state.shadow_color.with_alpha(self.state.shadow_color.a * 0.25);
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    let rx = w / 2.0 + blur;
    let ry = h / 2.0 + blur;
    let g = Gradient::Radial { x0: cx, y0: cy, r0: w.hypot(h) / 3.0, x1: cx, y1: cy, r1: rx.max(ry), stops: vec![
      ColorStop { t: 0.0, color: sc },
      ColorStop { t: 1.0, color: Color::TRANSPARENT },
    ]};
    self.save();
    self.state.fill = Fill::Gradient(g);
    self.set_shadow(Color::TRANSPARENT, 0.0);
    self.fill_ellipse(cx, cy, rx, ry);
    self.restore();
  }

  fn stroke_sample(&self, x: f32, y: f32) -> Color {
    match &self.state.stroke {
      Fill::Solid(c) => *c,
      Fill::Gradient(g) => g.sample(x, y),
    }
  }

  // --- textured (glyph atlas layer 0, images layers 1+) ---

  pub fn push_textured_quad(&mut self, layer: u32, x: f32, y: f32, w: f32, h: f32, uv: [f32; 4], color: Color) {
    let t0 = self.state.transform.apply(x, y);
    let t1 = self.state.transform.apply(x + w, y);
    let t2 = self.state.transform.apply(x + w, y + h);
    let t3 = self.state.transform.apply(x, y + h);
    let col = color.with_alpha(color.a * self.state.global_alpha);
    self.push_tri(
      Vertex::textured(t0, col, [uv[0], uv[1]], layer),
      Vertex::textured(t1, col, [uv[2], uv[1]], layer),
      Vertex::textured(t2, col, [uv[2], uv[3]], layer),
    );
    self.push_tri(
      Vertex::textured(t0, col, [uv[0], uv[1]], layer),
      Vertex::textured(t2, col, [uv[2], uv[3]], layer),
      Vertex::textured(t3, col, [uv[0], uv[3]], layer),
    );
  }

  pub fn push_atlas_layer(&mut self, layer: u32, rgba: Vec<u8>, w: u32, h: u32) {
    if layer >= super::renderer::TEXTURE_LAYERS {
      return;
    }
    self.atlases.push(AtlasUpload { layer, rgba, width: w, height: h });
  }

  /// Canvas `fillText` equivalent: bakes a run into a per-frame glyph atlas
  /// and pushes one textured quad. `y` is the baseline (like `fillText`).
  /// `fill` may be a gradient, sampled at glyph position while baking.
  pub fn draw_text(
    &mut self,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    family: &str,
    weight: f32,
    align: super::text::TextAlign,
    fill: Fill,
    opacity: f32,
    opts: &super::text::TextOpts,
  ) {
    let Some(font) = super::text::select_font(family, weight) else { return };
    let Some(atl) = super::text::rasterize(font, text, font_size, &fill, opts) else { return };
    let layer = self.next_text_layer;
    if layer >= super::text::TEXT_LAYERS {
      return;
    }
    self.next_text_layer += 1;
    let dx = match align {
      super::text::TextAlign::Left => 0.0,
      super::text::TextAlign::Center => -atl.advance / 2.0,
      super::text::TextAlign::Right => -atl.advance,
    };
    self.atlases.push(AtlasUpload {
      layer,
      rgba: atl.rgba,
      width: atl.atlas_w,
      height: atl.atlas_h,
    });
    self.push_textured_quad(
      layer,
      x + dx + atl.left,
      y - atl.ascent + atl.top,
      atl.width,
      atl.height,
      [0.0, 0.0, 1.0, 1.0],
      Color::rgba(1.0, 1.0, 1.0, opacity.clamp(0.0, 1.0)),
    );
  }

  // --- output ---

  pub fn finish(self) -> Mesh {
    Mesh {
      verts: self.verts,
      idx: self.idx,
      clear: self.clear,
      atlases: self.atlases,
    }
  }
}

pub struct Mesh {
  pub verts: Vec<Vertex>,
  pub idx: Vec<u32>,
  pub clear: Color,
  pub atlases: Vec<AtlasUpload>,
}

impl Mesh {
  pub fn is_empty(&self) -> bool {
    self.idx.is_empty()
  }
}
