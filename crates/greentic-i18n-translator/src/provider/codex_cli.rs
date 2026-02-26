use crate::json_map::JsonMap;
use crate::provider::TranslatorProvider;
use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMode {
    Auto,
    Browser,
    Device,
    ApiKey,
}

impl AuthMode {
    pub fn as_str(self) -> &'static str {
        match self {
            AuthMode::Auto => "auto",
            AuthMode::Browser => "browser",
            AuthMode::Device => "device",
            AuthMode::ApiKey => "api-key",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodexCliConfig {
    pub auth_mode: AuthMode,
    pub codex_home: Option<PathBuf>,
    pub api_key_stdin: bool,
}

pub struct CodexCliProvider {
    config: CodexCliConfig,
}

impl CodexCliProvider {
    pub fn new(config: CodexCliConfig) -> Self {
        Self { config }
    }

    fn codex_env_command(&self) -> Command {
        let mut cmd = Command::new("codex");
        if let Some(home) = &self.config.codex_home {
            cmd.env("CODEX_HOME", home);
        }
        cmd
    }

    fn login_status(&self) -> Result<bool, String> {
        let output = self
            .codex_env_command()
            .arg("login")
            .arg("status")
            .output()
            .map_err(|err| format!("failed to run `codex login status`: {err}"))?;
        Ok(output.status.success())
    }

    fn login_browser(&self) -> Result<(), String> {
        let output = self
            .codex_env_command()
            .arg("login")
            .output()
            .map_err(|err| format!("failed to run `codex login`: {err}"))?;
        if output.status.success() {
            return Ok(());
        }
        Err(format!(
            "`codex login` failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }

    fn login_device(&self) -> Result<(), String> {
        let output = self
            .codex_env_command()
            .arg("login")
            .arg("--device-auth")
            .output()
            .map_err(|err| format!("failed to run `codex login --device-auth`: {err}"))?;
        if output.status.success() {
            return Ok(());
        }
        Err(format!(
            "`codex login --device-auth` failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }

    fn login_with_api_key(&self) -> Result<(), String> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| "OPENAI_API_KEY is not set for API key auth".to_string())?;
        let mut child = self
            .codex_env_command()
            .arg("login")
            .arg("--with-api-key")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| format!("failed to run `codex login --with-api-key`: {err}"))?;

        {
            let Some(stdin) = child.stdin.as_mut() else {
                return Err("failed to open stdin for `codex login --with-api-key`".to_string());
            };
            stdin
                .write_all(api_key.as_bytes())
                .map_err(|err| format!("failed writing API key to codex stdin: {err}"))?;
            stdin
                .write_all(b"\n")
                .map_err(|err| format!("failed writing API key terminator: {err}"))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|err| format!("failed waiting for `codex login --with-api-key`: {err}"))?;
        if output.status.success() {
            return Ok(());
        }
        Err(format!(
            "`codex login --with-api-key` failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }

    pub fn build_prompt(
        lang: &str,
        items: &[(String, String)],
        glossary: Option<&JsonMap>,
        retry_feedback: Option<&str>,
    ) -> String {
        let mut prompt = String::new();
        prompt.push_str("You translate English UI strings into target language.\n");
        prompt.push_str(&format!("Target language: {lang}\n"));
        prompt.push_str("Rules:\n");
        prompt.push_str("- Keep the exact same number of `{}` placeholders.\n");
        prompt.push_str("- Preserve backtick spans exactly, including inner content.\n");
        prompt.push_str("- Keep exact newline count.\n");
        prompt.push_str("- Return JSON object only, mapping keys to translated strings.\n");
        prompt.push_str("- Do not add explanations, markdown, or extra keys.\n");

        if let Some(glossary) = glossary
            && !glossary.is_empty()
        {
            prompt.push_str("Glossary (preferred terms):\n");
            for (k, v) in glossary {
                prompt.push_str(&format!("- {k} => {v}\n"));
            }
        }

        if let Some(feedback) = retry_feedback {
            prompt.push_str("Previous output failed validation:\n");
            prompt.push_str(feedback);
            prompt.push('\n');
        }

        prompt.push_str("Input JSON:\n");
        prompt.push_str("{\n");
        for (idx, (key, value)) in items.iter().enumerate() {
            let escaped_key = serde_json::to_string(key).unwrap_or_else(|_| "\"\"".to_string());
            let escaped_value = serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string());
            let suffix = if idx + 1 == items.len() { "" } else { "," };
            prompt.push_str(&format!("  {escaped_key}: {escaped_value}{suffix}\n"));
        }
        prompt.push_str("}\n");
        prompt
    }

    pub fn parse_translation_response(raw: &str) -> Result<JsonMap, String> {
        let value: serde_json::Value = serde_json::from_str(raw)
            .map_err(|err| format!("provider did not return valid JSON: {err}"))?;
        let object = value
            .as_object()
            .ok_or_else(|| "provider JSON response must be an object".to_string())?;

        let mut out = JsonMap::new();
        for (key, value) in object {
            let text = value
                .as_str()
                .ok_or_else(|| format!("provider key `{key}` is not a string"))?;
            out.insert(key.clone(), text.to_string());
        }
        Ok(out)
    }

    fn run_codex_prompt(&self, prompt: &str) -> Result<String, String> {
        let output = self
            .codex_env_command()
            .arg("exec")
            .arg(prompt)
            .output()
            .map_err(|err| format!("failed to run `codex exec`: {err}"))?;

        if !output.status.success() {
            return Err(format!(
                "`codex exec` failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        String::from_utf8(output.stdout).map_err(|err| format!("provider output not UTF-8: {err}"))
    }
}

impl TranslatorProvider for CodexCliProvider {
    fn ensure_auth(&self) -> Result<(), String> {
        match self.config.auth_mode {
            AuthMode::ApiKey => return self.login_with_api_key(),
            AuthMode::Browser => return self.login_browser(),
            AuthMode::Device => return self.login_device(),
            AuthMode::Auto => {}
        }

        if self.login_status()? {
            return Ok(());
        }
        if self.login_browser().is_ok() {
            return Ok(());
        }
        if self.config.api_key_stdin || env::var("OPENAI_API_KEY").is_ok() {
            return self.login_with_api_key();
        }
        Err(
            "codex auth unavailable: browser login failed and OPENAI_API_KEY is not set"
                .to_string(),
        )
    }

    fn translate_batch(
        &self,
        lang: &str,
        items: &[(String, String)],
        glossary: Option<&JsonMap>,
        retry_feedback: Option<&str>,
    ) -> Result<JsonMap, String> {
        let prompt = Self::build_prompt(lang, items, glossary, retry_feedback);
        let output = self.run_codex_prompt(&prompt)?;
        Self::parse_translation_response(output.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::CodexCliProvider;
    use crate::json_map::JsonMap;

    #[test]
    fn prompt_builder_contains_validation_rules() {
        let items = vec![("k".to_string(), "raw:\n{}".to_string())];
        let prompt = CodexCliProvider::build_prompt("fr", &items, None, None);
        assert!(prompt.contains("same number of `{}` placeholders"));
        assert!(prompt.contains("Preserve backtick spans exactly"));
        assert!(prompt.contains("Keep exact newline count"));
        assert!(prompt.contains("\"k\": \"raw:\\n{}\""));
    }

    #[test]
    fn parse_json_response_extracts_map() {
        let parsed = CodexCliProvider::parse_translation_response("{\"hello\":\"Bonjour\"}")
            .expect("response should parse");
        assert_eq!(parsed.get("hello"), Some(&"Bonjour".to_string()));
    }

    #[test]
    fn prompt_builder_renders_glossary_and_feedback() {
        let items = vec![("k".to_string(), "v".to_string())];
        let mut glossary = JsonMap::new();
        glossary.insert("CLI".to_string(), "CLI".to_string());
        let prompt =
            CodexCliProvider::build_prompt("de", &items, Some(&glossary), Some("bad output"));
        assert!(prompt.contains("Glossary"));
        assert!(prompt.contains("CLI => CLI"));
        assert!(prompt.contains("Previous output failed validation"));
        assert!(prompt.contains("bad output"));
    }
}
