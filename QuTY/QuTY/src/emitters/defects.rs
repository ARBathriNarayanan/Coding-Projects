use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SiliconDefect {
    GCenter,   // 1278 nm
    TCenter,   // 1326 nm
    CiCenter,  // 1280 nm
    Al1Center, // 1300 nm
    WCenter,   // 1218 nm
    Er3Plus,   // 1536 nm
}

impl SiliconDefect {
    pub fn name(&self) -> &'static str {
        match self {
            SiliconDefect::GCenter => "G-Center (Carbon Pair + Interstitial)",
            SiliconDefect::TCenter => "T-Center (Carbon-Hydrogen Complex)",
            SiliconDefect::CiCenter => "C_i-Center (Interstitial Carbon)",
            SiliconDefect::Al1Center => "Al_1-Center (Aluminum-Related Defect)",
            SiliconDefect::WCenter => "W-Center (Tri-Interstitial Cluster)",
            SiliconDefect::Er3Plus => "Er^3+ Erbium Ion in Silicon",
        }
    }

    pub fn wavelength_nm(&self) -> f64 {
        match self {
            SiliconDefect::GCenter => 1278.4,
            SiliconDefect::TCenter => 1326.0,
            SiliconDefect::CiCenter => 1280.0,
            SiliconDefect::Al1Center => 1300.2,
            SiliconDefect::WCenter => 1218.0,
            SiliconDefect::Er3Plus => 1536.0,
        }
    }

    pub fn zpl_fraction(&self) -> f64 {
        match self {
            SiliconDefect::GCenter => 0.15,
            SiliconDefect::TCenter => 0.23,
            SiliconDefect::CiCenter => 0.10,
            SiliconDefect::Al1Center => 0.30,
            SiliconDefect::WCenter => 0.08,
            SiliconDefect::Er3Plus => 0.90,
        }
    }

    pub fn lifetime_ns(&self) -> f64 {
        match self {
            SiliconDefect::GCenter => 35.0,
            SiliconDefect::TCenter => 940.0,
            SiliconDefect::CiCenter => 25.0,
            SiliconDefect::Al1Center => 45.0,
            SiliconDefect::WCenter => 20.0,
            SiliconDefect::Er3Plus => 1_000_000.0,
        }
    }

    pub fn spin_t2_us(&self) -> f64 {
        match self {
            SiliconDefect::GCenter => 2.0,
            SiliconDefect::TCenter => 2100.0,
            SiliconDefect::CiCenter => 1.5,
            SiliconDefect::Al1Center => 10.0,
            SiliconDefect::WCenter => 0.5,
            SiliconDefect::Er3Plus => 10000.0,
        }
    }
}

