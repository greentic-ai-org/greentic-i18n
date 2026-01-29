use crate::I18nTag;

#[derive(Debug, Clone)]
pub struct TagDetails {
    pub language: String,
    pub script: Option<String>,
    pub region: Option<String>,
    pub variants: Vec<String>,
    pub extensions: Vec<Extension>,
}

#[derive(Debug, Clone)]
pub struct Extension {
    pub key: String,
    pub values: Vec<String>,
}

pub fn parse_tag_details(tag: &I18nTag) -> TagDetails {
    let raw = tag.as_str();
    let mut main = raw;
    let mut extension_part: Option<&str> = None;
    if let Some(idx) = raw.find("-u-") {
        main = &raw[..idx];
        extension_part = Some(&raw[idx + 3..]);
    }

    let parts: Vec<&str> = main.split('-').filter(|p| !p.is_empty()).collect();
    let language = parts
        .first()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "und".to_string());

    let mut idx = 1;
    let mut script = None;
    if parts
        .get(idx)
        .is_some_and(|part| part.len() == 4 && part.chars().all(|c| c.is_ascii_alphabetic()))
    {
        script = Some(parts[idx].to_string());
        idx += 1;
    }

    let mut region = None;
    if let Some(candidate) = parts.get(idx)
        && ((candidate.len() == 2 && candidate.chars().all(|c| c.is_ascii_alphabetic()))
            || (candidate.len() == 3 && candidate.chars().all(|c| c.is_ascii_digit())))
    {
        region = Some(candidate.to_string());
        idx += 1;
    }

    let variants = parts[idx..].iter().map(|s| s.to_string()).collect();
    let extensions = parse_extensions(extension_part.unwrap_or(""));

    TagDetails {
        language,
        script,
        region,
        variants,
        extensions,
    }
}

fn parse_extensions(input: &str) -> Vec<Extension> {
    let tokens: Vec<&str> = input.split('-').filter(|t| !t.is_empty()).collect();
    let mut extensions = Vec::new();
    let mut idx = 0;
    while idx < tokens.len() {
        let key = tokens[idx];
        if !is_extension_key(key) {
            idx += 1;
            continue;
        }
        idx += 1;
        let mut values = Vec::new();
        while idx < tokens.len() && !is_extension_key(tokens[idx]) {
            values.push(tokens[idx].to_string());
            idx += 1;
        }
        extensions.push(Extension {
            key: key.to_string(),
            values,
        });
    }
    extensions
}

fn is_extension_key(token: &str) -> bool {
    let len = token.len();
    token.chars().all(|c| c.is_ascii_lowercase()) && (len == 2 || len == 4)
}

pub fn extension_value(details: &TagDetails, key: &str) -> Option<String> {
    details
        .extensions
        .iter()
        .find(|ext| ext.key == key)
        .and_then(|ext| {
            if ext.values.is_empty() {
                None
            } else {
                Some(ext.values.join("-"))
            }
        })
}

pub fn build_parent_chain(final_tag: &I18nTag) -> Vec<I18nTag> {
    let mut chain = Vec::new();
    chain.push(final_tag.clone());
    let base = final_tag
        .as_str()
        .split("-u-")
        .next()
        .unwrap_or(final_tag.as_str());
    let mut parts: Vec<&str> = base.split('-').collect();
    while parts.len() > 1 {
        parts.pop();
        if let Ok(candidate) = I18nTag::new(&parts.join("-")) {
            chain.push(candidate);
        }
    }
    chain
}

pub fn direction_for_language(lang: &str) -> super::Direction {
    match lang {
        "ar" | "he" | "fa" | "ur" | "ps" | "dv" | "yi" => super::Direction::Rtl,
        _ => super::Direction::Ltr,
    }
}

pub fn lenient_first_day(region: Option<&str>) -> &'static str {
    match region {
        Some("US") => "sun",
        Some("GB") | Some("EU") => "mon",
        Some("SA") => "sat",
        _ => "mon",
    }
}

pub fn lenient_hour_cycle(region: Option<&str>) -> &'static str {
    match region {
        Some("US") => "h12",
        Some("SA") => "h23",
        _ => "h23",
    }
}
