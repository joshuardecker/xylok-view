/// Load ckl and cklb files into a benchmark.
pub mod ckl;
/// Detect the format of a benchmark file, such as xccdf, ckl, cklb, or xylok.
pub mod detection;
/// Load xccdf files into a benchmark.
pub mod xccdf;
/// Load xylok toml files into a benchmark.
pub mod xylok;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

use crate::parse::{
    ckl::{CKLB, CKLStatus},
    xylok::XylokToml,
};

/// If file format ever changes, this number will to.
/// It will make deserialization of old format fail.
const CACHE_VERSION: u16 = 2;

/// The overarching benchmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Benchmark {
    pub id: String,
    pub title: String,

    pub rules: BTreeMap<String, Rule>,

    cache_version: u16,
}

/// Each check / rule of a benchmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub group_id: String,
    pub rule_id: String,
    pub stig_id: Option<String>,
    pub severity: Severity,
    pub title: String,
    pub vuln_discussion: String,
    pub check_text: String,
    pub fix_text: String,
    pub cci_refs: Option<Vec<String>>,
    pub false_positives: Option<String>,
    pub false_negatives: Option<String>,
    pub documentable: Option<bool>,

    // If this is derived from a CKL, this field can be filled.
    pub ckl_status: Option<CKLStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(try_from = "u64", into = "u64")]
pub enum Severity {
    Unknown,
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Unknown => "Unknown",
            Severity::VeryLow => "Very Low",
            Severity::Low => "Low",
            Severity::Medium => "Medium",
            Severity::High => "High",
            Severity::VeryHigh => "Very High",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Unknown => write!(f, "Unknown"),
            Severity::VeryLow => write!(f, "Very Low"),
            Severity::Low => write!(f, "Low"),
            Severity::Medium => write!(f, "Medium"),
            Severity::High => write!(f, "High"),
            Severity::VeryHigh => write!(f, "Very High"),
        }
    }
}

impl From<Severity> for u64 {
    fn from(v: Severity) -> u64 {
        match v {
            Severity::Unknown => 0,
            Severity::VeryLow => 1,
            Severity::Low => 2,
            Severity::Medium => 3,
            Severity::High => 4,
            Severity::VeryHigh => 5,
        }
    }
}

impl TryFrom<u64> for Severity {
    type Error = String;
    fn try_from(v: u64) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Severity::Unknown),
            1 => Ok(Severity::VeryLow),
            2 => Ok(Severity::Low),
            3 => Ok(Severity::Medium),
            4 => Ok(Severity::High),
            5 => Ok(Severity::VeryHigh),
            _ => Err(format!("unknown nist_impact value: {v}")),
        }
    }
}

/// Different formats a benchmark can be loaded from.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum Format {
    // Carries the raw XML string so detection and loading share one read.
    XccdfV1_1(String),

    // This version is unsupported, but can easily be detected.
    // Lets detect the version, and pass a useful error message along
    // when the user tries to load this version. This version is only
    // used in SCAP's, which do not contain all necessary information.
    XccdfV1_2,

    // So easy to parse that passing Xylok toml around is easy
    // and saves doing redundant work.
    Xylok(XylokToml),

    CKL(String),

    // So easy to parse that passing around the JSON is easy
    // and saves doing redundant work.
    CKLB(CKLB),
}

impl Benchmark {
    pub fn empty() -> Self {
        Self {
            id: String::new(),
            title: String::new(),

            rules: BTreeMap::new(),

            cache_version: CACHE_VERSION,
        }
    }

    pub fn save(&self) -> Option<()> {
        use std::fs::File;
        use std::fs::create_dir_all;
        use std::io::Write;

        let mut cache_dir = dirs::cache_dir()?;

        // Create the save directory if it does not exist.
        cache_dir.push("stig-view/");
        create_dir_all(&cache_dir).ok()?;

        // Add proper file extensions.
        cache_dir.push(format!("{}.msgpack.zstd", self.id.clone()));

        let mut file = File::create(cache_dir).ok()?;

        // Serialize the benchmark into bytes in the MessagePack format.
        let benchmark_bytes = rmp_serde::to_vec(self).ok()?;

        // Compress it to shrink file size using zstd.
        let compressed = zstd::encode_all(&*benchmark_bytes, 3).ok()?;

        file.write_all(&compressed).ok()?;

        Some(())
    }

    /// Load the given file path as a benchmark.
    /// Does not check if path is a valid benchmark, will just return an error.
    pub fn load<P: AsRef<Path>>(path: P) -> Option<Benchmark> {
        use std::fs::read;

        let compressed_bytes = read(path).ok()?;

        let benchmark_bytes = zstd::decode_all(&*compressed_bytes).ok()?;

        let benchmark: Benchmark = rmp_serde::from_slice(benchmark_bytes.as_slice()).ok()?;

        if benchmark.cache_version != CACHE_VERSION {
            return None;
        }

        Some(benchmark)
    }
}

impl Rule {
    /// Convert this field into a string the GUI can display.
    pub fn documentable_str(&self) -> &'static str {
        match self.documentable {
            Some(true) => "True",
            _ => "False",
        }
    }
}
