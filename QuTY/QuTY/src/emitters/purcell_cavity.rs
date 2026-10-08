use crate::emitters::defects::SiliconDefect;

#[derive(Debug, Clone)]
pub struct PurcellCavitySpec {
    pub defect: SiliconDefect,
    pub cavity_q: f64,
    pub mode_volume_lambda3: f64,
    pub coupling_g_ghz: f64,
    pub cavity_loss_kappa_ghz: f64,
    pub detuning_ghz: f64,
}

pub struct EmitterErrorEstimator;

impl EmitterErrorEstimator {
    pub fn calculate_purcell_factor(spec: &PurcellCavitySpec) -> f64 {
        let n = 3.45; // Silicon refractive index
        let f_max = (3.0 / (4.0 * std::f64::consts::PI * std::f64::consts::PI)) * (spec.cavity_q / spec.mode_volume_lambda3) * (1.0 / (n * n * n));
        let lorentzian = 1.0 / (1.0 + 4.0 * (spec.detuning_ghz / spec.cavity_loss_kappa_ghz).powi(2));
        f_max * lorentzian * spec.defect.zpl_fraction()
    }

    pub fn compute_rabi_dynamics(spec: &PurcellCavitySpec, num_points: usize) -> Vec<(f64, f64)> {
        let mut points = Vec::with_capacity(num_points);
        let max_time_ns = 10.0;
        let g = spec.coupling_g_ghz * 2.0 * std::f64::consts::PI;
        let kappa = spec.cavity_loss_kappa_ghz * 2.0 * std::f64::consts::PI;
        let delta = spec.detuning_ghz * 2.0 * std::f64::consts::PI;

        let omega_rabi = (g * g + (delta / 2.0).powi(2)).sqrt();

        for i in 0..num_points {
            let t = (i as f64 / (num_points - 1) as f64) * max_time_ns;
            let env = (-kappa * t / 4.0).exp();
            let pe = env * (omega_rabi * t).cos().powi(2);
            points.push((t, pe.clamp(0.0, 1.0)));
        }

        points
    }
}

