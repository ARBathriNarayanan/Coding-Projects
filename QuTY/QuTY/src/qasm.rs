use crate::physical_specs::PhysicalTransmonSpec;
use num_complex::Complex64;
use rand::Rng;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct QasmProgram {
    pub num_qubits: usize,
    pub instructions: Vec<String>,
}

impl QasmProgram {
    pub fn from_str(qasm: &str) -> Result<Self, String> {
        let mut num_qubits = 3;
        let mut instructions = Vec::new();

        for line in qasm.lines().map(str::trim) {
            if line.starts_with("qreg") {
                if let (Some(s), Some(e)) = (line.find('['), line.find(']')) {
                    if let Ok(n) = line[s + 1..e].parse::<usize>() {
                        num_qubits = n.clamp(1, 16);
                    }
                }
            } else if !line.is_empty()
                && !line.starts_with("//")
                && !line.starts_with("OPENQASM")
                && !line.starts_with("include")
                && !line.starts_with("creg")
            {
                instructions.push(line.to_string());
            }
        }

        Ok(Self {
            num_qubits,
            instructions,
        })
    }

    pub fn simulate_ideal(&self) -> Vec<Complex64> {
        let n = self.num_qubits;
        let dim = 1 << n;
        let mut state = vec![Complex64::new(0.0, 0.0); dim];
        if dim > 0 {
            state[0] = Complex64::new(1.0, 0.0);
        }

        for inst in &self.instructions {
            Self::apply_instruction(&mut state, n, inst);
        }

        state
    }

    pub fn run_noisy_simulation(
        &self,
        spec: &PhysicalTransmonSpec,
        shots: usize,
    ) -> HashMap<String, usize> {
        let mut results = HashMap::new();
        if shots == 0 {
            return results;
        }

        let ideal_state = self.simulate_ideal();
        let dim = 1 << self.num_qubits;
        let mut probs: Vec<f64> = ideal_state.iter().map(|c| c.norm_sqr()).collect();

        let total_prob: f64 = probs.iter().sum();
        if total_prob > 0.0 {
            for p in &mut probs {
                *p /= total_prob;
            }
        } else if dim > 0 {
            probs[0] = 1.0;
        }

        let decoherence_rate = ((spec.gate_time_1q_us / spec.t1_us)
            + (spec.gate_time_1q_us / spec.t2_us))
            .clamp(0.0, 0.5);

        let mut cum_probs = Vec::with_capacity(dim);
        let mut acc = 0.0;
        for &p in &probs {
            acc += p;
            cum_probs.push(acc);
        }

        let mut rng = rand::thread_rng();

        for _ in 0..shots {
            let mut val = if rng.gen::<f64>() >= decoherence_rate {
                let r: f64 = rng.gen();
                let mut chosen = 0;
                for (idx, &cp) in cum_probs.iter().enumerate() {
                    if r <= cp {
                        chosen = idx;
                        break;
                    }
                }
                chosen
            } else {
                rng.gen_range(0..dim)
            };

            for q in 0..self.num_qubits {
                let bit = (val >> q) & 1;
                if bit == 0 {
                    if rng.gen::<f64>() < spec.readout_p01 {
                        val ^= 1 << q;
                    }
                } else if rng.gen::<f64>() < spec.readout_p10 {
                    val ^= 1 << q;
                }
            }

            let bitstring = format!("{:0width$b}", val, width = self.num_qubits);
            *results.entry(bitstring).or_insert(0) += 1;
        }

        results
    }

    fn apply_1q<F>(state: &mut [Complex64], n: usize, q: usize, f: F)
    where
        F: Fn(Complex64, Complex64) -> (Complex64, Complex64),
    {
        let dim = 1 << n;
        for i in 0..dim {
            if (i >> q) & 1 == 0 {
                let j = i | (1 << q);
                let (u, v) = f(state[i], state[j]);
                state[i] = u;
                state[j] = v;
            }
        }
    }

    pub fn apply_instruction(state: &mut [Complex64], n: usize, inst: &str) {
        let clean = inst.trim().trim_end_matches(';').trim();
        let dim = 1 << n;

        if clean.starts_with("h ") {
            if let Some(q) = Self::parse_qubit_idx(&clean[2..]) {
                if q < n {
                    let inv = 1.0 / std::f64::consts::SQRT_2;
                    Self::apply_1q(state, n, q, |u, v| ((u + v) * inv, (u - v) * inv));
                }
            }
        } else if clean.starts_with("x ") {
            if let Some(q) = Self::parse_qubit_idx(&clean[2..]) {
                if q < n {
                    Self::apply_1q(state, n, q, |u, v| (v, u));
                }
            }
        } else if clean.starts_with("y ") {
            if let Some(q) = Self::parse_qubit_idx(&clean[2..]) {
                if q < n {
                    let iu = Complex64::new(0.0, 1.0);
                    Self::apply_1q(state, n, q, |u, v| (-iu * v, iu * u));
                }
            }
        } else if clean.starts_with("z ") {
            if let Some(q) = Self::parse_qubit_idx(&clean[2..]) {
                if q < n {
                    Self::apply_1q(state, n, q, |u, v| (u, -v));
                }
            }
        } else if clean.starts_with("rx(") || clean.starts_with("ry(") || clean.starts_with("rz(") {
            let prefix = if clean.starts_with("rx(") {
                "rx("
            } else if clean.starts_with("ry(") {
                "ry("
            } else {
                "rz("
            };
            if let Some((angle, q)) = Self::parse_rot_gate(clean, prefix) {
                if q < n {
                    let half = angle / 2.0;
                    match prefix {
                        "rx(" => {
                            let c = Complex64::new(half.cos(), 0.0);
                            let s_i = Complex64::new(0.0, -half.sin());
                            Self::apply_1q(state, n, q, |u, v| (u * c + v * s_i, u * s_i + v * c));
                        }
                        "ry(" => {
                            let c = half.cos();
                            let s = half.sin();
                            Self::apply_1q(state, n, q, |u, v| (u * c - v * s, u * s + v * c));
                        }
                        "rz(" => {
                            let p_neg = Complex64::new(half.cos(), -half.sin());
                            let p_pos = Complex64::new(half.cos(), half.sin());
                            Self::apply_1q(state, n, q, |u, v| (u * p_neg, v * p_pos));
                        }
                        _ => {}
                    }
                }
            }
        } else if clean.starts_with("cx ") || clean.starts_with("cnot ") {
            let rem = if clean.starts_with("cx ") {
                &clean[3..]
            } else {
                &clean[5..]
            };
            if let Some((c, t)) = Self::parse_two_qubits(rem) {
                if c < n && t < n && c != t {
                    for i in 0..dim {
                        if ((i >> c) & 1 == 1) && ((i >> t) & 1 == 0) {
                            state.swap(i, i | (1 << t));
                        }
                    }
                }
            }
        } else if clean.starts_with("cz ") {
            if let Some((c, t)) = Self::parse_two_qubits(&clean[3..]) {
                if c < n && t < n && c != t {
                    for i in 0..dim {
                        if ((i >> c) & 1 == 1) && ((i >> t) & 1 == 1) {
                            state[i] = -state[i];
                        }
                    }
                }
            }
        } else if clean.starts_with("swap ") {
            if let Some((a, b)) = Self::parse_two_qubits(&clean[5..]) {
                if a < n && b < n && a != b {
                    for i in 0..dim {
                        if ((i >> a) & 1 == 1) && ((i >> b) & 1 == 0) {
                            let j = i ^ (1 << a) ^ (1 << b);
                            state.swap(i, j);
                        }
                    }
                }
            }
        }
    }

    fn parse_two_qubits(rem: &str) -> Option<(usize, usize)> {
        let parts: Vec<&str> = rem.split(',').collect();
        if parts.len() == 2 {
            let q1 = Self::parse_qubit_idx(parts[0])?;
            let q2 = Self::parse_qubit_idx(parts[1])?;
            Some((q1, q2))
        } else {
            None
        }
    }

    fn parse_rot_gate(line: &str, prefix: &str) -> Option<(f64, usize)> {
        let after = line.strip_prefix(prefix)?;
        let close = after.find(')')?;
        let angle = Self::parse_angle(&after[..close]);
        let q = Self::parse_qubit_idx(after[close + 1..].trim())?;
        Some((angle, q))
    }

    fn parse_angle(token: &str) -> f64 {
        let s = token.trim().to_lowercase().replace(' ', "");
        let pi = std::f64::consts::PI;
        if s.contains("pi") {
            if s == "pi" {
                pi
            } else if s == "-pi" {
                -pi
            } else if let Some(denom) = s.strip_prefix("pi/") {
                denom.parse::<f64>().map(|d| pi / d).unwrap_or(pi)
            } else if let Some(denom) = s.strip_prefix("-pi/") {
                denom.parse::<f64>().map(|d| -pi / d).unwrap_or(-pi)
            } else if s.contains("*pi/") {
                let parts: Vec<&str> = s.split("*pi/").collect();
                let num = parts.get(0).and_then(|v| v.parse::<f64>().ok()).unwrap_or(1.0);
                let den = parts.get(1).and_then(|v| v.parse::<f64>().ok()).unwrap_or(1.0);
                (num * pi) / den
            } else if s.contains("*pi") {
                s.replace("*pi", "").parse::<f64>().unwrap_or(1.0) * pi
            } else {
                pi
            }
        } else {
            s.parse::<f64>().unwrap_or(0.0)
        }
    }

    fn parse_qubit_idx(token: &str) -> Option<usize> {
        let start = token.find('[')?;
        let end = token.find(']')?;
        token[start + 1..end].trim().parse::<usize>().ok()
    }
}
