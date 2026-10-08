use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwarePlatform {
    SuperconductingTransmon,
    TrappedIon,
}

impl HardwarePlatform {
    pub fn label(&self) -> &'static str {
        match self {
            HardwarePlatform::SuperconductingTransmon => "⚛ Superconducting Transmon",
            HardwarePlatform::TrappedIon => "⚡ Trapped Ion (Aria-1)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalTransmonSpec {
    pub josephson_energy_ej: f64,
    pub charging_energy_ec: f64,
    pub t1_us: f64,
    pub t2_us: f64,
    pub readout_p01: f64,
    pub readout_p10: f64,
    pub flux_noise_amplitude: f64,
    pub thermal_population_nth: f64,
    pub gate_time_1q_us: f64,
    pub gate_time_2q_us: f64,
}

impl Default for PhysicalTransmonSpec {
    fn default() -> Self {
        Self {
            josephson_energy_ej: 18.0,
            charging_energy_ec: 0.28,
            t1_us: 120.0,
            t2_us: 80.0,
            readout_p01: 0.015,
            readout_p10: 0.020,
            flux_noise_amplitude: 1e-4,
            thermal_population_nth: 0.01,
            gate_time_1q_us: 0.020,
            gate_time_2q_us: 0.050,
        }
    }
}

impl PhysicalTransmonSpec {
    pub fn qubit_frequency_ghz(&self) -> f64 {
        (8.0 * self.josephson_energy_ej * self.charging_energy_ec).sqrt() - self.charging_energy_ec
    }

    pub fn ej_ec_ratio(&self) -> f64 {
        self.josephson_energy_ej / self.charging_energy_ec
    }

    pub fn anharmonicity_ghz(&self) -> f64 {
        -self.charging_energy_ec
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IonSpecies {
    Yb171, // 171Yb+ (12.64 GHz hyperfine clock qubit)
    Ca40,  // 40Ca+ (729 nm optical quadrupole qubit)
    Ba133, // 133Ba+ (visible optical/hyperfine qubit)
}

impl IonSpecies {
    pub fn name(&self) -> &'static str {
        match self {
            IonSpecies::Yb171 => "171Yb+ (Hyperfine Clock, 12.6 GHz)",
            IonSpecies::Ca40 => "40Ca+ (Optical Quadrupole, 729 nm)",
            IonSpecies::Ba133 => "133Ba+ (Barium Visible, 1.7 GHz)",
        }
    }

    pub fn natural_t1_s(&self) -> f64 {
        match self {
            IonSpecies::Yb171 => 1e6,
            IonSpecies::Ca40 => 1.16,
            IonSpecies::Ba133 => 80.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalIonTrapSpec {
    pub species: IonSpecies,
    pub name: String,
    pub t1_s: f64,
    pub t2_s: f64,
    pub secular_freq_axial_mhz: f64,
    pub secular_freq_radial_mhz: f64,
    pub heating_rate_quanta_s: f64, // \dot{\bar{n}} motional heating
    pub gate_time_1q_us: f64,
    pub gate_time_2q_us: f64,       // Mølmer-Sørensen gate
    pub raman_scattering_prob: f64,
    pub readout_p01: f64,
    pub readout_p10: f64,
    pub all_to_all_connectivity: bool,
}

impl Default for PhysicalIonTrapSpec {
    fn default() -> Self {
        Self::ionq_aria1()
    }
}

impl PhysicalIonTrapSpec {
    pub fn ionq_aria1() -> Self {
        Self {
            species: IonSpecies::Yb171,
            name: "IonQ Aria-1 (Yb-171)".to_string(),
            t1_s: 1000.0,
            t2_s: 1.2,
            secular_freq_axial_mhz: 0.35,
            secular_freq_radial_mhz: 3.10,
            heating_rate_quanta_s: 8.0,
            gate_time_1q_us: 10.0,
            gate_time_2q_us: 180.0,
            raman_scattering_prob: 0.0003,
            readout_p01: 0.003,
            readout_p10: 0.005,
            all_to_all_connectivity: true,
        }
    }

    pub fn optical_ca40() -> Self {
        Self {
            species: IonSpecies::Ca40,
            name: "Optical Ca-40 Trap".to_string(),
            t1_s: 1.16,
            t2_s: 0.25,
            secular_freq_axial_mhz: 0.80,
            secular_freq_radial_mhz: 2.50,
            heating_rate_quanta_s: 25.0,
            gate_time_1q_us: 5.0,
            gate_time_2q_us: 100.0,
            raman_scattering_prob: 0.0010,
            readout_p01: 0.006,
            readout_p10: 0.008,
            all_to_all_connectivity: true,
        }
    }

    pub fn high_noise_surface() -> Self {
        Self {
            species: IonSpecies::Yb171,
            name: "High-Noise Surface Trap".to_string(),
            t1_s: 50.0,
            t2_s: 0.08,
            secular_freq_axial_mhz: 0.25,
            secular_freq_radial_mhz: 1.80,
            heating_rate_quanta_s: 150.0,
            gate_time_1q_us: 15.0,
            gate_time_2q_us: 250.0,
            raman_scattering_prob: 0.0025,
            readout_p01: 0.020,
            readout_p10: 0.035,
            all_to_all_connectivity: true,
        }
    }
}
