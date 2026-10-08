#[allow(dead_code)]
pub struct QuantumNoiseChannel;

#[allow(dead_code)]
impl QuantumNoiseChannel {
    pub fn apply_amplitude_damping(p1: f64, t1_us: f64, gate_time_us: f64) -> f64 {
        let gamma = 1.0 - (-gate_time_us / t1_us).exp();
        p1 * (1.0 - gamma)
    }

    pub fn apply_phase_damping(coherence: f64, t2_us: f64, gate_time_us: f64) -> f64 {
        let gamma_phi = 1.0 - (-gate_time_us / t2_us).exp();
        coherence * (1.0 - gamma_phi)
    }
}
