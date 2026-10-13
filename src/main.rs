//! SOUNDCORE native Linux configurator: eframe/egui frontend over
//! `openscq30-lib`. Same design language as the ATTACK SHARK app (which
//! follows the craft-launcher language): square corners, 1px hairlines, mono
//! uppercase labels, dark/light tokens, bento cards, header/footer.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, mpsc};

use egui::epaint::text::VariationCoords;
use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, FontTweak, RichText,
    Stroke, TextStyle, Ui, Vec2, Visuals,
};
use macaddr::MacAddr6;
use openscq30_lib::device::OpenSCQ30Device;
use openscq30_lib::settings::{Setting, SettingId, Value};
use openscq30_lib::OpenSCQ30Session;

// ---------------------------------------------------------------- theme ---

#[derive(Clone, Copy)]
struct Tokens {
    dark: bool,
    bg: Color32,
    sunken: Color32,
    ink: Color32,
    strong: Color32,
    muted: Color32,
    faint: Color32,
    line: Color32,
    line_strong: Color32,
    invert_bg: Color32,
    invert_fg: Color32,
    accent: Color32,
    accent_ink: Color32,
    good: Color32,
    warn: Color32,
    bad: Color32,
}

impl Tokens {
    fn dark() -> Self {
        Self {
            dark: true,
            bg: Color32::from_rgb(0x12, 0x13, 0x16),
            sunken: Color32::from_rgb(0x06, 0x06, 0x07),
            ink: Color32::from_rgb(0xf2, 0xf1, 0xee),
            strong: Color32::WHITE,
            muted: Color32::from_rgba_unmultiplied(0xf2, 0xf1, 0xee, 153),
            faint: Color32::from_rgba_unmultiplied(0xf2, 0xf1, 0xee, 97),
            line: Color32::from_rgba_unmultiplied(0xf2, 0xf1, 0xee, 38),
            line_strong: Color32::from_rgba_unmultiplied(0xf2, 0xf1, 0xee, 102),
            invert_bg: Color32::WHITE,
            invert_fg: Color32::from_rgb(0x0b, 0x0b, 0x0c),
            accent: Color32::from_rgb(0x2d, 0x81, 0xff),
            accent_ink: Color32::from_rgb(0x74, 0xaa, 0xff),
            good: Color32::from_rgb(0x3f, 0xd6, 0xb8),
            warn: Color32::from_rgb(0xff, 0xc1, 0x4d),
            bad: Color32::from_rgb(0xff, 0x62, 0x59),
        }
    }
    fn light() -> Self {
        Self {
            dark: false,
            bg: Color32::from_rgb(0xf2, 0xf1, 0xee),
            sunken: Color32::from_rgb(0xe9, 0xe8, 0xe4),
            ink: Color32::from_rgb(0x10, 0x10, 0x14),
            strong: Color32::BLACK,
            muted: Color32::from_rgba_unmultiplied(0x10, 0x10, 0x14, 158),
            faint: Color32::from_rgba_unmultiplied(0x10, 0x10, 0x14, 107),
            line: Color32::from_rgba_unmultiplied(0x10, 0x10, 0x14, 41),
            line_strong: Color32::from_rgba_unmultiplied(0x10, 0x10, 0x14, 102),
            invert_bg: Color32::from_rgb(0x10, 0x10, 0x14),
            invert_fg: Color32::from_rgb(0xf2, 0xf1, 0xee),
            accent: Color32::from_rgb(0x2d, 0x81, 0xff),
            accent_ink: Color32::from_rgb(0x16, 0x59, 0xc4),
            good: Color32::from_rgb(0x0a, 0x75, 0x63),
            warn: Color32::from_rgb(0x8a, 0x58, 0x00),
            bad: Color32::from_rgb(0xb3, 0x26, 0x1e),
        }
    }
    fn get(ctx: &egui::Context) -> Self {
        if ctx.theme() == egui::Theme::Dark {
            Self::dark()
        } else {
            Self::light()
        }
    }
    fn hairline(&self) -> Stroke {
        Stroke::new(1.0, self.line)
    }
}

const CONTROL_H: f32 = 34.0;
const HUD_SIZE: f32 = 11.0;
const HUD_SPACING: f32 = HUD_SIZE * 0.15;
const BUTTON_SPACING: f32 = HUD_SIZE * 0.12;
const HEADER_H: f32 = 48.0;
const FOOTER_H: f32 = 52.0;

/// Inter 500.
fn medium() -> FontFamily {
    FontFamily::Name("medium".into())
}
/// Archivo 620 at 118% width: display headings.
fn display() -> FontFamily {
    FontFamily::Name("display".into())
}
/// Geist Mono 500: labels and badges.
fn mono() -> FontFamily {
    FontFamily::Name("mono".into())
}
/// Geist Mono 700: buttons.
fn mono_bold() -> FontFamily {
    FontFamily::Name("mono-bold".into())
}

fn install_fonts(ctx: &egui::Context) {
    const INTER: &[u8] = include_bytes!("/home/leo/craft-launcher/assets/fonts/Inter-Variable.ttf");
    const ARCHIVO: &[u8] =
        include_bytes!("/home/leo/craft-launcher/assets/fonts/Archivo-Variable.ttf");
    const GEIST_MONO: &[u8] =
        include_bytes!("/home/leo/craft-launcher/assets/fonts/GeistMono-Variable.ttf");
    let mut fonts = FontDefinitions::default();
    let fallback: Vec<String> = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let faces: [(&str, &[u8], Vec<([u8; 4], f32)>, FontFamily); 6] = [
        ("Inter", INTER, vec![(*b"wght", 400.0), (*b"opsz", 14.0)], FontFamily::Proportional),
        ("Inter-Medium", INTER, vec![(*b"wght", 500.0), (*b"opsz", 14.0)], medium()),
        ("Archivo-Display", ARCHIVO, vec![(*b"wght", 620.0), (*b"wdth", 118.0)], display()),
        ("GeistMono", GEIST_MONO, vec![(*b"wght", 400.0)], FontFamily::Monospace),
        ("GeistMono-Medium", GEIST_MONO, vec![(*b"wght", 500.0)], mono()),
        ("GeistMono-Bold", GEIST_MONO, vec![(*b"wght", 700.0)], mono_bold()),
    ];
    for (name, bytes, coords, family) in faces {
        let tweak = FontTweak { coords: VariationCoords::new(coords), ..Default::default() };
        fonts.font_data.insert(name.to_owned(), Arc::new(FontData::from_static(bytes).tweak(tweak)));
        let mut stack = vec![name.to_owned()];
        stack.extend(fallback.iter().cloned());
        fonts.families.insert(family, stack);
    }
    ctx.set_fonts(fonts);
}

fn visuals(t: &Tokens) -> Visuals {
    let mut v = if t.dark { Visuals::dark() } else { Visuals::light() };
    v.panel_fill = t.bg;
    v.window_fill = t.bg;
    v.extreme_bg_color = t.sunken;
    v.text_edit_bg_color = Some(t.sunken);
    v.faint_bg_color = t.sunken;
    v.code_bg_color = t.sunken;
    v.window_stroke = Stroke::new(1.0, t.line_strong);
    v.window_corner_radius = CornerRadius::ZERO;
    v.menu_corner_radius = CornerRadius::ZERO;
    v.override_text_color = Some(t.ink);
    v.hyperlink_color = t.accent_ink;
    v.selection.bg_fill = t.accent;
    v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    v.warn_fg_color = t.warn;
    v.error_fg_color = t.bad;
    for (w, fill, stroke, fg) in [
        (&mut v.widgets.noninteractive, t.bg, t.line, t.muted),
        (&mut v.widgets.inactive, t.line, t.line, t.ink),
        (&mut v.widgets.hovered, t.sunken, t.line_strong, t.strong),
        (&mut v.widgets.active, t.sunken, t.line_strong, t.strong),
        (&mut v.widgets.open, t.sunken, t.line_strong, t.strong),
    ] {
        w.bg_fill = fill;
        w.weak_bg_fill = fill;
        w.bg_stroke = Stroke::new(1.0, stroke);
        w.corner_radius = CornerRadius::ZERO;
        w.fg_stroke = Stroke::new(1.0, fg);
        w.expansion = 0.0;
    }
    v
}

fn apply_style(ctx: &egui::Context) {
    install_fonts(ctx);
    ctx.set_visuals_of(egui::Theme::Dark, visuals(&Tokens::dark()));
    ctx.set_visuals_of(egui::Theme::Light, visuals(&Tokens::light()));
    ctx.all_styles_mut(|s| {
        s.spacing.item_spacing = Vec2::new(8.0, 6.0);
        s.spacing.icon_spacing = 10.0;
        s.spacing.button_padding = Vec2::new(12.0, 6.0);
        s.spacing.interact_size.y = CONTROL_H;
        s.text_styles = [
            (TextStyle::Heading, FontId::new(24.0, display())),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(13.5, medium())),
            (TextStyle::Small, FontId::new(12.0, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(13.0, mono())),
        ]
        .into();
    });
}

// --------------------------------------------------------------- widgets ---

fn caps(
    ui: &Ui,
    text: &str,
    size: f32,
    family: FontFamily,
    spacing: f32,
    color: Color32,
) -> std::sync::Arc<egui::Galley> {
    let format = egui::TextFormat {
        font_id: FontId::new(size, family),
        color,
        extra_letter_spacing: spacing,
        ..Default::default()
    };
    ui.painter().layout_job(egui::text::LayoutJob::single_section(text.to_uppercase(), format))
}

fn hud_galley(ui: &Ui, text: &str, color: Color32) -> std::sync::Arc<egui::Galley> {
    caps(ui, text, HUD_SIZE, mono(), HUD_SPACING, color)
}

fn paint_left(ui: &Ui, rect: egui::Rect, galley: std::sync::Arc<egui::Galley>, color: Color32) {
    let pos = egui::pos2(rect.left(), rect.center().y - galley.size().y / 2.0);
    ui.painter().galley_with_override_text_color(pos, galley, color);
}

fn paint_centred(ui: &Ui, rect: egui::Rect, galley: std::sync::Arc<egui::Galley>, color: Color32) {
    ui.painter().galley_with_override_text_color(rect.center() - galley.size() / 2.0, galley, color);
}

fn hud(ui: &mut Ui, text: &str, color: Color32) {
    let galley = hud_galley(ui, text, color);
    let (rect, _) = ui.allocate_exact_size(galley.size(), egui::Sense::hover());
    paint_left(ui, rect, galley, color);
}

fn heading(ui: &mut Ui, text: &str, t: &Tokens) {
    ui.label(
        RichText::new(text)
            .size(24.0)
            .family(display())
            .color(t.strong)
            .extra_letter_spacing(-0.02 * 24.0),
    );
}

fn eyebrow(ui: &mut Ui, index: &str, label: &str, t: &Tokens) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        let galley = caps(ui, &format!("{index} / "), 13.0, mono(), HUD_SPACING, t.faint);
        let (rect, _) = ui.allocate_exact_size(galley.size(), egui::Sense::hover());
        paint_left(ui, rect, galley, t.faint);
        let galley = caps(ui, label, 13.0, mono(), HUD_SPACING, t.muted);
        let (rect, _) = ui.allocate_exact_size(galley.size(), egui::Sense::hover());
        paint_left(ui, rect, galley, t.muted);
    });
}

fn badge(ui: &mut Ui, text: &str, color: Color32, t: &Tokens) {
    let galley = caps(ui, text, 10.0, mono(), 1.0, color);
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(galley.size().x + 12.0, 20.0), egui::Sense::hover());
    ui.painter().rect(rect, CornerRadius::ZERO, t.sunken, t.hairline(), egui::StrokeKind::Inside);
    paint_centred(ui, rect, galley, color);
}

/// Solid chip with bold mono label.
fn tag(ui: &mut Ui, text: &str, fill: Color32, ink: Color32) {
    let galley = caps(ui, text, HUD_SIZE, mono_bold(), HUD_SPACING, ink);
    let (rect, _) =
        ui.allocate_exact_size(galley.size() + Vec2::new(16.0, 8.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::ZERO, fill);
    paint_centred(ui, rect, galley, ink);
}

/// Full-width nav row: mono label, inverts on hover or when active, accent
/// spine on the active row.
fn navlink(ui: &mut Ui, index: &str, label: &str, active: bool, t: &Tokens) -> egui::Response {
    let txt = if index.is_empty() { label.to_uppercase() } else { format!("{index}  {label}") };
    let h = 34.0;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), h), egui::Sense::click());
    let on = active || resp.hovered();
    let col = if on { t.invert_fg } else { t.muted };
    let galley = hud_galley(ui, &txt, col);
    if on {
        ui.painter().rect_filled(rect, CornerRadius::ZERO, t.invert_bg);
    }
    if active {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(rect.min, Vec2::new(2.0, h)),
            CornerRadius::ZERO,
            t.accent,
        );
    }
    paint_left(
        ui,
        egui::Rect::from_min_size(
            egui::pos2(rect.min.x + 12.0, rect.min.y),
            Vec2::new(rect.width() - 12.0, h),
        ),
        galley,
        col,
    );
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Slim square progress bar with a caption above it.
fn meter(ui: &mut Ui, frac: Option<f32>, caption: &str, color: Color32, t: &Tokens) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, CONTROL_H), egui::Sense::hover());
    let p = ui.painter();
    p.text(
        egui::pos2(rect.left(), rect.top() + 9.0),
        egui::Align2::LEFT_CENTER,
        caption,
        FontId::new(12.0, medium()),
        t.muted,
    );
    let bar =
        egui::Rect::from_min_size(egui::pos2(rect.left(), rect.bottom() - 4.0), Vec2::new(w, 4.0));
    p.rect_filled(bar, CornerRadius::ZERO, t.line);
    if let Some(f) = frac {
        p.rect_filled(
            egui::Rect::from_min_size(bar.min, Vec2::new(bar.width() * f.clamp(0.0, 1.0), bar.height())),
            CornerRadius::ZERO,
            color,
        );
    }
}

/// Square dropdown in the site language: hairline box, selected value in
/// mono caps, caret drawn by hand; the popup is a stack of navlink rows
/// that invert on hover. Returns the picked index, if any.
fn dropdown(
    ui: &mut Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    selected: &str,
    options: &[String],
    t: &Tokens,
) -> Option<usize> {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, CONTROL_H), egui::Sense::click());
    let hovered = resp.hovered();
    let p = ui.painter();
    p.rect(
        rect,
        CornerRadius::ZERO,
        if hovered { t.sunken } else { t.bg },
        t.hairline(),
        egui::StrokeKind::Inside,
    );
    let g = hud_galley(ui, selected, if hovered { t.strong } else { t.ink });
    paint_left(
        ui,
        egui::Rect::from_min_size(
            egui::pos2(rect.left() + 12.0, rect.top()),
            Vec2::new(rect.width() - 36.0, rect.height()),
        ),
        g,
        if hovered { t.strong } else { t.ink },
    );
    let cx = rect.right() - 18.0;
    let cy = rect.center().y;
    p.add(egui::Shape::convex_polygon(
        vec![
            egui::pos2(cx - 5.0, cy - 3.0),
            egui::pos2(cx + 5.0, cy - 3.0),
            egui::pos2(cx, cy + 3.0),
        ],
        t.muted,
        Stroke::NONE,
    ));
    let mut out = None;
    egui::Popup::menu(&resp).id(egui::Id::new(id)).show(|ui| {
        ui.set_min_width(rect.width());
        // Long lists scroll inside the popup instead of blowing up the page.
        let rows = |ui: &mut Ui, out: &mut Option<usize>| {
            for (i, opt) in options.iter().enumerate() {
                if navlink(ui, "", opt, opt == selected, t).clicked() {
                    *out = Some(i);
                    ui.close();
                }
            }
        };
        if options.len() > 7 {
            egui::ScrollArea::vertical().max_height(7.0 * 34.0).show(ui, |ui| {
                rows(ui, &mut out);
            });
        } else {
            rows(ui, &mut out);
        }
    });
    resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    out
}

/// Hairline card on the flat page surface: same fill as the page, 1px
/// outline, 20px padding.
fn card(ui: &mut Ui, t: &Tokens, add: impl FnOnce(&mut Ui)) {
    egui::Frame::new()
        .fill(t.bg)
        .stroke(t.hairline())
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), add);
        });
}

/// Square button with a bold mono upper-case label. Primary is a solid
/// inverted block; secondary is outlined and inverts on hover.
fn sbutton(ui: &mut Ui, text: &str, primary: bool, t: &Tokens) -> egui::Response {
    let galley = caps(ui, text, HUD_SIZE, mono_bold(), BUTTON_SPACING, t.ink);
    let (rect, resp) =
        ui.allocate_exact_size(Vec2::new(galley.size().x + 32.0, CONTROL_H), egui::Sense::click());
    let enabled = ui.is_enabled();
    let hovered = resp.hovered() && enabled;
    let (fill, stroke, color) = if primary {
        (t.invert_bg.gamma_multiply(if hovered { 0.8 } else { 1.0 }), Stroke::NONE, t.invert_fg)
    } else if hovered {
        (t.invert_bg, Stroke::NONE, t.invert_fg)
    } else {
        (t.bg, Stroke::new(1.0, t.line_strong), t.ink)
    };
    let fade = |c: Color32| if enabled { c } else { c.gamma_multiply(0.5) };
    ui.painter().rect(
        rect,
        CornerRadius::ZERO,
        fade(fill),
        Stroke::new(stroke.width, fade(stroke.color)),
        egui::StrokeKind::Inside,
    );
    paint_centred(ui, rect, galley, fade(color));
    if enabled { resp.on_hover_cursor(egui::CursorIcon::PointingHand) } else { resp }
}

// ---------------------------------------------------------------- backend ---

const FALLBACK_MAC: &str = "F0:AE:66:EB:CB:31";

const BUTTON_IDS: [SettingId; 6] = [
    SettingId::LeftSinglePress,
    SettingId::RightSinglePress,
    SettingId::LeftDoublePress,
    SettingId::RightDoublePress,
    SettingId::LeftLongPress,
    SettingId::RightLongPress,
];

const BUTTON_LABELS: [&str; 6] = [
    "left single",
    "right single",
    "left double",
    "right double",
    "left long",
    "right long",
];

#[derive(Clone, Default)]
struct ButtonState {
    value: Option<String>,
    options: Vec<String>,
}

#[derive(Clone, Default)]
struct Snapshot {
    mac: String,
    model: String,
    eq_preset: Option<String>,
    eq_options: Vec<String>,
    buttons: Vec<ButtonState>,
    gaming: Option<bool>,
    touch_tone: Option<bool>,
    batt_l: Option<String>,
    batt_r: String,
    charging_l: bool,
    charging_r: bool,
    fw_l: String,
    fw_r: String,
    serial: String,
    tws: String,
}

impl Snapshot {
    fn batt_frac(s: &str) -> Option<f32> {
        let (a, b) = s.split_once('/')?;
        let (a, b): (f32, f32) = (a.trim().parse().ok()?, b.trim().parse().ok()?);
        if b > 0.0 { Some(a / b) } else { None }
    }
    fn batt_pct(s: &str) -> String {
        match Self::batt_frac(s) {
            Some(f) => format!("{}% — reported by the buds", (f * 100.0).round() as u32),
            None => s.to_owned(),
        }
    }
}

enum Cmd {
    Connect,
    Set(SettingId, Value),
}

enum Event {
    Snapshot(Snapshot),
    Status(String),
    Error(String),
}

fn db_path() -> std::path::PathBuf {
    dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp")).join("openscq30").join("database.sqlite")
}

fn info_text(device: &Arc<dyn OpenSCQ30Device + Send + Sync>, id: SettingId) -> String {
    match device.setting(&id) {
        Some(Setting::Information { translated_value, value }) => {
            if translated_value.is_empty() { value } else { translated_value }
        }
        Some(s) => Value::from(s).to_string(),
        None => "—".into(),
    }
}

/// Battery readings are `Information` with a raw `level/max` value plus a
/// localized percent string — the meter needs the raw form, never the
/// translated one.
fn batt_text(device: &Arc<dyn OpenSCQ30Device + Send + Sync>, id: SettingId) -> String {
    match device.setting(&id) {
        Some(Setting::Information { value, .. }) => value,
        Some(s) => Value::from(s).to_string(),
        None => "—".into(),
    }
}

fn toggle(device: &Arc<dyn OpenSCQ30Device + Send + Sync>, id: SettingId) -> Option<bool> {
    match device.setting(&id) {
        Some(Setting::Toggle { value }) => Some(value),
        _ => None,
    }
}

fn fetch_snapshot(
    device: &Arc<dyn OpenSCQ30Device + Send + Sync>,
    mac: String,
) -> Snapshot {
    let mut snap = Snapshot { mac, ..Default::default() };
    if let Some(Setting::PresetEqualizerProfileSelect { select, value, .. }) =
        device.setting(&SettingId::PresetEqualizerProfile)
    {
        snap.eq_options = select.options.iter().map(|s| s.to_string()).collect();
        snap.eq_preset = value.map(|s| s.to_string());
    }
    for (id, _label) in BUTTON_IDS.iter().zip(BUTTON_LABELS) {
        match device.setting(id) {
            Some(Setting::OptionalSelect { setting, value }) => snap.buttons.push(ButtonState {
                value: value.map(|s| s.to_string()),
                options: setting.options.iter().map(|s| s.to_string()).collect(),
            }),
            Some(Setting::Select { setting, value }) => snap.buttons.push(ButtonState {
                value: Some(value.to_string()),
                options: setting.options.iter().map(|s| s.to_string()).collect(),
            }),
            _ => snap.buttons.push(ButtonState::default()),
        }
    }
    snap.gaming = toggle(device, SettingId::GamingMode);
    snap.touch_tone = toggle(device, SettingId::TouchTone);
    snap.batt_l = Some(batt_text(device, SettingId::BatteryLevelLeft));
    snap.batt_r = batt_text(device, SettingId::BatteryLevelRight);
    snap.charging_l = toggle(device, SettingId::IsChargingLeft).unwrap_or(false);
    snap.charging_r = toggle(device, SettingId::IsChargingRight).unwrap_or(false);
    snap.fw_l = info_text(device, SettingId::FirmwareVersionLeft);
    snap.fw_r = info_text(device, SettingId::FirmwareVersionRight);
    snap.serial = info_text(device, SettingId::SerialNumber);
    snap.tws = info_text(device, SettingId::TwsStatus);
    snap
}

async fn worker_loop(cmd_rx: mpsc::Receiver<Cmd>, ev_tx: mpsc::Sender<Event>) {
    let send = |ev: Event| {
        let _ = ev_tx.send(ev);
    };
    let session = match OpenSCQ30Session::new(db_path()).await {
        Ok(s) => s,
        Err(e) => {
            send(Event::Error(format!("database: {e}")));
            return;
        }
    };
    let mut device: Option<Arc<dyn OpenSCQ30Device + Send + Sync>> = None;
    let mut mac = String::new();

    // initial connect
    match session.paired_devices().await {
        Ok(paired) => match paired.iter().find(|d| !d.is_demo) {
            Some(p) => match session.connect(p.mac_address).await {
                Ok(dev) => {
                    mac = p.mac_address.to_string();
                    send(Event::Status(format!("connected to {mac}")));
                    device = Some(dev.clone());
                    send(Event::Snapshot(fetch_snapshot(&dev, mac.clone())));
                }
                Err(e) => send(Event::Error(format!("connect failed: {e}"))),
            },
            None => send(Event::Error(
                "no paired device — pair with: openscq30 paired-devices add".into(),
            )),
        },
        Err(e) => send(Event::Error(format!("database: {e}"))),
    }

    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            Cmd::Connect => {
                device = None;
                match session.paired_devices().await {
                    Ok(paired) => match paired.iter().find(|d| !d.is_demo) {
                        Some(p) => match session.connect(p.mac_address).await {
                            Ok(dev) => {
                                mac = p.mac_address.to_string();
                                send(Event::Status(format!("reconnected to {mac}")));
                                device = Some(dev.clone());
                                send(Event::Snapshot(fetch_snapshot(&dev, mac.clone())));
                            }
                            Err(e) => send(Event::Error(format!("connect failed: {e}"))),
                        },
                        None => send(Event::Error("no paired device".into())),
                    },
                    Err(e) => send(Event::Error(format!("database: {e}"))),
                }
            }
            Cmd::Set(id, value) => {
                let label = format!("{id}");
                match &device {
                    Some(dev) => {
                        // fallback MAC when nothing paired yet
                        match dev.set_setting_values(vec![(id, value)]).await {
                            Ok(()) => {
                                let snap = fetch_snapshot(dev, mac.clone());
                                let echo = match id {
                                    SettingId::PresetEqualizerProfile => snap
                                        .eq_preset
                                        .clone()
                                        .unwrap_or_else(|| "—".into()),
                                    _ => String::new(),
                                };
                                if echo.is_empty() {
                                    send(Event::Status(format!("{label}: applied")));
                                } else {
                                    send(Event::Status(format!("{label}: buds confirm {echo}")));
                                }
                                send(Event::Snapshot(snap));
                            }
                            Err(e) => send(Event::Error(format!("{label} failed: {e}"))),
                        }
                    }
                    None => {
                        // try direct MAC fallback once
                        match FALLBACK_MAC.parse::<MacAddr6>() {
                            Ok(addr) => match session.connect(addr).await {
                                Ok(dev) => {
                                    mac = addr.to_string();
                                    match dev.set_setting_values(vec![(id, value)]).await {
                                        Ok(()) => {
                                            let snap = fetch_snapshot(&dev, mac.clone());
                                            send(Event::Status(format!("{label}: applied")));
                                            device = Some(dev);
                                            send(Event::Snapshot(snap));
                                        }
                                        Err(e) => send(Event::Error(format!("{label} failed: {e}"))),
                                    }
                                }
                                Err(e) => send(Event::Error(format!("connect failed: {e}"))),
                            },
                            Err(e) => send(Event::Error(format!("no device: {e}"))),
                        }
                    }
                }
            }
        }
    }
}

// ------------------------------------------------------------------- app ---

struct SoundcoreApp {
    cmd_tx: mpsc::Sender<Cmd>,
    ev_rx: mpsc::Receiver<Event>,
    snap: Option<Snapshot>,
    status: String,
    err: String,
    eq_pick: Option<String>,
    btn_picks: HashMap<usize, Option<String>>,
}

impl SoundcoreApp {
    fn new() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel::<Cmd>();
        let (ev_tx, ev_rx) = mpsc::channel::<Event>();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            rt.block_on(worker_loop(cmd_rx, ev_tx));
        });
        Self {
            cmd_tx,
            ev_rx,
            snap: None,
            status: "connecting…".into(),
            err: String::new(),
            eq_pick: None,
            btn_picks: HashMap::new(),
        }
    }

    fn drain(&mut self) {
        while let Ok(ev) = self.ev_rx.try_recv() {
            match ev {
                Event::Snapshot(s) => {
                    self.eq_pick = s.eq_preset.clone();
                    self.btn_picks = s
                        .buttons
                        .iter()
                        .enumerate()
                        .map(|(i, b)| (i, b.value.clone()))
                        .collect();
                    self.snap = Some(s);
                    self.err.clear();
                }
                Event::Status(s) => self.status = s,
                Event::Error(e) => self.err = e,
            }
        }
    }

    fn send(&mut self, cmd: Cmd) {
        let _ = self.cmd_tx.send(cmd);
    }
}

impl eframe::App for SoundcoreApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let t = Tokens::get(&ctx);
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
        self.drain();

        // header: 48px bar — wordmark cell, tag, subtitle; status badges right.
        egui::Panel::top("header")
            .frame(egui::Frame::new().fill(t.bg))
            .exact_size(HEADER_H)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let (cell, _) = ui.allocate_exact_size(Vec2::new(232.0, HEADER_H), egui::Sense::hover());
                    ui.painter().rect_filled(cell, CornerRadius::ZERO, t.bg);
                    let mut job = egui::text::LayoutJob::default();
                    job.append("SOUND", 0.0, egui::TextFormat { font_id: FontId::new(17.0, display()), color: t.strong, ..Default::default() });
                    job.append("CORE", 0.0, egui::TextFormat { font_id: FontId::new(17.0, display()), color: t.accent_ink, ..Default::default() });
                    let g = ui.painter().layout_job(job);
                    ui.painter().galley(egui::pos2(cell.left() + 20.0, cell.center().y - g.size().y / 2.0), g, t.strong);
                    ui.painter().vline(cell.right(), cell.y_range(), t.hairline());
                    ui.add_space(12.0);
                    tag(ui, "R50I", t.invert_bg, t.invert_fg);
                    ui.add_space(8.0);
                    hud(ui, "native linux configurator", t.faint);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(20.0);
                        match &self.snap {
                            Some(s) => {
                                let pct = Snapshot::batt_frac(&s.batt_r).map(|f| (f * 100.0).round() as u32);
                                match pct {
                                    Some(p) => {
                                        let c = if p > 50 { t.good } else if p > 20 { t.warn } else { t.bad };
                                        badge(ui, &format!("battery {p}%"), c, &t);
                                    }
                                    None => badge(ui, "battery —", t.faint, &t),
                                }
                                ui.add_space(6.0);
                                badge(ui, "connected", t.good, &t);
                            }
                            None => {
                                badge(ui, "no device", t.bad, &t);
                            }
                        }
                    });
                });
            });

        // footer: 34px bar — status left, refresh + reconnect + error right.
        egui::Panel::bottom("footer")
            .frame(egui::Frame::new().fill(t.bg).inner_margin(egui::Margin::symmetric(20, 0)))
            .exact_size(FOOTER_H)
            .show(ui, |ui| {
                // Stretch the row to the full bar height so status, buttons
                // and error all sit on the vertical centre.
                let h = ui.available_height();
                ui.horizontal_centered(|ui| {
                    ui.set_min_height(h);
                    ui.spacing_mut().item_spacing.x = 0.0;
                    hud(ui, &self.status, t.muted);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if sbutton(ui, "refresh", true, &t).clicked() {
                            self.send(Cmd::Connect);
                        }
                        ui.add_space(8.0);
                        if sbutton(ui, "reconnect", false, &t).clicked() {
                            self.send(Cmd::Connect);
                        }
                        ui.add_space(12.0);
                        if !self.err.is_empty() {
                            hud(ui, &self.err, t.bad);
                        }
                    });
                });
            });

        // centre: one page — device row on top, sound full width, controls
        // row (buttons + modes) below.
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(t.bg).inner_margin(egui::Margin { left: 20, right: 20, top: 12, bottom: 8 }))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    eyebrow(ui, "dv", "device", &t);
                    ui.add_space(6.0);
                    if ui.available_width() < 760.0 {
                        self.page_device(ui, &t);
                        ui.add_space(8.0);
                        self.page_power(ui, &t);
                    } else {
                        ui.columns(2, |cols| {
                            self.page_device(&mut cols[0], &t);
                            self.page_power(&mut cols[1], &t);
                        });
                    }
                    ui.add_space(8.0);
                    eyebrow(ui, "sn", "sound", &t);
                    ui.add_space(6.0);
                    self.page_eq(ui, &t);
                    ui.add_space(8.0);
                    eyebrow(ui, "ct", "controls", &t);
                    ui.add_space(6.0);
                    if ui.available_width() < 560.0 {
                        self.page_buttons(ui, &t);
                        ui.add_space(12.0);
                        self.page_modes(ui, &t);
                    } else {
                        ui.columns(2, |cols| {
                            self.page_buttons(&mut cols[0], &t);
                            self.page_modes(&mut cols[1], &t);
                        });
                    }
                });
            });
    }
}

impl SoundcoreApp {
    fn page_device(&mut self, ui: &mut Ui, t: &Tokens) {
        card(ui, t, |ui| {
            hud(ui, "link status", t.muted);
            ui.add_space(4.0);
            match &self.snap {
                Some(s) => {
                    egui::Grid::new("devstat").num_columns(2).spacing([16.0, 4.0]).show(ui, |ui| {
                        hud(ui, "mac", t.faint);
                        ui.monospace(&s.mac);
                        ui.end_row();
                        hud(ui, "serial", t.faint);
                        ui.monospace(&s.serial);
                        ui.end_row();
                        hud(ui, "firmware l", t.faint);
                        ui.monospace(&s.fw_l);
                        ui.end_row();
                        hud(ui, "firmware r", t.faint);
                        ui.monospace(&s.fw_r);
                        ui.end_row();
                        hud(ui, "tws", t.faint);
                        ui.monospace(&s.tws);
                        ui.end_row();
                    });
                }
                None => {
                    ui.label(RichText::new("Buds not reachable. Keep them connected, then Reconnect.").color(t.warn));
                }
            }
        });
    }

    fn page_power(&mut self, ui: &mut Ui, t: &Tokens) {
        card(ui, t, |ui| {
            hud(ui, "battery", t.muted);
            ui.add_space(2.0);
            match &self.snap {
                Some(s) => {
                    let l = s.batt_l.as_deref().unwrap_or("—");
                    let frac = Snapshot::batt_frac(l);
                    let c = frac.map(|f| if f * 100.0 > 50.0 { t.good } else if f * 100.0 > 20.0 { t.warn } else { t.bad }).unwrap_or(t.faint);
                    let cap = format!("left {} — {}{}", Snapshot::batt_pct(l), "reported by the buds", if s.charging_l { " · charging" } else { "" });
                    meter(ui, frac, &cap, c, t);
                    ui.add_space(2.0);
                    let frac_r = Snapshot::batt_frac(&s.batt_r);
                    let c_r = frac_r.map(|f| if f * 100.0 > 50.0 { t.good } else if f * 100.0 > 20.0 { t.warn } else { t.bad }).unwrap_or(t.faint);
                    let cap_r = format!("right {} — {}{}", Snapshot::batt_pct(&s.batt_r), "reported by the buds", if s.charging_r { " · charging" } else { "" });
                    meter(ui, frac_r, &cap_r, c_r, t);
                }
                None => meter(ui, None, "no reading yet", t.faint, t),
            }
        });
    }

    fn page_eq(&mut self, ui: &mut Ui, t: &Tokens) {
        card(ui, t, |ui| {
            hud(ui, "equalizer preset", t.muted);
            ui.add_space(4.0);
            let (options, current) = match &self.snap {
                Some(s) => (s.eq_options.clone(), self.eq_pick.clone().or(s.eq_preset.clone())),
                None => (Vec::new(), None),
            };
            if options.is_empty() {
                hud(ui, "no reading yet", t.faint);
                return;
            }
            let sel = current.clone().unwrap_or_else(|| "—".into());
            if let Some(i) = dropdown(ui, "eqpreset", &sel, &options, t) {
                let opt = options[i].clone();
                self.eq_pick = Some(opt.clone());
                self.send(Cmd::Set(
                    SettingId::PresetEqualizerProfile,
                    Value::OptionalString(Some(Cow::Owned(opt))),
                ));
            }
            ui.add_space(2.0);
            hud(ui, "writes straight to the buds — footer confirms what they echo back", t.faint);
        });
    }

    fn page_buttons(&mut self, ui: &mut Ui, t: &Tokens) {
        card(ui, t, |ui| {
            hud(ui, "touch map", t.muted);
            ui.add_space(4.0);
            let rows: Vec<(String, Option<String>, Vec<String>)> = match &self.snap {
                Some(s) => BUTTON_LABELS
                    .iter()
                    .enumerate()
                    .map(|(i, label)| {
                        let b = s.buttons.get(i).cloned().unwrap_or_default();
                        (label.to_string(), self.btn_picks.get(&i).cloned().unwrap_or(b.value), b.options)
                    })
                    .collect(),
                None => Vec::new(),
            };
            if rows.is_empty() {
                hud(ui, "no reading yet", t.faint);
                return;
            }
            for (i, (label, current, options)) in rows.iter().enumerate() {
                ui.horizontal(|ui| {
                    hud(ui, label, t.muted);
                    let sel = current.clone().unwrap_or_else(|| "—".into());
                    if let Some(pick) = dropdown(ui, format!("btn{i}"), &sel, options, t) {
                        let opt = options[pick].clone();
                        self.btn_picks.insert(i, Some(opt.clone()));
                        self.send(Cmd::Set(
                            BUTTON_IDS[i],
                            Value::OptionalString(Some(Cow::Owned(opt))),
                        ));
                    }
                });
            }
            ui.add_space(2.0);
            hud(ui, "single / double / long press per bud", t.faint);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if sbutton(ui, "reset buttons", false, t).clicked() {
                        self.send(Cmd::Set(SettingId::ResetButtonsToDefault, Value::Bool(false)));
                    }
                });
            });
        });
    }

    fn page_modes(&mut self, ui: &mut Ui, t: &Tokens) {
        card(ui, t, |ui| {
            hud(ui, "modes", t.muted);
            ui.add_space(4.0);
            let (gaming, tone) = match &self.snap {
                Some(s) => (s.gaming, s.touch_tone),
                None => (None, None),
            };
            match gaming {
                Some(on) => {
                    if navlink(ui, "", "gaming mode", on, t).clicked() {
                        self.send(Cmd::Set(SettingId::GamingMode, Value::Bool(!on)));
                    }
                }
                None => hud(ui, "gaming mode —", t.faint),
            }
            match tone {
                Some(on) => {
                    if navlink(ui, "", "touch tone", on, t).clicked() {
                        self.send(Cmd::Set(SettingId::TouchTone, Value::Bool(!on)));
                    }
                }
                None => hud(ui, "touch tone —", t.faint),
            }
        });
    }
}

fn main() {
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1120.0, 720.0]).with_title("SOUNDCORE"),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native("SOUNDCORE", opts, Box::new(|cc| {
        apply_style(&cc.egui_ctx);
        Ok(Box::new(SoundcoreApp::new()))
    })) {
        eprintln!("UI_FAIL: {e}");
    }
}
