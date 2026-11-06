use eframe::egui::{self, Color32, CornerRadius, FontId, Frame, Margin, RichText, Stroke, Vec2};
use std::{fs, path::PathBuf};
use wowmc::{candidates, parse_snapshot, revision, snapshots, Library};

const INK: Color32 = Color32::from_rgb(25, 34, 48);
const MUTED: Color32 = Color32::from_rgb(91, 104, 120);
const CANVAS: Color32 = Color32::from_rgb(248, 250, 252);
const SURFACE: Color32 = Color32::WHITE;
const BORDER: Color32 = Color32::from_rgb(221, 227, 234);
const ACCENT: Color32 = Color32::from_rgb(42, 79, 170);
const ACCENT_LIGHT: Color32 = Color32::from_rgb(235, 241, 253);

#[derive(PartialEq)]
enum View {
    Library,
    Import,
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1080.0, 760.0])
            .with_min_inner_size([720.0, 560.0]),
        ..Default::default()
    };
    eframe::run_native(
        "wowmc",
        options,
        Box::new(|cc| {
            set_theme(&cc.egui_ctx);
            let path = wowmc::config::Config::library_path(None)?;
            let installation = candidates().first().cloned().unwrap_or_default();
            Ok(Box::new(App::new(path, installation)))
        }),
    )
}

fn set_theme(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    if let Ok(data) = fs::read(r"C:\Windows\Fonts\segoeui.ttf") {
        fonts
            .font_data
            .insert("system-ui".into(), egui::FontData::from_owned(data).into());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "system-ui".into());
    }
    ctx.set_fonts(fonts);
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::light();
    style.visuals.override_text_color = Some(INK);
    style.visuals.panel_fill = CANVAS;
    style.visuals.extreme_bg_color = SURFACE;
    style.visuals.faint_bg_color = Color32::from_rgb(244, 247, 251);
    style.visuals.selection.bg_fill = ACCENT_LIGHT;
    style.visuals.selection.stroke = Stroke::new(1.5, ACCENT);
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    for widget in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
    ] {
        widget.corner_radius = CornerRadius::same(7);
    }
    style.visuals.widgets.inactive.bg_fill = SURFACE;
    style.visuals.widgets.inactive.weak_bg_fill = SURFACE;
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    style.visuals.widgets.hovered.bg_fill = ACCENT_LIGHT;
    style.visuals.widgets.hovered.weak_bg_fill = ACCENT_LIGHT;
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    style.visuals.widgets.active.bg_fill = ACCENT_LIGHT;
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.5, ACCENT);
    style.spacing.item_spacing = Vec2::new(10.0, 10.0);
    style.spacing.button_padding = Vec2::new(13.0, 8.0);
    style.spacing.interact_size.y = 36.0;
    style.spacing.text_edit_width = 250.0;
    style
        .text_styles
        .insert(egui::TextStyle::Body, FontId::proportional(15.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, FontId::proportional(14.0));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, FontId::proportional(26.0));
    style
        .text_styles
        .insert(egui::TextStyle::Monospace, FontId::monospace(14.0));
    ctx.set_style(style);
}
fn primary(text: &str) -> egui::Button<'_> {
    egui::Button::new(RichText::new(text).color(Color32::WHITE).strong())
        .fill(ACCENT)
        .stroke(Stroke::NONE)
        .corner_radius(7)
}
fn nav_button(text: &str, selected: bool) -> egui::Button<'_> {
    let fill = if selected {
        ACCENT_LIGHT
    } else {
        Color32::TRANSPARENT
    };
    let text_color = if selected { ACCENT } else { INK };
    egui::Button::new(RichText::new(text).color(text_color))
        .fill(fill)
        .stroke(Stroke::NONE)
        .corner_radius(7)
}
fn label(ui: &mut egui::Ui, text: &str) -> egui::Id {
    ui.label(RichText::new(text).size(13.0).strong().color(INK))
        .id
}
fn muted(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.label(RichText::new(text.into()).size(13.0).color(MUTED));
}
fn card() -> Frame {
    Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(10)
        .inner_margin(Margin::same(24))
}

struct App {
    path: PathBuf,
    lib: Library,
    selected: Option<usize>,
    name: String,
    body: String,
    scope: String,
    icon: String,
    query: String,
    installation: String,
    files: Vec<PathBuf>,
    import_file: String,
    import_scope: String,
    preview: Vec<wowmc::SnapshotMacro>,
    status: String,
    dirty: bool,
    view: View,
    confirm_close: bool,
}
impl App {
    fn new(path: PathBuf, installation: PathBuf) -> Self {
        let (lib, status) = match Library::load(&path) {
            Ok(l) => (l, String::new()),
            Err(e) => (
                Library::default(),
                format!("Cannot load library: {e:#}. Resolve the error, then reload the library."),
            ),
        };
        Self {
            path,
            lib,
            selected: None,
            name: String::new(),
            body: String::new(),
            scope: "account".into(),
            icon: String::new(),
            query: String::new(),
            installation: installation.display().to_string(),
            files: vec![],
            import_file: String::new(),
            import_scope: "account".into(),
            preview: vec![],
            status,
            dirty: false,
            view: View::Library,
            confirm_close: false,
        }
    }
    fn select(&mut self, i: usize) {
        let m = &self.lib.macros[i];
        self.selected = Some(i);
        self.name = m.current.name.clone();
        self.body = m.current.body.clone();
        self.scope = m.current.scope.clone();
        self.icon = m.current.icon.clone().unwrap_or_default();
        self.dirty = false;
    }
    fn reload_library(&mut self) -> anyhow::Result<()> {
        let next = Library::load(&self.path)?;
        let selected_id = self.selected.map(|i| self.lib.macros[i].id);
        let selected =
            selected_id.and_then(|id| next.macros.iter().position(|m| m.id == id && !m.deleted));
        if self.dirty && selected_id.is_some() && selected.is_none() {
            anyhow::bail!("This macro was removed elsewhere. Copy your draft or discard changes before reloading.");
        }
        self.lib = next;
        self.selected = selected;
        if !self.dirty {
            if let Some(i) = selected {
                self.select(i);
            } else {
                self.clear_draft();
            }
        }
        self.status = if self.dirty {
            "Library reloaded. Your draft is kept; review it before saving."
        } else {
            "Library reloaded."
        }
        .into();
        Ok(())
    }
    fn new_macro(&mut self) {
        if self.dirty {
            self.status = "Save your changes before switching macros.".into();
        } else {
            self.clear_draft();
            self.view = View::Library;
        }
    }
    fn clear_draft(&mut self) {
        self.selected = None;
        self.name.clear();
        self.body.clear();
        self.icon.clear();
        self.scope = "account".into();
        self.dirty = false;
    }
    fn save(&mut self) -> anyhow::Result<()> {
        if self.lib.version != 1 {
            anyhow::bail!("Library was not loaded; resolve the error and reload the library")
        }
        let r = revision(
            self.name.clone(),
            self.body.clone(),
            if self.icon.is_empty() {
                None
            } else {
                Some(self.icon.clone())
            },
            self.scope.clone(),
        )?;
        let id = self.selected.map(|i| self.lib.macros[i].id);
        let id = self.lib.save_macro(&self.path, id, r)?;
        self.selected = Some(self.lib.resolve(&id.to_string(), false)?);
        self.dirty = false;
        Ok(())
    }
    fn apply_import(&mut self) -> anyhow::Result<usize> {
        if self.lib.version != 1 {
            anyhow::bail!("Library was not loaded; cannot import")
        }
        let count = self.lib.import(
            &self.path,
            &self.preview,
            &self.import_file,
            &self.import_scope,
        )?;
        self.preview.clear();
        Ok(count)
    }
    fn sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("library")
            .exact_width(252.0)
            .frame(
                Frame::new()
                    .fill(Color32::from_rgb(247, 249, 252))
                    .inner_margin(Margin::same(18)),
            )
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    Frame::new()
                        .fill(ACCENT)
                        .corner_radius(8)
                        .inner_margin(Margin::symmetric(10, 5))
                        .show(ui, |ui| {
                            ui.label(RichText::new("W").size(18.0).strong().color(Color32::WHITE));
                        });
                    ui.label(RichText::new("wowmc").size(19.0).strong());
                });
                ui.add_space(26.0);
                ui.label(RichText::new("WORKSPACE").size(11.0).strong().color(MUTED));
                ui.add_space(3.0);
                if ui
                    .add_sized(
                        [ui.available_width(), 38.0],
                        nav_button("Library", self.view == View::Library),
                    )
                    .clicked()
                {
                    if self.view != View::Library && self.lib.version == 1 {
                        self.status.clear();
                    }
                    self.view = View::Library;
                }
                if ui
                    .add_sized(
                        [ui.available_width(), 38.0],
                        nav_button("Import from WoW", self.view == View::Import),
                    )
                    .clicked()
                {
                    if self.view != View::Import && self.lib.version == 1 {
                        self.status.clear();
                    }
                    self.view = View::Import;
                }
                ui.add_space(18.0);
                ui.separator();
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("YOUR MACROS")
                            .size(11.0)
                            .strong()
                            .color(MUTED),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        muted(
                            ui,
                            self.lib
                                .macros
                                .iter()
                                .filter(|m| !m.deleted)
                                .count()
                                .to_string(),
                        );
                    });
                });
                let search_label = label(ui, "Search");
                ui.add_sized(
                    [ui.available_width(), 36.0],
                    egui::TextEdit::singleline(&mut self.query).hint_text("Search macros"),
                )
                .labelled_by(search_label);
                if ui
                    .add_sized(
                        [ui.available_width(), 36.0],
                        egui::Button::new("+  New macro"),
                    )
                    .clicked()
                {
                    self.new_macro();
                }
                if ui
                    .add_sized(
                        [ui.available_width(), 36.0],
                        egui::Button::new("Reload library (keep draft)"),
                    )
                    .clicked()
                {
                    if let Err(error) = self.reload_library() {
                        self.status = format!("Could not reload: {error:#}");
                    }
                }
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .id_salt("macro_list")
                    .show(ui, |ui| {
                        let q = self.query.to_lowercase();
                        let mut chosen = None;
                        let mut count = 0;
                        for (i, m) in self.lib.macros.iter().enumerate() {
                            if !m.deleted
                                && (m.current.name.to_lowercase().contains(&q)
                                    || m.current.body.to_lowercase().contains(&q))
                            {
                                count += 1;
                                let title = format!("{}\n{}", m.current.name, m.current.scope);
                                if ui
                                    .add_sized(
                                        [ui.available_width(), 52.0],
                                        nav_button(
                                            &title,
                                            self.selected == Some(i) && self.view == View::Library,
                                        ),
                                    )
                                    .clicked()
                                {
                                    chosen = Some(i);
                                }
                            }
                        }
                        if count == 0 {
                            muted(
                                ui,
                                if q.is_empty() {
                                    "No macros yet."
                                } else {
                                    "No matching macros."
                                },
                            );
                        }
                        if let Some(i) = chosen {
                            if self.dirty {
                                self.status =
                                    "Save or discard your changes before switching macros.".into();
                            } else {
                                self.select(i);
                                self.view = View::Library;
                                self.status.clear();
                            }
                        }
                    });
            });
    }
    fn editor(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let was_dirty = self.dirty;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(if self.selected.is_some() {
                        "Edit macro"
                    } else {
                        "New macro"
                    })
                    .size(27.0)
                    .strong(),
                );
            });
        });
        ui.add_space(18.0);
        card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new("Macro details").size(17.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    muted(
                        ui,
                        if self.dirty {
                            "Unsaved changes"
                        } else if self.selected.is_some() {
                            "Saved in library"
                        } else {
                            "Draft"
                        },
                    );
                });
            });
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(12.0);
            let name_label = label(ui, "Name");
            self.dirty |= ui
                .add_sized(
                    [ui.available_width(), 38.0],
                    egui::TextEdit::singleline(&mut self.name),
                )
                .labelled_by(name_label)
                .changed();
            ui.add_space(8.0);
            ui.columns(2, |columns| {
                let scope_label = label(&mut columns[0], "Scope");
                self.dirty |= columns[0]
                    .add_sized(
                        [columns[0].available_width(), 38.0],
                        egui::TextEdit::singleline(&mut self.scope),
                    )
                    .labelled_by(scope_label)
                    .changed();
                let icon_label = label(&mut columns[1], "Icon (optional)");
                self.dirty |= columns[1]
                    .add_sized(
                        [columns[1].available_width(), 38.0],
                        egui::TextEdit::singleline(&mut self.icon),
                    )
                    .labelled_by(icon_label)
                    .changed();
            });
            ui.add_space(14.0);
            let body_label = label(ui, "Macro text");
            self.dirty |= ui
                .add(
                    egui::TextEdit::multiline(&mut self.body)
                        .desired_rows(8)
                        .desired_width(f32::INFINITY)
                        .code_editor(),
                )
                .labelled_by(body_label)
                .changed();
            muted(
                ui,
                format!(
                    "{} characters · Client limits may vary",
                    self.body.chars().count()
                ),
            );
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(12.0);
            ui.horizontal_wrapped(|ui| {
                if ui.add(primary("Save macro")).clicked() {
                    self.status = match self.save() {
                        Ok(()) => "Saved in library. This does not change WoW.".into(),
                        Err(e) => format!("Could not save: {e:#}"),
                    };
                }
                if ui.button("Copy for WoW").clicked() {
                    ctx.copy_text(self.body.clone());
                    self.status = format!(
                        "Copied. In WoW, create or edit '{}' in scope '{}' and paste the body.",
                        self.name, self.scope
                    );
                }
                if self.dirty && ui.button("Discard changes").clicked() {
                    if let Some(i) = self.selected {
                        self.select(i);
                    } else {
                        self.dirty = false;
                        self.new_macro();
                    }
                    self.status = "Changes discarded.".into();
                }
            });
        });
        if !was_dirty && self.dirty {
            self.status.clear();
        }
    }
    fn import(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Import from WoW").size(27.0).strong());
        ui.add_space(18.0);
        card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            muted(ui, "Local snapshots may not reflect your current in-game macros. Importing does not change game files."); ui.add_space(8.0);
            let folder_label = label(ui, "Forever folder");
            ui.horizontal(|ui| { let width=(ui.available_width()-150.0).max(100.0); ui.add_sized([width, 34.0], egui::TextEdit::singleline(&mut self.installation)).labelled_by(folder_label);
                if ui.button("Find snapshots").clicked() { match snapshots(&PathBuf::from(&self.installation)) { Ok(files) => { self.files=files; self.status=format!("Found {} snapshots.", self.files.len()); }, Err(e) => self.status=format!("Could not inspect: {e:#}") } }
            });
            if !self.files.is_empty() { ui.add_space(8.0); label(ui, "Snapshots");
                for f in &self.files {
                    let display = f.strip_prefix(PathBuf::from(&self.installation).join("WTF").join("Account")).unwrap_or(f).display().to_string().replace('\\', " / ");
                    if ui.add_sized([ui.available_width(), 34.0], egui::Button::new(display).wrap()).on_hover_text(f.display().to_string()).clicked() { self.import_file=f.display().to_string(); self.preview.clear(); }
                }
            }
            ui.add_space(8.0); let path_label = label(ui, "Snapshot path");
            if ui.add(egui::TextEdit::singleline(&mut self.import_file).desired_width(f32::INFINITY)).labelled_by(path_label).changed() { self.preview.clear(); }
            ui.add_space(4.0); let scope_label = label(ui, "Import scope"); ui.add(egui::TextEdit::singleline(&mut self.import_scope).desired_width(f32::INFINITY)).labelled_by(scope_label);
            if ui.button("Preview macros").clicked() { match fs::read_to_string(&self.import_file).map_err(anyhow::Error::from).and_then(|s| parse_snapshot(&s)) { Ok(p) => { self.status=format!("Previewing {} macros; existing names will be skipped.", p.len()); self.preview=p; }, Err(e) => { self.preview.clear(); self.status=format!("Could not preview: {e:#}"); } } }
            if !self.preview.is_empty() { ui.separator(); label(ui, "Preview");
                match self.lib.plan_import(&self.preview, &self.import_file, &self.import_scope) {
                    Ok(plan) => { for item in plan.items { ui.label(format!("{}: {}", item.name, item.decision.label())); } }
                    Err(error) => { ui.label(format!("Cannot import: {error:#}")); }
                }
                if ui.add(primary("Import new names")).clicked() {
                    self.status=match self.apply_import() { Ok(count) => format!("Imported {count} macros into the library. Game files were not changed."), Err(e) => format!("Could not save import: {e:#}") };
                }
            }
        });
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) && self.dirty {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm_close = true;
        }
        if self.confirm_close {
            egui::Window::new("Unsaved changes")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Your changes have not been saved.");
                    ui.horizontal(|ui| {
                        if ui.button("Keep editing").clicked() {
                            self.confirm_close = false;
                        }
                        if ui.button("Discard and close").clicked() {
                            self.dirty = false;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
        }
        egui::TopBottomPanel::bottom("status")
            .frame(
                Frame::new()
                    .fill(SURFACE)
                    .stroke(Stroke::new(1.0, BORDER))
                    .inner_margin(Margin::symmetric(20, 8)),
            )
            .show(ctx, |ui| {
                let message = if self.status.is_empty() {
                    "Local library · Changes do not install macros in WoW"
                } else {
                    &self.status
                };
                ui.label(RichText::new(message).size(13.0).color(
                    if self.status.starts_with("Could not") || self.status.starts_with("Cannot") {
                        Color32::from_rgb(166, 50, 48)
                    } else {
                        MUTED
                    },
                ));
            });
        self.sidebar(ctx);
        egui::CentralPanel::default()
            .frame(Frame::new().fill(CANVAS).inner_margin(Margin::same(28)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("content")
                    .show(ui, |ui| {
                        ui.set_width((ctx.screen_rect().width() - 306.0).min(780.0));
                        match self.view {
                            View::Library => self.editor(ui, ctx),
                            View::Import => self.import(ui),
                        }
                    });
            });
    }
}

#[cfg(test)]
mod transition_tests {
    use super::*;
    #[test]
    fn conflict_reload_preserves_draft_and_follows_record_id_after_reordering() {
        let fixture = tempfile::tempdir().unwrap();
        let path = fixture.path().join("library.json");
        let mut app = App::new(path.clone(), PathBuf::new());
        app.name = "A".into();
        app.body = "/say initial".into();
        app.dirty = true;
        app.save().unwrap();
        let id = app.lib.macros[0].id;
        app.body = "/say draft".into();
        app.dirty = true;
        let mut other = Library::load(&path).unwrap();
        other
            .save_macro(
                &path,
                Some(id),
                revision("A".into(), "/say external".into(), None, "account".into()).unwrap(),
            )
            .unwrap();
        other
            .save_macro(
                &path,
                None,
                revision("B".into(), "/say other".into(), None, "account".into()).unwrap(),
            )
            .unwrap();
        other.macros.swap(0, 1);
        other.save(&path).unwrap();
        assert!(app.save().is_err());
        assert!(app.save().is_err());
        assert!(app.dirty);
        assert_eq!(app.body, "/say draft");
        app.reload_library().unwrap();
        assert_eq!(app.selected, Some(1));
        assert!(app.dirty);
        assert_eq!(app.body, "/say draft");
        app.save().unwrap();
        let saved = Library::load(&path).unwrap();
        let record = &saved.macros[saved.resolve(&id.to_string(), false).unwrap()];
        assert_eq!(record.current.body, "/say draft");
        assert_eq!(record.history.last().unwrap().body, "/say external");
        assert_eq!(saved.macros[0].current.body, "/say other");
    }

    #[test]
    fn reload_never_silently_recreates_an_externally_deleted_draft() {
        let fixture = tempfile::tempdir().unwrap();
        let path = fixture.path().join("library.json");
        let mut app = App::new(path.clone(), PathBuf::new());
        app.name = "A".into();
        app.body = "/say initial".into();
        app.save().unwrap();
        app.body = "/say draft".into();
        app.dirty = true;
        let mut other = Library::load(&path).unwrap();
        other.delete_macro(&path, "A").unwrap();
        assert!(app.reload_library().is_err());
        assert!(app.dirty);
        assert_eq!(app.body, "/say draft");
        app.dirty = false;
        app.reload_library().unwrap();
        assert!(app.selected.is_none());
        assert!(app.body.is_empty());
        assert!(Library::load(&path).unwrap().macros[0].deleted);
    }
    #[test]
    fn failed_save_keeps_library_and_draft_distinct() {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("blocked");
        fs::write(&root, "not a directory").unwrap();
        let mut app = App::new(fixture.path().join("library.json"), PathBuf::new());
        app.path = root.join("library.json");
        app.lib = Library::load(&app.path).unwrap();
        app.name = "Test".into();
        app.body = "/say hi".into();
        app.dirty = true;
        assert!(app.save().is_err());
        assert!(
            app.lib.macros.is_empty(),
            "failed save must not change the library in memory"
        );
        assert!(app.selected.is_none());
        assert!(app.dirty);
        fs::remove_file(root).unwrap();
        app.path = fixture.path().join("library.json");
        app.save().unwrap();
        assert!(!app.dirty);
        let persisted = Library::load(&app.path).unwrap();
        assert_eq!(persisted.macros[0].current.body, "/say hi");
    }
    #[test]
    fn failed_import_keeps_preview_and_leaves_library_unchanged() {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("blocked");
        fs::write(&root, "not a directory").unwrap();
        let mut app = App::new(fixture.path().join("library.json"), PathBuf::new());
        app.path = root.join("library.json");
        app.lib = Library::load(&app.path).unwrap();
        app.preview = vec![wowmc::SnapshotMacro {
            name: "A".into(),
            icon: "1".into(),
            body: "/say hi".into(),
        }];
        assert!(app.apply_import().is_err());
        assert!(app.lib.macros.is_empty());
        assert_eq!(app.preview.len(), 1);
        fs::remove_file(root).unwrap();
        app.path = fixture.path().join("library.json");
        assert_eq!(app.apply_import().unwrap(), 1);
        assert!(app.preview.is_empty());
        assert_eq!(Library::load(&app.path).unwrap().macros.len(), 1);
    }
}
