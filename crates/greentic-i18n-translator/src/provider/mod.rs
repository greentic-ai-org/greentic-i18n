pub mod codex_cli;

use crate::json_map::JsonMap;

pub trait TranslatorProvider {
    fn ensure_auth(&self) -> Result<(), String>;

    fn translate_batch(
        &self,
        lang: &str,
        items: &[(String, String)],
        glossary: Option<&JsonMap>,
        retry_feedback: Option<&str>,
    ) -> Result<JsonMap, String>;
}
