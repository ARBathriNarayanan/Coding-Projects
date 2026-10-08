use crate::emitters::defects::SiliconDefect;

pub struct OpticsHbtHom;

impl OpticsHbtHom {
    pub fn calculate_hom_visibility(defect: &SiliconDefect, spin_bath_noise_mhz: f64) -> f64 {
        let t1_ns = defect.lifetime_ns();
        let t2_us = defect.spin_t2_us();
        let t2_ns = t2_us * 1000.0;

        // Pure dephasing rate from spin bath noise
        let gamma_pure = spin_bath_noise_mhz / 1000.0; // GHz equivalent
        let t2_star_ns = 1.0 / ((1.0 / t2_ns) + gamma_pure);

        // HOM visibility V_HOM = T2 / (2 * T1)
        (t2_star_ns / (2.0 * t1_ns)).clamp(0.0, 1.0)
    }

    pub fn compute_hahn_echo_decay(
        defect: &SiliconDefect,
        spin_bath_noise_mhz: f64,
        num_points: usize,
    ) -> Vec<(f64, f64)> {
        let t2_us = defect.spin_t2_us();
        let gamma_bath = spin_bath_noise_mhz / 10.0; // Bath noise dephasing factor
        let t2_eff = t2_us / (1.0 + gamma_bath);

        let max_t = (t2_eff * 3.0).max(1.0);
        let mut points = Vec::with_capacity(num_points);

        for i in 0..num_points {
            let t = (i as f64 / (num_points - 1) as f64) * max_t;
            // Hahn echo Gaussian decay profile
            let decay = (-(t / t2_eff).powi(2)).exp();
            points.push((t, decay));
        }

        points
    }
}
