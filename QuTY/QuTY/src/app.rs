use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};
use std::collections::HashMap;

use crate::circuit_grid::{AlgorithmPreset, CircuitGrid, GateType, GridCell};
use crate::emitters::{
    EmitterErrorEstimator, OpticsHbtHom, PurcellCavitySpec, SiliconDefect,
};
use crate::physical_specs::{
    HardwarePlatform, IonSpecies, PhysicalIonTrapSpec, PhysicalTransmonSpec,
};
use crate::postprocessing::{
    CustomSyndromeRule, DecodeMode, Postprocessor, QECCodeConfig, QECCodePreset,
};
use crate::qasm::QasmProgram;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AppMode {
    TransmonQubits,
    QuantumEmitters,
    PostprocessingQec,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum PostprocessProtocol {
    ReadoutMitigation,
    QecStabilizer,
    ZeroNoiseExtrapolation,
}

pub struct TransmonApp {
    pub mode: AppMode,

    // --- Hardware Platform & Physics Specs ---
    pub hardware_platform: HardwarePlatform,
    pub spec: PhysicalTransmonSpec,
    pub ion_spec: PhysicalIonTrapSpec,

    // --- Circuit & Simulation State ---
    pub grid: CircuitGrid,
    pub selected_preset: Option<AlgorithmPreset>,
    pub selected_gate: Option<GateType>,
    pub selected_cnot_control: usize,
    pub selected_angle: f64,
    pub qasm_text: String,
    pub shots: usize,
    pub simulation_results: Option<HashMap<String, usize>>,
    pub transmon_error_msg: Option<String>,

    // --- Quantum Emitter State ---
    pub selected_defect: SiliconDefect,
    pub cavity_q: f64,
    pub mode_volume_lambda3: f64,
    pub coupling_g_ghz: f64,
    pub cavity_loss_kappa_ghz: f64,
    pub detuning_ghz: f64,
    pub spin_bath_noise_mhz: f64,

    // --- Postprocessing & QEC State ---
    pub postprocessing_protocol: PostprocessProtocol,
    pub qec_config: QECCodeConfig,
    pub custom_python_script: String,
    pub zne_scale_factors: Vec<f64>,
    pub zne_measured_vals: Vec<f64>,
    pub readout_p01_mitigation: f64,
    pub readout_p10_mitigation: f64,
    pub new_syndrome_input: String,
    pub new_qubit_idx_input: usize,
}

impl Default for TransmonApp {
    fn default() -> Self {
        let grid = CircuitGrid::default();
        let qasm_text = grid.to_qasm();
        let qec_config = QECCodeConfig::default();
        let custom_python_script = Postprocessor::generate_python_postprocessing_script(&qec_config);

        let mut app = Self {
            mode: AppMode::TransmonQubits,

            // Hardware & Specs
            hardware_platform: HardwarePlatform::SuperconductingTransmon,
            spec: PhysicalTransmonSpec::default(),
            ion_spec: PhysicalIonTrapSpec::ionq_aria1(),

            // Circuit Defaults
            grid,
            selected_preset: Some(AlgorithmPreset::Ghz3Qubit),
            selected_gate: Some(GateType::H),
            selected_cnot_control: 0,
            selected_angle: std::f64::consts::FRAC_PI_2,
            qasm_text,
            shots: 1024,
            simulation_results: None,
            transmon_error_msg: None,

            // Emitter Defaults
            selected_defect: SiliconDefect::GCenter,
            cavity_q: 10000.0,
            mode_volume_lambda3: 0.80,
            coupling_g_ghz: 0.25,
            cavity_loss_kappa_ghz: 0.10,
            detuning_ghz: 0.0,
            spin_bath_noise_mhz: 0.20,

            // Postprocessing Defaults
            postprocessing_protocol: PostprocessProtocol::QecStabilizer,
            qec_config,
            custom_python_script,
            zne_scale_factors: vec![1.0, 1.5, 2.0, 3.0],
            zne_measured_vals: vec![0.85, 0.76, 0.68, 0.52],
            readout_p01_mitigation: 0.03,
            readout_p10_mitigation: 0.04,
            new_syndrome_input: "10".to_string(),
            new_qubit_idx_input: 0,
        };
        app.run_transmon_simulation();
        app
    }
}

impl TransmonApp {
    fn sync_from_grid(&mut self) {
        self.qasm_text = self.grid.to_qasm();
        self.run_transmon_simulation();
    }

    fn run_transmon_simulation(&mut self) {
        self.transmon_error_msg = None;
        match QasmProgram::from_str(&self.qasm_text) {
            Ok(program) => {
                let results = match self.hardware_platform {
                    HardwarePlatform::SuperconductingTransmon => {
                        program.run_noisy_simulation(&self.spec, self.shots)
                    }
                    HardwarePlatform::TrappedIon => {
                        program.run_noisy_simulation_ion(&self.ion_spec, self.shots)
                    }
                };
                self.simulation_results = Some(results);
            }
            Err(err) => {
                self.transmon_error_msg = Some(err);
                self.simulation_results = None;
            }
        }
    }

    // =========================================================================
    // WORKSPACE 1: QUANTUM HARDWARE & CIRCUIT SIMULATION
    // =========================================================================
    fn render_transmon_mode(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("hardware_controls")
            .default_width(310.0)
            .min_width(250.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Quantum Hardware Console");
                ui.add_space(4.0);

                // Hardware Platform Selector: Transmon vs. Trapped Ion
                ui.horizontal(|ui| {
                    let transmon_btn = ui.selectable_label(
                        self.hardware_platform == HardwarePlatform::SuperconductingTransmon,
                        "⚛ Transmon",
                    );
                    if transmon_btn.clicked() {
                        self.hardware_platform = HardwarePlatform::SuperconductingTransmon;
                        self.run_transmon_simulation();
                    }

                    let ion_btn = ui.selectable_label(
                        self.hardware_platform == HardwarePlatform::TrappedIon,
                        "⚡ Trapped Ion",
                    );
                    if ion_btn.clicked() {
                        self.hardware_platform = HardwarePlatform::TrappedIon;
                        self.run_transmon_simulation();
                    }
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    match self.hardware_platform {
                        HardwarePlatform::SuperconductingTransmon => {
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Transmon Physical Parameters").strong().color(Color32::LIGHT_BLUE));
                                ui.add(egui::Slider::new(&mut self.spec.josephson_energy_ej, 5.0..=35.0).text("E_J (GHz)"));
                                ui.add(egui::Slider::new(&mut self.spec.charging_energy_ec, 0.10..=0.50).text("E_C (GHz)"));

                                let freq = self.spec.qubit_frequency_ghz();
                                let ratio = self.spec.ej_ec_ratio();
                                ui.label(format!("Transmon Freq f_01 : {:.2} GHz", freq));
                                ui.label(format!("E_J / E_C Ratio   : {:.1}", ratio));
                                ui.label(format!("Anharmonicity α   : {:.2} GHz", self.spec.anharmonicity_ghz()));
                                if ratio < 20.0 {
                                    ui.colored_label(Color32::KHAKI, "Warning: Low E_J/E_C (<20) increases charge noise.");
                                }
                            });

                            ui.add_space(8.0);
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Decoherence & Relaxation Times").strong());
                                ui.add(egui::Slider::new(&mut self.spec.t1_us, 1.0..=300.0).text("T_1 Decay (us)"));
                                ui.add(egui::Slider::new(&mut self.spec.t2_us, 1.0..=300.0).text("T_2 Dephasing (us)"));
                            });

                            ui.add_space(8.0);
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Readout Confusion Matrix").strong());
                                ui.add(egui::Slider::new(&mut self.spec.readout_p01, 0.0..=0.15).text("P(1|0) False Positive"));
                                ui.add(egui::Slider::new(&mut self.spec.readout_p10, 0.0..=0.15).text("P(0|1) False Negative"));
                            });

                            ui.add_space(8.0);
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Transmon Presets").strong());
                                ui.horizontal(|ui| {
                                    if ui.button("IBM Eagle").clicked() {
                                        self.spec = PhysicalTransmonSpec {
                                            josephson_energy_ej: 18.0,
                                            charging_energy_ec: 0.28,
                                            t1_us: 120.0,
                                            t2_us: 80.0,
                                            readout_p01: 0.012,
                                            readout_p10: 0.018,
                                            flux_noise_amplitude: 1e-4,
                                            thermal_population_nth: 0.01,
                                            gate_time_1q_us: 0.02,
                                            gate_time_2q_us: 0.05,
                                        };
                                        self.run_transmon_simulation();
                                    }
                                    if ui.button("High Noise").clicked() {
                                        self.spec = PhysicalTransmonSpec {
                                            josephson_energy_ej: 12.0,
                                            charging_energy_ec: 0.35,
                                            t1_us: 15.0,
                                            t2_us: 10.0,
                                            readout_p01: 0.06,
                                            readout_p10: 0.08,
                                            flux_noise_amplitude: 5e-4,
                                            thermal_population_nth: 0.04,
                                            gate_time_1q_us: 0.02,
                                            gate_time_2q_us: 0.05,
                                        };
                                        self.run_transmon_simulation();
                                    }
                                });
                            });
                        }
                        HardwarePlatform::TrappedIon => {
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Trapped-Ion Architecture").strong().color(Color32::from_rgb(241, 196, 15)));
                                ui.label(format!("Natural Atomic T_1: {:.1} s", self.ion_spec.species.natural_t1_s()));
                                egui::ComboBox::from_label("Species")
                                    .selected_text(self.ion_spec.species.name())
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut self.ion_spec.species, IonSpecies::Yb171, "171Yb+ (Clock, 12.6 GHz)");
                                        ui.selectable_value(&mut self.ion_spec.species, IonSpecies::Ca40, "40Ca+ (Optical, 729 nm)");
                                        ui.selectable_value(&mut self.ion_spec.species, IonSpecies::Ba133, "133Ba+ (Visible, 1.7 GHz)");
                                    });

                                ui.colored_label(Color32::from_rgb(46, 204, 113), "⚡ Native All-to-All Coulomb Crystal Bus Active");
                            });

                            ui.add_space(8.0);
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Secular Trapping & Phonon Noise").strong());
                                ui.add(egui::Slider::new(&mut self.ion_spec.secular_freq_radial_mhz, 0.5..=5.0).text("Radial Freq (MHz)"));
                                ui.add(egui::Slider::new(&mut self.ion_spec.heating_rate_quanta_s, 0.1..=200.0).logarithmic(true).text("Motional Heating dn/dt (quanta/s)"));
                                ui.add(egui::Slider::new(&mut self.ion_spec.t2_s, 0.01..=10.0).logarithmic(true).text("T_2 Coherence (s)"));
                            });

                            ui.add_space(8.0);
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Raman Gate Durations & Optical Errors").strong());
                                ui.add(egui::Slider::new(&mut self.ion_spec.gate_time_2q_us, 50.0..=400.0).text("MS Gate 2Q (us)"));
                                ui.add(egui::Slider::new(&mut self.ion_spec.raman_scattering_prob, 0.0001..=0.005).logarithmic(true).text("Raman Scatter Err"));
                                ui.add(egui::Slider::new(&mut self.ion_spec.readout_p01, 0.001..=0.05).text("Fluorescence P(1|0)"));
                                ui.add(egui::Slider::new(&mut self.ion_spec.readout_p10, 0.001..=0.05).text("Fluorescence P(0|1)"));
                            });

                            ui.add_space(8.0);
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Ion Trap Presets").strong());
                                ui.horizontal(|ui| {
                                    if ui.button("IonQ Aria-1").clicked() {
                                        self.ion_spec = PhysicalIonTrapSpec::ionq_aria1();
                                        self.run_transmon_simulation();
                                    }
                                    if ui.button("Optical Ca-40").clicked() {
                                        self.ion_spec = PhysicalIonTrapSpec::optical_ca40();
                                        self.run_transmon_simulation();
                                    }
                                    if ui.button("High Noise").clicked() {
                                        self.ion_spec = PhysicalIonTrapSpec::high_noise_surface();
                                        self.run_transmon_simulation();
                                    }
                                });
                            });
                        }
                    }

                    ui.add_space(8.0);
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Execution Shots").strong());
                        ui.add(egui::Slider::new(&mut self.shots, 128..=8192).text("Shots"));
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Preset Algorithm Toolbar (Phase 1 P0)
            ui.horizontal(|ui| {
                ui.heading("Circuit Composer");
                ui.separator();
                ui.label(egui::RichText::new("📚 Preset Algorithms:").strong().color(Color32::from_rgb(241, 196, 15)));

                let current_name = self.selected_preset.map(|p| p.name()).unwrap_or("Custom Circuit");
                egui::ComboBox::from_id_source("preset_algo_dropdown")
                    .selected_text(current_name)
                    .show_ui(ui, |ui| {
                        let presets = [
                            AlgorithmPreset::Ghz3Qubit,
                            AlgorithmPreset::BellState,
                            AlgorithmPreset::Grover2Qubit,
                            AlgorithmPreset::DeutschJozsaBalanced,
                            AlgorithmPreset::QuantumTeleportation,
                        ];
                        for p in presets {
                            if ui.selectable_label(self.selected_preset == Some(p), p.name()).clicked() {
                                self.selected_preset = Some(p);
                                self.grid = CircuitGrid::from_preset(p);
                                self.sync_from_grid();
                            }
                        }
                    });
            });
            ui.separator();

            // Gate Palette Toolbar
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Gate Palette:").strong());
                let gates = [
                    GateType::H,
                    GateType::X,
                    GateType::Y,
                    GateType::Z,
                    GateType::Rx,
                    GateType::Ry,
                    GateType::Rz,
                    GateType::Cnot,
                    GateType::Cz,
                    GateType::Swap,
                    GateType::Measure,
                ];
                for g in gates {
                    let is_sel = self.selected_gate == Some(g);
                    let stroke = if is_sel {
                        Stroke::new(2.5_f32, Color32::YELLOW)
                    } else {
                        Stroke::NONE
                    };
                    let text_color = Color32::WHITE;
                    let btn_text = if is_sel {
                        format!("[*] {}", g.label())
                    } else {
                        g.label().to_string()
                    };

                    let btn = egui::Button::new(
                        egui::RichText::new(btn_text).color(text_color).strong(),
                    )
                    .fill(g.color())
                    .stroke(stroke);

                    if ui.add(btn).clicked() {
                        self.selected_gate = Some(g);
                    }
                }

                if matches!(self.selected_gate, Some(GateType::Rx | GateType::Ry | GateType::Rz)) {
                    ui.separator();
                    ui.label(egui::RichText::new("Angle θ:").strong().color(Color32::LIGHT_BLUE));
                    ui.add(egui::Slider::new(&mut self.selected_angle, -std::f64::consts::PI..=std::f64::consts::PI).text("rad"));
                    if ui.button("π/4").clicked() {
                        self.selected_angle = std::f64::consts::FRAC_PI_4;
                    }
                    if ui.button("π/2").clicked() {
                        self.selected_angle = std::f64::consts::FRAC_PI_2;
                    }
                    if ui.button("π").clicked() {
                        self.selected_angle = std::f64::consts::PI;
                    }
                    if ui.button("-π/2").clicked() {
                        self.selected_angle = -std::f64::consts::FRAC_PI_2;
                    }
                } else if matches!(self.selected_gate, Some(GateType::Cnot | GateType::Cz | GateType::Swap)) {
                    ui.separator();
                    let lbl = if self.selected_gate == Some(GateType::Swap) {
                        "Partner:"
                    } else {
                        "Control:"
                    };
                    ui.label(egui::RichText::new(lbl).strong().color(Color32::LIGHT_BLUE));
                    egui::ComboBox::from_id_source("multi_q_combo")
                        .selected_text(format!("q[{}]", self.selected_cnot_control.min(self.grid.num_qubits.saturating_sub(1))))
                        .show_ui(ui, |ui| {
                            for q_idx in 0..self.grid.num_qubits {
                                ui.selectable_value(&mut self.selected_cnot_control, q_idx, format!("q[{}]", q_idx));
                            }
                        });
                }

                ui.separator();
                if ui.button("[+] Qubit").clicked() {
                    self.grid.add_qubit();
                    self.selected_preset = None;
                    self.sync_from_grid();
                }
                if ui.button("[-] Qubit").clicked() {
                    self.grid.remove_qubit();
                    self.selected_preset = None;
                    self.sync_from_grid();
                }
                if ui.button("[Run Grid]").clicked() {
                    self.sync_from_grid();
                }
            });

            ui.add_space(8.0);

            let mut grid_changed = false;
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Circuit Wire Grid").strong());
                    ui.label("(Click on wire to place selected gate. Multi-qubit gates use control/partner above)");
                });
                for q in 0..self.grid.num_qubits {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("q[{}] --", q)).monospace().strong());
                        for s in 0..self.grid.num_steps {
                            let cell = self.grid[q][s].clone();

                            let is_control_for = (0..self.grid.num_qubits).find(|&other_q| {
                                other_q != q
                                    && (self.grid.grid[other_q][s].gate == Some(GateType::Cnot)
                                        || self.grid.grid[other_q][s].gate == Some(GateType::Cz))
                                    && self.grid.grid[other_q][s].control_qubit == Some(q)
                            });

                            let (text, fill) = if let Some(g) = cell.gate {
                                match g {
                                    GateType::Rx | GateType::Ry | GateType::Rz => {
                                        let th = cell.param.unwrap_or(0.0);
                                        (format!("{}({:.2})", g.label(), th), g.color())
                                    }
                                    GateType::Cnot => {
                                        let ctrl = cell.control_qubit.unwrap_or(0);
                                        (format!("(+)c{}", ctrl), g.color())
                                    }
                                    GateType::Cz => {
                                        let ctrl = cell.control_qubit.unwrap_or(0);
                                        (format!("(+)z{}", ctrl), g.color())
                                    }
                                    GateType::Swap => {
                                        let p = cell.control_qubit.unwrap_or(0);
                                        (format!("x q{}", p), g.color())
                                    }
                                    _ => (g.label().to_string(), g.color()),
                                }
                            } else if let Some(target_q) = is_control_for {
                                (format!("• ->q{}", target_q), Color32::from_rgb(41, 128, 185))
                            } else {
                                (" -- ".to_string(), Color32::from_gray(50))
                            };

                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(text).color(Color32::WHITE),
                                    )
                                    .fill(fill),
                                )
                                .clicked()
                            {
                                self.selected_preset = None;
                                if let Some(g) = self.selected_gate {
                                    match g {
                                        GateType::Rx | GateType::Ry | GateType::Rz => {
                                            if cell.gate == Some(g) && cell.param == Some(self.selected_angle) {
                                                self.grid[q][s].gate = None;
                                                self.grid[q][s].param = None;
                                            } else {
                                                self.grid[q][s].gate = Some(g);
                                                self.grid[q][s].param = Some(self.selected_angle);
                                            }
                                            grid_changed = true;
                                        }
                                        GateType::Cnot | GateType::Cz => {
                                            let ctrl = if self.selected_cnot_control != q {
                                                self.selected_cnot_control
                                            } else if q > 0 {
                                                q - 1
                                            } else {
                                                1.min(self.grid.num_qubits.saturating_sub(1))
                                            };

                                            if cell.gate == Some(g) {
                                                self.grid[q][s].gate = None;
                                                self.grid[q][s].control_qubit = None;
                                            } else {
                                                self.grid[q][s].gate = Some(g);
                                                self.grid[q][s].control_qubit = Some(ctrl);
                                            }
                                            grid_changed = true;
                                        }
                                        GateType::Swap => {
                                            let partner = if self.selected_cnot_control != q {
                                                self.selected_cnot_control
                                            } else {
                                                (q + 1) % self.grid.num_qubits
                                            };
                                            if cell.gate == Some(GateType::Swap) {
                                                self.grid[q][s] = GridCell::default();
                                                self.grid[partner][s] = GridCell::default();
                                            } else {
                                                self.grid[q][s] = GridCell {
                                                    gate: Some(GateType::Swap),
                                                    control_qubit: Some(partner),
                                                    param: None,
                                                };
                                                self.grid[partner][s] = GridCell {
                                                    gate: Some(GateType::Swap),
                                                    control_qubit: Some(q),
                                                    param: None,
                                                };
                                            }
                                            grid_changed = true;
                                        }
                                        _ => {
                                            if cell.gate == Some(g) {
                                                self.grid[q][s].gate = None;
                                                self.grid[q][s].control_qubit = None;
                                                self.grid[q][s].param = None;
                                            } else {
                                                self.grid[q][s].gate = Some(g);
                                                self.grid[q][s].control_qubit = None;
                                                self.grid[q][s].param = None;
                                            }
                                            grid_changed = true;
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
            });

            if grid_changed {
                self.sync_from_grid();
            }

            ui.add_space(10.0);

            // QASM Editor & Direct Input Mode
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("OpenQASM 2.0 Direct Editor").strong());
                    ui.separator();
                    if ui.button("▶ Run from QASM Text").clicked() {
                        self.run_transmon_simulation();
                    }
                    if ui.button("🔄 Sync QASM -> Circuit Grid").clicked() {
                        match CircuitGrid::from_qasm(&self.qasm_text) {
                            Ok(new_grid) => {
                                self.grid = new_grid;
                                self.selected_preset = None;
                                self.run_transmon_simulation();
                            }
                            Err(e) => {
                                self.transmon_error_msg = Some(e);
                            }
                        }
                    }
                    if ui.button("🔄 Sync Grid -> QASM").clicked() {
                        self.sync_from_grid();
                    }
                });

                ui.add(
                    egui::TextEdit::multiline(&mut self.qasm_text)
                        .font(egui::TextStyle::Monospace)
                        .desired_rows(7)
                        .desired_width(f32::INFINITY),
                );
            });
        });

        // Right Panel: Visual Graphical Histogram (Phase 1 P0)
        egui::SidePanel::right("transmon_results")
            .default_width(300.0)
            .min_width(240.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Hardware Shot Histogram");
                ui.separator();

                if let Some(err) = &self.transmon_error_msg {
                    ui.colored_label(Color32::LIGHT_RED, format!("Error: {}", err));
                } else if let Some(counts) = &self.simulation_results {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("Platform: {}", self.hardware_platform.label())).strong());
                    });
                    ui.label(format!("Total Shots Executed: {}", self.shots));
                    ui.add_space(8.0);

                    let mut sorted_keys: Vec<_> = counts.keys().collect();
                    sorted_keys.sort();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Measured State Distribution").strong());
                            ui.add_space(4.0);

                            for k in sorted_keys {
                                let cnt = counts[k];
                                let pct = (cnt as f64 / self.shots as f64) * 100.0;
                                let frac = (cnt as f32 / self.shots as f32).clamp(0.0, 1.0);

                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(format!("|{}>", k)).monospace().strong());

                                    // Graphical color bar
                                    let bar_color = if frac > 0.40 {
                                        Color32::from_rgb(46, 204, 113) // Emerald Green for dominant peaks
                                    } else if frac > 0.15 {
                                        Color32::from_rgb(52, 152, 219) // Dodger Blue for superpositions
                                    } else {
                                        Color32::from_rgb(127, 140, 141) // Slate Gray for noise floor
                                    };

                                    let (rect, _response) = ui.allocate_exact_size(Vec2::new(100.0, 14.0), egui::Sense::hover());
                                    let painter = ui.painter_at(rect);

                                    // Background track
                                    painter.rect_filled(rect, 2.0, Color32::from_rgb(30, 36, 48));

                                    // Filled bar
                                    let mut fill_rect = rect;
                                    fill_rect.set_width((rect.width() * frac).max(1.0));
                                    painter.rect_filled(fill_rect, 2.0, bar_color);

                                    ui.label(egui::RichText::new(format!("{:4} ({:4.1}%)", cnt, pct)).monospace());
                                });
                            }
                        });
                    });
                }
            });
    }

    // =========================================================================
    // WORKSPACE 2: SILICON QUANTUM EMITTERS
    // =========================================================================
    fn render_emitters_mode(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("emitter_controls")
            .default_width(320.0)
            .min_width(250.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Silicon Quantum Emitters");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("1. Silicon Color Center Defect").strong());
                        egui::ComboBox::from_label("Defect Registry")
                            .selected_text(self.selected_defect.name())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.selected_defect, SiliconDefect::GCenter, "G-Center (1278 nm)");
                                ui.selectable_value(&mut self.selected_defect, SiliconDefect::TCenter, "T-Center (1326 nm)");
                                ui.selectable_value(&mut self.selected_defect, SiliconDefect::CiCenter, "C_i-Center (1280 nm)");
                                ui.selectable_value(&mut self.selected_defect, SiliconDefect::Al1Center, "Al_1-Center (1300 nm)");
                                ui.selectable_value(&mut self.selected_defect, SiliconDefect::WCenter, "W-Center (1218 nm)");
                                ui.selectable_value(&mut self.selected_defect, SiliconDefect::Er3Plus, "Er^3+ Ion (1536 nm)");
                            });

                        ui.label(format!("Emission Wavelength lambda : {:.1} nm", self.selected_defect.wavelength_nm()));
                        ui.label(format!("Zero-Phonon Line (ZPL)   : {:.0}%", self.selected_defect.zpl_fraction() * 100.0));
                        ui.label(format!("Lifetime T_1              : {:.1} ns", self.selected_defect.lifetime_ns()));
                        ui.label(format!("Spin Coherence T_2        : {:.1} us", self.selected_defect.spin_t2_us()));
                    });

                    ui.add_space(8.0);
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("2. Nanophotonic Purcell Cavity").strong());
                        ui.add(egui::Slider::new(&mut self.cavity_q, 100.0..=100000.0).logarithmic(true).text("Cavity Q-Factor"));
                        ui.add(egui::Slider::new(&mut self.mode_volume_lambda3, 0.1..=5.0).text("Mode Vol V (lambda/n)^3"));
                        ui.add(egui::Slider::new(&mut self.coupling_g_ghz, 0.01..=2.0).text("Coupling g (GHz)"));
                        ui.add(egui::Slider::new(&mut self.cavity_loss_kappa_ghz, 0.01..=5.0).text("Cavity Loss kappa (GHz)"));
                        ui.add(egui::Slider::new(&mut self.detuning_ghz, -2.0..=2.0).text("Detuning Delta (GHz)"));
                    });

                    ui.add_space(8.0);
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("3. Optical Dephasing & Noise").strong());
                        ui.add(egui::Slider::new(&mut self.spin_bath_noise_mhz, 0.01..=10.0).text("Spin Bath Noise (MHz)"));
                    });
                });
            });

        let cavity_spec = PurcellCavitySpec {
            defect: self.selected_defect,
            cavity_q: self.cavity_q,
            mode_volume_lambda3: self.mode_volume_lambda3,
            coupling_g_ghz: self.coupling_g_ghz,
            cavity_loss_kappa_ghz: self.cavity_loss_kappa_ghz,
            detuning_ghz: self.detuning_ghz,
        };

        let f_purcell = EmitterErrorEstimator::calculate_purcell_factor(&cavity_spec);
        let rabi_points = EmitterErrorEstimator::compute_rabi_dynamics(&cavity_spec, 100);
        let v_hom = OpticsHbtHom::calculate_hom_visibility(&self.selected_defect, self.spin_bath_noise_mhz);
        let hahn_points = OpticsHbtHom::compute_hahn_echo_decay(&self.selected_defect, self.spin_bath_noise_mhz, 100);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Quantum Emitter Dynamics & Single-Photon Optics");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                // --- PLOT 1: Vacuum Rabi Dynamics ---
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Vacuum Rabi Dynamics P_e(t)").strong());
                    ui.label("Time-evolution of excited state population under non-Hermitian Hamiltonian H_eff:");
                    ui.add_space(4.0);

                    let (rect, _response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 160.0), egui::Sense::hover());
                    let painter = ui.painter_at(rect);

                    painter.rect_filled(rect, 4.0, Color32::from_rgb(20, 24, 32));
                    painter.rect_stroke(rect, 4.0, Stroke::new(1.0_f32, Color32::from_rgb(50, 60, 80)));

                    for ratio in [0.25_f32, 0.50_f32, 0.75_f32] {
                        let y = rect.bottom() - ratio * rect.height();
                        painter.line_segment(
                            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                            Stroke::new(0.5_f32, Color32::from_gray(40)),
                        );
                    }

                    if rabi_points.len() > 1 {
                        let max_t = rabi_points.last().map(|p| p.0).unwrap_or(10.0);
                        let points_to_draw: Vec<Pos2> = rabi_points
                            .iter()
                            .map(|(t, pe)| {
                                let x = rect.left() + (t / max_t) as f32 * rect.width();
                                let y = rect.bottom() - (*pe as f32 * 0.90_f32 + 0.05_f32) * rect.height();
                                Pos2::new(x, y)
                            })
                            .collect();

                        for window in points_to_draw.windows(2) {
                            painter.line_segment([window[0], window[1]], Stroke::new(2.0_f32, Color32::LIGHT_BLUE));
                        }
                    }

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Time: 0.0 ns").small().monospace());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new("10.0 ns").small().monospace());
                        });
                    });
                });

                ui.add_space(12.0);

                // --- PLOT 2: Hahn Echo Spin Coherence Decay ---
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Hahn Echo Spin Coherence Decay S(t)").strong());
                    ui.label(format!("HOM Photon Indistinguishability Visibility V_HOM: {:.1}%", v_hom * 100.0));
                    ui.add_space(4.0);

                    let (rect, _response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 160.0), egui::Sense::hover());
                    let painter = ui.painter_at(rect);

                    painter.rect_filled(rect, 4.0, Color32::from_rgb(20, 24, 32));
                    painter.rect_stroke(rect, 4.0, Stroke::new(1.0_f32, Color32::from_rgb(50, 60, 80)));

                    for ratio in [0.25_f32, 0.50_f32, 0.75_f32] {
                        let y = rect.bottom() - ratio * rect.height();
                        painter.line_segment(
                            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                            Stroke::new(0.5_f32, Color32::from_gray(40)),
                        );
                    }

                    if hahn_points.len() > 1 {
                        let max_t = hahn_points.last().map(|p| p.0).unwrap_or(10.0);
                        let points_to_draw: Vec<Pos2> = hahn_points
                            .iter()
                            .map(|(t, sig)| {
                                let x = rect.left() + (t / max_t) as f32 * rect.width();
                                let y = rect.bottom() - (*sig as f32 * 0.90_f32 + 0.05_f32) * rect.height();
                                Pos2::new(x, y)
                            })
                            .collect();

                        for window in points_to_draw.windows(2) {
                            painter.line_segment([window[0], window[1]], Stroke::new(2.0_f32, Color32::GREEN));
                        }
                    }

                    let max_t_us = hahn_points.last().map(|p| p.0).unwrap_or(0.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Time: 0.0 us").small().monospace());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(format!("{:.1} us", max_t_us)).small().monospace());
                        });
                    });
                });
            });
        });

        egui::SidePanel::right("emitter_metrics")
            .default_width(260.0)
            .min_width(200.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Emitter & Cavity Metrics");
                ui.separator();

                ui.group(|ui| {
                    ui.label(egui::RichText::new("Cavity Purcell Enhancement").strong());
                    ui.label(format!("Purcell Factor F_P : {:.1}x", f_purcell));
                    let enhanced_lifetime = self.selected_defect.lifetime_ns() / (1.0 + f_purcell * self.selected_defect.zpl_fraction());
                    ui.label(format!("Enhanced T_1      : {:.2} ns", enhanced_lifetime));
                });

                ui.add_space(8.0);

                ui.group(|ui| {
                    ui.label(egui::RichText::new("Regime Classification").strong());
                    if self.coupling_g_ghz > self.cavity_loss_kappa_ghz / 4.0 {
                        ui.colored_label(Color32::GREEN, "Strong Coupling Regime (Vacuum Rabi)");
                    } else {
                        ui.colored_label(Color32::LIGHT_BLUE, "Weak Coupling Regime (Purcell Decay)");
                    }
                });
            });
    }

    // =========================================================================
    // WORKSPACE 3: POSTPROCESSING & QUANTUM ERROR CORRECTION (QEC)
    // =========================================================================
    fn render_postprocessing_mode(&mut self, ctx: &egui::Context) {
        let raw_counts = if let Some(res) = &self.simulation_results {
            res.clone()
        } else {
            let mut mock = HashMap::new();
            mock.insert("0000".to_string(), 1950);
            mock.insert("1111".to_string(), 1920);
            mock.insert("1000".to_string(), 110);
            mock.insert("0110".to_string(), 60);
            mock.insert("1001".to_string(), 56);
            mock
        };

        egui::SidePanel::left("postprocess_controls")
            .default_width(320.0)
            .min_width(260.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("QEC & Postprocessing Engine");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("1. Select Engine Protocol").strong());
                        ui.selectable_value(
                            &mut self.postprocessing_protocol,
                            PostprocessProtocol::QecStabilizer,
                            "Stabilizer QEC Pipeline",
                        );
                        ui.selectable_value(
                            &mut self.postprocessing_protocol,
                            PostprocessProtocol::ReadoutMitigation,
                            "Readout Confusion Mitigation",
                        );
                        ui.selectable_value(
                            &mut self.postprocessing_protocol,
                            PostprocessProtocol::ZeroNoiseExtrapolation,
                            "Zero-Noise Extrapolation (ZNE)",
                        );
                    });

                    ui.add_space(8.0);

                    match self.postprocessing_protocol {
                        PostprocessProtocol::QecStabilizer => {
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("2. QEC Code Preset").strong());
                                egui::ComboBox::from_label("Code Selection")
                                    .selected_text(self.qec_config.preset.name())
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut self.qec_config.preset,
                                            QECCodePreset::Repetition3QubitBitFlip,
                                            "3-Qubit Bit-Flip Repetition Code",
                                        );
                                        ui.selectable_value(
                                            &mut self.qec_config.preset,
                                            QECCodePreset::ErrorDetecting4Qubit,
                                            "4-Qubit Error-Detecting Code",
                                        );
                                        ui.selectable_value(
                                            &mut self.qec_config.preset,
                                            QECCodePreset::Perfect5Qubit,
                                            "5-Qubit Perfect Code",
                                        );
                                        ui.selectable_value(
                                            &mut self.qec_config.preset,
                                            QECCodePreset::Steane7Qubit,
                                            "7-Qubit Steane Code",
                                        );
                                    });

                                ui.add_space(6.0);
                                ui.label(egui::RichText::new("3. Pipeline Strategy").strong());
                                ui.selectable_value(
                                    &mut self.qec_config.decode_mode,
                                    DecodeMode::ActiveSyndromeCorrection,
                                    "Active Syndrome Bit-Flipping",
                                );
                                ui.selectable_value(
                                    &mut self.qec_config.decode_mode,
                                    DecodeMode::PostSelectionDiscarding,
                                    "Post-Selection Discarding",
                                );
                            });

                            if self.qec_config.decode_mode == DecodeMode::ActiveSyndromeCorrection {
                                ui.add_space(6.0);
                                ui.group(|ui| {
                                    ui.label(egui::RichText::new("Syndrome Recovery Rules").strong());
                                    ui.checkbox(&mut self.qec_config.enable_auto_majority_vote, "Use Standard Codebook Lookup");

                                    if !self.qec_config.enable_auto_majority_vote {
                                        ui.separator();
                                        ui.label(egui::RichText::new("Active Custom Rules:").small().strong());
                                        let mut to_remove = None;
                                        for (idx, rule) in self.qec_config.custom_rules.iter().enumerate() {
                                            ui.horizontal(|ui| {
                                                ui.label(format!("Synd [{}] -> Flip Q[{}]", rule.syndrome_bits, rule.flip_qubit_idx));
                                                if ui.button("❌").clicked() {
                                                    to_remove = Some(idx);
                                                }
                                            });
                                        }
                                        if let Some(rem_idx) = to_remove {
                                            self.qec_config.custom_rules.remove(rem_idx);
                                        }

                                        ui.add_space(6.0);
                                        ui.label(egui::RichText::new("Add Custom Rule:").small().strong());
                                        ui.horizontal(|ui| {
                                            ui.label("Synd:");
                                            ui.add(egui::TextEdit::singleline(&mut self.new_syndrome_input).desired_width(45.0));
                                            ui.label("Flip Q:");
                                            let max_q = self.qec_config.preset.physical_qubits().saturating_sub(1);
                                            ui.add(egui::DragValue::new(&mut self.new_qubit_idx_input).clamp_range(0..=max_q));
                                            if ui.button("➕ Add").clicked() {
                                                let synd = self.new_syndrome_input.trim().to_string();
                                                if !synd.is_empty() {
                                                    self.qec_config.custom_rules.push(CustomSyndromeRule {
                                                        syndrome_bits: synd,
                                                        flip_qubit_idx: self.new_qubit_idx_input,
                                                    });
                                                }
                                            }
                                        });

                                        ui.add_space(4.0);
                                        if ui.button("↺ Reset Defaults").clicked() {
                                            self.qec_config.custom_rules = vec![
                                                CustomSyndromeRule { syndrome_bits: "10".to_string(), flip_qubit_idx: 0 },
                                                CustomSyndromeRule { syndrome_bits: "11".to_string(), flip_qubit_idx: 1 },
                                                CustomSyndromeRule { syndrome_bits: "01".to_string(), flip_qubit_idx: 2 },
                                            ];
                                        }
                                    }
                                });
                            }
                        }
                        PostprocessProtocol::ReadoutMitigation => {
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("Readout Confusion Parameters").strong());
                                ui.add(egui::Slider::new(&mut self.readout_p01_mitigation, 0.0..=0.20).text("P(1|0) False Pos"));
                                ui.add(egui::Slider::new(&mut self.readout_p10_mitigation, 0.0..=0.20).text("P(0|1) False Neg"));
                            });
                        }
                        PostprocessProtocol::ZeroNoiseExtrapolation => {
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("ZNE Noise Scaling Factors").strong());
                                ui.label("Noise scale factors lambda: [1.0, 1.5, 2.0, 3.0]");
                            });
                        }
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Postprocessing Results & In-Engine Console");
            ui.separator();

            match self.postprocessing_protocol {
                PostprocessProtocol::QecStabilizer => {
                    let qec_result = Postprocessor::process_qec_pipeline(&raw_counts, &self.qec_config);

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("ℹ️ What is Stabilizer QEC & Syndrome Decoding?").strong().color(Color32::LIGHT_BLUE));
                            ui.label(
                                "Quantum Error Correction protects quantum data by encoding logical qubits into entangled physical registers:\n\
                                 • Syndrome Measurement: Parity-check stabilizer operators detect bit-flips without measuring or destroying superposition data.\n\
                                 • Active Syndrome Bit-Flipping: Non-zero syndrome bitstrings map to targeted Pauli-X operations on corrupted physical qubits.\n\
                                 • Post-Selection: Discards runs with detected parity violations (e.g. odd parity), boosting fidelity at the cost of retention."
                            );
                        });

                        ui.add_space(8.0);
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Physical State Histogram Comparison").strong());
                            let mut sorted_keys: Vec<_> = raw_counts.keys().cloned().collect();
                            for k in qec_result.corrected_physical_counts.keys() {
                                if !sorted_keys.contains(k) {
                                    sorted_keys.push(k.clone());
                                }
                            }
                            sorted_keys.sort();

                            for key in &sorted_keys {
                                let raw_cnt = *raw_counts.get(key).unwrap_or(&0);
                                let corr_cnt = *qec_result.corrected_physical_counts.get(key).unwrap_or(&0);
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(format!("|{}>", key)).monospace().strong());
                                    ui.label(format!("Raw: {:4}  |  Processed: {:4}", raw_cnt, corr_cnt));
                                });
                            }
                        });

                        ui.add_space(12.0);
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Option B: In-Engine Script Console").strong());
                                if ui.button("Refresh Template").clicked() {
                                    self.custom_python_script =
                                        Postprocessor::generate_python_postprocessing_script(&self.qec_config);
                                }
                            });

                            ui.add(
                                egui::TextEdit::multiline(&mut self.custom_python_script)
                                    .font(egui::TextStyle::Monospace)
                                    .desired_rows(8)
                                    .desired_width(f32::INFINITY),
                            );
                        });
                    });
                }
                PostprocessProtocol::ReadoutMitigation => {
                    let total_shots: usize = raw_counts.values().sum();
                    let mitigated = Postprocessor::mitigate_readout_errors_1q(
                        &raw_counts,
                        total_shots,
                        self.readout_p01_mitigation,
                        self.readout_p10_mitigation,
                    );

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("ℹ️ What is Readout Confusion Mitigation?").strong().color(Color32::LIGHT_BLUE));
                            ui.label(
                                "Measurement detectors on physical hardware experience classification errors:\n\
                                 • False Positive P(1|0): Qubit in |0> misclassified as |1> due to thermal excitation or resonator ring-up. \n\
                                 • False Negative P(0|1): Qubit in |1> decays to |0> during measurement readout integration.\n\n\
                                 Matrix Inversion Unfolding:\n\
                                 [P_0_mit, P_1_mit]^T = M^(-1) * [P_0_raw, P_1_raw]^T,\n\
                                 where M = [[1 - P(1|0), P(0|1)], [P(1|0), 1 - P(0|1)]].\n\
                                 Inverting M removes detector distortion and recovers the ideal pre-readout quantum probabilities."
                            );
                        });

                        ui.add_space(8.0);
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Readout Confusion Matrix Inversion").strong());
                            ui.label("Raw Measured Probabilities vs. Inverted Unfolded Distribution:");
                            ui.add_space(8.0);

                            for state in ["0", "1"] {
                                let raw_cnt = *raw_counts.get(state).unwrap_or(&0);
                                let raw_p = if total_shots > 0 {
                                    raw_cnt as f64 / total_shots as f64
                                } else {
                                    0.0
                                };
                                let mit_p = *mitigated.get(state).unwrap_or(&0.0);

                                ui.group(|ui| {
                                    ui.label(format!("State |{}> : Raw P = {:.1}%  -->  Mitigated P = {:.1}%", state, raw_p * 100.0, mit_p * 100.0));
                                });
                            }
                        });
                    });
                }
                PostprocessProtocol::ZeroNoiseExtrapolation => {
                    let zne_val = Postprocessor::extrapolate_zero_noise(&self.zne_scale_factors, &self.zne_measured_vals);

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("ℹ️ What is Zero-Noise Extrapolation (ZNE)?").strong().color(Color32::LIGHT_BLUE));
                            ui.label(
                                "Zero-Noise Extrapolation (ZNE) is a NISQ error mitigation technique requiring no extra physical qubits:\n\
                                 1. Noise Amplification (λ): Artificially amplify gate noise by factors λ (e.g. 1.0x, 1.5x, 2.0x, 3.0x)\n\
                                    via pulse stretching or identity insertion U -> U (U† U)^n.\n\
                                 2. Expectation Sampling: Measure observable expectation values <O>(λ) at each amplified noise level. \n\
                                 3. Zero-Noise Extrapolation: Fit a curve across measured points and extrapolate to the zero-noise limit λ -> 0.\n\n\
                                 The intercept at λ = 0 recovers the error-free expectation value without physical ancillae."
                            );
                        });

                        ui.add_space(8.0);
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Zero-Noise Extrapolation (ZNE) Plot").strong());
                            ui.label(format!("Extrapolated Zero-Noise Expectation Value E(lambda=0): {:.3}", zne_val));
                            ui.add_space(8.0);

                            let (rect, _response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 160.0), egui::Sense::hover());
                            let painter = ui.painter_at(rect);

                            painter.rect_filled(rect, 4.0, Color32::from_rgb(20, 24, 32));
                            painter.rect_stroke(rect, 4.0, Stroke::new(1.0_f32, Color32::from_rgb(50, 60, 80)));

                            for idx in 0..self.zne_scale_factors.len() {
                                let sf = self.zne_scale_factors[idx];
                                let val = self.zne_measured_vals[idx];
                                let x = rect.left() + (sf as f32 / 3.5_f32) * rect.width();
                                let y = rect.bottom() - (val as f32 * 0.80_f32 + 0.10_f32) * rect.height();
                                painter.circle_filled(Pos2::new(x, y), 4.0_f32, Color32::YELLOW);
                            }

                            // Extrapolated point at lambda=0
                            let x0 = rect.left();
                            let y0 = rect.bottom() - (zne_val as f32 * 0.80_f32 + 0.10_f32) * rect.height();
                            painter.circle_filled(Pos2::new(x0, y0), 6.0_f32, Color32::GREEN);
                        });
                    });
                }
            }
        });

        egui::SidePanel::right("postprocess_output")
            .default_width(270.0)
            .min_width(200.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("QEC Metrics & Retention");
                ui.separator();

                if self.postprocessing_protocol == PostprocessProtocol::QecStabilizer {
                    let qec_result = Postprocessor::process_qec_pipeline(&raw_counts, &self.qec_config);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Execution Metrics").strong());
                            ui.label(format!("Total Input Shots : {}", qec_result.total_input_shots));
                            ui.label(format!("Retained Shots    : {}", qec_result.retained_shots));
                            ui.label(format!("Discarded Shots   : {}", qec_result.discarded_shots));
                            ui.label(format!("Retention Rate    : {:.2}%", qec_result.retention_rate * 100.0));
                        });

                        ui.add_space(8.0);
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Logical State Distribution").strong());
                            let mut sorted_logical: Vec<_> = qec_result.logical_counts.keys().collect();
                            sorted_logical.sort();

                            for l_state in sorted_logical {
                                let cnt = qec_result.logical_counts[l_state];
                                let pct = if qec_result.retained_shots > 0 {
                                    (cnt as f64 / qec_result.retained_shots as f64) * 100.0
                                } else {
                                    0.0
                                };
                                ui.label(format!("|{}>_L : {} shots ({:.1}%)", l_state, cnt, pct));
                            }
                        });
                    });
                }
            });
    }
}

impl eframe::App for TransmonApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_navigation").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading("Qty Quantum Hardware Console");
                ui.add_space(20.0);
                ui.label(egui::RichText::new("Workspace:").strong());

                ui.selectable_value(
                    &mut self.mode,
                    AppMode::TransmonQubits,
                    "⚛ Hardware & Circuits",
                );
                ui.selectable_value(
                    &mut self.mode,
                    AppMode::QuantumEmitters,
                    "🔬 Silicon Emitters",
                );
                ui.selectable_value(
                    &mut self.mode,
                    AppMode::PostprocessingQec,
                    "🛡 Postprocessing & QEC",
                );
            });
            ui.add_space(4.0);
        });

        match self.mode {
            AppMode::TransmonQubits => self.render_transmon_mode(ctx),
            AppMode::QuantumEmitters => self.render_emitters_mode(ctx),
            AppMode::PostprocessingQec => self.render_postprocessing_mode(ctx),
        }
    }
}
