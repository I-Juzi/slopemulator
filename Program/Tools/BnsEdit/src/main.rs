#![cfg_attr(windows, windows_subsystem = "windows")]
//! bnsedit: change client data files with simple buttons. Changes live in packs, like mods.
//! Nothing in the game folder changes until the user clicks Build & Play.

mod backend;
mod demo;
mod install;
mod labels;
mod packs;
mod real;
mod rules;
mod settings;
mod theme;

use backend::{Backend, Field, Value};
use egui::{Color32, RichText, Sense, Stroke};
use packs::{Pack, PackChange, Packs};
use rules::{Cond, How, Op};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use theme::*;

const DARK_ACCENT: Color32 = Color32::from_rgb(0x0B, 0x4F, 0x48);
const DARK_ORANGE: Color32 = Color32::from_rgb(0x8A, 0x3B, 0x08);

#[derive(Clone, Copy, PartialEq, Debug)]
enum Screen {
    Home,
    Find,
    Many,
    Changes,
    Setup,
}

/// What the search looks in.
#[derive(Clone, PartialEq, Debug)]
enum Filter {
    All,
    Kind(usize),
    Table(String),
}

/// Result of reading a game layout (schema) from an exe on a worker thread.
enum ReadOut {
    Done {
        dir: PathBuf,
        wide: bool,
        tables: usize,
        unknown: String,
    },
    Protected(String),
    Failed(String),
}

/// Window to choose the target of a link field.
struct Picker {
    table: String,
    record: usize,
    field: Field,
    target: String,
    query: String,
    results: Option<Vec<usize>>,
}

/// The "change many at once" sentence.
struct RuleForm {
    table: String,
    c1: Cond,
    use_c2: bool,
    c2: Cond,
    target: String,
    how: How,
    value: String,
}

struct PreviewRow {
    record: usize,
    now: Value,
    after: Value,
}

struct Preview {
    table: String,
    target: String,
    label: String,
    scanned: usize,
    rows: Vec<PreviewRow>,
}

struct App {
    backend: Box<dyn Backend>,
    screen: Screen,
    spec: real::OpenSpec,
    loading: Option<Receiver<Result<real::Real, String>>>,
    last_game: Option<PathBuf>,
    message: Option<(String, bool)>,
    now: f64,
    // Find
    filter: Filter,
    query: String,
    query_seen: String,
    query_time: f64,
    searched: Option<(Filter, String)>,
    results: Vec<(String, usize)>,
    selected: Option<(String, usize)>,
    show_all: bool,
    buffers: HashMap<(String, usize), String>,
    picker: Option<Picker>,
    // Packs
    packs: Packs,
    history: Vec<Packs>,
    future: Vec<Packs>,
    changed_rows: HashSet<(String, usize)>,
    renaming: Option<(usize, String)>,
    confirm_delete: Option<usize>,
    // Many
    rule: RuleForm,
    preview: Option<Preview>,
    // Setup
    reading: Option<(bool, Receiver<ReadOut>)>,
    protected: [Option<String>; 2],
    // Build & Play
    build_open: bool,
    pack_done: Option<String>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);
        let empty = || Cond {
            field: String::new(),
            op: Op::Is,
            value: String::new(),
        };
        Self {
            backend: Box::new(demo::Demo::new()),
            // BNSEDIT_SCREEN opens that screen first (for screenshots).
            screen: match std::env::var("BNSEDIT_SCREEN").as_deref() {
                Ok("find") => Screen::Find,
                Ok("many") => Screen::Many,
                Ok("changes") => Screen::Changes,
                Ok("setup") => Screen::Setup,
                _ => Screen::Home,
            },
            spec: real::OpenSpec::default(),
            loading: None,
            last_game: None,
            message: None,
            now: 0.0,
            filter: Filter::All,
            query: std::env::var("BNSEDIT_QUERY").unwrap_or_default(),
            query_seen: String::new(),
            query_time: 0.0,
            searched: None,
            results: Vec::new(),
            selected: None,
            show_all: false,
            buffers: HashMap::new(),
            picker: None,
            packs: Packs::default(),
            history: Vec::new(),
            future: Vec::new(),
            changed_rows: HashSet::new(),
            renaming: None,
            confirm_delete: None,
            rule: RuleForm {
                table: "item".into(),
                c1: empty(),
                use_c2: false,
                c2: empty(),
                target: String::new(),
                how: How::Percent,
                value: "50".into(),
            },
            preview: None,
            reading: None,
            protected: [None, None],
            build_open: false,
            pack_done: None,
        }
        .with_saved()
        .with_args(&cc.egui_ctx)
    }

    fn with_saved(mut self) -> Self {
        let saved = settings::Settings::load();
        self.spec.schema32 = saved.schema32;
        self.spec.schema64 = saved.schema64;
        self.last_game = saved.game.filter(|g| g.is_dir());
        if let Some(g) = &self.last_game {
            self.spec.game = g.clone();
        }
        self
    }

    fn save_settings(&self) {
        let game = if self.spec.game.as_os_str().is_empty() {
            self.last_game.clone()
        } else {
            Some(self.spec.game.clone())
        };
        settings::Settings {
            game,
            schema32: self.spec.schema32.clone(),
            schema64: self.spec.schema64.clone(),
        }
        .save();
    }

    /// `bnsedit [GAME_FOLDER] [--schema32 DIR] [--schema64 DIR]`
    fn with_args(mut self, ctx: &egui::Context) -> Self {
        let mut args = std::env::args().skip(1);
        let mut open = false;
        while let Some(a) = args.next() {
            match a.as_str() {
                "--schema32" => self.spec.schema32 = args.next().map(Into::into),
                "--schema64" => self.spec.schema64 = args.next().map(Into::into),
                _ => {
                    self.spec.game = a.into();
                    open = true;
                }
            }
        }
        if open {
            self.start_open(ctx);
        }
        self
    }

    fn has_layout(&self) -> bool {
        self.spec.schema32.is_some() || self.spec.schema64.is_some()
    }

    fn start_open(&mut self, ctx: &egui::Context) {
        if !self.has_layout() {
            self.screen = Screen::Setup;
            self.message = Some((
                "Set a game layout first. Click \"Read from Client.exe…\" or \"Use a schema folder…\".".into(),
                false,
            ));
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let spec = self.spec.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(real::Real::open(spec));
            ctx.request_repaint();
        });
        self.loading = Some(rx);
    }

    fn poll_loading(&mut self) {
        let Some(rx) = &self.loading else { return };
        match rx.try_recv() {
            Ok(Ok(real)) => {
                self.loading = None;
                self.backend = Box::new(real);
                self.last_game = Some(self.spec.game.clone());
                self.save_settings();
                self.pack_done = None;
                self.selected = None;
                self.searched = None;
                self.preview = None;
                self.buffers.clear();
                self.history.clear();
                self.future.clear();
                self.packs = Packs::load();
                let (moved, failed) = self.sync();
                if moved + failed > 0 {
                    self.message = Some((
                        format!(
                            "{} saved changes do not fit these game files. They stay in their pack, but bnsedit does not use them.",
                            moved + failed
                        ),
                        false,
                    ));
                }
                if self.screen == Screen::Setup {
                    self.screen = Screen::Home;
                }
            }
            Ok(Err(e)) => {
                self.loading = None;
                let text = if e.contains("version or kind mismatch") {
                    format!("The game layout does not match these game files. They are from different game versions.\nOpen \"Game setup\". Use a layout from the same game version as the files.\n({e})")
                } else if e.contains("missing width-specific") {
                    "Set both game layouts first: 32-bit and 64-bit. Open \"Game setup\".".to_string()
                } else {
                    format!("Could not open the game files.\n{e}")
                };
                self.screen = Screen::Setup;
                self.message = Some((text, true));
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(_) => {
                self.loading = None;
                self.message = Some(("Opening stopped unexpectedly.".into(), true));
            }
        }
    }

    fn pick_game(&mut self, ctx: &egui::Context) {
        if let Some(dir) = rfd::FileDialog::new()
            .set_title("Pick the game folder with datafile.bin")
            .pick_folder()
        {
            self.spec.game = dir;
            self.save_settings();
            self.start_open(ctx);
        }
    }

    fn is_demo(&self) -> bool {
        self.backend.game().build == "demo"
    }

    // ---------- packs ----------

    /// Puts the data layer in the state the packs ask for. Returns (moved rows, failed sets).
    fn sync(&mut self) -> (usize, usize) {
        let (mut moved, mut failed) = (0, 0);
        for (c, want) in self.packs.desired() {
            if let Some(k) = c.key
                && self.backend.link_key(&c.table, c.record) != Some(k)
            {
                moved += 1;
                continue;
            }
            match self.backend.value(&c.table, c.record, &c.field, c.repeat) {
                Some(cur) if packs::same(&cur, &want) => {}
                Some(_) => {
                    if self
                        .backend
                        .set(&c.table, c.record, &c.field, c.repeat, want)
                        .is_err()
                    {
                        failed += 1;
                    }
                }
                None => failed += 1,
            }
        }
        self.after_change();
        (moved, failed)
    }

    fn after_change(&mut self) {
        self.changed_rows = self
            .packs
            .list
            .iter()
            .filter(|p| p.on)
            .flat_map(|p| p.changes.iter().map(|c| (c.table.clone(), c.record)))
            .collect();
        self.buffers.clear();
        if !self.is_demo()
            && let Err(e) = self.packs.save()
        {
            self.message = Some((format!("Could not save your changes.\n{e}"), true));
        }
    }

    fn remember(&mut self) {
        self.history.push(self.packs.clone());
        if self.history.len() > 100 {
            self.history.remove(0);
        }
        self.future.clear();
    }

    fn undo(&mut self) {
        if let Some(p) = self.history.pop() {
            self.future.push(std::mem::replace(&mut self.packs, p));
            self.sync();
        }
    }

    fn redo(&mut self) {
        if let Some(p) = self.future.pop() {
            self.history.push(std::mem::replace(&mut self.packs, p));
            self.sync();
        }
    }

    /// One change from the screen: into the data and into the active pack.
    fn set_field(&mut self, table: &str, record: usize, field: &Field, value: Value) {
        let old = field.original.clone().unwrap_or_else(|| field.value.clone());
        if let Err(e) = self
            .backend
            .set(table, record, &field.name, field.repeat, value.clone())
        {
            self.message = Some((e, true));
            return;
        }
        self.remember();
        let change = PackChange {
            table: table.into(),
            record,
            key: self.backend.link_key(table, record),
            title: self.backend.title(table, record),
            field: field.name.clone(),
            repeat: field.repeat,
            label: field_label(field, false),
            old,
            new: value,
        };
        let pack = self.packs.active_mut();
        let was_on = pack.on;
        pack.on = true;
        pack.put(change);
        if was_on {
            self.after_change();
        } else {
            self.sync();
        }
    }

    // ---------- search ----------

    fn kind_table(&self, kind: usize) -> Option<String> {
        let tables = self.backend.tables();
        labels::KINDS[kind]
            .2
            .iter()
            .find_map(|c| tables.iter().find(|t| t.name == *c))
            .map(|t| t.name.clone())
    }

    fn refresh_results(&mut self, ctx: &egui::Context) {
        if self.query != self.query_seen {
            self.query_seen = self.query.clone();
            self.query_time = self.now;
        }
        let key = (self.filter.clone(), self.query.trim().to_string());
        if self.searched.as_ref() == Some(&key) {
            return;
        }
        // Wait until typing stops, so big tables are searched once.
        if !self.query.is_empty() && self.now - self.query_time < 0.35 {
            ctx.request_repaint_after(std::time::Duration::from_millis(120));
            return;
        }
        let q = key.1.clone();
        let mut out = Vec::new();
        match &self.filter {
            Filter::All if q.is_empty() => {}
            Filter::All => {
                for k in 0..labels::KINDS.len() {
                    if let Some(t) = self.kind_table(k) {
                        out.extend(self.backend.search(&t, &q, 150).into_iter().map(|r| (t.clone(), r)));
                    }
                }
            }
            Filter::Kind(k) => {
                if let Some(t) = self.kind_table(*k) {
                    out.extend(self.backend.search(&t, &q, 500).into_iter().map(|r| (t.clone(), r)));
                }
            }
            Filter::Table(t) => {
                out.extend(self.backend.search(t, &q, 500).into_iter().map(|r| (t.clone(), r)));
            }
        }
        self.results = out;
        self.searched = Some(key);
        // BNSEDIT_PICK=1 selects the first result (for screenshots).
        if self.selected.is_none() && std::env::var_os("BNSEDIT_PICK").is_some() {
            self.selected = self.results.first().cloned();
        }
    }

    // ---------- shared pieces ----------

    fn header(&mut self, ui: &mut egui::Ui) {
        let total = self.packs.total();
        egui::Panel::top("header")
            .frame(
                egui::Frame::new()
                    .fill(PAPER)
                    .inner_margin(egui::Margin::symmetric(24, 12))
                    .stroke(Stroke::new(1.0, LINE)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if logo(ui).clicked() {
                        self.screen = Screen::Home;
                    }
                    ui.add_space(8.0);
                    let tabs = [
                        (Screen::Home, "Home".to_string()),
                        (Screen::Find, "Change one thing".to_string()),
                        (Screen::Many, "Change many".to_string()),
                        (Screen::Changes, format!("My changes ({total})")),
                        (Screen::Setup, "Game setup".to_string()),
                    ];
                    for (s, label) in tabs {
                        if tab(ui, &label, self.screen == s).clicked() {
                            self.screen = s;
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let b = primary("Build & Play  ▶");
                        if ui
                            .add_enabled(total > 0 || self.is_demo(), b)
                            .on_disabled_hover_text("Make a change first.")
                            .clicked()
                        {
                            self.build_open = true;
                            self.pack_done = None;
                        }
                        if ui
                            .add_enabled(!self.future.is_empty(), egui::Button::new("Redo"))
                            .on_hover_text("Ctrl+Y")
                            .clicked()
                        {
                            self.redo();
                        }
                        if ui
                            .add_enabled(!self.history.is_empty(), egui::Button::new("Undo"))
                            .on_hover_text("Ctrl+Z")
                            .clicked()
                        {
                            self.undo();
                        }
                        ui.label(
                            RichText::new(self.packs.active_name())
                                .strong()
                                .color(DARK_ACCENT),
                        )
                        .on_hover_text("Change this on the \"My changes\" page.");
                        ui.label(muted("New changes go into"));
                    });
                });
            });
    }

    // ---------- Home ----------

    fn home(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                centered(ui, 1180.0, |ui| {
                    ui.add_space(36.0);
                    ui.label(big("What do you want to do?", 44.0));
                    ui.label(
                        muted("Pick one. Your game files stay safe until you click Build & Play, and you can always put the originals back.")
                            .size(19.0),
                    );
                    ui.add_space(22.0);
                    self.setup_bar(ui);
                    ui.add_space(22.0);
                    let w = ((ui.available_width() - 40.0) / 3.0).max(260.0);
                    let total = self.packs.total();
                    ui.horizontal_wrapped(|ui| {
                        if big_card(ui, w, "⌕", "Change one thing",
                            "Find an item, an NPC, a skill or any text. Change it with simple buttons.",
                            "Start here →", ACCENT, ACCENT_SOFT, true).clicked()
                        {
                            self.screen = Screen::Find;
                        }
                        if big_card(ui, w, "≡", "Change many at once",
                            "Make a rule: every level 50 weapon costs half. See what it changes first.",
                            "Make a rule →", BLUE, BLUE_SOFT, false).clicked()
                        {
                            self.screen = Screen::Many;
                        }
                        if big_card(ui, w, "▤", "My changes",
                            &format!("{total} changes in {}. Turn packs on or off, like mods. Share a pack as one file.", plural(self.packs.list.len(), "pack")),
                            "Open →", ORANGE, ORANGE_SOFT, false).clicked()
                        {
                            self.screen = Screen::Changes;
                        }
                    });
                });
            });
        });
    }

    fn setup_bar(&mut self, ui: &mut egui::Ui) {
        let demo = self.is_demo();
        let fill = if demo { ORANGE_SOFT } else { CARD };
        card_frame()
            .fill(fill)
            .stroke(Stroke::new(1.0, if demo { ORANGE } else { LINE }))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(big("Game setup", 20.0));
                        ui.horizontal(|ui| {
                            let mark = |ok: bool| if ok { ("✔", ACCENT) } else { ("✖", RED) };
                            let game = if demo {
                                "Demo data. Open your game to edit it.".to_string()
                            } else {
                                format!("Game: {}", self.spec.game.display())
                            };
                            for (ok, text) in [
                                (!demo, game.as_str()),
                                (self.spec.schema32.is_some(), "32-bit layout"),
                                (self.spec.schema64.is_some(), "64-bit layout"),
                            ] {
                                let (m, c) = mark(ok);
                                ui.label(RichText::new(m).color(c).strong());
                                ui.label(text);
                                ui.add_space(10.0);
                            }
                        });
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Game setup").clicked() {
                            self.screen = Screen::Setup;
                        }
                        if demo && let Some(last) = self.last_game.clone() {
                            if ui
                                .add(primary("Open my game"))
                                .on_hover_text(last.display().to_string())
                                .clicked()
                            {
                                self.spec.game = last;
                                self.start_open(&ui.ctx().clone());
                            }
                        } else if demo && ui.add(primary("Pick game folder…")).clicked() {
                            self.pick_game(&ui.ctx().clone());
                        }
                    });
                });
            });
    }

    // ---------- Find ----------

    fn find(&mut self, ui: &mut egui::Ui) {
        self.refresh_results(&ui.ctx().clone());
        egui::Panel::top("findbar")
            .frame(
                egui::Frame::new()
                    .fill(GROUND)
                    .inner_margin(egui::Margin::symmetric(24, 14)),
            )
            .show(ui, |ui| {
                let edit = egui::TextEdit::singleline(&mut self.query)
                    .hint_text("Type a name, a number or any text")
                    .font(egui::FontId::proportional(20.0))
                    .margin(egui::Margin::symmetric(16, 12))
                    .desired_width(f32::INFINITY);
                ui.add(edit);
                ui.horizontal(|ui| {
                    if chip(ui, "All", self.filter == Filter::All).clicked() {
                        self.filter = Filter::All;
                    }
                    for (i, (label, _, _)) in labels::KINDS.iter().enumerate() {
                        if self.kind_table(i).is_some()
                            && chip(ui, label, self.filter == Filter::Kind(i)).clicked()
                        {
                            self.filter = Filter::Kind(i);
                        }
                    }
                    let other = match &self.filter {
                        Filter::Table(t) => t.clone(),
                        _ => "Other…".into(),
                    };
                    let mut pick = None;
                    egui::ComboBox::from_id_salt("othertable")
                        .selected_text(other)
                        .height(420.0)
                        .show_ui(ui, |ui| {
                            for t in self.backend.tables() {
                                if ui.selectable_label(false, format!("{}  ({})", t.name, t.records)).clicked() {
                                    pick = Some(t.name);
                                }
                            }
                        });
                    if let Some(t) = pick {
                        self.filter = Filter::Table(t);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let n = self.results.len();
                        ui.label(muted(if self.searched.as_ref().map(|s| &s.0) != Some(&self.filter)
                            || self.searched.as_ref().map(|s| s.1.as_str()) != Some(self.query.trim())
                        {
                            "Looking…".to_string()
                        } else {
                            format!("{n} found")
                        }));
                    });
                });
            });

        egui::Panel::left("results")
            .resizable(true)
            .default_size(400.0)
            .size_range(340.0..=720.0)
            .frame(
                egui::Frame::new()
                    .fill(GROUND)
                    .inner_margin(egui::Margin::same(14))
                    .stroke(Stroke::new(1.0, LINE)),
            )
            .show(ui, |ui| {
                let done = self.searched.is_some();
                if self.results.is_empty() && done {
                    ui.add_space(30.0);
                    ui.vertical_centered(|ui| {
                        if self.query.trim().is_empty() && self.filter == Filter::All {
                            ui.label(big("Type to search", 20.0));
                            ui.label(muted("Or pick Items, NPCs, Skills or Text to see a list."));
                        } else {
                            ui.label(big("Nothing found.", 20.0));
                            ui.label(muted("Try fewer letters."));
                        }
                    });
                    return;
                }
                let row_h = 58.0;
                let results = self.results.clone();
                egui::ScrollArea::vertical().auto_shrink([false, false]).show_rows(ui, row_h, results.len(), |ui, range| {
                    for (t, r) in &results[range] {
                        let title = self.backend.title(t, *r);
                        let sub = format!("{} · {}", labels::kind_of(t), self.backend.subtitle(t, *r));
                        let changed = self.changed_rows.contains(&(t.clone(), *r));
                        let on = self.selected.as_ref() == Some(&(t.clone(), *r));
                        if result_row(ui, row_h, &title, &sub, changed, on).clicked() {
                            self.selected = Some((t.clone(), *r));
                            self.buffers.clear();
                        }
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(GROUND).inner_margin(egui::Margin::same(24)))
            .show(ui, |ui| {
                let Some((table, record)) = self.selected.clone() else {
                    ui.add_space(80.0);
                    ui.vertical_centered(|ui| {
                        ui.label(big("Pick something on the left", 26.0));
                        ui.label(muted("Then change it here. Changes go into your pack."));
                    });
                    return;
                };
                self.record_view(ui, &table, record);
            });
    }

    fn record_view(&mut self, ui: &mut egui::Ui, table: &str, record: usize) {
        let title = self.backend.title(table, record);
        let sub = self.backend.subtitle(table, record);
        ui.horizontal(|ui| {
            ui.label(big(&title, 30.0));
            pill(ui, &labels::kind_of(table), BLUE_SOFT, BLUE);
            if self.changed_rows.contains(&(table.to_string(), record)) {
                pill(ui, "changed", ORANGE_SOFT, DARK_ORANGE);
            }
        });
        ui.label(muted(sub));
        ui.add_space(10.0);
        let fields = self.backend.fields(table, record);
        let mut simple: Vec<&Field> = fields
            .iter()
            .filter(|f| labels::friendly(&f.name).is_some() && !matches!(f.value, Value::Raw { .. }))
            .collect();
        simple.sort_by_key(|f| (labels::order(&f.name), f.repeat));
        let simple_view = !self.show_all && simple.len() >= 2;
        let visible: Vec<&Field> = if simple_view {
            simple
        } else if self.show_all {
            fields.iter().collect()
        } else {
            fields.iter().filter(|f| !f.hidden).take(12).collect()
        };
        let expert = self.show_all;
        egui::ScrollArea::vertical().show(ui, |ui| {
            let col = 300.0;
            let avail = ui.available_width() - 14.0;
            let per_row = ((avail + 16.0) / (col + 16.0)).floor().max(1.0);
            let w = (avail - 16.0 * (per_row - 1.0)) / per_row;
            ui.spacing_mut().item_spacing = egui::vec2(16.0, 16.0);
            let mut commit = None;
            for row in visible.chunks(per_row as usize) {
                ui.horizontal_top(|ui| {
                    for f in row {
                        if let Some(v) = self.field_card(ui, w, table, record, f, expert) {
                            commit = Some(((*f).clone(), v));
                        }
                    }
                });
            }
            if let Some((f, v)) = commit {
                self.set_field(table, record, &f, v);
            }
            ui.add_space(8.0);
            let label = if self.show_all {
                "Show the simple view".to_string()
            } else {
                format!("Show all {} fields (for experts)", fields.len())
            };
            if ui.button(label).clicked() {
                self.show_all = !self.show_all;
            }
        });
    }

    /// Draws one field. Returns a new value when the user commits a change.
    fn field_card(
        &mut self,
        ui: &mut egui::Ui,
        w: f32,
        table: &str,
        record: usize,
        f: &Field,
        expert: bool,
    ) -> Option<Value> {
        let mut out = None;
        let changed = f.original.is_some();
        let frame = card_frame().stroke(if changed {
            Stroke::new(2.0, ORANGE)
        } else {
            Stroke::new(1.0, LINE)
        });
        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(w - 40.0);
                ui.set_min_height(104.0);
                ui.label(RichText::new(field_label(f, expert)).strong().color(MUTED));
                if f.locked {
                    ui.disable();
                }
                match &f.value {
                    Value::Text(t) => {
                        let key = (f.name.clone(), f.repeat);
                        let buf = self.buffers.entry(key).or_insert_with(|| t.clone());
                        let r = ui.add(
                            egui::TextEdit::singleline(buf)
                                .desired_width(f32::INFINITY)
                                .margin(egui::Margin::symmetric(12, 10)),
                        );
                        if r.lost_focus() && *buf != *t {
                            out = Some(Value::Text(buf.clone()));
                        }
                    }
                    Value::Int(v) => {
                        ui.horizontal(|ui| {
                            if ui.add(egui::Button::new("−").min_size(egui::vec2(44.0, 44.0))).clicked() {
                                out = Some(Value::Int(v.saturating_sub(1)));
                            }
                            let mut n = *v;
                            let mut dv = egui::DragValue::new(&mut n).speed(1.0);
                            if let (Some(lo), Some(hi)) = (f.min, f.max) {
                                dv = dv.range(lo as i64..=hi as i64);
                            }
                            let gap = ui.spacing().item_spacing.x;
                            let r = ui.add_sized([ui.available_width() - 44.0 - gap, 44.0], dv);
                            if r.changed() && n != *v {
                                out = Some(Value::Int(n));
                            }
                            if ui.add(egui::Button::new("+").min_size(egui::vec2(44.0, 44.0))).clicked() {
                                out = Some(Value::Int(v.saturating_add(1)));
                            }
                        });
                    }
                    Value::Float(v) => {
                        let mut n = *v;
                        let mut dv = egui::DragValue::new(&mut n).speed(0.1);
                        if let (Some(lo), Some(hi)) = (f.min, f.max) {
                            dv = dv.range(lo as f32..=hi as f32);
                        }
                        if ui.add_sized([ui.available_width(), 44.0], dv).changed() && n != *v {
                            out = Some(Value::Float(n));
                        }
                    }
                    Value::Bool(b) => {
                        ui.horizontal(|ui| {
                            for (label, val) in [("Yes", true), ("No", false)] {
                                let on = *b == val;
                                if option_button(ui, label, on).clicked() && !on {
                                    out = Some(Value::Bool(val));
                                }
                            }
                        });
                    }
                    Value::Enum { index, names } => {
                        let name = |i: u64| names.get(i as usize).cloned().unwrap_or_else(|| format!("#{i}"));
                        if names.len() <= 6 && !names.is_empty() {
                            ui.horizontal_wrapped(|ui| {
                                for (i, n) in names.iter().enumerate() {
                                    let on = *index == i as u64;
                                    if option_button(ui, n, on).clicked() && !on {
                                        out = Some(Value::Enum { index: i as u64, names: names.clone() });
                                    }
                                }
                            });
                        } else {
                            egui::ComboBox::from_id_salt((&f.name, f.repeat, table, record))
                                .selected_text(name(*index))
                                .width(ui.available_width())
                                .height(400.0)
                                .show_ui(ui, |ui| {
                                    for (i, n) in names.iter().enumerate() {
                                        if ui.selectable_label(*index == i as u64, n).clicked() {
                                            out = Some(Value::Enum { index: i as u64, names: names.clone() });
                                        }
                                    }
                                });
                        }
                    }
                    Value::Ref { table: target, key, title } => {
                        let text = title.clone().unwrap_or_else(|| "(not found)".into());
                        ui.add(egui::Label::new(RichText::new(text).strong()).truncate());
                        ui.horizontal(|ui| {
                            if ui.button("Pick another…").clicked() {
                                self.picker = Some(Picker {
                                    table: table.into(),
                                    record,
                                    field: f.clone(),
                                    target: target.clone(),
                                    query: String::new(),
                                    results: None,
                                });
                            }
                            if *key != 0 && ui.button("None").clicked() {
                                out = Some(Value::Ref { table: target.clone(), key: 0, title: Some("(none)".into()) });
                            }
                        });
                        ui.label(muted(format!("Links to: {}", labels::kind_of(target))).small());
                    }
                    Value::Raw { kind, bytes } => {
                        let hex: String = bytes.iter().take(16).map(|b| format!("{b:02x} ")).collect();
                        ui.label(RichText::new(hex).monospace());
                        ui.label(muted(format!("{kind} · cannot be changed")).small());
                    }
                }
                match (&f.original, f.min, f.max) {
                    (Some(o), _, _) => {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("Changed · was {}", short(o)))
                                    .small()
                                    .strong()
                                    .color(DARK_ORANGE),
                            );
                            if ui.small_button("Put back").clicked() {
                                out = Some(o.clone());
                            }
                        });
                    }
                    (None, _, Some(hi)) if matches!(f.value, Value::Text(_)) => {
                        ui.label(muted(format!("Up to {hi} letters")).small());
                    }
                    (None, Some(lo), Some(hi)) if hi - lo <= 10_000_000.0 => {
                        ui.label(muted(format!("From {lo} to {hi}")).small());
                    }
                    _ => {}
                }
            })
        });
        out
    }

    fn picker_window(&mut self, ctx: &egui::Context) {
        let Some(mut p) = self.picker.take() else { return };
        if !self.backend.tables().iter().any(|t| t.name == p.target) {
            self.message = Some((format!("The list \"{}\" is not in these game files.", p.target), true));
            return;
        }
        if p.results.is_none() {
            p.results = Some(self.backend.search(&p.target, &p.query, 300));
        }
        let who = self.backend.title(&p.table, p.record);
        let mut open = true;
        let mut chosen = None;
        egui::Window::new(format!("{} for {who}?", field_label(&p.field, false)))
            .collapsible(false)
            .default_size([600.0, 580.0])
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(muted("Pick one. You see names, not numbers."));
                ui.horizontal(|ui| {
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut p.query)
                            .hint_text("Type a name, then press Enter")
                            .desired_width(ui.available_width() - 90.0),
                    );
                    if (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                        | ui.button("Search").clicked()
                    {
                        p.results = None;
                    }
                });
                let results = p.results.clone().unwrap_or_default();
                if results.is_empty() {
                    ui.label(muted("Nothing found. Try fewer letters."));
                }
                let target = p.target.clone();
                egui::ScrollArea::vertical().max_height(440.0).show_rows(ui, 30.0, results.len(), |ui, range| {
                    for &r in &results[range] {
                        let title = self.backend.title(&target, r);
                        if ui
                            .add(egui::Button::new(&title).min_size(egui::vec2(ui.available_width(), 28.0)))
                            .on_hover_text(self.backend.subtitle(&target, r))
                            .clicked()
                        {
                            chosen = Some((r, title));
                        }
                    }
                });
                if results.len() >= 300 {
                    ui.label(muted("First 300 shown. Type more to narrow the list.").small());
                }
            });
        if let Some((r, title)) = chosen {
            match self.backend.link_key(&p.target, r) {
                Some(key) => {
                    let value = Value::Ref { table: p.target.clone(), key, title: Some(title) };
                    self.set_field(&p.table, p.record, &p.field, value);
                }
                None => self.message = Some(("This one cannot be linked. Pick another.".into(), true)),
            }
            return;
        }
        if open {
            self.picker = Some(p);
        }
    }

    // ---------- Many ----------

    /// Fields that a rule can use, friendly ones first: (code name, shown name).
    fn rule_fields(&mut self, table: &str) -> Vec<(String, String, Field)> {
        let mut v: Vec<(String, String, Field)> = self
            .backend
            .fields(table, 0)
            .into_iter()
            .filter(|f| f.repeat == 0 && !f.locked && !matches!(f.value, Value::Raw { .. }))
            .map(|f| {
                let shown = match labels::friendly(&f.name) {
                    Some(n) => format!("{n}  ({})", f.name),
                    None => f.name.clone(),
                };
                (f.name.clone(), shown, f)
            })
            .collect();
        v.sort_by_key(|(n, _, _)| labels::order(n) == usize::MAX);
        v
    }

    fn many(&mut self, ui: &mut egui::Ui) {
        let tables = self.backend.tables();
        if !tables.iter().any(|t| t.name == self.rule.table)
            && let Some(t) = self.kind_table(0).or(tables.first().map(|t| t.name.clone()))
        {
            self.rule.table = t;
        }
        let table = self.rule.table.clone();
        let fields = self.rule_fields(&table);
        let shown = |code: &str| {
            fields
                .iter()
                .find(|f| f.0 == code)
                .map_or_else(|| "pick a field".to_string(), |f| f.1.clone())
        };
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                centered(ui, 1100.0, |ui| {
                    ui.add_space(28.0);
                    ui.label(big("Change many at once", 40.0));
                    ui.label(muted("Make a rule. See what it changes. Then add it to your pack.").size(18.0));
                    ui.add_space(18.0);
                    card_frame().inner_margin(egui::Margin::same(26)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.spacing_mut().item_spacing = egui::vec2(10.0, 14.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.label(big("Every", 20.0));
                            let mut pick = None;
                            egui::ComboBox::from_id_salt("ruletable")
                                .selected_text(labels::kind_of(&table))
                                .height(420.0)
                                .show_ui(ui, |ui| {
                                    for k in 0..labels::KINDS.len() {
                                        if let Some(t) = self.kind_table(k)
                                            && ui.selectable_label(t == table, labels::kind_of(&t)).clicked()
                                        {
                                            pick = Some(t);
                                        }
                                    }
                                    ui.separator();
                                    for t in &tables {
                                        if ui.selectable_label(t.name == table, &t.name).clicked() {
                                            pick = Some(t.name.clone());
                                        }
                                    }
                                });
                            if let Some(t) = pick {
                                self.rule.table = t;
                                self.rule.c1.field.clear();
                                self.rule.c2.field.clear();
                                self.rule.target.clear();
                                self.preview = None;
                            }
                        });
                        for second in [false, true] {
                            if second && !self.rule.use_c2 {
                                if ui.button("+ Add a second condition").clicked() {
                                    self.rule.use_c2 = true;
                                }
                                continue;
                            }
                            ui.horizontal_wrapped(|ui| {
                                ui.label(big(if second { "and" } else { "where" }, 20.0));
                                let c = if second { &mut self.rule.c2 } else { &mut self.rule.c1 };
                                let cur = shown(&c.field);
                                field_combo(ui, ("cond", second), &fields, &mut c.field, &cur);
                                egui::ComboBox::from_id_salt(("op", second))
                                    .selected_text(c.op.label())
                                    .show_ui(ui, |ui| {
                                        for op in Op::ALL {
                                            ui.selectable_value(&mut c.op, op, op.label());
                                        }
                                    });
                                ui.add(egui::TextEdit::singleline(&mut c.value).hint_text("value").desired_width(180.0));
                                if let Some(f) = fields.iter().find(|f| f.0 == c.field) {
                                    ui.label(muted(example(&f.2.value)).small());
                                }
                                if second && ui.button("✕").on_hover_text("Remove this condition").clicked() {
                                    self.rule.use_c2 = false;
                                }
                            });
                        }
                        ui.horizontal_wrapped(|ui| {
                            ui.label(big("change", 20.0));
                            let cur = shown(&self.rule.target);
                            field_combo(ui, "target", &fields, &mut self.rule.target, &cur);
                            ui.label(big("and", 20.0));
                            egui::ComboBox::from_id_salt("how")
                                .selected_text(self.rule.how.label())
                                .show_ui(ui, |ui| {
                                    for h in How::ALL {
                                        ui.selectable_value(&mut self.rule.how, h, h.label());
                                    }
                                });
                            ui.add(egui::TextEdit::singleline(&mut self.rule.value).desired_width(120.0));
                        });
                        ui.add_space(6.0);
                        let ready = !self.rule.c1.field.is_empty() && !self.rule.target.is_empty();
                        if ui
                            .add_enabled(ready, primary("See what it changes"))
                            .on_disabled_hover_text("Pick a field for \"where\" and for \"change\".")
                            .clicked()
                        {
                            self.run_preview(&fields);
                        }
                    });
                    ui.add_space(18.0);
                    self.preview_card(ui);
                });
            });
        });
    }

    fn run_preview(&mut self, fields: &[(String, String, Field)]) {
        let r = &self.rule;
        let Some(target) = fields.iter().find(|f| f.0 == r.target).map(|f| f.2.clone()) else {
            return;
        };
        let count = self
            .backend
            .tables()
            .iter()
            .find(|t| t.name == r.table)
            .map_or(0, |t| t.records);
        let mut conds = vec![r.c1.clone()];
        if r.use_c2 && !r.c2.field.is_empty() {
            conds.push(r.c2.clone());
        }
        let mut names: Vec<String> = conds.iter().map(|c| c.field.clone()).collect();
        names.push(r.target.clone());
        let (how, input, table) = (r.how, r.value.clone(), r.table.clone());
        let mut rows = Vec::new();
        for record in 0..count {
            let vals = self.backend.values(&table, record, &names);
            let ok = conds
                .iter()
                .zip(&vals)
                .all(|(c, v)| v.as_ref().is_some_and(|v| c.matches(v)));
            if !ok {
                continue;
            }
            let Some(now) = vals.last().cloned().flatten() else { continue };
            if let Some(after) = rules::apply(how, &input, &now, target.min, target.max) {
                rows.push(PreviewRow { record, now, after });
            }
        }
        self.preview = Some(Preview {
            table,
            target: target.name.clone(),
            label: field_label(&target, false),
            scanned: count,
            rows,
        });
    }

    fn preview_card(&mut self, ui: &mut egui::Ui) {
        let Some(p) = &self.preview else { return };
        let (n, scanned, table) = (p.rows.len(), p.scanned, p.table.clone());
        let mut add = false;
        card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(big(format!("{n} will change"), 24.0));
                ui.label(muted(format!("Looked at {scanned} {}.", labels::kind_of(&table).to_lowercase())));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let label = format!("Add {n} changes to \"{}\"", self.packs.active_name());
                    if ui.add_enabled(n > 0, primary(&label)).clicked() {
                        add = true;
                    }
                });
            });
            if n == 0 {
                ui.label(muted("Nothing matches. Check the value you typed."));
                return;
            }
            let p = self.preview.as_ref().unwrap();
            let col = (ui.available_width() - 40.0) / 4.0;
            ui.horizontal(|ui| {
                ui.add_sized([col * 2.0, 24.0], egui::Label::new(RichText::new("Name").strong()));
                ui.add_sized([col, 24.0], egui::Label::new(RichText::new(format!("{} now", p.label)).strong()));
                ui.add_sized([col, 24.0], egui::Label::new(RichText::new("After").strong()));
            });
            ui.separator();
            let rows: Vec<(usize, String, String)> = p
                .rows
                .iter()
                .map(|r| (r.record, short(&r.now), short(&r.after)))
                .collect();
            egui::ScrollArea::vertical().max_height(420.0).show_rows(ui, 28.0, rows.len(), |ui, range| {
                for (record, now, after) in &rows[range] {
                    let title = self.backend.title(&table, *record);
                    ui.horizontal(|ui| {
                        ui.add_sized([col * 2.0, 26.0], egui::Label::new(title).truncate());
                        ui.add_sized([col, 26.0], egui::Label::new(muted(now)));
                        ui.add_sized([col, 26.0], egui::Label::new(RichText::new(after).strong().color(ACCENT)));
                    });
                }
            });
        });
        if add {
            self.add_preview();
        }
    }

    fn add_preview(&mut self) {
        let Some(p) = self.preview.take() else { return };
        self.remember();
        let mut changes = Vec::with_capacity(p.rows.len());
        for r in &p.rows {
            changes.push(PackChange {
                table: p.table.clone(),
                record: r.record,
                key: self.backend.link_key(&p.table, r.record),
                title: self.backend.title(&p.table, r.record),
                field: p.target.clone(),
                repeat: 0,
                label: p.label.clone(),
                old: r.now.clone(),
                new: r.after.clone(),
            });
        }
        let n = changes.len();
        let pack = self.packs.active_mut();
        pack.on = true;
        for c in changes {
            pack.put(c);
        }
        let (moved, failed) = self.sync();
        let name = self.packs.active_name();
        self.message = Some((
            if moved + failed == 0 {
                format!("{n} changes are in \"{name}\".")
            } else {
                format!("{n} changes are in \"{name}\". {} could not be used.", moved + failed)
            },
            false,
        ));
    }

    // ---------- My changes ----------

    fn changes(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                centered(ui, 1100.0, |ui| {
                    ui.add_space(28.0);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(big("My changes", 40.0));
                            ui.label(muted("Packs work like mods. Turn a pack on or off. New changes go into the pack marked ADDING HERE.").size(17.0));
                        });
                    });
                    ui.horizontal(|ui| {
                        if ui.add(primary("+ New pack")).clicked() {
                            self.remember();
                            let name = self.packs.free_name("New pack");
                            self.packs.list.push(Pack::new(&name));
                            self.packs.active = self.packs.list.len() - 1;
                            self.after_change();
                        }
                        if ui.button("Load a shared pack…").clicked()
                            && let Some(f) = rfd::FileDialog::new().add_filter("bnsedit pack", &["json"]).pick_file()
                        {
                            match packs::import(&f) {
                                Ok(mut p) => {
                                    self.remember();
                                    p.name = self.packs.free_name(&p.name);
                                    let n = p.changes.len();
                                    self.packs.list.push(p);
                                    let (moved, failed) = self.sync();
                                    self.message = Some((
                                        format!("Pack loaded with {n} changes.{}", if moved + failed > 0 {
                                            format!(" {} do not fit these game files and are not used.", moved + failed)
                                        } else {
                                            String::new()
                                        }),
                                        false,
                                    ));
                                }
                                Err(e) => self.message = Some((e, true)),
                            }
                        }
                    });
                    ui.add_space(12.0);
                    for i in 0..self.packs.list.len() {
                        if i >= self.packs.list.len() {
                            break;
                        }
                        self.pack_card(ui, i);
                        ui.add_space(12.0);
                    }
                });
            });
        });
    }

    fn pack_card(&mut self, ui: &mut egui::Ui, i: usize) {
        let active = self.packs.active == i;
        let mut action: Option<PackAction> = None;
        card_frame()
            .stroke(if active { Stroke::new(2.0, ACCENT) } else { Stroke::new(1.0, LINE) })
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let p = &self.packs.list[i];
                ui.horizontal(|ui| {
                    let mut on = p.on;
                    if toggle(ui, &mut on).on_hover_text("On or off").changed() {
                        action = Some(PackAction::Toggle);
                    }
                    match &mut self.renaming {
                        Some((j, name)) if *j == i => {
                            let r = ui.add(egui::TextEdit::singleline(name).desired_width(280.0));
                            r.request_focus();
                            if ui.button("Save name").clicked()
                                || (r.lost_focus() && ui.input(|k| k.key_pressed(egui::Key::Enter)))
                            {
                                action = Some(PackAction::Rename(name.clone()));
                            }
                        }
                        _ => {
                            ui.label(big(&p.name, 22.0));
                        }
                    }
                    ui.label(muted(format!("{} changes · {}", p.changes.len(), if p.on { "on" } else { "off" })));
                    if active {
                        pill(ui, "ADDING HERE", ACCENT_SOFT, DARK_ACCENT);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.confirm_delete == Some(i) {
                            if ui.add(egui::Button::new(RichText::new("Yes, delete").color(Color32::WHITE)).fill(RED)).clicked() {
                                action = Some(PackAction::Delete);
                            }
                            if ui.button("Keep").clicked() {
                                self.confirm_delete = None;
                            }
                        } else if ui.button("Delete").clicked() {
                            self.confirm_delete = Some(i);
                        }
                        if ui.button("Share…").on_hover_text("Save this pack as one file").clicked() {
                            action = Some(PackAction::Share);
                        }
                        if ui.button("Rename").clicked() {
                            self.renaming = Some((i, p.name.clone()));
                        }
                        if !active && ui.button("Add here").clicked() {
                            action = Some(PackAction::Activate);
                        }
                    });
                });
                if p.changes.is_empty() {
                    ui.label(muted("No changes in this pack yet."));
                    return;
                }
                ui.add_space(4.0);
                let n = p.changes.len();
                egui::CollapsingHeader::new(format!("See the {n} changes"))
                    .id_salt(("pack", i))
                    .default_open(n <= 20)
                    .show(ui, |ui| {
                        let shown = n.min(500);
                        egui::ScrollArea::vertical().id_salt(("packlist", i)).max_height(420.0).show_rows(
                            ui,
                            32.0,
                            shown,
                            |ui, range| {
                                for (j, c) in p.changes[range.clone()].iter().enumerate() {
                                    let j = range.start + j;
                                    ui.horizontal(|ui| {
                                        pill(ui, &labels::kind_of(&c.table), BLUE_SOFT, BLUE);
                                        ui.add(egui::Label::new(RichText::new(&c.title).strong()).truncate());
                                        ui.label(muted(format!("{}:", if c.label.is_empty() { &c.field } else { &c.label })));
                                        ui.label(RichText::new(short(&c.old)).strikethrough().color(MUTED));
                                        ui.label("→");
                                        ui.label(RichText::new(short(&c.new)).strong().color(ACCENT));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.small_button("✕").on_hover_text("Remove this change").clicked() {
                                                action = Some(PackAction::Remove(j));
                                            }
                                            if ui.small_button("Open").clicked() {
                                                action = Some(PackAction::Open(c.table.clone(), c.record));
                                            }
                                        });
                                    });
                                }
                            },
                        );
                        if n > 500 {
                            ui.label(muted(format!("and {} more", n - 500)));
                        }
                    });
            });
        if let Some(a) = action {
            self.pack_action(i, a);
        }
    }

    fn pack_action(&mut self, i: usize, a: PackAction) {
        match a {
            PackAction::Toggle => {
                self.remember();
                self.packs.list[i].on = !self.packs.list[i].on;
                self.sync();
            }
            PackAction::Rename(name) => {
                let name = name.trim();
                if !name.is_empty() {
                    self.remember();
                    self.packs.list[i].name = name.into();
                    self.after_change();
                }
                self.renaming = None;
            }
            PackAction::Activate => {
                self.remember();
                self.packs.active = i;
                self.sync();
            }
            PackAction::Delete => {
                self.remember();
                self.packs.list.remove(i);
                if self.packs.list.is_empty() {
                    self.packs.list.push(Pack::new("My changes"));
                }
                if self.packs.active >= i && self.packs.active > 0 {
                    self.packs.active -= 1;
                }
                self.confirm_delete = None;
                // Changes of the deleted pack go back to their old values.
                let gone = self.history.last().map(|h| h.list[i].changes.clone()).unwrap_or_default();
                self.sync();
                self.revert_orphans(&gone);
            }
            PackAction::Share => {
                let p = &self.packs.list[i];
                if let Some(f) = rfd::FileDialog::new()
                    .set_file_name(format!("{}.json", p.name))
                    .add_filter("bnsedit pack", &["json"])
                    .save_file()
                {
                    match packs::export(p, &f) {
                        Ok(()) => self.message = Some((format!("Pack saved to {}.", f.display()), false)),
                        Err(e) => self.message = Some((e, true)),
                    }
                }
            }
            PackAction::Remove(j) => {
                self.remember();
                let c = self.packs.list[i].changes.remove(j);
                self.sync();
                self.revert_orphans(&[c]);
            }
            PackAction::Open(table, record) => {
                self.selected = Some((table, record));
                self.buffers.clear();
                self.screen = Screen::Find;
            }
        }
    }

    /// Fields no pack touches any more go back to their original value.
    fn revert_orphans(&mut self, gone: &[PackChange]) {
        let still: HashSet<packs::Key> = self
            .packs
            .list
            .iter()
            .flat_map(|p| p.changes.iter().map(PackChange::key))
            .collect();
        for c in gone.iter().filter(|c| !still.contains(&c.key())) {
            if self.backend.link_key(&c.table, c.record) != c.key && c.key.is_some() {
                continue;
            }
            if let Some(cur) = self.backend.value(&c.table, c.record, &c.field, c.repeat)
                && !packs::same(&cur, &c.old)
            {
                let _ = self.backend.set(&c.table, c.record, &c.field, c.repeat, c.old.clone());
            }
        }
        self.after_change();
    }

    // ---------- Game setup ----------

    fn start_read(&mut self, wide: bool, exe: PathBuf, ctx: &egui::Context) {
        let Some(base) = settings::config_dir() else {
            self.message = Some(("No settings folder on this computer.".into(), true));
            return;
        };
        let stem = exe.file_stem().map_or("exe".into(), |s| s.to_string_lossy().into_owned());
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let out = base.join("schemas").join(format!("{stem}-{stamp}"));
        let (tx, rx) = std::sync::mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let r = match bnsschema::export(&exe, None, &out) {
                Ok(found) => ReadOut::Done {
                    dir: out,
                    wide: found.width == bnsschema::Width32or64::Width64,
                    tables: found.tables,
                    unknown: if found.unknown_types.is_empty() {
                        String::new()
                    } else {
                        format!("{:?}", found.unknown_types)
                    },
                },
                Err(bnsschema::Error::Refused(why)) => ReadOut::Protected(why),
                Err(e) => {
                    let _ = std::fs::remove_dir_all(&out);
                    ReadOut::Failed(e.to_string())
                }
            };
            let _ = tx.send(r);
            ctx.request_repaint();
        });
        self.protected[wide as usize] = None;
        self.reading = Some((wide, rx));
    }

    fn poll_reading(&mut self) {
        let Some((card, rx)) = &self.reading else { return };
        let card = *card;
        let out = match rx.try_recv() {
            Ok(o) => o,
            Err(std::sync::mpsc::TryRecvError::Empty) => return,
            Err(_) => ReadOut::Failed("Reading stopped unexpectedly.".into()),
        };
        self.reading = None;
        match out {
            ReadOut::Done { dir, wide, tables, unknown } => {
                if wide {
                    self.spec.schema64 = Some(dir);
                } else {
                    self.spec.schema32 = Some(dir);
                }
                self.save_settings();
                let bits = if wide { 64 } else { 32 };
                let mut text = format!("Read {tables} tables. The {bits}-bit game layout is ready.");
                if wide != card {
                    text += &format!("\nThis exe is {bits}-bit, so it went to the {bits}-bit card.");
                }
                if !unknown.is_empty() {
                    text += &format!("\nSome field types are unknown ({unknown}). You can see them, but not change them.");
                }
                text += "\nUse the exe from the same game version as the data files.";
                if !self.is_demo() {
                    text += "\nOpen the game again to use the new layout.";
                }
                self.message = Some((text, false));
            }
            ReadOut::Protected(why) => self.protected[card as usize] = Some(why),
            ReadOut::Failed(e) => {
                self.message = Some((format!("No game layout found in this exe.\n{e}"), true))
            }
        }
    }

    fn setup(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                centered(ui, 1000.0, |ui| {
                    ui.add_space(28.0);
                    ui.label(big("Game setup", 40.0));
                    ui.label(muted("Two things: where your game is, and the game layout. The layout tells bnsedit what each list holds.").size(17.0));
                    ui.add_space(16.0);
                    self.game_card(ui);
                    ui.add_space(12.0);
                    self.layout_card(ui, false);
                    ui.add_space(12.0);
                    self.layout_card(ui, true);
                    ui.add_space(12.0);
                    card_frame().fill(PAPER).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new("Protected files").strong());
                        ui.label(muted("Some exe files are protected (packed). bnsedit says so and stops. It never unpacks, runs or changes an exe. Use a schema folder for that game instead."));
                    });
                });
            });
        });
    }

    fn game_card(&mut self, ui: &mut egui::Ui) {
        let demo = self.is_demo();
        let game = self.spec.game.clone();
        card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(big("Game folder", 22.0));
                    if game.as_os_str().is_empty() {
                        ui.label(muted("Not picked yet. Pick the folder that holds datafile.bin."));
                    } else {
                        ui.label(muted(game.display().to_string()));
                        ui.horizontal(|ui| {
                            for f in install::FILES {
                                let ok = game.join(f).is_file();
                                ui.label(RichText::new(if ok { "✔" } else { "✖" }).color(if ok { ACCENT } else { MUTED }).strong());
                                ui.label(if ok { RichText::new(f) } else { muted(f) });
                                ui.add_space(6.0);
                            }
                        });
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Pick another folder…").clicked() {
                        self.pick_game(&ui.ctx().clone());
                    }
                    let can_open = !game.as_os_str().is_empty() && self.has_layout();
                    let label = if demo { "Open it" } else { "Open again" };
                    if can_open && ui.add(primary(label)).clicked() {
                        self.start_open(&ui.ctx().clone());
                    }
                    if !demo {
                        pill(ui, "OPEN", ACCENT_SOFT, DARK_ACCENT);
                    }
                });
            });
        });
    }

    fn layout_card(&mut self, ui: &mut egui::Ui, wide: bool) {
        let cur = if wide { self.spec.schema64.clone() } else { self.spec.schema32.clone() };
        let reading = self.reading.as_ref().is_some_and(|r| r.0 == wide);
        let protected = self.protected[wide as usize].clone();
        let (badge, fill, color) = if reading {
            ("READING…", BLUE_SOFT, BLUE)
        } else if protected.is_some() {
            ("PROTECTED", ORANGE_SOFT, DARK_ORANGE)
        } else if cur.as_ref().is_some_and(|p| p.is_dir()) {
            ("READY", ACCENT_SOFT, DARK_ACCENT)
        } else {
            ("NOT SET", GROUND, MUTED)
        };
        let (title, exe, hint) = if wide {
            ("64-bit game layout", "BNSR.exe", "For datafile64.bin. Read it from BNSR.exe or Client64.exe.")
        } else {
            ("32-bit game layout", "Client.exe", "For datafile.bin. Read it from Client.exe.")
        };
        card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(big(title, 22.0));
                pill(ui, badge, fill, color);
                if reading {
                    ui.add(egui::Spinner::new().color(BLUE));
                }
            });
            ui.label(muted(hint));
            if let Some(p) = &cur {
                ui.label(RichText::new(p.display().to_string()).small().color(MUTED));
            }
            if let Some(why) = &protected {
                egui::Frame::new()
                    .fill(ORANGE_SOFT)
                    .corner_radius(egui::CornerRadius::same(12))
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new("This exe is protected. bnsedit does not open protected files.").strong().color(DARK_ORANGE));
                        ui.label(RichText::new(format!("Reason: {why}")).small().color(DARK_ORANGE));
                        ui.label(RichText::new("Use a schema folder for this game instead.").color(DARK_ORANGE));
                    });
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let busy = self.reading.is_some();
                if ui.add_enabled(!busy, primary(&format!("Read from {exe}…"))).clicked()
                    && let Some(p) = rfd::FileDialog::new()
                        .set_title(format!("Pick {exe}"))
                        .add_filter("exe", &["exe"])
                        .pick_file()
                {
                    self.start_read(wide, p, &ui.ctx().clone());
                }
                if ui.add_enabled(!busy, egui::Button::new("Use a schema folder…")).clicked()
                    && let Some(p) = rfd::FileDialog::new().set_title("Pick a TableDefs folder").pick_folder()
                {
                    if wide { self.spec.schema64 = Some(p) } else { self.spec.schema32 = Some(p) }
                    self.protected[wide as usize] = None;
                    self.save_settings();
                }
                if let Some(src) = cur.clone().filter(|p| p.is_dir()) {
                    if ui.button("Save this layout as files…").on_hover_text("Copy the layout files to a folder you pick").clicked()
                        && let Some(dest) = rfd::FileDialog::new().set_title("Pick a folder for the layout files").pick_folder()
                    {
                        let name = src.file_name().map_or_else(|| "layout".into(), |n| n.to_owned());
                        match copy_dir(&src, &dest.join(name)) {
                            Ok((n, to)) => self.message = Some((format!("{n} layout files saved to {}.", to.display()), false)),
                            Err(e) => self.message = Some((e, true)),
                        }
                    }
                    if ui.button("Forget").clicked() {
                        if wide { self.spec.schema64 = None } else { self.spec.schema32 = None }
                        self.save_settings();
                    }
                }
            });
        });
    }

    // ---------- Build & Play ----------

    fn build_window(&mut self, ctx: &egui::Context) {
        if !self.build_open {
            return;
        }
        let game = (!self.is_demo()).then(|| self.spec.game.clone());
        let total = self.packs.total();
        let on = self.packs.list.iter().filter(|p| p.on).count();
        let problems = self.backend.problems();
        let errors: Vec<_> = problems.iter().filter(|p| p.error).collect();
        let done = self.pack_done.is_some();
        let mut open = true;
        egui::Window::new("Build & Play")
            .collapsible(false)
            .resizable(false)
            .default_width(640.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.set_width(620.0);
                let check = if game.is_none() {
                    "Demo data cannot be built. Open your game first.".to_string()
                } else if !errors.is_empty() {
                    format!("{} problems. Fix them first.", errors.len())
                } else {
                    format!("{total} changes in {on} packs that are on. No problems.")
                };
                step(ui, 1, game.is_some() && errors.is_empty(), "Check", &check);
                for p in errors.iter().take(5) {
                    ui.label(RichText::new(format!("{} · row {} · {}: {}", p.table, p.record, p.field, p.message)).small().color(RED));
                }
                step(ui, 2, done, "Back up", "Your original files are copied once to bnsedit-backup in the game folder.");
                step(ui, 3, done, "Pack", "Builds new datafile and localfile, 32 and 64-bit.");
                step(ui, 4, done, "Put into game", "Replaces the files in the game folder. Close the game first.");
                ui.add_space(8.0);
                let ready = game.is_some() && errors.is_empty() && !done;
                let label = if done { "Done. Start the game to test." } else { "Build & Play" };
                if ui.add_enabled(ready, primary(label).min_size(egui::vec2(ui.available_width(), 56.0))).clicked()
                    && let Some(g) = &game
                {
                    self.pack_into_game(g);
                }
                if let Some(d) = &self.pack_done {
                    ui.label(RichText::new(d).color(ACCENT).strong());
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(game.is_some() && errors.is_empty(), egui::Button::new("Build to another folder…"))
                        .clicked()
                        && let Some(dir) = rfd::FileDialog::new().set_title("Pick an empty folder").pick_folder()
                    {
                        match self.backend.pack(&dir) {
                            Ok(rep) => {
                                self.pack_done = Some(format!("{} files written to {}. Copy them into the game yourself.", rep.files.len(), dir.display()))
                            }
                            Err(e) => self.message = Some((e, true)),
                        }
                    }
                    let has = game.as_ref().is_some_and(|g| !install::backed_up(g).is_empty());
                    if ui
                        .add_enabled(has, egui::Button::new("Put the original files back"))
                        .on_disabled_hover_text("No backup yet.")
                        .clicked()
                        && let Some(g) = &game
                    {
                        match install::restore(g) {
                            Ok(_) => {
                                self.pack_done = None;
                                self.message = Some(("Original files are back in the game folder.".into(), false));
                            }
                            Err(e) => self.message = Some((e, true)),
                        }
                    }
                });
                ui.label(muted("If the game does not accept the files, put the originals back. Do not try to force it.").small());
            });
        if !open {
            self.build_open = false;
        }
    }

    /// Pack into a work folder, back up, replace the game files, remove the work folder.
    fn pack_into_game(&mut self, game: &Path) {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let work = install::backup_dir(game).join(format!("pack-{stamp}"));
        let result = self.backend.pack(&work).and_then(|_| install::install(game, &work));
        let _ = std::fs::remove_dir_all(&work);
        match result {
            Ok(names) => self.pack_done = Some(format!("Put into the game: {}.", names.join(", "))),
            Err(e) => self.message = Some((e, true)),
        }
    }
}

enum PackAction {
    Toggle,
    Rename(String),
    Activate,
    Delete,
    Share,
    Remove(usize),
    Open(String, usize),
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.now = ui.input(|i| i.time);
        self.poll_loading();
        self.poll_reading();
        if self.loading.is_some() {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.add_space(ui.available_height() / 3.0);
                ui.vertical_centered(|ui| {
                    ui.add(egui::Spinner::new().size(48.0).color(ACCENT));
                    ui.add_space(12.0);
                    ui.label(big("Opening game files…", 28.0));
                    ui.label(muted("Big files take a few seconds."));
                });
            });
            return;
        }
        let typing = ui.ctx().memory(|m| m.focused().is_some());
        let (undo, redo) = ui.input(|i| {
            (
                i.modifiers.command && i.key_pressed(egui::Key::Z) && !i.modifiers.shift,
                i.modifiers.command
                    && (i.key_pressed(egui::Key::Y) || (i.modifiers.shift && i.key_pressed(egui::Key::Z))),
            )
        });
        if !typing && undo {
            self.undo();
        }
        if !typing && redo {
            self.redo();
        }
        self.header(ui);
        match self.screen {
            Screen::Home => self.home(ui),
            Screen::Find => self.find(ui),
            Screen::Many => self.many(ui),
            Screen::Changes => self.changes(ui),
            Screen::Setup => self.setup(ui),
        }
        self.picker_window(ui.ctx());
        self.build_window(&ui.ctx().clone());
        if let Some((text, error)) = self.message.clone() {
            let mut open = true;
            egui::Window::new(if error { "Problem" } else { "Note" })
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    ui.label(RichText::new(text).color(if error { RED } else { INK }));
                    if ui.button("OK").clicked() {
                        self.message = None;
                    }
                });
            if !open {
                self.message = None;
            }
        }
    }
}

// ---------- helpers ----------

/// Friendly name, or the code name for experts.
fn field_label(f: &Field, expert: bool) -> String {
    let base = match labels::friendly(&f.name) {
        Some(n) if !expert => n.to_string(),
        _ => return if expert { format!("{} [{}]", f.name, f.repeat) } else { f.label() },
    };
    if f.repeat > 0 { format!("{base} {}", f.repeat + 1) } else { base }
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 { format!("1 {word}") } else { format!("{n} {word}s") }
}

fn example(v: &Value) -> String {
    match v {
        Value::Int(_) => "a number, like 50".into(),
        Value::Float(_) => "a number, like 1.5".into(),
        Value::Bool(_) => "yes or no".into(),
        Value::Enum { names, .. } => {
            let some: Vec<_> = names.iter().take(4).cloned().collect();
            if some.is_empty() { "a number".into() } else { format!("like {}", some.join(", ")) }
        }
        Value::Ref { .. } => "a number (the key)".into(),
        Value::Text(_) => "some text".into(),
        Value::Raw { .. } => String::new(),
    }
}

fn field_combo(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    fields: &[(String, String, Field)],
    cur: &mut String,
    shown: &str,
) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(shown)
        .width(300.0)
        .height(440.0)
        .show_ui(ui, |ui| {
            let mut friendly = true;
            for (code, label, _) in fields {
                if friendly && labels::friendly(code).is_none() {
                    friendly = false;
                    ui.separator();
                    ui.label(muted("For experts").small());
                }
                ui.selectable_value(cur, code.clone(), label);
            }
        });
}

/// Copies the files of `src` into `dest` (made new). Returns (files, dest).
fn copy_dir(src: &Path, dest: &Path) -> Result<(usize, PathBuf), String> {
    if dest.exists() {
        return Err(format!("{} is already there. Pick another folder.", dest.display()));
    }
    std::fs::create_dir_all(dest).map_err(|e| format!("Cannot make {}: {e}", dest.display()))?;
    let mut n = 0;
    for e in std::fs::read_dir(src).map_err(|e| e.to_string())? {
        let e = e.map_err(|e| e.to_string())?;
        if e.file_type().map_err(|e| e.to_string())?.is_file() {
            std::fs::copy(e.path(), dest.join(e.file_name())).map_err(|e| e.to_string())?;
            n += 1;
        }
    }
    Ok((n, dest.to_path_buf()))
}

// ---------- widgets ----------

fn logo(ui: &mut egui::Ui) -> egui::Response {
    let (rect, r) = ui.allocate_exact_size(egui::vec2(150.0, 40.0), Sense::click());
    let p = ui.painter();
    let tile = egui::Rect::from_min_size(rect.min, egui::vec2(40.0, 40.0));
    p.rect_filled(tile, 12.0, ACCENT);
    let c = tile.center();
    p.circle_filled(c + egui::vec2(0.0, 5.0), 7.0, Color32::WHITE);
    for (dx, dy) in [(-10.0, -3.0), (-4.0, -9.0), (4.0, -9.0), (10.0, -3.0)] {
        p.circle_filled(c + egui::vec2(dx, dy), 3.0, Color32::WHITE);
    }
    p.text(
        egui::pos2(tile.right() + 12.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        "bnsedit",
        egui::FontId::proportional(22.0),
        INK,
    );
    r.on_hover_text("Home")
}

fn pill(ui: &mut egui::Ui, text: &str, fill: Color32, color: Color32) {
    egui::Frame::new()
        .fill(fill)
        .corner_radius(egui::CornerRadius::same(255))
        .inner_margin(egui::Margin::symmetric(12, 5))
        .show(ui, |ui| ui.label(RichText::new(text).small().strong().color(color)));
}

fn tab(ui: &mut egui::Ui, label: &str, on: bool) -> egui::Response {
    let text = if on {
        RichText::new(label).strong().color(DARK_ACCENT)
    } else {
        RichText::new(label).color(MUTED)
    };
    ui.add(
        egui::Button::new(text)
            .fill(if on { ACCENT_SOFT } else { Color32::TRANSPARENT })
            .stroke(Stroke::NONE),
    )
}

fn chip(ui: &mut egui::Ui, label: &str, on: bool) -> egui::Response {
    option_button(ui, label, on)
}

fn option_button(ui: &mut egui::Ui, label: &str, on: bool) -> egui::Response {
    let b = egui::Button::new(if on {
        RichText::new(label).strong().color(Color32::WHITE)
    } else {
        RichText::new(label)
    })
    .fill(if on { ACCENT } else { CARD })
    .corner_radius(egui::CornerRadius::same(22))
    .min_size(egui::vec2(64.0, 40.0));
    ui.add(b)
}

fn centered(ui: &mut egui::Ui, max: f32, add: impl FnOnce(&mut egui::Ui)) {
    let w = ui.available_width().min(max);
    let pad = ((ui.available_width() - w) / 2.0).max(0.0);
    ui.horizontal(|ui| {
        ui.add_space(pad);
        ui.vertical(|ui| {
            ui.set_width(w);
            add(ui);
        });
    });
}

#[allow(clippy::too_many_arguments)]
fn big_card(
    ui: &mut egui::Ui,
    w: f32,
    icon: &str,
    title: &str,
    text: &str,
    action: &str,
    color: Color32,
    soft: Color32,
    main: bool,
) -> egui::Response {
    let inner = card_frame()
        .stroke(if main { Stroke::new(2.0, color) } else { Stroke::new(1.0, LINE) })
        .corner_radius(egui::CornerRadius::same(24))
        .inner_margin(egui::Margin::same(28))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(w - 56.0);
                ui.set_min_height(220.0);
                egui::Frame::new()
                    .fill(soft)
                    .corner_radius(egui::CornerRadius::same(18))
                    .inner_margin(egui::Margin::same(14))
                    .show(ui, |ui| ui.label(RichText::new(icon).size(30.0).color(color)));
                ui.add_space(6.0);
                ui.label(big(title, 26.0));
                ui.label(muted(text).size(16.0));
                ui.add_space(8.0);
                ui.label(RichText::new(action).strong().color(color));
            })
        });
    let r = ui.interact(inner.response.rect, ui.scope_id().with(title), Sense::click());
    if r.hovered() {
        ui.painter().rect_stroke(inner.response.rect, 24.0, Stroke::new(2.0, color), egui::StrokeKind::Inside);
    }
    r.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn result_row(ui: &mut egui::Ui, h: f32, title: &str, sub: &str, changed: bool, on: bool) -> egui::Response {
    let (rect, r) = ui.allocate_exact_size(egui::vec2(ui.available_width(), h - 4.0), Sense::click());
    let p = ui.painter();
    if on {
        p.rect(rect, 14.0, CARD, Stroke::new(2.0, ACCENT), egui::StrokeKind::Inside);
    } else if r.hovered() {
        p.rect_filled(rect, 14.0, PAPER);
    }
    let x = rect.left() + 14.0;
    let clip = |s: &str, n: usize| {
        if s.chars().count() > n { format!("{}…", s.chars().take(n).collect::<String>()) } else { s.to_string() }
    };
    let max = ((rect.width() - 110.0) / 8.0).max(10.0) as usize;
    p.text(egui::pos2(x, rect.top() + 10.0), egui::Align2::LEFT_TOP, clip(title, max), egui::FontId::proportional(16.0), INK);
    p.text(egui::pos2(x, rect.top() + 31.0), egui::Align2::LEFT_TOP, clip(sub, max + 8), egui::FontId::proportional(13.0), MUTED);
    if changed {
        let tag = egui::Rect::from_min_size(egui::pos2(rect.right() - 82.0, rect.center().y - 12.0), egui::vec2(70.0, 24.0));
        p.rect_filled(tag, 12.0, ORANGE_SOFT);
        p.text(tag.center(), egui::Align2::CENTER_CENTER, "changed", egui::FontId::proportional(12.0), DARK_ORANGE);
    }
    r.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn toggle(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let (rect, mut r) = ui.allocate_exact_size(egui::vec2(64.0, 36.0), Sense::click());
    if r.clicked() {
        *on = !*on;
        r.mark_changed();
    }
    let t = ui.ctx().animate_bool_responsive(r.id, *on);
    let p = ui.painter();
    let bg = if *on { ACCENT } else { Color32::from_rgb(0xD6, 0xCF, 0xBF) };
    p.rect_filled(rect, 18.0, bg);
    let x = egui::lerp(rect.left() + 18.0..=rect.right() - 18.0, t);
    p.circle_filled(egui::pos2(x, rect.center().y), 14.0, Color32::WHITE);
    r
}

fn step(ui: &mut egui::Ui, n: usize, ok: bool, title: &str, text: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), Sense::hover());
        let p = ui.painter();
        if ok {
            p.circle_filled(rect.center(), 18.0, ACCENT);
            p.text(rect.center(), egui::Align2::CENTER_CENTER, "✔", egui::FontId::proportional(16.0), Color32::WHITE);
        } else {
            p.circle(rect.center(), 17.0, GROUND, Stroke::new(2.0, Color32::from_rgb(0xD6, 0xCF, 0xBF)));
            p.text(rect.center(), egui::Align2::CENTER_CENTER, n.to_string(), egui::FontId::proportional(15.0), MUTED);
        }
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(18.0).strong());
            ui.label(muted(text));
        });
    });
    ui.add_space(6.0);
}

fn short(v: &Value) -> String {
    match v {
        Value::Int(i) => i.to_string(),
        Value::Float(f) => format!("{f}"),
        Value::Bool(b) => if *b { "yes" } else { "no" }.into(),
        Value::Enum { index, names } => names.get(*index as usize).cloned().unwrap_or_else(|| format!("#{index}")),
        Value::Ref { key, title, .. } => title.clone().unwrap_or_else(|| format!("#{key}")),
        Value::Text(t) => format!("\"{}\"", t.chars().take(40).collect::<String>()),
        Value::Raw { .. } => "raw".into(),
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("bnsedit")
            .with_inner_size([1440.0, 920.0])
            .with_min_inner_size([960.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native("bnsedit", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
