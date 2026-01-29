use assert_cmd::cargo::cargo_bin_cmd;
use greentic_i18n_lib::normalize_tag;
use serde_json::Value;

#[test]
fn resolve_json_matches_schema_v1() -> Result<(), Box<dyn std::error::Error>> {
    let tag = "ar-OM-u-ca-islamic-civil-cu-omr-nu-arabext-ss-yes-kl-kf-upper-co-phonebk-tz-Asia/Muscat-fw-sat-hc-h23";
    let mut cmd = cargo_bin_cmd!("greentic-i18n");
    cmd.args(["resolve", tag, "--json"]);
    let output = cmd.output()?;
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout)?;

    assert_eq!(value["schema_version"], "v1");
    let canonical = normalize_tag(tag)?.as_str().to_string();
    assert_eq!(value["tag"], canonical);
    assert!(
        value["fallback_chain"]
            .as_array()
            .unwrap()
            .first()
            .unwrap()
            .as_str()
            .unwrap()
            == canonical
    );

    let profile = &value["profile"];
    assert_eq!(profile["language"], "ar");
    assert_eq!(profile["direction"], "rtl");
    assert_eq!(profile["timezone"], "asia/muscat");
    assert_eq!(profile["decimal_separator"], ".");
    assert!(profile["variants"].is_array());

    let samples = &value["samples"];
    let number_sample = samples["number"].as_str().unwrap();
    assert!(number_sample.replace('_', "").contains("1234"));
    assert!(
        samples["currency"].as_str().unwrap().starts_with("USD")
            || samples["currency"].as_str().unwrap().starts_with("OMR")
    );
    assert!(samples["date"].as_str().unwrap().contains('/'));
    assert!(samples["time"].as_str().unwrap().contains(':'));
    assert!(samples["datetime"].as_str().unwrap().ends_with("UTC"));

    Ok(())
}
