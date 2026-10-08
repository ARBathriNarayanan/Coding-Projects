use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pre-configured Quantum Error Correction (QEC) Code Presets
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QECCodePreset {
    Repetition3QubitBitFlip, // 3 physical -> 1 logical qubit
    ErrorDetecting4Qubit,    // 4 physical -> 2 logical qubits (arXiv:2412.04791v2)
    Perfect5Qubit,           // 5 physical -> 1 logical qubit
    Steane7Qubit,            // 7 physical -> 1 logical qubit CSS code
    Custom,                  // User-defined physical (N) -> logical (K) mapping
}

impl QECCodePreset {
    pub fn name(&self) -> &'static str {
        match self {
            QECCodePreset::Repetition3QubitBitFlip => "3-Qubit Bit-Flip Repetition Code",
            QECCodePreset::ErrorDetecting4Qubit => "4-Qubit Error-Detecting Code (Deutsch-Jozsa)",
            QECCodePreset::Perfect5Qubit => "5-Qubit Perfect Code",
            QECCodePreset::Steane7Qubit => "7-Qubit Steane CSS Code",
            QECCodePreset::Custom => "Custom N-Qubit Code",
        }
    }

    pub fn physical_qubits(&self) -> usize {
        match self {
            QECCodePreset::Repetition3QubitBitFlip => 3,
            QECCodePreset::ErrorDetecting4Qubit => 4,
            QECCodePreset::Perfect5Qubit => 5,
            QECCodePreset::Steane7Qubit => 7,
            QECCodePreset::Custom => 3,
        }
    }

    #[allow(dead_code)]
    pub fn logical_qubits(&self) -> usize {
        match self {
            QECCodePreset::Repetition3QubitBitFlip => 1,
            QECCodePreset::ErrorDetecting4Qubit => 2,
            QECCodePreset::Perfect5Qubit => 1,
            QECCodePreset::Steane7Qubit => 1,
            QECCodePreset::Custom => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DecodeMode {
    ActiveSyndromeCorrection,
    PostSelectionDiscarding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSyndromeRule {
    pub syndrome_bits: String,
    pub flip_qubit_idx: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QECCodeConfig {
    pub preset: QECCodePreset,
    pub decode_mode: DecodeMode,
    pub enable_auto_majority_vote: bool,
    pub custom_rules: Vec<CustomSyndromeRule>,
    pub custom_script_code: String,
}

impl Default for QECCodeConfig {
    fn default() -> Self {
        let default_rules = vec![
            CustomSyndromeRule { syndrome_bits: "10".to_string(), flip_qubit_idx: 0 },
            CustomSyndromeRule { syndrome_bits: "11".to_string(), flip_qubit_idx: 1 },
            CustomSyndromeRule { syndrome_bits: "01".to_string(), flip_qubit_idx: 2 },
        ];

        Self {
            preset: QECCodePreset::Repetition3QubitBitFlip,
            decode_mode: DecodeMode::ActiveSyndromeCorrection,
            enable_auto_majority_vote: true,
            custom_rules: default_rules,
            custom_script_code: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QECPipelineResult {
    pub corrected_physical_counts: HashMap<String, usize>,
    pub logical_counts: HashMap<String, usize>,
    #[allow(dead_code)]
    pub syndrome_histogram: HashMap<String, usize>,
    pub total_input_shots: usize,
    pub retained_shots: usize,
    pub discarded_shots: usize,
    pub retention_rate: f64,
    #[allow(dead_code)]
    pub logical_error_rate: f64,
}

pub struct Postprocessor;

impl Postprocessor {
    pub fn mitigate_readout_errors_1q(
        raw_counts: &HashMap<String, usize>,
        total_shots: usize,
        p01: f64,
        p10: f64,
    ) -> HashMap<String, f64> {
        if total_shots == 0 {
            return HashMap::new();
        }
        let n0 = *raw_counts.get("0").unwrap_or(&0) as f64 / total_shots as f64;
        let n1 = *raw_counts.get("1").unwrap_or(&0) as f64 / total_shots as f64;

        let det = 1.0 - p10 - p01;
        if det.abs() < 1e-6 {
            let mut res = HashMap::new();
            res.insert("0".to_string(), n0);
            res.insert("1".to_string(), n1);
            return res;
        }

        let p0_ideal = ((1.0 - p01) * n0 - p01 * n1) / det;
        let p1_ideal = (-p10 * n0 + (1.0 - p10) * n1) / det;

        let p0_clamped = p0_ideal.clamp(0.0, 1.0);
        let p1_clamped = p1_ideal.clamp(0.0, 1.0);
        let sum = (p0_clamped + p1_clamped).max(1e-9);

        let mut mitigated = HashMap::new();
        mitigated.insert("0".to_string(), p0_clamped / sum);
        mitigated.insert("1".to_string(), p1_clamped / sum);
        mitigated
    }

    pub fn process_qec_pipeline(
        raw_counts: &HashMap<String, usize>,
        config: &QECCodeConfig,
    ) -> QECPipelineResult {
        let mut corrected_physical: HashMap<String, usize> = HashMap::new();
        let mut logical_counts: HashMap<String, usize> = HashMap::new();
        let mut syndrome_histogram: HashMap<String, usize> = HashMap::new();

        let mut total_input_shots = 0;
        let mut retained_shots = 0;
        let mut discarded_shots = 0;
        let mut uncorrected_logical_errors = 0;

        for (bitstring, &count) in raw_counts {
            total_input_shots += count;

            match config.preset {
                QECCodePreset::Repetition3QubitBitFlip => {
                    if bitstring.len() != 3 {
                        *corrected_physical.entry(bitstring.clone()).or_insert(0) += count;
                        retained_shots += count;
                        continue;
                    }

                    let bits: Vec<u8> = bitstring.chars().map(|c| if c == '1' { 1 } else { 0 }).collect();
                    let (s1, s2) = (bits[0] ^ bits[1], bits[1] ^ bits[2]);
                    let syndrome_str = format!("{}{}", s1, s2);
                    *syndrome_histogram.entry(syndrome_str.clone()).or_insert(0) += count;

                    if config.decode_mode == DecodeMode::PostSelectionDiscarding {
                        if syndrome_str != "00" {
                            discarded_shots += count;
                            continue;
                        }
                    }

                    let mut corrected_bits = bits.clone();
                    if config.decode_mode == DecodeMode::ActiveSyndromeCorrection {
                        if config.enable_auto_majority_vote {
                            match syndrome_str.as_str() {
                                "10" => corrected_bits[0] ^= 1,
                                "11" => corrected_bits[1] ^= 1,
                                "01" => corrected_bits[2] ^= 1,
                                _ => {}
                            }
                        } else {
                            for rule in &config.custom_rules {
                                if rule.syndrome_bits == syndrome_str && rule.flip_qubit_idx < corrected_bits.len() {
                                    corrected_bits[rule.flip_qubit_idx] ^= 1;
                                    break;
                                }
                            }
                        }
                    }

                    retained_shots += count;
                    let corr_str: String = corrected_bits.iter().map(|b| if *b == 1 { '1' } else { '0' }).collect();
                    *corrected_physical.entry(corr_str.clone()).or_insert(0) += count;

                    let logical_bit = if corr_str == "111" { "1" } else { "0" };
                    *logical_counts.entry(logical_bit.to_string()).or_insert(0) += count;

                    if corr_str != "000" && corr_str != "111" {
                        uncorrected_logical_errors += count;
                    }
                }

                QECCodePreset::ErrorDetecting4Qubit => {
                    if bitstring.len() != 4 {
                        *corrected_physical.entry(bitstring.clone()).or_insert(0) += count;
                        retained_shots += count;
                        continue;
                    }

                    let bits: Vec<u8> = bitstring.chars().map(|c| if c == '1' { 1 } else { 0 }).collect();
                    let parity = (bits[0] + bits[1] + bits[2] + bits[3]) % 2;
                    let syndrome_str = format!("P{}", parity);
                    *syndrome_histogram.entry(syndrome_str.clone()).or_insert(0) += count;

                    if config.decode_mode == DecodeMode::PostSelectionDiscarding {
                        if parity != 0 {
                            discarded_shots += count;
                            continue;
                        }
                    }

                    let mut corrected_bits = bits.clone();
                    if config.decode_mode == DecodeMode::ActiveSyndromeCorrection {
                        if !config.enable_auto_majority_vote {
                            for rule in &config.custom_rules {
                                if rule.syndrome_bits == syndrome_str && rule.flip_qubit_idx < corrected_bits.len() {
                                    corrected_bits[rule.flip_qubit_idx] ^= 1;
                                    break;
                                }
                            }
                        }
                    }

                    retained_shots += count;
                    let corr_str: String = corrected_bits.iter().map(|b| if *b == 1 { '1' } else { '0' }).collect();
                    *corrected_physical.entry(corr_str.clone()).or_insert(0) += count;

                    let logical_state = match corr_str.as_str() {
                        "0000" | "1111" => "00",
                        "1100" | "0011" => "01",
                        "1010" | "0101" => "10",
                        "0110" | "1001" => "11",
                        _ => "Unmapped",
                    };
                    *logical_counts.entry(logical_state.to_string()).or_insert(0) += count;
                }

                _ => {
                    let mut corrected_bits: Vec<u8> = bitstring.chars().map(|c| if c == '1' { 1 } else { 0 }).collect();
                    let syndrome_str = if corrected_bits.len() >= 2 {
                        format!("{}{}", corrected_bits[0] ^ corrected_bits[1], corrected_bits[1] ^ corrected_bits.last().copied().unwrap_or(0))
                    } else {
                        "0".to_string()
                    };
                    *syndrome_histogram.entry(syndrome_str.clone()).or_insert(0) += count;

                    if config.decode_mode == DecodeMode::ActiveSyndromeCorrection && !config.enable_auto_majority_vote {
                        for rule in &config.custom_rules {
                            if rule.syndrome_bits == syndrome_str && rule.flip_qubit_idx < corrected_bits.len() {
                                corrected_bits[rule.flip_qubit_idx] ^= 1;
                                break;
                            }
                        }
                    }

                    retained_shots += count;
                    let corr_str: String = corrected_bits.iter().map(|b| if *b == 1 { '1' } else { '0' }).collect();
                    *corrected_physical.entry(corr_str.clone()).or_insert(0) += count;
                    let logical_str = corr_str.chars().next().unwrap_or('0').to_string();
                    *logical_counts.entry(logical_str).or_insert(0) += count;
                }
            }
        }

        let retention_rate = if total_input_shots > 0 {
            retained_shots as f64 / total_input_shots as f64
        } else {
            0.0
        };

        let logical_error_rate = if retained_shots > 0 {
            uncorrected_logical_errors as f64 / retained_shots as f64
        } else {
            0.0
        };

        QECPipelineResult {
            corrected_physical_counts: corrected_physical,
            logical_counts,
            syndrome_histogram,
            total_input_shots,
            retained_shots,
            discarded_shots,
            retention_rate,
            logical_error_rate,
        }
    }

    pub fn extrapolate_zero_noise(scale_factors: &[f64], values: &[f64]) -> f64 {
        if scale_factors.len() < 2 || scale_factors.len() != values.len() {
            return values.first().copied().unwrap_or(0.0);
        }

        let n = scale_factors.len() as f64;
        let sum_x: f64 = scale_factors.iter().sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = scale_factors.iter().zip(values.iter()).map(|(x, y)| x * y).sum();
        let sum_x2: f64 = scale_factors.iter().map(|x| x * x).sum();

        let denom = n * sum_x2 - sum_x * sum_x;
        if denom.abs() < 1e-9 {
            return sum_y / n;
        }

        let slope = (n * sum_xy - sum_x * sum_y) / denom;
        (sum_y - slope * sum_x) / n
    }

    pub fn generate_python_postprocessing_script(config: &QECCodeConfig) -> String {
        format!(
            r#"# Auto-Generated In-Engine Postprocessing Script
# Active Preset: {} ({})

def in_engine_postprocess(raw_shots):
    retained_shots = {{}}
    discarded_count = 0
    
    for bitstring, count in raw_shots.items():
        if len(bitstring) == {}:
            retained_shots[bitstring] = count
        else:
            discarded_count += count
            
    return retained_shots, discarded_count

raw_shots = {{"0000": 1950, "1111": 1920, "1000": 110, "0110": 60}}
retained, discarded = in_engine_postprocess(raw_shots)
print("Retained Shots :", retained)
print("Discarded Shots:", discarded)
"#,
            config.preset.name(),
            if config.decode_mode == DecodeMode::ActiveSyndromeCorrection { "Active Correction" } else { "Post-Selection" },
            config.preset.physical_qubits()
        )
    }
}
