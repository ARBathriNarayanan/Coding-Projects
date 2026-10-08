use eframe::egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateType {
    H,
    X,
    Y,
    Z,
    Rx,
    Ry,
    Rz,
    Cnot,
    Cz,
    Swap,
    Measure,
}

impl GateType {
    pub fn label(&self) -> &'static str {
        match self {
            GateType::H => "H",
            GateType::X => "X",
            GateType::Y => "Y",
            GateType::Z => "Z",
            GateType::Rx => "Rx",
            GateType::Ry => "Ry",
            GateType::Rz => "Rz",
            GateType::Cnot => "CX",
            GateType::Cz => "CZ",
            GateType::Swap => "SWAP",
            GateType::Measure => "M",
        }
    }

    pub fn color(&self) -> Color32 {
        match self {
            GateType::H => Color32::from_rgb(41, 128, 185),
            GateType::X => Color32::from_rgb(192, 57, 43),
            GateType::Y => Color32::from_rgb(142, 68, 173),
            GateType::Z => Color32::from_rgb(39, 174, 96),
            GateType::Rx => Color32::from_rgb(211, 84, 0),
            GateType::Ry => Color32::from_rgb(125, 60, 152),
            GateType::Rz => Color32::from_rgb(22, 160, 133),
            GateType::Cnot => Color32::from_rgb(230, 126, 34),
            GateType::Cz => Color32::from_rgb(52, 73, 94),
            GateType::Swap => Color32::from_rgb(180, 130, 20),
            GateType::Measure => Color32::from_rgb(127, 140, 141),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GridCell {
    pub gate: Option<GateType>,
    pub control_qubit: Option<usize>,
    pub param: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlgorithmPreset {
    Ghz3Qubit,
    BellState,
    Grover2Qubit,
    DeutschJozsaBalanced,
    QuantumTeleportation,
}

impl AlgorithmPreset {
    pub fn name(&self) -> &'static str {
        match self {
            AlgorithmPreset::Ghz3Qubit => "3-Qubit GHZ Entanglement (|000> + |111>)",
            AlgorithmPreset::BellState => "2-Qubit Bell State Φ+ (|00> + |11>)",
            AlgorithmPreset::Grover2Qubit => "Grover's Search 2-Qubit (Marked |11>)",
            AlgorithmPreset::DeutschJozsaBalanced => "Deutsch-Jozsa (Balanced Oracle f(x)=x)",
            AlgorithmPreset::QuantumTeleportation => "Quantum Teleportation (Alice -> Bob)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitGrid {
    pub num_qubits: usize,
    pub num_steps: usize,
    pub grid: Vec<Vec<GridCell>>,
}

impl Default for CircuitGrid {
    fn default() -> Self {
        Self::from_preset(AlgorithmPreset::Ghz3Qubit)
    }
}

impl std::ops::Index<usize> for CircuitGrid {
    type Output = Vec<GridCell>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.grid[index]
    }
}

impl std::ops::IndexMut<usize> for CircuitGrid {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.grid[index]
    }
}

impl CircuitGrid {
    pub fn from_preset(preset: AlgorithmPreset) -> Self {
        match preset {
            AlgorithmPreset::Ghz3Qubit => {
                let num_qubits = 3;
                let num_steps = 5;
                let mut grid = vec![vec![GridCell::default(); num_steps]; num_qubits];

                grid[0][0].gate = Some(GateType::H);

                grid[1][1].gate = Some(GateType::Cnot);
                grid[1][1].control_qubit = Some(0);

                grid[2][2].gate = Some(GateType::Cnot);
                grid[2][2].control_qubit = Some(1);

                for q in 0..3 {
                    grid[q][3].gate = Some(GateType::Measure);
                }

                Self { num_qubits, num_steps, grid }
            }
            AlgorithmPreset::BellState => {
                let num_qubits = 2;
                let num_steps = 4;
                let mut grid = vec![vec![GridCell::default(); num_steps]; num_qubits];

                grid[0][0].gate = Some(GateType::H);

                grid[1][1].gate = Some(GateType::Cnot);
                grid[1][1].control_qubit = Some(0);

                grid[0][2].gate = Some(GateType::Measure);
                grid[1][2].gate = Some(GateType::Measure);

                Self { num_qubits, num_steps, grid }
            }
            AlgorithmPreset::Grover2Qubit => {
                let num_qubits = 2;
                let num_steps = 9;
                let mut grid = vec![vec![GridCell::default(); num_steps]; num_qubits];

                // 1. Equal Superposition
                grid[0][0].gate = Some(GateType::H);
                grid[1][0].gate = Some(GateType::H);

                // 2. Oracle for |11>: CZ gate marks |11> with -1 phase
                grid[1][1].gate = Some(GateType::Cz);
                grid[1][1].control_qubit = Some(0);

                // 3. Diffusion operator: H -> X -> CZ -> X -> H
                grid[0][2].gate = Some(GateType::H);
                grid[1][2].gate = Some(GateType::H);

                grid[0][3].gate = Some(GateType::X);
                grid[1][3].gate = Some(GateType::X);

                grid[1][4].gate = Some(GateType::Cz);
                grid[1][4].control_qubit = Some(0);

                grid[0][5].gate = Some(GateType::X);
                grid[1][5].gate = Some(GateType::X);

                grid[0][6].gate = Some(GateType::H);
                grid[1][6].gate = Some(GateType::H);

                // 4. Measure
                grid[0][7].gate = Some(GateType::Measure);
                grid[1][7].gate = Some(GateType::Measure);

                Self { num_qubits, num_steps, grid }
            }
            AlgorithmPreset::DeutschJozsaBalanced => {
                let num_qubits = 2;
                let num_steps = 6;
                let mut grid = vec![vec![GridCell::default(); num_steps]; num_qubits];

                // Ancilla q[1] initialized to |1>
                grid[1][0].gate = Some(GateType::X);

                // Superposition on input and ancilla (|->)
                grid[0][1].gate = Some(GateType::H);
                grid[1][1].gate = Some(GateType::H);

                // Balanced oracle f(x) = x -> CNOT(0 -> 1)
                grid[1][2].gate = Some(GateType::Cnot);
                grid[1][2].control_qubit = Some(0);

                // Interference on input
                grid[0][3].gate = Some(GateType::H);

                // Measurement: input q[0] gives |1> indicating balanced!
                grid[0][4].gate = Some(GateType::Measure);
                grid[1][4].gate = Some(GateType::Measure);

                Self { num_qubits, num_steps, grid }
            }
            AlgorithmPreset::QuantumTeleportation => {
                let num_qubits = 3;
                let num_steps = 7;
                let mut grid = vec![vec![GridCell::default(); num_steps]; num_qubits];

                // Prepare state on q[0] to teleport (|+)
                grid[0][0].gate = Some(GateType::H);

                // Entangle Bell pair between Alice q[1] and Bob q[2]
                grid[1][1].gate = Some(GateType::H);
                grid[2][2].gate = Some(GateType::Cnot);
                grid[2][2].control_qubit = Some(1);

                // Alice Bell measurement: CNOT(q0 -> q1), H(q0)
                grid[1][3].gate = Some(GateType::Cnot);
                grid[1][3].control_qubit = Some(0);

                grid[0][4].gate = Some(GateType::H);

                // Measurement
                for q in 0..3 {
                    grid[q][5].gate = Some(GateType::Measure);
                }

                Self { num_qubits, num_steps, grid }
            }
        }
    }

    pub fn add_qubit(&mut self) {
        if self.num_qubits < 8 {
            self.num_qubits += 1;
            self.grid.push(vec![GridCell::default(); self.num_steps]);
        }
    }

    pub fn remove_qubit(&mut self) {
        if self.num_qubits > 1 {
            self.num_qubits -= 1;
            self.grid.pop();
            for q in 0..self.num_qubits {
                for s in 0..self.num_steps {
                    if let Some(ctrl) = self.grid[q][s].control_qubit {
                        if ctrl >= self.num_qubits {
                            self.grid[q][s].control_qubit = Some(0);
                        }
                    }
                }
            }
        }
    }

    pub fn to_qasm(&self) -> String {
        let mut qasm = format!(
            "OPENQASM 2.0;\ninclude \"qelib1.inc\";\nqreg q[{}];\ncreg c[{}];\n",
            self.num_qubits, self.num_qubits
        );

        for step in 0..self.num_steps {
            for q in 0..self.num_qubits {
                let cell = &self.grid[q][step];
                if let Some(gate) = cell.gate {
                    match gate {
                        GateType::H => qasm.push_str(&format!("h q[{}];\n", q)),
                        GateType::X => qasm.push_str(&format!("x q[{}];\n", q)),
                        GateType::Y => qasm.push_str(&format!("y q[{}];\n", q)),
                        GateType::Z => qasm.push_str(&format!("z q[{}];\n", q)),
                        GateType::Rx => {
                            let th = cell.param.unwrap_or(std::f64::consts::FRAC_PI_2);
                            qasm.push_str(&format!("rx({:.4}) q[{}];\n", th, q));
                        }
                        GateType::Ry => {
                            let th = cell.param.unwrap_or(std::f64::consts::FRAC_PI_2);
                            qasm.push_str(&format!("ry({:.4}) q[{}];\n", th, q));
                        }
                        GateType::Rz => {
                            let th = cell.param.unwrap_or(std::f64::consts::FRAC_PI_2);
                            qasm.push_str(&format!("rz({:.4}) q[{}];\n", th, q));
                        }
                        GateType::Cnot => {
                            let mut ctrl = cell.control_qubit.unwrap_or(if q > 0 { q - 1 } else { 1 });
                            if ctrl >= self.num_qubits || ctrl == q {
                                ctrl = if q > 0 { 0 } else { 1.min(self.num_qubits.saturating_sub(1)) };
                            }
                            if ctrl != q {
                                qasm.push_str(&format!("cx q[{}],q[{}];\n", ctrl, q));
                            }
                        }
                        GateType::Cz => {
                            let mut ctrl = cell.control_qubit.unwrap_or(if q > 0 { q - 1 } else { 1 });
                            if ctrl >= self.num_qubits || ctrl == q {
                                ctrl = if q > 0 { 0 } else { 1.min(self.num_qubits.saturating_sub(1)) };
                            }
                            if ctrl != q {
                                qasm.push_str(&format!("cz q[{}],q[{}];\n", ctrl, q));
                            }
                        }
                        GateType::Swap => {
                            let partner = cell.control_qubit.unwrap_or(if q > 0 { q - 1 } else { 1 });
                            if partner < self.num_qubits && q < partner {
                                qasm.push_str(&format!("swap q[{}],q[{}];\n", q, partner));
                            }
                        }
                        GateType::Measure => {
                            qasm.push_str(&format!("measure q[{}] -> c[{}];\n", q, q));
                        }
                    }
                }
            }
        }
        qasm
    }

    pub fn from_qasm(qasm: &str) -> Result<Self, String> {
        let mut num_qubits = 3;
        for line in qasm.lines().map(str::trim) {
            if line.starts_with("qreg") {
                if let (Some(s), Some(e)) = (line.find('['), line.find(']')) {
                    if let Ok(n) = line[s + 1..e].parse::<usize>() {
                        num_qubits = n.clamp(1, 8);
                        break;
                    }
                }
            }
        }

        let num_steps = 10;
        let mut grid = vec![vec![GridCell::default(); num_steps]; num_qubits];
        let mut qubit_timeline = vec![0; num_qubits];

        for line in qasm.lines().map(|l| l.trim().trim_end_matches(';').trim()) {
            if line.is_empty()
                || line.starts_with("//")
                || line.starts_with("OPENQASM")
                || line.starts_with("include")
                || line.starts_with("qreg")
                || line.starts_with("creg")
            {
                continue;
            }

            let single_op = if line.starts_with("h ") {
                Some((GateType::H, &line[2..]))
            } else if line.starts_with("x ") {
                Some((GateType::X, &line[2..]))
            } else if line.starts_with("y ") {
                Some((GateType::Y, &line[2..]))
            } else if line.starts_with("z ") {
                Some((GateType::Z, &line[2..]))
            } else if line.starts_with("measure ") {
                Some((GateType::Measure, &line[8..]))
            } else {
                None
            };

            if let Some((gate, target)) = single_op {
                if let Some(q) = Self::parse_qubit_idx(target) {
                    if q < num_qubits {
                        let st = qubit_timeline[q].min(num_steps - 1);
                        grid[q][st].gate = Some(gate);
                        qubit_timeline[q] = (st + 1).min(num_steps);
                    }
                }
                continue;
            }

            let mut is_rot = false;
            for (prefix, gate) in [("rx(", GateType::Rx), ("ry(", GateType::Ry), ("rz(", GateType::Rz)] {
                if line.starts_with(prefix) {
                    if let Some((angle, q)) = Self::parse_rot_gate(line, prefix) {
                        if q < num_qubits {
                            let st = qubit_timeline[q].min(num_steps - 1);
                            grid[q][st].gate = Some(gate);
                            grid[q][st].param = Some(angle);
                            qubit_timeline[q] = (st + 1).min(num_steps);
                        }
                    }
                    is_rot = true;
                    break;
                }
            }
            if is_rot {
                continue;
            }

            let two_q = if line.starts_with("cx ") {
                Some((GateType::Cnot, &line[3..]))
            } else if line.starts_with("cnot ") {
                Some((GateType::Cnot, &line[5..]))
            } else if line.starts_with("cz ") {
                Some((GateType::Cz, &line[3..]))
            } else if line.starts_with("swap ") {
                Some((GateType::Swap, &line[5..]))
            } else {
                None
            };

            if let Some((gate, rem)) = two_q {
                let parts: Vec<&str> = rem.split(',').collect();
                if parts.len() == 2 {
                    if let (Some(a), Some(b)) = (Self::parse_qubit_idx(parts[0]), Self::parse_qubit_idx(parts[1])) {
                        if a < num_qubits && b < num_qubits && a != b {
                            let st = qubit_timeline[a].max(qubit_timeline[b]).min(num_steps - 1);
                            if gate == GateType::Swap {
                                grid[a][st] = GridCell { gate: Some(GateType::Swap), control_qubit: Some(b), param: None };
                                grid[b][st] = GridCell { gate: Some(GateType::Swap), control_qubit: Some(a), param: None };
                            } else {
                                grid[b][st] = GridCell { gate: Some(gate), control_qubit: Some(a), param: None };
                            }
                            qubit_timeline[a] = (st + 1).min(num_steps);
                            qubit_timeline[b] = (st + 1).min(num_steps);
                        }
                    }
                }
            }
        }

        Ok(Self {
            num_qubits,
            num_steps,
            grid,
        })
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
