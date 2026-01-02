use crate::error::{AppError, AppResult};
use serde_json::{json, Value};
use time::format_description::well_known::Rfc3339;
use time::{OffsetDateTime, UtcOffset};

pub struct DateExtraction {
    pub json: Value,
    pub lines: Vec<String>,
}

#[derive(Clone, Copy)]
pub enum DateMode {
    Utc,
    Local,
    Offset(UtcOffset),
}

pub fn extract_dates(payload: &Value, mode: Option<DateMode>) -> AppResult<DateExtraction> {
    let Some(mode) = mode else {
        return Ok(DateExtraction {
            json: json!({}),
            lines: Vec::new(),
        });
    };

    let mut json_map = serde_json::Map::new();
    let mut lines = Vec::new();

    if let Some(obj) = payload.as_object() {
        for key in ["exp", "nbf", "iat"] {
            if let Some(val) = obj.get(key) {
                if let Some(num) = val.as_i64() {
                    let rendered = format_timestamp(num, mode)?;
                    json_map.insert(key.to_string(), json!({ "raw": num, "rfc3339": rendered }));
                    lines.push(format!("{key}: {num} -> {rendered}"));
                }
            }
        }
    }

    Ok(DateExtraction {
        json: Value::Object(json_map),
        lines,
    })
}

pub fn parse_date_mode(input: Option<String>) -> AppResult<Option<DateMode>> {
    let Some(raw) = input else {
        return Ok(None);
    };
    let val = raw.trim().to_lowercase();
    if val == "utc" {
        return Ok(Some(DateMode::Utc));
    }
    if val == "local" {
        return Ok(Some(DateMode::Local));
    }
    if let Some(offset) = parse_offset(&val)? {
        return Ok(Some(DateMode::Offset(offset)));
    }
    Err(AppError::invalid_claims(
        "invalid --date value; expected utc, local, or +HH:MM",
    ))
}

fn parse_offset(input: &str) -> AppResult<Option<UtcOffset>> {
    if !(input.starts_with('+') || input.starts_with('-')) {
        return Ok(None);
    }
    let sign = if input.starts_with('-') { -1 } else { 1 };
    let parts: Vec<&str> = input[1..].split(':').collect();
    if parts.len() != 2 {
        return Err(AppError::invalid_claims(
            "invalid offset format; use +HH:MM",
        ));
    }
    let hours: i8 = parts[0]
        .parse()
        .map_err(|_| AppError::invalid_claims("invalid offset hours"))?;
    let mins: i8 = parts[1]
        .parse()
        .map_err(|_| AppError::invalid_claims("invalid offset minutes"))?;
    let offset = UtcOffset::from_hms(sign * hours, sign * mins, 0)
        .map_err(|_| AppError::invalid_claims("invalid offset value"))?;
    Ok(Some(offset))
}

fn format_timestamp(ts: i64, mode: DateMode) -> AppResult<String> {
    let odt = OffsetDateTime::from_unix_timestamp(ts)
        .map_err(|_| AppError::invalid_claims("invalid timestamp"))?;
    let adjusted = match mode {
        DateMode::Utc => odt.to_offset(UtcOffset::UTC),
        DateMode::Local => {
            let offset = UtcOffset::current_local_offset()
                .map_err(|_| AppError::invalid_claims("unable to determine local offset"))?;
            odt.to_offset(offset)
        }
        DateMode::Offset(offset) => odt.to_offset(offset),
    };
    adjusted
        .format(&Rfc3339)
        .map_err(|e| AppError::invalid_claims(format!("format timestamp failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn date_mode_parses_utc_local_offset() {
        assert!(matches!(
            parse_date_mode(Some("utc".into())).unwrap(),
            Some(DateMode::Utc)
        ));
        assert!(matches!(
            parse_date_mode(Some("local".into())).unwrap(),
            Some(DateMode::Local)
        ));
        assert!(matches!(
            parse_date_mode(Some("+02:00".into())).unwrap(),
            Some(DateMode::Offset(_))
        ));
    }

    #[test]
    fn extract_dates_empty_when_missing() {
        let payload = json!({ "sub": "123" });
        let out = extract_dates(&payload, None).unwrap();
        assert!(out.json.as_object().unwrap().is_empty());
    }
}
