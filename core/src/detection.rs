use std::fs::{File, read_to_string};
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

use crate::CKLB;
use crate::{Format, XylokToml};

/// Detect the format of STIG the user provided given a path.
/// If its not a STIG, that is still returned as an error.
pub fn detect_stig_format<P: AsRef<Path>>(path: P) -> Option<Format> {
    match path.as_ref().extension().and_then(|os_str| os_str.to_str()) {
        // Attempt to deserialize the toml as a Xylok Benchmark.
        Some("toml") => {
            let toml_str = read_to_string(path).ok()?;

            let xylok_toml: XylokToml = toml::from_str(&toml_str).ok()?;

            Some(Format::Xylok(xylok_toml))
        }

        // Look in the xml for version keywords to detect its version.
        Some("xml") => {
            let xml = std::fs::read_to_string(path.as_ref()).ok()?;

            detect_xccdf_str(&xml)
        }

        // Unzip the file, and then check for key words in the xml.
        Some("zip") => detect_xccdf_in_zip(path.as_ref()),

        // Just attempt to parse the data, without looking for a keyword.
        Some("ckl") => {
            let xml = std::fs::read_to_string(path.as_ref()).ok()?;

            Some(Format::CKL(xml))
        }

        Some("cklb") => {
            let json_str = read_to_string(path).ok()?;

            let cklb_benchmark: CKLB = serde_json::from_str(&json_str).ok()?;

            Some(Format::CKLB(cklb_benchmark))
        }

        _ => None,
    }
}

/// Detect the XCCDF version from a raw XML string.
/// For XccdfV1_1/V1_2 the string is moved into the variant so the caller does not
/// need to re-read (or re-unzip) the file.
fn detect_xccdf_str(xml: &str) -> Option<Format> {
    let xml_tree = roxmltree::Document::parse(xml).ok()?;

    let str = xml_tree
        .descendants()
        .find(|node| node.tag_name().name() == "Benchmark")?
        .tag_name()
        .namespace()
        .unwrap_or("");

    if str.contains("checklists.nist.gov/xccdf/1.2") {
        Some(Format::XccdfV1_2)
    } else if str.contains("checklists.nist.gov/xccdf/1.1") {
        Some(Format::XccdfV1_1(xml.to_owned()))
    } else {
        None
    }
}

/// See if the input zip contains an XCCDF STIG.
fn detect_xccdf_in_zip(path: &Path) -> Option<Format> {
    let mut archive = ZipArchive::new(File::open(path).ok()?).ok()?;

    let xml_names: Vec<String> = archive
        .file_names()
        .filter(|name| name.ends_with(".xml"))
        .map(|name| name.to_owned())
        .collect();

    for name in &xml_names {
        let mut entry = match archive.by_name(name) {
            Ok(e) => e,
            Err(_) => continue,
        };

        let mut xml = String::new();
        if entry.read_to_string(&mut xml).is_err() {
            continue;
        }

        if let Some(format) = detect_xccdf_str(&xml) {
            return Some(format);
        }
    }

    None
}

#[test]
fn test_xccdfv1_1_detection() {
    let format = detect_stig_format("../test_assets/U_RHEL_8_V2R6_STIG.zip");
    assert!(matches!(format, Some(Format::XccdfV1_1(_))));
}

#[test]
fn test_xccdfv1_2_detection() {
    let format =
        detect_stig_format("../test_assets/U_MS_Windows_10_V3R7_STIG_SCAP_1-3_Benchmark.zip");
    assert!(matches!(format, Some(Format::XccdfV1_2)));
}

#[test]
fn test_xylok_detection() {
    let format = detect_stig_format("../test_assets/packed.toml");
    assert!(matches!(format, Some(Format::Xylok(_))));
}
