use std::fs;

use zed_extension_api::{
    self as zed,
    lsp::{Completion, Symbol},
    GithubReleaseOptions, LanguageServerInstallationStatus,
};

struct NorminetteExtension {
    cache: Option<String>,
}

impl NorminetteExtension {
    pub fn asset_name(&self) -> String {
        let (platform, arch) = zed::current_platform();
        format!(
            "norminette_lsp-{arch}-{os}",
            arch = match arch {
                zed::Architecture::Aarch64 => "aarch64",
                zed::Architecture::X86 => "x86",
                zed::Architecture::X8664 => "x86_64",
            },
            os = match platform {
                zed::Os::Mac => "apple-darwin",
                zed::Os::Linux => "unknown-linux-gnu",
                zed::Os::Windows => "windows",
            }
        )
    }
}

impl zed::Extension for NorminetteExtension {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self { cache: None }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        match worktree.which(&self.asset_name()) {
            Some(path) => Ok(zed::Command {
                command: path,
                args: vec![],
                env: vec![],
            }),
            None => match &self.cache {
                Some(path) if fs::metadata(path).map_or(false, |stat| stat.is_file()) => {
                    Ok(zed::Command {
                        command: path.clone(),
                        args: vec![],
                        env: vec![],
                    })
                }
                _ => {
                    zed::set_language_server_installation_status(
                        language_server_id,
                        &LanguageServerInstallationStatus::CheckingForUpdate,
                    );

                    let release = zed::latest_github_release(
                        "DavidLee18/norminette_lsp",
                        GithubReleaseOptions {
                            require_assets: true,
                            pre_release: false,
                        },
                    )?;

                    let asset_name = self.asset_name();

                    let asset = release
                        .assets
                        .iter()
                        .find(|asset| asset.name == asset_name)
                        .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

                    let version_dir = format!("norminette_lsp_{}", release.version);
                    let binary_path = format!("{}/{}", version_dir, asset_name);

                    if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
                        zed::set_language_server_installation_status(
                            &language_server_id,
                            &zed::LanguageServerInstallationStatus::Downloading,
                        );

                        zed::download_file(
                            &asset.download_url,
                            &version_dir,
                            zed::DownloadedFileType::Uncompressed,
                        )
                        .map_err(|e| format!("failed to download file: {e}"))?;

                        zed::make_file_executable(&binary_path)?;

                        let entries = fs::read_dir(".")
                            .map_err(|e| format!("failed to list working directory {e}"))?;
                        for entry in entries {
                            let entry =
                                entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                            if entry.file_name().to_str() != Some(&version_dir) {
                                fs::remove_dir_all(&entry.path()).ok();
                            }
                        }
                    }

                    self.cache = Some(binary_path.clone());
                    Ok(zed::Command {
                        command: binary_path,
                        args: vec![],
                        env: vec![],
                    })
                }
            },
        }
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> zed::Result<Option<zed::serde_json::Value>> {
        Ok(None)
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> zed::Result<Option<zed::serde_json::Value>> {
        Ok(None)
    }

    fn label_for_completion(
        &self,
        _language_server_id: &zed::LanguageServerId,
        _completion: Completion,
    ) -> Option<zed::CodeLabel> {
        None
    }

    fn label_for_symbol(
        &self,
        _language_server_id: &zed::LanguageServerId,
        _symbol: Symbol,
    ) -> Option<zed::CodeLabel> {
        None
    }

    fn complete_slash_command_argument(
        &self,
        _command: zed::SlashCommand,
        _args: Vec<String>,
    ) -> zed::Result<Vec<zed::SlashCommandArgumentCompletion>, String> {
        Ok(Vec::new())
    }

    fn run_slash_command(
        &self,
        _command: zed::SlashCommand,
        _args: Vec<String>,
        _worktree: Option<&zed::Worktree>,
    ) -> zed::Result<zed::SlashCommandOutput, String> {
        Err("`run_slash_command` not implemented".to_string())
    }

    fn suggest_docs_packages(&self, _provider: String) -> zed::Result<Vec<String>, String> {
        Ok(Vec::new())
    }

    fn index_docs(
        &self,
        _provider: String,
        _package: String,
        _database: &zed::KeyValueStore,
    ) -> zed::Result<(), String> {
        Err("`index_docs` not implemented".to_string())
    }
}

zed::register_extension!(NorminetteExtension);
