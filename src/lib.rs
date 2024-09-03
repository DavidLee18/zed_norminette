use zed_extension_api::{
    self as zed,
    lsp::{Completion, Symbol},
};

struct NorminetteExtension;

impl zed::Extension for NorminetteExtension {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        Err("`language_server_command` not implemented".to_string())
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<Option<zed::serde_json::Value>> {
        Ok(None)
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<Option<zed::serde_json::Value>> {
        Ok(None)
    }

    fn label_for_completion(
        &self,
        language_server_id: &zed::LanguageServerId,
        completion: Completion,
    ) -> Option<zed::CodeLabel> {
        None
    }

    fn label_for_symbol(
        &self,
        language_server_id: &zed::LanguageServerId,
        symbol: Symbol,
    ) -> Option<zed::CodeLabel> {
        None
    }

    fn complete_slash_command_argument(
        &self,
        command: zed::SlashCommand,
        args: Vec<String>,
    ) -> zed::Result<Vec<zed::SlashCommandArgumentCompletion>, String> {
        Ok(Vec::new())
    }

    fn run_slash_command(
        &self,
        command: zed::SlashCommand,
        args: Vec<String>,
        worktree: Option<&zed::Worktree>,
    ) -> zed::Result<zed::SlashCommandOutput, String> {
        Err("`run_slash_command` not implemented".to_string())
    }

    fn suggest_docs_packages(&self, _provider: String) -> zed::Result<Vec<String>, String> {
        Ok(Vec::new())
    }

    fn index_docs(
        &self,
        provider: String,
        package: String,
        database: &zed::KeyValueStore,
    ) -> zed::Result<(), String> {
        Err("`index_docs` not implemented".to_string())
    }
}

zed::register_extension!(NorminetteExtension);
