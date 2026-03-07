use std::cmp::Ordering;

use crate::error::VersionError;

#[derive(Debug, Eq, PartialEq)]
struct ParsedVersion {
    core: Vec<u64>,
    pre_release: Option<Vec<PreReleaseIdentifier>>,
}

#[derive(Debug, Eq, PartialEq)]
enum PreReleaseIdentifier {
    Numeric(u64),
    AlphaNumeric(String),
}

pub fn is_newer_release(current: &str, candidate: &str) -> Result<bool, VersionError> {
    let current = parse_version(current)?;
    let candidate = parse_version(candidate)?;

    Ok(compare_versions(&candidate, &current).is_gt())
}

fn parse_version(input: &str) -> Result<ParsedVersion, VersionError> {
    let normalized = input.trim().trim_start_matches('v');
    let normalized = normalized
        .split_once('+')
        .map_or(normalized, |(without_build, _)| without_build);
    let (core, pre_release) = normalized
        .split_once('-')
        .map_or((normalized, None), |(core, pre)| (core, Some(pre)));

    let core = core
        .split('.')
        .map(|segment| {
            segment
                .parse::<u64>()
                .map_err(|_| VersionError::InvalidVersion(input.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    if core.is_empty() {
        return Err(VersionError::InvalidVersion(input.to_string()));
    }

    let pre_release = pre_release
        .map(|value| {
            value
                .split('.')
                .map(|segment| {
                    if segment.is_empty() {
                        return Err(VersionError::InvalidVersion(input.to_string()));
                    }

                    match segment.parse::<u64>() {
                        Ok(number) => Ok(PreReleaseIdentifier::Numeric(number)),
                        Err(_) => Ok(PreReleaseIdentifier::AlphaNumeric(segment.to_string())),
                    }
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;

    Ok(ParsedVersion { core, pre_release })
}

fn compare_versions(left: &ParsedVersion, right: &ParsedVersion) -> Ordering {
    let core_len = left.core.len().max(right.core.len());
    for index in 0..core_len {
        let ordering = left
            .core
            .get(index)
            .unwrap_or(&0)
            .cmp(right.core.get(index).unwrap_or(&0));
        if !ordering.is_eq() {
            return ordering;
        }
    }

    compare_pre_release(left.pre_release.as_deref(), right.pre_release.as_deref())
}

fn compare_pre_release(
    left: Option<&[PreReleaseIdentifier]>,
    right: Option<&[PreReleaseIdentifier]>,
) -> Ordering {
    match (left, right) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(left), Some(right)) => {
            let len = left.len().max(right.len());
            for index in 0..len {
                match (left.get(index), right.get(index)) {
                    (Some(left), Some(right)) => {
                        let ordering = compare_identifier(left, right);
                        if !ordering.is_eq() {
                            return ordering;
                        }
                    }
                    (Some(_), None) => return Ordering::Greater,
                    (None, Some(_)) => return Ordering::Less,
                    (None, None) => return Ordering::Equal,
                }
            }

            Ordering::Equal
        }
    }
}

fn compare_identifier(left: &PreReleaseIdentifier, right: &PreReleaseIdentifier) -> Ordering {
    match (left, right) {
        (PreReleaseIdentifier::Numeric(left), PreReleaseIdentifier::Numeric(right)) => {
            left.cmp(right)
        }
        (PreReleaseIdentifier::Numeric(_), PreReleaseIdentifier::AlphaNumeric(_)) => Ordering::Less,
        (PreReleaseIdentifier::AlphaNumeric(_), PreReleaseIdentifier::Numeric(_)) => {
            Ordering::Greater
        }
        (PreReleaseIdentifier::AlphaNumeric(left), PreReleaseIdentifier::AlphaNumeric(right)) => {
            left.cmp(right)
        }
    }
}
