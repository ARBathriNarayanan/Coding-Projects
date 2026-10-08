use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalTransmonSpec {
    pub josephson_energy_ej: f64,
    pub charging_energy_ec: f64,
    pub flux_noise_amplitude: f64,
    pub thermal_population_nth: f64,
    pub gate_time_1q_us: f64,
    pub gate_time_2q_us: f64,
    pub t1_us: f64,
    pub t2_us: f64,
    pub readout_p01: f64,
    pub readout_p10: f64,
}

impl Default for PhysicalTransmonSpec {
    fn default() -> Self {
        Self {
            josephson_energy_ej: 18.0,
            charging_energy_ec: 0.28,
            flux_noise_amplitude: 1e-4,
            thermal_population_nth: 0.01,
            gate_time_1q_us: 0.02,
            gate_time_2q_us: 0.05,
            t1_us: 100.0,
            t2_us: 70.0,
            readout_p01: 0.015,
            readout_p10: 0.025,
        }
    }
}

impl PhysicalTransmonSpec {
    pub fn qubit_frequency_ghz(&self) -> f64 {
        (8.0 * self.josephson_energy_ej * self.charging_energy_ec).sqrt() - self.charging_energy_ec
    }

    pub fn ej_ec_ratio(&self) -> f64 {
        if self.charging_energy_ec > 0.0 {
            self.josephson_energy_ej / self.charging_energy_ec
        } else {
            0.0
        }
    }
}
