use crate::error::{AppError, AppResult};
use humantime::parse_duration;
use serde_json::{json, Map, Value};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default, Debug, Clone)]
pub struct StandardClaims {
    pub iss: Option<String>,
    pub sub: Option<String>,
    pub aud: Vec<String>,
    pub jti: Option<String>,
    pub iat: Option<String>,
    pub nbf: Option<String>,
    pub exp: Option<String>,
    pub no_iat: bool,
}

pub fn build_claims(
    base: Value,
    claim_files: Vec<Value>,
    standard: StandardClaims,
    claim_kv: Vec<String>,
    keep_order: bool,
) -> AppResult<Value> {
    let mut obj = into_object(base, "claims JSON")?;

    for file_val in claim_files {
        let file_obj = into_object(file_val, "claim file JSON")?;
        for (k, v) in file_obj {
            obj.insert(k, v);
        }
    }

    apply_standard_claims(&mut obj, standard)?;

    for kv in claim_kv {
        let (k, v) = parse_claim_kv(&kv)?;
        obj.insert(k, v);
    }

    if keep_order {
        return Ok(Value::Object(obj));
    }

    let mut entries: Vec<(String, Value)> = obj.into_iter().collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut sorted = Map::new();
    for (k, v) in entries {
        sorted.insert(k, v);
    }
    Ok(Value::Object(sorted))
}

fn into_object(val: Value, label: &str) -> AppResult<Map<String, Value>> {
    match val {
        Value::Object(map) => Ok(map),
        _ => Err(AppError::invalid_claims(format!(
            "{label} must be a JSON object"
        ))),
    }
}

fn apply_standard_claims(obj: &mut Map<String, Value>, standard: StandardClaims) -> AppResult<()> {
    if let Some(iss) = standard.iss {
        obj.insert("iss".to_string(), Value::String(iss));
    }
    if let Some(sub) = standard.sub {
        obj.insert("sub".to_string(), Value::String(sub));
    }
    if !standard.aud.is_empty() {
        if standard.aud.len() == 1 {
            obj.insert("aud".to_string(), Value::String(standard.aud[0].clone()));
        } else {
            obj.insert(
                "aud".to_string(),
                Value::Array(standard.aud.into_iter().map(Value::String).collect()),
            );
        }
    }
    if let Some(jti) = standard.jti {
        obj.insert("jti".to_string(), Value::String(jti));
    }

    let now = now_epoch();

    if standard.no_iat {
        obj.remove("iat");
    } else if let Some(iat) = standard.iat {
        let ts = parse_time(&iat, now)?;
        obj.insert("iat".to_string(), json!(ts));
    }

    if let Some(nbf) = standard.nbf {
        let ts = parse_time(&nbf, now)?;
        obj.insert("nbf".to_string(), json!(ts));
    }

    if let Some(exp) = standard.exp {
        let ts = parse_time(&exp, now)?;
        obj.insert("exp".to_string(), json!(ts));
    }

    Ok(())
}

pub fn parse_claim_kv(input: &str) -> AppResult<(String, Value)> {
    let mut parts = input.splitn(2, '=');
    let key = parts.next().unwrap_or("").trim();
    let val = parts.next().unwrap_or("").trim();
    if key.is_empty() {
        return Err(AppError::invalid_claims("claim key is required"));
    }
    if val.is_empty() {
        return Err(AppError::invalid_claims(format!(
            "claim '{key}' is missing a value"
        )));
    }

    let parsed =
        serde_json::from_str::<Value>(val).unwrap_or_else(|_| Value::String(val.to_string()));
    Ok((key.to_string(), parsed))
}

pub fn parse_time(spec: &str, now: i64) -> AppResult<i64> {
    let raw = spec.trim();
    if raw.is_empty() {
        return Err(AppError::invalid_claims("time value is empty"));
    }
    if raw == "now" {
        return Ok(now);
    }
    if let Ok(val) = raw.parse::<i64>() {
        return Ok(val);
    }
    let mut sign = 1i64;
    let mut text = raw.to_string();
    if text.to_lowercase().contains("ago") {
        sign = -1;
        text = text.replace("ago", "");
    }
    let mut text = text.trim().to_string();
    if text.starts_with('-') {
        sign = -1;
        text = text.trim_start_matches('-').trim().to_string();
    } else if text.starts_with('+') {
        text = text.trim_start_matches('+').trim().to_string();
    }
    if text.is_empty() {
        return Err(AppError::invalid_claims("time value is empty"));
    }
    if text.chars().any(|c| c.is_alphabetic()) {
        let dur = parse_duration(&text)
            .map_err(|e| AppError::invalid_claims(format!("invalid duration '{raw}': {e}")))?;
        let secs = dur.as_secs() as i64;
        return Ok(now + sign * secs);
    }

    Err(AppError::invalid_claims(format!(
        "invalid time value '{raw}'"
    )))
}

pub fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_claim_kv_handles_json_and_strings() {
        let (k, v) = parse_claim_kv("count=3").unwrap();
        assert_eq!(k, "count");
        assert_eq!(v, json!(3));

        let (k, v) = parse_claim_kv("flag=true").unwrap();
        assert_eq!(k, "flag");
        assert_eq!(v, json!(true));

        let (k, v) = parse_claim_kv("name=alice").unwrap();
        assert_eq!(k, "name");
        assert_eq!(v, json!("alice"));
    }

    #[test]
    fn parse_time_supports_now_and_durations() {
        let now = 1_000;
        assert_eq!(parse_time("now", now).unwrap(), now);
        assert_eq!(parse_time("1050", now).unwrap(), 1050);
        assert_eq!(parse_time("+1h", now).unwrap(), now + 3600);
        assert_eq!(parse_time("-30m", now).unwrap(), now - 1800);
        assert_eq!(parse_time("2 days", now).unwrap(), now + 172_800);
        assert_eq!(parse_time("2 days ago", now).unwrap(), now - 172_800);
    }

    #[test]
    fn standard_claims_follow_spec_types() {
        let standard = StandardClaims {
            iss: Some("issuer".to_string()),
            sub: Some("subject".to_string()),
            aud: vec!["api".to_string(), "mobile".to_string()],
            jti: Some("jwt-id".to_string()),
            iat: Some("1700000000".to_string()),
            nbf: Some("1700000100".to_string()),
            exp: Some("1700000200".to_string()),
            no_iat: false,
        };
        let claims =
            build_claims(json!({}), Vec::new(), standard, Vec::new(), false).expect("claims");
        let obj = claims.as_object().expect("object");
        assert_eq!(obj.get("iss").and_then(Value::as_str), Some("issuer"));
        assert_eq!(obj.get("sub").and_then(Value::as_str), Some("subject"));
        assert_eq!(obj.get("jti").and_then(Value::as_str), Some("jwt-id"));
        assert!(obj.get("aud").expect("aud").is_array());
        assert!(obj.get("iat").expect("iat").is_number());
        assert!(obj.get("nbf").expect("nbf").is_number());
        assert!(obj.get("exp").expect("exp").is_number());

        let standard = StandardClaims {
            aud: vec!["single".to_string()],
            ..StandardClaims::default()
        };
        let claims =
            build_claims(json!({}), Vec::new(), standard, Vec::new(), false).expect("claims");
        assert!(claims.get("aud").expect("aud").is_string());

        let standard = StandardClaims {
            iat: Some("1700000000".to_string()),
            no_iat: true,
            ..StandardClaims::default()
        };
        let claims = build_claims(json!({ "iat": 1 }), Vec::new(), standard, Vec::new(), false)
            .expect("claims");
        assert!(claims.get("iat").is_none());
    }
}
