//! Hestia GUI
//!
//! Graphical user interface prototype for the Hestia EDR system using egui/eframe.

use eframe::{egui, App, Frame, NativeOptions};
use egui::{Context, RichText};
use log::LevelFilter;
use std::time::{Duration, Instant};

fn main() {
    // Console logger for quick diagnostics
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Info)
        .try_init();

    let native_options = NativeOptions::default();
    let _ = eframe::run_native(
        "Hestia GUI",
        native_options,
        Box::new(|_cc| Ok(Box::new(HestiaApp::default()))),
    );
}

#[derive(Default)]
struct HestiaApp {
    // Toggles / settings
    edr_enabled: bool,
    realtime_monitoring: bool,
    quarantine_enabled: bool,
    send_telemetry: bool,

    // Action state
    current_scan: Option<ScanState>,

    // Log buffer
    logs: Vec<String>,

    // UI state
    selected_tab: Tab,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScanKind {
    Process,
    Disk,
}

struct ScanState {
    kind: ScanKind,
    start: Instant,
    halfway_logged: bool,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
enum Tab {
    #[default]
    Dashboard,
    Logs,
    Settings,
}

impl HestiaApp {
    fn log_line(&mut self, msg: impl Into<String>) {
        const MAX_LINES: usize = 500;
        let timestamp = chrono_like_now();
        let line = format!("[{}] {}", timestamp, msg.into());
        self.logs.push(line);
        if self.logs.len() > MAX_LINES {
            let drain = self.logs.len() - MAX_LINES;
            self.logs.drain(0..drain);
        }
    }

    fn start_scan(&mut self, kind: ScanKind) {
        if self.current_scan.is_some() {
            self.log_line("A scan is already running");
            return;
        }
        match kind {
            ScanKind::Process => self.log_line("Process scan started"),
            ScanKind::Disk => self.log_line("Disk scan started"),
        }
        self.current_scan = Some(ScanState {
            kind,
            start: Instant::now(),
            halfway_logged: false,
        });
    }

    fn tick_scan(&mut self) {
        // Avoid double-borrow of `self` by staging log lines
        let mut logs: Vec<String> = Vec::new();
        let mut clear_scan = false;
        if let Some(state) = &mut self.current_scan {
            let elapsed = state.start.elapsed();
            if !state.halfway_logged && elapsed >= Duration::from_secs(1) {
                let msg = match state.kind {
                    ScanKind::Process => "Process scan: 50%",
                    ScanKind::Disk => "Disk scan: 50%",
                };
                logs.push(msg.to_string());
                state.halfway_logged = true;
            }
            if elapsed >= Duration::from_secs(2) {
                let msg = match state.kind {
                    ScanKind::Process => "Process scan completed",
                    ScanKind::Disk => "Disk scan completed",
                };
                logs.push(msg.to_string());
                clear_scan = true;
            }
        }
        for l in logs {
            self.log_line(l);
        }
        if clear_scan {
            self.current_scan = None;
        }
    }
}

impl App for HestiaApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // Advance any ongoing simulated work
        self.tick_scan();

        // Left: vertical tabs
        egui::SidePanel::left("tabs")
            .default_width(160.0)
            .resizable(false)
            .show(ctx, |ui| {
                let mut style = ui.style_mut();

                ui.heading("Hestia");
                ui.separator();
                ui.add_space(8.0);

                let tabs = [
                    (Tab::Dashboard, "Dashboard"),
                    (Tab::Logs, "Logs"),
                    (Tab::Settings, "Settings"),
                ];

                for (index, (tab, label)) in tabs.iter().enumerate() {
                    let selected = self.selected_tab == *tab;
                    let text = RichText::new(*label).size(20.0);

                    if index == tabs.len() - 1 {
                        ui.allocate_space(ui.available_size_before_wrap());
                    }

                    let resp = ui.add_sized(
                        [ui.available_width(), 60.0],
                        egui::Button::new(text).selected(selected),
                    );
                    if resp.clicked() {
                        self.selected_tab = *tab;
                    }

                    if index < tabs.len() - 1 {
                        ui.add_space(4.0);
                    }
                }
            });

        // Center: selected tab content
        egui::CentralPanel::default().show(ctx, |ui| match self.selected_tab {
            Tab::Dashboard => {
                ui.vertical_centered(|ui| {
                    ui.heading("Dashboard");
                    ui.label("Welcome to Hestia. Dashboard content coming soon.");
                });
            }
            Tab::Logs => {
                ui.heading("Logs");
                ui.separator();

                // Build a single string view. For a prototype this is fine.
                let mut buffer = String::with_capacity(self.logs.iter().map(|s| s.len() + 1).sum());
                for line in &self.logs {
                    buffer.push_str(line);
                    buffer.push('\n');
                }

                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let mut text = buffer;
                        ui.add(
                            egui::TextEdit::multiline(&mut text)
                                .font(egui::TextStyle::Monospace)
                                .desired_rows(20)
                                .lock_focus(true)
                                .desired_width(f32::INFINITY)
                                .interactive(false),
                        );
                    });

                ui.add_space(8.0);
                if ui.button("Clear Logs").clicked() {
                    self.logs.clear();
                }
            }
            Tab::Settings => {
                ui.heading("Settings");
                ui.separator();

                ui.checkbox(&mut self.edr_enabled, "EDR Enabled");
                ui.toggle_value(&mut self.realtime_monitoring, "Realtime Monitoring");
                ui.toggle_value(&mut self.quarantine_enabled, "Quarantine");
                ui.toggle_value(&mut self.send_telemetry, "Send Telemetry");

                ui.separator();
                ui.heading("Actions");
                let busy = self.current_scan.is_some();

                if ui
                    .add_enabled(!busy, egui::Button::new("Run Process Scan"))
                    .clicked()
                {
                    self.start_scan(ScanKind::Process);
                }

                if ui
                    .add_enabled(!busy, egui::Button::new("Run Disk Scan"))
                    .clicked()
                {
                    self.start_scan(ScanKind::Disk);
                }

                ui.separator();
                let status = if self.edr_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                };
                ui.label(RichText::new(format!("Status: {}", status)).strong());
                if busy {
                    ui.label("Scan in progress.");
                }
            }
        });

        // Request another frame to keep animations/timers responsive
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

// Tiny timestamp helper without extra dependencies
fn chrono_like_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0));
    let secs = now.as_secs();
    let ms = now.subsec_millis();
    format!("{}.{:03}", secs, ms)
}
