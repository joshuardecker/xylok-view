use regex::Regex;
use std::collections::HashMap;
use stig_view_core::Benchmark;

use crate::app::Pinned;

/// A command sent by the user can be two things:
/// - Look for a key phrase using regex.
/// - Reset the filter back to nothing.
#[derive(Debug, Clone)]
pub enum Command {
    Phrase(String),
    Reset,
}

/// Parse the given str into a command that can be run on a benchmark.
pub fn parse_command(input: &str) -> Option<Command> {
    let phrase = input.trim().to_string();

    if phrase.is_empty() {
        None
    } else if &phrase == "reset" {
        Some(Command::Reset)
    } else {
        Some(Command::Phrase(phrase))
    }
}

/// Run the given command on a given benchmark, updating what STIGs are pinned.
pub fn run_search_cmd(
    cmd: Command,
    benchmark: &Benchmark,
    mut pins: HashMap<String, Pinned>,
) -> Option<HashMap<String, Pinned>> {
    match cmd {
        Command::Phrase(phrase) => {
            let re = Regex::new(&phrase).ok()?;

            for (name, rule) in benchmark.rules.iter() {
                let is_match = re.is_match(&rule.group_id)
                    || re.is_match(&rule.rule_id)
                    || rule.stig_id.as_deref().is_some_and(|id| re.is_match(id))
                    || re.is_match(&rule.title)
                    || re.is_match(&rule.vuln_discussion)
                    || re.is_match(&rule.check_text)
                    || re.is_match(&rule.fix_text)
                    || rule
                        .cci_refs
                        .as_deref()
                        .unwrap_or(&[])
                        .iter()
                        .any(|cci| re.is_match(cci))
                    || rule
                        .false_positives
                        .as_deref()
                        .is_some_and(|false_p| re.is_match(false_p))
                    || rule
                        .false_negatives
                        .as_deref()
                        .is_some_and(|false_n| re.is_match(false_n));

                if is_match {
                    match pins.get(name).unwrap_or(&Pinned::Not) {
                        Pinned::Not => {
                            let _ = pins.insert(name.to_owned(), Pinned::ByFilter);
                        }
                        Pinned::ByUser => {
                            let _ = pins.insert(name.to_owned(), Pinned::ByFilterAndUser);
                        }
                        // If already pinned, do nothing.
                        _ => (),
                    }

                    continue;
                } else {
                    match pins.get(name).unwrap_or(&Pinned::Not) {
                        Pinned::ByFilter => {
                            let _ = pins.insert(name.to_owned(), Pinned::Not);
                        }
                        Pinned::ByFilterAndUser => {
                            let _ = pins.insert(name.to_owned(), Pinned::ByUser);
                        }
                        // If its not pinned and shouldnt be pinned, do nothing.
                        _ => (),
                    }
                }
            }
        }

        Command::Reset => {
            pins.iter_mut()
                .for_each(|(_name, value)| *value = Pinned::Not);
        }
    }

    Some(pins)
}
