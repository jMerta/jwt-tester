use directories::ProjectDirs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn normalize_opt_string(input: Option<String>) -> Option<String> {
    input.and_then(|val| {
        let trimmed = val.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub(super) fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for tag in tags {
        let trimmed = tag.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !out.iter().any(|existing| existing == trimmed) {
            out.push(trimmed.to_string());
        }
    }
    out
}

pub(super) fn serialize_tags(tags: &[String]) -> Option<String> {
    if tags.is_empty() {
        None
    } else {
        serde_json::to_string(tags).ok()
    }
}

pub(super) fn parse_tags(raw: Option<String>) -> Vec<String> {
    raw.and_then(|val| serde_json::from_str::<Vec<String>>(&val).ok())
        .unwrap_or_default()
}

pub(super) fn default_data_dir() -> Option<PathBuf> {
    ProjectDirs::from("dev", "jwt-tester", "jwt-tester").map(|d| d.data_dir().to_path_buf())
}

pub(super) fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::{normalize_opt_string, normalize_tags, parse_tags, serialize_tags};

    #[test]
    fn normalize_opt_string_handles_empty() {
        assert_eq!(normalize_opt_string(Some("  ".to_string())), None);
        assert_eq!(
            normalize_opt_string(Some(" notes ".to_string())),
            Some("notes".to_string())
        );
    }

    #[test]
    fn normalize_tags_trims_and_dedupes() {
        let tags = normalize_tags(vec![
            " alpha ".to_string(),
            "beta".to_string(),
            "alpha".to_string(),
            "".to_string(),
        ]);
        assert_eq!(tags, vec!["alpha".to_string(), "beta".to_string()]);
    }

    #[test]
    fn tags_roundtrip_json() {
        let tags = vec!["one".to_string(), "two".to_string()];
        let raw = serialize_tags(&tags).expect("serialize tags");
        let parsed = parse_tags(Some(raw));
        assert_eq!(parsed, tags);
    }
}
