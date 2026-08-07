# Visual Parity Report — Legacy Tauri vs Native Slint

Dibuat: 6 Agustus 2026 · `docs/parity/`

## Cara Verifikasi

| Item | Metode |
| :--- | :--- |
| Screenshot legacy | Mockup HTML statis dari `legacy_tauri_backup/src` (index.css + JSX), di-screenshot dengan Chrome headless → `legacy_shot.png` |
| **Screenshot Slint NATIVE** | **`cargo run --features ui-screenshot --example ui_screenshot -- [--out path] [--tab ...] [--style ...] [--hover x,y] [--modal export]`** — me-render **AppWindow nyata** (ui/*.slint, output build.rs yang sama dengan app) ke PNG via backend testing + software renderer, tanpa display → `native_*.png` |
| Screenshot Slint (mockup) | Mockup HTML statis (untuk perbandingan historis) → `slint_shot.png` |
| Perbandingan visual | `comparison.html` (buka di browser) |

> **Tool screenshot native (`examples/ui_screenshot.rs`)**: satu-satunya cara verifikasi pixel-akurat tanpa display (app berjalan di Wayland, tidak tertangkap xwd). Didukung oleh `i-slint-backend-testing` (headless) + `i-slint-renderer-software`, digate di balik feature opsional `ui-screenshot` agar `cargo build`/`cargo test` normal tidak menarik dep tree software renderer. Versi dijaga lockstep dengan `slint` (keduanya 1.17.x) sehingga Cargo menyatukannya ke satu `i-slint-core` (diverifikasi: 1 copy di Cargo.lock). Contoh pemakaian:
> - `cargo run --features ui-screenshot --example ui_screenshot -- --out docs/parity/native_shot.png` (default: tab style, style spectrum)
> - `--tab colors` / `--tab export` / `--modal export` untuk state lain
> - `--hover 950,140` untuk mensimulasikan pointer move → micro-interactions (scale + glow) ter-render

> **Catatan penting:** Legacy **tidak dapat dijalankan langsung** — `package.json`/`npm` sudah dipurge saat migrasi.

## 🎨 Perbandingan Warna (Tema)

| Elemen | Legacy Tauri | Native Slint (sekarang) | Status |
| :--- | :--- | :--- | :--- |
| Background utama | `#000000` (hitam murni) | `#000000` | 🟢 SAMA |
| Navbar / Audio bar | `#000000` + border gold | `#0a0a0a` | 🟢 SAMA |
| Control panel | `rgba(10,10,10,0.96)` | `#0a0a0a` | 🟢 SAMA |
| Canvas visualizer | `#000` | `#000000` | 🟢 SAMA |
| Modal background | `#0a0a0a` + glow gold | `#0a0a0a` | 🟢 SAMA |
| **Accent utama** | **`#ffd700` (GOLD)** | **`#ffd700`** | 🟢 SAMA |
| Accent widget bawaan | Gold (`#ffd700` runtime accent) | Gold (`SlintContext.set_accent_color`) | 🟢 SAMA |
| Tombol CTA (Open/Export) | Gradient gold, teks hitam | Solid gold (Button `primary`) | 🟢 SAMA (look) |
| Text utama | `#f8fafc` | `#f8fafc` | 🟢 SAMA |
| Text muted | `#a3a3a3` | `#a3a3a3` | 🟢 SAMA |
| Text sub | `#737373` | `#737373` | 🟢 SAMA |
| Radius card | `10px`/`16px` | `12px` | 🟡 Bed kecil |
| Color theme data (cyberpunk) | `#00f0ff`/`#ff007f`/`#ffe600` | Sama (data string) | 🟢 SAMA |

**Kesimpulan:** tema legacy = **hitam + gold** (`--accent-cyan: #ffd700`). Tema ini kini **sudah diterapkan** ke Slint: semua chrome UI di `ui/*.slint` memakai palet legacy, dan widget bawaan (Button/Slider/ComboBox/CheckBox) di-re-hue ke gold via `SlintContext::set_accent_color(#ffd700)` + dark color-scheme (lihat `src/lib.rs`).

## 📐 Layout & Spacing

| Dimensi | Legacy | Slint | Status |
| :--- | :--- | :--- | :--- |
| Navbar height | 60px | 60px | 🟢 SAMA |
| Navbar padding/gap | padding 0 20px, gap 16px | 20px / 16px | 🟢 SAMA |
| Navbar border-bawah | 1px gold `rgba(255,215,0,0.2)` | `#ffd70033` | 🟢 SAMA |
| Brand | Icon 36px gold-gradient + nama + badge gold | Icon 36px + AudioWave + badge Studio | 🟢 SAMA |
| Control panel width | 380px | 380px | 🟢 SAMA |
| Audio bar height | 64px | 64px | 🟢 SAMA |
| Panel tabs | 6 tab | 6 tab | 🟢 SAMA (urutan Text/Effects tukar) |
| Style selector | Grid kartu 2 kolom, 19 style | Grid 2 kolom, 19 kartu | 🟢 SAMA |

## 🧩 Fitur UI

| Fitur | Legacy | Slint | Status |
| :--- | :---: | :---: | :--- |
| 19 visualizer styles | ✅ | ✅ (setelah paritas) | 🟢 SAMA |
| Bar gap/width/rounding | ✅ | ✅ | 🟢 SAMA |
| Mirror waves | ✅ | ✅ | 🟢 SAMA |
| Fire 3D dims (api3D) | ✅ | ✅ | 🟢 SAMA |
| Radial center image | ✅ | ✅ | 🟢 SAMA |
| Peak markers | ✅ | ✅ | 🟢 SAMA |
| Theme presets | ✅ | ✅ | 🟢 SAMA |
| Save/Load preset (.awpreset) | ✅ | ✅ | 🟢 SAMA |
| Mute button | ✅ | ✅ | 🟢 SAMA |
| Export settings (aspect/res/FPS/format/encoder/FFT) | ✅ | ✅ | 🟢 SAMA |
| Hardware modal (GPU/encoder/OS/rescan) | ✅ | ✅ | 🟢 SAMA |
| Listen (mic input) | ✅ | ✅ (via `arecord` ALSA) | 🟢 SAMA* |
| Pop-out preview | ✅ | ❌ | 🔴 Belum diport |
| Window controls custom | ✅ | ⚠️ Native decorations | 🔴 Beda |
| Ticker CredibleMark | ✅ | ✅ (marquee gold, klik → About) | 🟢 SAMA |
| Fullscreen F11 | ✅ | ❌ | 🔴 Belum diport |

## ✅ Yang SUDAH paritas (dari sesi sebelumnya)

- 19 style visualizer lengkap di ComboBox (sebelumnya hanya 14)
- Semua kontrol StyleTab (bar gap/width/rounding, mirror, fire dims, radial image)
- Tab panel: Background & Effects (bukan "Bg"/"FX"), tab Export ditambahkan
- Mute button di audio bar
- Preset selector + Save/Load Preset di navbar
- Export modal diperkaya (aspect ratio, format, encoder, FFT, include audio) + bug sinkronisasi export ke config diperbaiki
- Hardware modal diperkaya (GPU list, encoder table, OS, rescan)

## ✅ Perubahan tema (sesi ini)

- Semua chrome warna di `ui/*.slint` disamakan dengan palet legacy: background `#000000`, panel `#0a0a0a`, accent gold `#ffd700`, teks `#f8fafc`/`#a3a3a3`/`#737373`
- Data color-theme dipertahankan agar identik dengan presets legacy (`cyberpunk` primary `#00f0ff`, secondary `#ff007f`, accent `#ffe600`)
- `src/lib.rs`: `WindowInner::context().set_accent_color(#ffd700)` + `set_color_scheme(Dark)` → seluruh widget bawaan (button, slider, combobox, checkbox, scrollbar) otomatis jadi gold pada latar hitam
- Tombol CTA utama (`Open Track`, `Export Video`, `Start Export`) dijadikan `primary: true` → solid gold seperti `btn-export` legacy
- Mockup & screenshot diperbarui: `slint_mockup.html` + `slint_shot.png` (verifikasi pixel: brand text gold `(232,215,0)` ≈ legacy `(248,234,181)`, canvas `(0,0,0)`)

## ✨ Micro-UI polish (sesi ini)

- **Navbar**: restrukturisasi ke 2 cluster (kiri/kanan) `justify-content: space-between` persis legacy — brand+ticker+Open Track+Listen di kiri, icon actions+Export+window controls di kanan; padding `0 20px`; IconBtn hover putih `rgba(255,255,255,0.12)` (bukan gold); window controls `gap: 2px` + border-left divider
- **Audio bar**: height 64px (sebelumnya 56px), padding `0 20px`, gap 16px; tombol Play/Pause jadi **lingkaran gold 36px dengan glow** (`drop-shadow` 12px) + Stop/Mute jadi **lingkaran 28px** (`.btn-control` legacy) — menggantikan widget Button persegi besar
- **Control panel**: width 380px (sebelumnya 360px); komponen `SectionTitle` baru (uppercase, 12px, bold, `letter-spacing: 0.5px`, hairline gold) menggantikan judul putih 13px yang tidak konsisten; komponen `SliderRow` baru (label kiri + nilai **gold** di kanan + track di bawah) menggantikan label+slider terpisah yang boros ruang; content padding 10px
- **Modals**: tombol ✕ close di pojok kanan atas header (28px, hover putih); ✕ pada ExportModal juga membatalkan render yang sedang berjalan (sama seperti tombol Cancel)
- **Bug text**: AboutModal "14 Visualizer Styles" → **19**
- **Micro-interactions hover/active** (StyleCard + TabBtn): `transform-scale-x/y` 1.04–1.05 (via properti reserved builtin — `scale-x`/`scale-y` BUKAN properti valid di Slint 1.17, hanya `transform-scale-*` Float32), `drop-shadow-blur` glow 6–12px gold, `animate` 120–150ms ease-out; StyleCard `mouse-cursor: pointer`; icon font-size tidak lagi di-animate (redundan dengan transform-scale, berisiko jitter reflow)
- **Micro-interactions navbar** (IconBtn + window controls): IconBtn scale 1.07 + glow gold 6px saat hover (120ms ease-out); window controls scale 1.06 (close 1.08) + glow putih/merah 5–8px (100ms ease-out); scale 1.1 diturunkan ke 1.07 atas review (konsisten dengan 1.04–1.05 kartu/tab)
- **Micro-interactions modal**: tombol ✕ di 3 modal scale 1.07 + glow putih 6px; komponen `ModalBtn` baru (primary gold/secondary dark, scale 1.04 + glow gold/putih, 120ms) menggantikan std Button Cancel/Start Export/Pindai Ulang/Close; **a11y dipertahankan**: ModalBtn dibungkus `FocusScope` (`focus-on-click` + `focus-on-tab-navigation`, Enter/Space memicu klik, gold focus ring saat `has-focus`); Start Export di-guard `if (!is-exporting)` agar tidak re-trigger saat rendering

## 🔴 Yang MASIH belum identik (prioritas perbaikan)

1. ~~Fitur belum diport: pop-out preview, window controls custom, F11 fullscreen~~ — **sudah diport di sesi sebelumnya** (PreviewWindow `on_close_requested` + window controls di navbar + F11/Escape fullscreen)

## 🎙️ Catatan Feasibility: Listen (mic) & cpal

- **cpal (Rust crate)**: memerlukan paket build **`libasound2-dev`** (via `alsa-sys`) yang **tidak terpasang** di sistem ini (`pkg-config alsa` MISSING). Crates `cpal-0.15.3`/`alsa-*` sudah ada di cache Cargo, tetapi tanpa header ALSA build akan gagal. Memasang `libasound2-dev` akan mengulangi ketergantungan C-library yang sengaja dihilangkan saat migrasi (lihat MIGRATION_PLAN.md).
- **Solusi yang dipakai**: subprocess **`arecord -f S16_LE -r 44100 -c 1 -t raw`** (ALSA, streaming PCM ke stdout) — terverifikasi berhasil (88.200 byte/detik = tepat 44.1kHz mono S16). Pola ini konsisten dengan playback yang sudah memakai `pw-play`. Fallback `pw-record` disediakan bila `arecord` tidak ada.
- **Device**: `arecord -l` mendeteksi `HDA Intel PCH / ALC671` (capture), meskipun `pactl` menampilkan 0 source (tidak ter-ekspos ke PipeWire/Pulse) — capture via ALSA langsung tetap berfungsi.
- **Status**: tombol 🎧 Listen → arecord streaming → ring buffer 2 detik → pipeline FFT yang sama dengan lagu (lihat `src/app_state.rs` `start_listen`/`mic_window`).

## File

- `legacy_mockup.html` — mockup UI legacy dari source
- `slint_mockup.html` — mockup UI Slint dari source
- `legacy_shot.png` / `slint_shot.png` — screenshot Chrome headless
- `comparison.html` — halaman perbandingan visual interaktif
