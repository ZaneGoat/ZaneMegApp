//! The shell: ribbon, sidebar, tab body and status bar (design §6; AC-10, AC-37).
//!
//! The four regions are **panels**, never rows: each one is given its own rect before the next
//! one is laid out, so the tab body is always the only flexible region and is always top-down
//! (design §1.3, L1/L4). That is the fix for the V4 launch panic, where the sidebar and the body
//! shared a `LeftToRight` row and the body was handed a negative width.
//!
//! The shell owns the readings ([`Sampler`]), the session's activity log and the current tab. A
//! tab paints itself and returns a *navigation request*; this file is the only place a tab switch
//! happens, and no tab can act on the machine (design §3 rule 3, §6).

use eframe::egui;
use std::time::{Duration, Instant};

use crate::app::activity::ActivityLog;
use crate::app::sidebar::sidebar;
use crate::app::statusbar::status_bar;
use crate::app::tabs::AppTab;
use crate::core::alerts::evaluate;
use crate::core::health::{BatteryInfo, HealthSnapshot, Sampler};
use crate::core::measured::Unit;
use crate::theme::*;
use crate::ui::tab_cleaner::TabCleaner;
use crate::ui::tab_controller::TabController;
use crate::ui::tab_hardware::TabHardware;
use crate::ui::tab_headset::TabHeadset;
use crate::ui::tab_idevice::TabIDevice;
use crate::ui::tab_mission_control::TabMissionControl;
use crate::ui::tab_security::TabSecurity;

pub mod activity;
pub mod sidebar;
pub mod statusbar;
pub mod tabs;

/// The navigation column: AC-37's 225 px sidebar card, plus the 10 px frame margin on each side
/// of it inside [`sidebar`].
const NAV_W: f32 = 245.0;

/// Mission Control is re-read at 1 Hz, so one repaint a second is all the shell needs. The old
/// shell asked for one every 120 ms forever, which is both a tax on the idle-CPU budget (design
/// §9) and a reason the sampler below is throttled by its own cadences rather than by the frame
/// rate. Hover and typing still repaint immediately: egui asks for a frame on input.
const REPAINT_AFTER: Duration = Duration::from_millis(1000);

/// A tab's ask of the shell after it painted (design §6: a tab returns a request, it never
/// switches or acts itself).
#[derive(Default)]
struct Dispatch {
    nav: Option<AppTab>,
    status: Option<(String, egui::Color32)>,
}

pub struct ZaneMegApp {
    pub current_tab: AppTab,
    /// The one source of every number on screen (design §5.2).
    pub sampler: Sampler,
    /// "What this app has done — this session only" (AC-15).
    pub activity: ActivityLog,
    pub tab_mission: TabMissionControl,
    pub tab_hardware: TabHardware,
    pub tab_controller: TabController,
    pub tab_cleaner: TabCleaner,
    pub tab_headset: TabHeadset,
    pub tab_security: TabSecurity,
    pub tab_idevice: TabIDevice,
    /// The last thing a tab reported, painted by the status bar as a note. Empty means nothing to
    /// report, not "Ready" repeated.
    pub status_message: String,
    pub status_color: egui::Color32,
    pub last_telemetry_tick: Instant,
}

impl ZaneMegApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_theme(&cc.egui_ctx);

        let now = Instant::now();
        let mut app = Self::with_sampler(Sampler::system(now), now);
        // The six V2 tabs keep their own readers; they are refreshed here so the first frame on
        // any of them is not a row of placeholders (AC-16).
        app.tab_hardware.update_telemetry();
        app.tab_cleaner.update_resources();
        app
    }

    /// The shell over a given sampler. Tests hand it a fixture tree instead of this laptop
    /// (design §10.1), which is the only difference from [`ZaneMegApp::new`].
    fn with_sampler(sampler: Sampler, now: Instant) -> Self {
        Self {
            // AC-10: Mission Control is the default tab.
            current_tab: AppTab::Mission,
            sampler,
            activity: ActivityLog::new(),
            tab_mission: TabMissionControl::new(),
            tab_hardware: TabHardware::new(),
            tab_controller: TabController::new(),
            tab_cleaner: TabCleaner::new(),
            tab_headset: TabHeadset::new(),
            tab_security: TabSecurity::new(),
            tab_idevice: TabIDevice::new(),
            status_message: String::new(),
            status_color: GREEN_OK,
            last_telemetry_tick: now,
        }
    }

    /// The six V2 tabs report through this; the status bar paints the note.
    pub fn set_status(&mut self, msg: String, color: egui::Color32) {
        self.status_message = msg;
        self.status_color = color;
    }

    /// The whole window, minus the `eframe::Frame` that only the runtime owns — so the tests can
    /// drive the real shell instead of a copy of it.
    pub fn shell(&mut self, ui: &mut egui::Ui) {
        let now = Instant::now();
        if self.last_telemetry_tick.elapsed() >= REPAINT_AFTER {
            self.last_telemetry_tick = now;
            self.sampler.tick(now);
            self.tab_hardware.update_telemetry();
            self.tab_cleaner.update_resources();
        }
        ui.ctx().request_repaint_after(REPAINT_AFTER);

        self.route_keys(ui);

        let snapshot = self.sampler.snapshot().clone();
        let rows = snapshot.health_rows(now);
        let alerts = evaluate(&snapshot, now);
        let mut nav = None;

        egui::Panel::top("ribbon")
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(BG_PANEL)
                    .stroke(egui::Stroke::new(1.0, BORDER_DARK))
                    .inner_margin(egui::Margin::symmetric(16, 8)),
            )
            .show(ui, |ui| self.ribbon(ui, &snapshot, now));

        let note = if self.status_message.is_empty() {
            None
        } else {
            Some((self.status_message.as_str(), self.status_color))
        };
        let current = self.current_tab;
        egui::Panel::bottom("status")
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(BG_PANEL)
                    .stroke(egui::Stroke::new(1.0, BORDER_DARK))
                    .inner_margin(egui::Margin::symmetric(14, 6)),
            )
            .show(ui, |ui| status_bar(ui, current, None, note));

        egui::Panel::left("nav")
            .resizable(false)
            .exact_size(NAV_W)
            .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(0, 6)))
            .show(ui, |ui| {
                if let Some(clicked) = sidebar(ui, current, &rows, alerts.len()) {
                    nav = Some(clicked);
                }
            });

        let dispatch = egui::CentralPanel::default()
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("tab_body")
                    .auto_shrink([false, false])
                    // A visible bar is how the reader knows the tab continues below the fold at
                    // 1024×640; hiding it until needed made the cut look like the end of the page.
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .show(ui, |ui| {
                        ui.vertical(|ui| self.dispatch(ui, &snapshot, now)).inner
                    })
                    .inner
            })
            .inner;

        nav = dispatch.nav.or(nav);
        if let Some(tab) = nav {
            self.switch_to(tab);
        }
        if let Some((msg, color)) = dispatch.status {
            self.set_status(msg, color);
        }
    }

    /// AC-37's shortcut routing, walked from the tab table itself so there is no second key map
    /// to drift. A modal will get first refusal here in 4b (design §6, "one function, ordered").
    fn route_keys(&mut self, ui: &egui::Ui) {
        ui.input(|input| {
            for tab in AppTab::ALL {
                if let Some(key) = tab.key()
                    && input.key_pressed(key)
                {
                    self.switch_to(tab);
                }
            }
        });
    }

    /// The one place the current tab changes.
    fn switch_to(&mut self, tab: AppTab) {
        if self.current_tab == tab {
            return;
        }
        self.current_tab = tab;
        // The Security tab's readings are two subprocesses (`ip`, `ss`), so they are refreshed
        // when she opens it rather than every second on the UI thread (design §9, D19).
        if tab == AppTab::Security {
            self.tab_security.refresh_state();
        }
    }

    /// Paints the active tab and collects its requests.
    fn dispatch(&mut self, ui: &mut egui::Ui, snapshot: &HealthSnapshot, now: Instant) -> Dispatch {
        let mut dispatch = Dispatch::default();
        let mut set_status = |msg: String, color: egui::Color32| {
            dispatch.status = Some((msg, color));
        };
        match self.current_tab {
            AppTab::Mission => {
                dispatch.nav = self.tab_mission.ui(ui, snapshot, now, &self.activity);
            }
            AppTab::Hardware => self.tab_hardware.ui(ui, &mut set_status),
            AppTab::Controller => self.tab_controller.ui(ui, &mut set_status),
            AppTab::Cleaner => self.tab_cleaner.ui(ui, &mut set_status),
            AppTab::Headset => self.tab_headset.ui(ui, &mut set_status),
            AppTab::Security => self.tab_security.ui(ui, &mut set_status),
            AppTab::IDevice => self.tab_idevice.ui(ui, &mut set_status),
        }
        dispatch
    }

    /// The ribbon: the app's identity, then the five numbers every tab shares, each through
    /// [`composite_text`] so a pill and a tile print the same words for the same reading.
    fn ribbon(&self, ui: &mut egui::Ui, snapshot: &HealthSnapshot, now: Instant) {
        ui.horizontal(|ui| {
            let pulse = pulse_factor(ui.ctx());
            let (dot, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
            ui.painter()
                .circle_filled(dot.center(), 4.5, RED_BRIGHT.gamma_multiply(pulse));

            ui.label(
                egui::RichText::new(format!("⚡ {}", crate::app_version()))
                    .strong()
                    .size(15.0)
                    .color(TEXT_MAIN),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.ribbon_pills(ui, snapshot, now);
            });
        });
    }

    /// Right-to-left, in priority order: the firewall state is the one T-29 says may never go
    /// missing, so it is placed first (rightmost) and the others are dropped when the window is
    /// too narrow for all five — which says less, but never says something false.
    fn ribbon_pills(&self, ui: &mut egui::Ui, snapshot: &HealthSnapshot, now: Instant) {
        let firewall = composite_text(&snapshot.firewall, now, |state| state.word().to_string());
        let battery = composite_text(&snapshot.battery, now, BatteryInfo::words);
        let disk = composite_text(&snapshot.disk_root, now, |disk| {
            format!("{:.0} %", disk.used_pct)
        });
        let memory = composite_text(&snapshot.mem, now, |mem| format!("{:.0} %", mem.used_pct));
        let cpu = metric_text(&snapshot.cpu_temp_c, now, Unit::Celsius, 0);

        let pills = [
            ("🛡️", "Firewall", firewall, hover_text(&snapshot.firewall, now)),
            ("🔋", "Battery", battery, hover_text(&snapshot.battery, now)),
            ("💾", "Disk", disk, hover_text(&snapshot.disk_root, now)),
            ("🧠", "RAM", memory, hover_text(&snapshot.mem, now)),
            ("🌡️", "CPU", cpu, hover_text(&snapshot.cpu_temp_c, now)),
        ];
        for (icon, label, text, hover) in pills {
            if !ribbon_pill(ui, icon, label, &text, &hover) {
                break;
            }
        }
    }
}

/// One ribbon pill, or nothing when even its short form cannot fit: `false` stops the caller.
///
/// A live or stale value prints in full. An `unavailable` one prints in full only when that fits,
/// and otherwise falls back to `Firewall unavailable` with the whole sentence in the hover — the
/// reason is then one hover away instead of painted over the window edge.
fn ribbon_pill(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    text: &MetricText,
    hover: &str,
) -> bool {
    let full = format!("{icon} {label} {}", text.words());
    let short = format!("{icon} {label} {}", metric_kind(text));
    let word = if status_pill_width(ui, &full) <= ui.available_width() {
        full
    } else if status_pill_width(ui, &short) <= ui.available_width() {
        short
    } else {
        return false;
    };
    let (fg, bg) = match text {
        MetricText::Number(_) => (TEXT_MAIN, BG_CARD),
        MetricText::Stale(_) => (AMBER_WARN, AMBER_SUBTLE),
        MetricText::Unavailable(_) => (TEXT_MUTED, BG_CARD),
    };
    status_pill(ui, &word, fg, bg).on_hover_text(hover);
    true
}

/// The one-word form of a cell: `live`, `stale`, `unavailable`.
fn metric_kind(text: &MetricText) -> &'static str {
    match text {
        MetricText::Number(_) => "live",
        MetricText::Stale(_) => "stale",
        MetricText::Unavailable(_) => "unavailable",
    }
}

impl eframe::App for ZaneMegApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.shell(ui);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::health::Roots;
    use std::path::Path;

    /// A shell over an empty fixture tree: every reading is `unavailable`, so no test here can
    /// read this laptop's temperature, battery or network (design §10.1).
    fn app_at(root: &Path, now: Instant) -> ZaneMegApp {
        let _ = std::fs::remove_dir_all(root);
        std::fs::create_dir_all(root).expect("fixture root");
        let roots = Roots {
            proc_root: root.join("proc"),
            sys_root: root.join("sys"),
            etc_root: root.join("etc"),
            root_fs: root.to_path_buf(),
        };
        ZaneMegApp::with_sampler(Sampler::new(roots, now), now)
    }

    /// Drives the real shell through egui at the widths that matter: the default size, the
    /// enforced minimum, and widths squeezed well below the sidebar. A panic here is the V4
    /// launch bug coming back (AC-02, AC-05, L1–L6).
    #[test]
    fn the_shell_paints_every_tab_at_every_width() {
        let now = Instant::now();
        let root = std::env::temp_dir().join(format!("zma-shell-{}", std::process::id()));
        let mut app = app_at(&root, now);

        for width in [1920.0_f32, 1140.0, 1024.0, 640.0, 320.0] {
            for tab in AppTab::ALL {
                app.current_tab = tab;
                let ctx = egui::Context::default();
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, 640.0),
                    )),
                    ..Default::default()
                };
                let output = ctx.run_ui(input, |ui| app.shell(ui));
                output.drop_without_applying_deltas();
            }
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    /// AC-10: Mission Control is where the app opens, and `F1` brings her back to it.
    #[test]
    fn mission_control_is_the_default_and_f1_returns_to_it() {
        let now = Instant::now();
        let root = std::env::temp_dir().join(format!("zma-shell-f1-{}", std::process::id()));
        let mut app = app_at(&root, now);
        assert_eq!(app.current_tab, AppTab::Mission);

        app.current_tab = AppTab::Cleaner;
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::F1,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        };
        let output = ctx.run_ui(input, |ui| app.route_keys(ui));
        output.drop_without_applying_deltas();
        assert_eq!(app.current_tab, AppTab::Mission);
        let _ = std::fs::remove_dir_all(&root);
    }
}
