//! Tiny CLI that exercises the core/format helpers.
mod cli_i18n;

use std::{
    env, process,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::cli_i18n::CliI18n;
use greentic_i18n_lib::{
    DefaultResolver, FormatFacade, I18n, I18nRequest, I18nResolver, normalize_tag,
    tag::{extension_value, parse_tag_details},
};
use serde_json::json;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let raw_args = env::args().skip(1).collect::<Vec<_>>();
    let requested_locale = requested_locale_from_args(&raw_args);
    let i18n = CliI18n::from_request(requested_locale.as_deref())?;
    let args = extract_global_locale(raw_args, &i18n)?;

    if args.is_empty() {
        print_usage(&i18n);
        return Ok(());
    }

    match args[0].as_str() {
        "normalize" => normalize_cmd(&args, &i18n),
        "id" => id_cmd(&args, &i18n),
        "resolve" => resolve_cmd(&args, &i18n),
        "--help" | "help" => {
            print_usage(&i18n);
            Ok(())
        }
        cmd => {
            print_usage(&i18n);
            Err(i18n.tf("cli.error.unknown_command", &[cmd]))
        }
    }
}

fn requested_locale_from_args(args: &[String]) -> Option<String> {
    let mut locale = None;
    let mut idx = 0usize;
    while idx < args.len() {
        let token = &args[idx];
        if token == "--locale" {
            if let Some(value) = args.get(idx + 1)
                && !value.starts_with('-')
            {
                locale = Some(value.to_string());
            }
            idx += 2;
            continue;
        }
        if let Some(value) = token.strip_prefix("--locale=") {
            if !value.is_empty() {
                locale = Some(value.to_string());
            }
            idx += 1;
            continue;
        }
        idx += 1;
    }
    locale
}

fn extract_global_locale(args: Vec<String>, i18n: &CliI18n) -> Result<Vec<String>, String> {
    let mut filtered = Vec::new();
    let mut idx = 0usize;
    while idx < args.len() {
        let token = &args[idx];
        if token == "--locale" {
            let Some(value) = args.get(idx + 1) else {
                return Err(i18n.t("cli.error.locale_needs_value"));
            };
            if value.starts_with('-') {
                return Err(i18n.t("cli.error.locale_needs_value"));
            }
            idx += 2;
            continue;
        }
        if let Some(value) = token.strip_prefix("--locale=") {
            if value.is_empty() {
                return Err(i18n.t("cli.error.locale_needs_value"));
            }
            idx += 1;
            continue;
        }
        filtered.push(token.to_string());
        idx += 1;
    }
    Ok(filtered)
}

fn normalize_cmd(args: &[String], i18n: &CliI18n) -> Result<(), String> {
    let tag = args
        .get(1)
        .ok_or_else(|| i18n.t("cli.error.normalize_requires_tag"))?;
    let canonical = normalize_tag(tag).map_err(|e| e.to_string())?;
    println!("{}", canonical.as_str());
    Ok(())
}

fn id_cmd(args: &[String], i18n: &CliI18n) -> Result<(), String> {
    let tag = args
        .get(1)
        .ok_or_else(|| i18n.t("cli.error.id_requires_tag"))?;
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

fn resolve_cmd(args: &[String], i18n: &CliI18n) -> Result<(), String> {
    let tag = args
        .get(1)
        .ok_or_else(|| i18n.t("cli.error.resolve_requires_tag"))?;
    let mut currency_arg: Option<String> = None;
    let mut idx = 2;
    let mut json_output = false;
    while idx < args.len() {
        match args[idx].as_str() {
            "--currency" => {
                let value = args
                    .get(idx + 1)
                    .ok_or_else(|| i18n.t("cli.error.currency_needs_value"))?;
                currency_arg = Some(value.to_string());
                idx += 2;
            }
            "--json" => {
                json_output = true;
                idx += 1;
            }
            unknown => return Err(i18n.tf("cli.error.unexpected_argument", &[unknown])),
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
    let (example_date, example_time) = format_example_date_time(now, i18n);
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
        println!("{}", i18n.tf("cli.resolve.tag", &[profile.tag.as_str()]));
        println!("{}", i18n.tf("cli.resolve.id", &[&id_string]));
        println!(
            "{}",
            i18n.tf(
                "cli.resolve.fallback_chain",
                &[&fallback_chain
                    .iter()
                    .map(|tag| tag.as_str())
                    .collect::<Vec<_>>()
                    .join(" -> ")]
            )
        );
        println!("{}", i18n.tf("cli.resolve.language", &[&details.language]));
        if let Some(script) = &details.script {
            println!("{}", i18n.tf("cli.resolve.script", &[script]));
        }
        if let Some(region) = &details.region {
            println!("{}", i18n.tf("cli.resolve.region", &[region]));
        }
        if !details.variants.is_empty() {
            println!(
                "{}",
                i18n.tf("cli.resolve.variants", &[&details.variants.join(", ")])
            );
        }
        println!("{}", i18n.tf("cli.resolve.calendar", &[&profile.calendar]));
        println!(
            "{}",
            i18n.tf("cli.resolve.number_system", &[&profile.numbering_system])
        );
        if let Some(currency_tag) = extension_value(&details, "cu") {
            println!("{}", i18n.tf("cli.resolve.currency_tag", &[&currency_tag]));
        }
        println!("{}", i18n.tf("cli.resolve.timezone", &[&profile.timezone]));
        println!(
            "{}",
            i18n.tf("cli.resolve.first_day", &[&profile.first_day])
        );
        println!(
            "{}",
            i18n.tf("cli.resolve.hour_cycle", &[&profile.hour_cycle])
        );
        if let Some(collation) = &profile.collation {
            println!("{}", i18n.tf("cli.resolve.collation", &[collation]));
        }
        if let Some(case_first) = &profile.case_first {
            println!("{}", i18n.tf("cli.resolve.case_first", &[case_first]));
        }
        if let Some(units) = &profile.units {
            println!("{}", i18n.tf("cli.resolve.units", &[units]));
        }
        let none_label = i18n.t("cli.resolve.none");
        let currency_text = profile.currency.as_deref().unwrap_or(&none_label);
        println!("{}", i18n.tf("cli.resolve.currency", &[currency_text]));
        println!(
            "{}",
            i18n.tf(
                "cli.resolve.decimal_separator",
                &[&profile.decimal_separator.to_string()]
            )
        );
        println!(
            "{}",
            i18n.tf("cli.resolve.direction", &[&profile.direction.to_string()])
        );
        println!(
            "{}",
            i18n.tf("cli.resolve.example_number", &[&example_number])
        );
        println!(
            "{}",
            i18n.tf("cli.resolve.example_currency", &[&example_currency])
        );
        println!("{}", i18n.tf("cli.resolve.example_date", &[&example_date]));
        println!("{}", i18n.tf("cli.resolve.example_time", &[&example_time]));
        println!(
            "{}",
            i18n.tf("cli.resolve.example_datetime", &[&example_datetime])
        );
    }

    Ok(())
}

fn print_usage(i18n: &CliI18n) {
    eprintln!("{}", i18n.t("cli.usage.title"));
    eprintln!("{}", i18n.t("cli.usage.commands_header"));
    eprintln!("{}", i18n.t("cli.usage.command.normalize"));
    eprintln!("{}", i18n.t("cli.usage.command.id"));
    eprintln!("{}", i18n.t("cli.usage.command.resolve"));
    eprintln!("{}", i18n.t("cli.usage.command.locale"));
    eprintln!("{}", i18n.t("cli.usage.command.help"));
    eprintln!();
    print_tag_notes(i18n);
}

fn print_tag_notes(i18n: &CliI18n) {
    eprintln!("{}", i18n.t("cli.notes.bcp47"));
    eprintln!("{}", i18n.t("cli.notes.try_header"));
    eprintln!("{}", i18n.t("cli.notes.try.normalize_en_gb"));
    eprintln!("{}", i18n.t("cli.notes.try.normalize_zh_hant_tw"));
    eprintln!("{}", i18n.t("cli.notes.try.id_fr_ca"));
    eprintln!("{}", i18n.t("cli.notes.try.resolve_es_eur"));
    eprintln!();
    eprintln!("{}", i18n.t("cli.notes.complex_header"));
    eprintln!("{}", i18n.t("cli.notes.complex.normalize"));
    eprintln!("{}", i18n.t("cli.notes.complex.resolve"));
}

fn format_example_date_time(when: SystemTime, i18n: &CliI18n) -> (String, String) {
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
            i18n.tf("cli.error.invalid_date", &[&err.to_string()]),
            i18n.tf("cli.error.invalid_time", &[&err.to_string()]),
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
