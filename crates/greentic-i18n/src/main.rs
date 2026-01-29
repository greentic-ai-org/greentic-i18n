//! Tiny CLI that exercises the core/format helpers.
use std::{
    env, process,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use greentic_i18n_lib::{
    DefaultResolver, FormatFacade, I18n, I18nRequest, I18nResolver, normalize_tag,
    tag::{extension_value, parse_tag_details},
};
use serde_json::json;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "normalize" => normalize_cmd(&args),
        "id" => id_cmd(&args),
        "resolve" => resolve_cmd(&args),
        "--help" | "help" => {
            print_usage();
            Ok(())
        }
        cmd => {
            print_usage();
            Err(format!("unknown command `{cmd}`"))
        }
    }
}

fn normalize_cmd(args: &[String]) -> Result<(), String> {
    let tag = args
        .get(2)
        .ok_or_else(|| "normalize requires a tag".to_string())?;
    let canonical = normalize_tag(tag).map_err(|e| e.to_string())?;
    println!("{}", canonical.as_str());
    Ok(())
}

fn id_cmd(args: &[String]) -> Result<(), String> {
    let tag = args.get(2).ok_or_else(|| "id requires a tag".to_string())?;
    let canonical = normalize_tag(tag).map_err(|e| e.to_string())?;
    let resolver: Arc<dyn I18nResolver> = Arc::new(DefaultResolver::default());
    let engine = I18n::new(resolver);
    let request = I18nRequest::new(Some(canonical.clone()), None);
    let resolution = engine
        .resolve_and_cache(request)
        .map_err(|e| e.to_string())?;
    println!("{}", resolution.profile.id.as_str());
    Ok(())
}

fn resolve_cmd(args: &[String]) -> Result<(), String> {
    let tag = args
        .get(2)
        .ok_or_else(|| "resolve requires a tag".to_string())?;
    let mut currency_arg: Option<String> = None;
    let mut idx = 3;
    let mut json_output = false;
    while idx < args.len() {
        match args[idx].as_str() {
            "--currency" => {
                let value = args
                    .get(idx + 1)
                    .ok_or_else(|| "--currency needs a value".to_string())?;
                currency_arg = Some(value.to_string());
                idx += 2;
            }
            "--json" => {
                json_output = true;
                idx += 1;
            }
            unknown => return Err(format!("unexpected argument `{unknown}`")),
        }
    }

    let canonical = normalize_tag(tag).map_err(|e| e.to_string())?;
    let resolver: Arc<dyn I18nResolver> = Arc::new(DefaultResolver::default());
    let engine = I18n::new(resolver);
    let request = I18nRequest::new(Some(canonical.clone()), currency_arg.clone());
    let resolution = engine
        .resolve_and_cache(request)
        .map_err(|e| e.to_string())?;
    let profile = resolution.profile;
    let fallback_chain = resolution.fallback_chain;
    let id_string = profile.id.as_str();

    let now = SystemTime::now();
    let example_number = profile.format_number(1234.56);
    let example_currency = profile.format_currency(42.0, currency_arg.as_deref());
    let example_datetime = profile.format_datetime(now);
    let (example_date, example_time) = format_example_date_time(now);
    let details = parse_tag_details(&profile.tag);
    if json_output {
        let profile_obj = json!({
            "language": details.language,
            "script": details.script,
            "region": details.region,
            "variants": details.variants,
            "calendar": profile.calendar,
            "number_system": profile.numbering_system,
            "currency_tag": extension_value(&details, "cu"),
            "currency": profile.currency,
            "timezone": profile.timezone,
            "first_day": profile.first_day,
            "hour_cycle": profile.hour_cycle,
            "collation": profile.collation,
            "case_first": profile.case_first,
            "units": profile.units,
            "direction": profile.direction.to_string(),
            "decimal_separator": profile.decimal_separator.to_string(),
        });
        let samples_obj = json!({
            "number": example_number,
            "currency": example_currency,
            "date": example_date,
            "time": example_time,
            "datetime": example_datetime,
        });
        let payload = json!({
            "schema_version": "v1",
            "tag": profile.tag.as_str(),
            "id": id_string,
            "fallback_chain": fallback_chain
                .iter()
                .map(|tag| tag.as_str())
                .collect::<Vec<_>>(),
            "profile": profile_obj,
            "samples": samples_obj,
        });
        let json_output = serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?;
        println!("{json_output}");
        return Ok(());
    } else {
        println!("tag             : {}", profile.tag.as_str());
        println!("id              : {}", id_string);
        println!(
            "fallback chain  : {}",
            fallback_chain
                .iter()
                .map(|tag| tag.as_str())
                .collect::<Vec<_>>()
                .join(" -> ")
        );
        println!("language        : {}", details.language);
        if let Some(script) = &details.script {
            println!("script          : {}", script);
        }
        if let Some(region) = &details.region {
            println!("region          : {}", region);
        }
        if !details.variants.is_empty() {
            println!("variants        : {}", details.variants.join(", "));
        }
        println!("calendar        : {}", profile.calendar);
        println!("number system   : {}", profile.numbering_system);
        if let Some(currency_tag) = extension_value(&details, "cu") {
            println!("currency tag    : {}", currency_tag);
        }
        println!("timezone        : {}", profile.timezone);
        println!("first day       : {}", profile.first_day);
        println!("hour cycle      : {}", profile.hour_cycle);
        if let Some(collation) = &profile.collation {
            println!("collation       : {}", collation);
        }
        if let Some(case_first) = &profile.case_first {
            println!("case first      : {}", case_first);
        }
        if let Some(units) = &profile.units {
            println!("units           : {}", units);
        }
        println!(
            "currency        : {}",
            profile.currency.as_deref().unwrap_or("none")
        );
        println!("decimal sep     : {}", profile.decimal_separator);
        println!("direction       : {}", profile.direction);
        println!("example number  : {example_number}");
        println!("example currency: {example_currency}");
        println!("example date    : {example_date}");
        println!("example time    : {example_time}");
        println!("example datetime: {example_datetime}");
    }

    Ok(())
}

fn print_usage() {
    eprintln!("Usage: greentic-i18n <command> [args]");
    eprintln!("Commands:");
    eprintln!("  normalize <tag>                   Canonicalize a locale tag");
    eprintln!("  id <tag>                          Print the stable I18nId");
    eprintln!("  resolve <tag> [--currency CODE] [--json]   Resolve a profile and show samples");
    eprintln!("  --help                            Show this help + tag guidance");
    eprintln!();
    print_tag_notes();
}

fn print_tag_notes() {
    eprintln!(
        "Tags follow BCP-47: language[-script][-region][-variants][-u-extension], and casing is normalized."
    );
    eprintln!("Try these commands:");
    eprintln!("  greentic-i18n normalize en-gb");
    eprintln!("  greentic-i18n normalize zh-hant-tw");
    eprintln!("  greentic-i18n id fr-CA-u-ca-gregory");
    eprintln!("  greentic-i18n resolve es --currency EUR");
    eprintln!();
    eprintln!(
        "Complex example with language/calendar/number/currency/date-time/unit/direction/script/variant/collation/timezone/first-day/hour-cycle:"
    );
    eprintln!(
        "  greentic-i18n normalize ar-OM-u-ca-islamic-civil-cu-omr-nu-arabext-ss-yes-kl-kf-upper-co-phonebk-tz-Asia/Muscat-fw-sat-hc-h23-unit-meter"
    );
    eprintln!(
        "  greentic-i18n resolve ar-OM-u-ca-islamic-civil-cu-omr-nu-arabext-ss-yes-kl-kf-upper-co-phonebk-tz-Asia/Muscat-fw-sat-hc-h23-unit-meter"
    );
}

fn format_example_date_time(when: SystemTime) -> (String, String) {
    match when.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            let days = (secs / 86_400) as i64;
            let seconds_of_day = secs % 86_400;
            let (year, month, day) = civil_from_days(days);
            let hour = seconds_of_day / 3_600;
            let minute = (seconds_of_day % 3_600) / 60;
            (
                format!("{:02}/{:02}/{:04}", day, month, year),
                format!("{:02}:{:02}", hour, minute),
            )
        }
        Err(err) => (
            format!("invalid date: {err}"),
            format!("invalid time: {err}"),
        ),
    }
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 {
        z / 146_097
    } else {
        (z - 146_096) / 146_097
    };
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year, m as u32, d as u32)
}
