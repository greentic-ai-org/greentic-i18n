use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub type JsonMap = BTreeMap<String, String>;

pub fn read_json_map(path: &Path) -> Result<JsonMap, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("failed reading {}: {err}", path.display()))?;
    parse_json_map(&raw)
}

pub fn parse_json_map(raw: &str) -> Result<JsonMap, String> {
    let value: Value =
        serde_json::from_str(raw).map_err(|err| format!("invalid JSON map content: {err}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "expected a top-level JSON object".to_string())?;

    let mut out = JsonMap::new();
    for (key, value) in object {
        let text = value
            .as_str()
            .ok_or_else(|| format!("key `{key}` must map to a string"))?;
        out.insert(key.clone(), text.to_string());
    }
    Ok(out)
}

pub fn write_json_map(path: &Path, map: &JsonMap) -> Result<(), String> {
    let mut serialized = serde_json::to_string_pretty(map)
        .map_err(|err| format!("failed serializing map as JSON: {err}"))?;
    serialized.push('\n');
    fs::write(path, serialized).map_err(|err| format!("failed writing {}: {err}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::{JsonMap, read_json_map, write_json_map};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("greentic-i18n-translator-{name}-{stamp}.json"))
    }

    fn fixture_path(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn round_trip_write_read_is_stable_and_preserves_content() {
        let path = unique_temp_path("roundtrip");
        let mut map = JsonMap::new();
        map.insert("z_key".to_string(), "last".to_string());
        map.insert("a_key".to_string(), "first".to_string());

        write_json_map(&path, &map).expect("write_json_map should work");
        let first_bytes = fs::read_to_string(&path).expect("must read first bytes");
        let parsed = read_json_map(&path).expect("read_json_map should work");
        assert_eq!(parsed, map);

        write_json_map(&path, &parsed).expect("second write should work");
        let second_bytes = fs::read_to_string(&path).expect("must read second bytes");
        assert_eq!(first_bytes, second_bytes);
        assert!(first_bytes.find("\"a_key\"").unwrap() < first_bytes.find("\"z_key\"").unwrap());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn sample_fixture_parses_and_values_are_preserved() {
        let map = read_json_map(&fixture_path("sample.json")).expect("fixture should parse");
        assert_eq!(map.len(), 3);
        assert_eq!(map.get("hello"), Some(&"Hello".to_string()));
        assert_eq!(map.get("raw_block"), Some(&"raw:\n{}".to_string()));
        assert_eq!(
            map.get("cmd"),
            Some(&"`greentic-i18n normalize en-gb`".to_string())
        );
    }
}
