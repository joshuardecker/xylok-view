use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::parse::{Benchmark, CACHE_VERSION, Rule, Severity};

/// A struct representing a JSON CKLB.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CKLB {
    pub title: String,
    pub id: String,
    pub stigs: Vec<CKLBStig>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CKLBStig {
    pub stig_name: String,
    pub stig_id: String,
    pub release_info: Option<String>,
    pub rules: Vec<CKLBRule>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CKLBRule {
    pub group_id: String,
    pub rule_id: String,
    pub rule_version: Option<String>,
    pub severity: String,
    pub rule_title: String,
    pub discussion: String,
    pub check_content: String,
    pub fix_text: String,
    pub ccis: Option<Vec<String>>,
    pub false_positives: Option<String>,
    pub false_negatives: Option<String>,
    pub documentable: Option<String>,
    #[serde(default)]
    pub ckl_status: Option<CKLStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CKLStatus {
    NotAFinding,
    Open,
    NotApplicable,
    NotReviewed,
}

impl CKLB {
    pub fn convert(self) -> Vec<Benchmark> {
        self.stigs
            .into_iter()
            .filter_map(|stig| stig.convert())
            .collect()
    }
}

impl CKLBStig {
    /// Convert a single CKLB Benchmark into a benchmark.
    fn convert(self) -> Option<Benchmark> {
        if self.stig_id.is_empty() || self.stig_name.is_empty() {
            return None;
        }

        let mut rules = BTreeMap::new();

        self.rules.into_iter().for_each(|rule| {
            if let Some(rule) = rule.convert() {
                rules.insert(rule.group_id.clone(), rule);
            }
        });

        if rules.is_empty() {
            return None;
        }

        Some(Benchmark {
            id: self.stig_id,
            title: self.stig_name,
            rules,
            cache_version: CACHE_VERSION,
        })
    }
}

impl CKLBRule {
    fn convert(self) -> Option<Rule> {
        Some(Rule {
            group_id: (!self.group_id.is_empty()).then_some(self.group_id)?,
            rule_id: (!self.rule_id.is_empty()).then_some(self.rule_id)?,
            stig_id: self.rule_version.filter(|s| !s.is_empty()),
            severity: parse_severity(&self.severity),
            title: (!self.rule_title.is_empty()).then_some(self.rule_title)?,
            vuln_discussion: (!self.discussion.is_empty()).then_some(self.discussion)?,
            check_text: (!self.check_content.is_empty()).then_some(self.check_content)?,
            fix_text: (!self.fix_text.is_empty()).then_some(self.fix_text)?,
            cci_refs: self.ccis.filter(|v| !v.is_empty()),
            false_positives: self.false_positives.filter(|s| !s.is_empty()),
            false_negatives: self.false_negatives.filter(|s| !s.is_empty()),
            documentable: self.documentable.map(|s| s.trim() == "true"),
            ckl_status: Some(self.ckl_status.unwrap_or(CKLStatus::NotReviewed)),
        })
    }
}

/// Load all benchmarks from a CKL xml string.
pub fn load_ckl(xml: &str) -> Vec<Benchmark> {
    let xml_tree = match roxmltree::Document::parse(xml) {
        Ok(doc) => doc,
        Err(_) => return Vec::new(),
    };

    xml_tree
        .descendants()
        .filter(|node| node.tag_name().name() == "iSTIG")
        .filter_map(parse_istig)
        .collect()
}

fn parse_istig(istig: roxmltree::Node) -> Option<Benchmark> {
    let mut benchmark = Benchmark::empty();

    // Benchmark id and title are stored as SI_DATA key-value pairs under STIG_INFO.
    let stig_info_node = istig
        .children()
        .find(|node| node.tag_name().name() == "STIG_INFO")?;

    for si_data in stig_info_node
        .children()
        .filter(|node| node.tag_name().name() == "SI_DATA")
    {
        let name = si_data
            .children()
            .find(|node| node.tag_name().name() == "SID_NAME")
            .and_then(|node| node.text())
            .unwrap_or("");

        let data = si_data
            .children()
            .find(|node| node.tag_name().name() == "SID_DATA")
            .and_then(|node| node.text())
            .unwrap_or("");

        match name {
            "stigid" => benchmark.id = data.to_owned(),
            "title" => benchmark.title = data.to_owned(),
            _ => {}
        }
    }

    if benchmark.id.is_empty() || benchmark.title.is_empty() {
        return None;
    }

    for vuln in istig
        .children()
        .filter(|node| node.tag_name().name() == "VULN")
    {
        let mut group_id = String::new();
        let mut rule_id = String::new();
        let mut stig_id: Option<String> = None;
        let mut severity_str = String::new();
        let mut title = String::new();
        let mut vuln_discussion = String::new();
        let mut check_text = String::new();
        let mut fix_text = String::new();
        let mut cci_refs: Vec<String> = Vec::new();
        let mut false_positives: Option<String> = None;
        let mut false_negatives: Option<String> = None;
        let mut documentable: Option<bool> = None;
        let mut ckl_status = CKLStatus::NotReviewed;

        for stig_data in vuln
            .children()
            .filter(|node| node.tag_name().name() == "STIG_DATA")
        {
            let attr = stig_data
                .children()
                .find(|node| node.tag_name().name() == "VULN_ATTRIBUTE")
                .and_then(|node| node.text())
                .unwrap_or("");

            let data = stig_data
                .children()
                .find(|node| node.tag_name().name() == "ATTRIBUTE_DATA")
                .and_then(|node| node.text())
                .unwrap_or("");

            match attr {
                "Vuln_Num" => group_id = data.to_owned(),
                "Rule_ID" => rule_id = data.trim_end_matches("_rule").to_owned(),
                "Rule_Ver" => stig_id = (!data.is_empty()).then(|| data.to_owned()),
                "Severity" => severity_str = data.to_owned(),
                "Rule_Title" => title = data.to_owned(),
                "Vuln_Discuss" => vuln_discussion = data.to_owned(),
                "Check_Content" => check_text = data.to_owned(),
                "Fix_Text" => fix_text = data.to_owned(),
                "CCI_REF" => {
                    if !data.is_empty() {
                        cci_refs.push(data.to_owned());
                    }
                }
                "False_Positives" => false_positives = (!data.is_empty()).then(|| data.to_owned()),
                "False_Negatives" => false_negatives = (!data.is_empty()).then(|| data.to_owned()),
                "Documentable" => documentable = Some(data.trim() == "true"),
                _ => {}
            }
        }

        if let Some(status_text) = vuln
            .children()
            .find(|node| node.tag_name().name() == "STATUS")
            .and_then(|node| node.text())
        {
            ckl_status = parse_ckl_status(status_text);
        }

        if group_id.is_empty()
            || rule_id.is_empty()
            || title.is_empty()
            || vuln_discussion.is_empty()
            || check_text.is_empty()
            || fix_text.is_empty()
        {
            continue;
        }

        let rule = Rule {
            group_id: group_id.clone(),
            rule_id,
            stig_id,
            severity: parse_severity(&severity_str),
            title,
            vuln_discussion,
            check_text,
            fix_text,
            cci_refs: (!cci_refs.is_empty()).then_some(cci_refs),
            false_positives,
            false_negatives,
            documentable,
            ckl_status: Some(ckl_status),
        };

        benchmark.rules.insert(group_id, rule);
    }

    if benchmark.rules.is_empty() {
        return None;
    }

    Some(benchmark)
}

fn parse_ckl_status(s: &str) -> CKLStatus {
    match s {
        "NotAFinding" => CKLStatus::NotAFinding,
        "Open" => CKLStatus::Open,
        "Not_Applicable" => CKLStatus::NotApplicable,
        _ => CKLStatus::NotReviewed,
    }
}

fn parse_severity(str: &str) -> Severity {
    match str {
        "high" => Severity::High,
        "medium" => Severity::Medium,
        "low" => Severity::Low,
        _ => Severity::Unknown,
    }
}
