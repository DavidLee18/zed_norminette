use std::fs;

use zed_extension_api::{
    self as zed,
    lsp::{Completion, Symbol},
    GithubReleaseOptions,
};

struct NorminetteExtension {
    cache: Option<String>,
    version: Option<String>,
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

    pub fn cache_new(
        &mut self,
        release: zed::GithubRelease,
        language_server_id: &zed::LanguageServerId,
    ) -> zed::Result<zed::Command> {
        let asset_name = self.asset_name();

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))
            .map_err(|e| {
                zed::set_language_server_installation_status(
                    language_server_id,
                    &zed::LanguageServerInstallationStatus::Failed(e.clone()),
                );
                e
            })?;

        let version_dir = String::from("norminette_lsp");
        let binary_path = format!("{}_{}", version_dir, release.version);

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                &language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &binary_path,
                zed::DownloadedFileType::Uncompressed,
            )
            .map_err(|e| {
                let s = format!("failed to download file: {e}");
                zed::set_language_server_installation_status(
                    language_server_id,
                    &zed::LanguageServerInstallationStatus::Failed(s.clone()),
                );
                s
            })?;

            zed::make_file_executable(&binary_path).map_err(|e| {
                let s = format!("failed to make file executable: {e}");
                zed::set_language_server_installation_status(
                    language_server_id,
                    &zed::LanguageServerInstallationStatus::Failed(s.clone()),
                );
                s
            })?;

            let entries = fs::read_dir(".").map_err(|e| {
                let s = format!("failed to list working directory {e}");
                zed::set_language_server_installation_status(
                    language_server_id,
                    &zed::LanguageServerInstallationStatus::Failed(s.clone()),
                );
                s
            })?;
            for entry in entries {
                let entry = entry.map_err(|e| {
                    let s = format!("failed to load directory entry {e}");
                    zed::set_language_server_installation_status(
                        language_server_id,
                        &zed::LanguageServerInstallationStatus::Failed(s.clone()),
                    );
                    s
                })?;
                if entry.file_name().to_str() != Some(&binary_path) {
                    fs::remove_file(&entry.path()).map_err(|e| {
                        let s = format!("failed to remove file: {e}");
                        zed::set_language_server_installation_status(
                            language_server_id,
                            &zed::LanguageServerInstallationStatus::Failed(s.clone()),
                        );
                        s
                    })?;
                }
            }
        }

        self.cache = Some(binary_path.clone());
        self.version = Some(release.version);
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::None,
        );
        Ok(zed::Command {
            command: binary_path,
            args: vec![],
            env: vec![],
        })
    }
}

impl zed::Extension for NorminetteExtension {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            cache: None,
            version: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            "DavidLee18/norminette_lsp",
            GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )
        .map_err(|e| {
            let s = format!("failed to find release: {e}");
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Failed(s.clone()),
            );
            s
        })?;
        match &self.version {
            Some(v) if v == &release.version => match self.cache.take() {
                Some(path) => Ok(zed::Command {
                    command: path,
                    args: vec![],
                    env: vec![],
                }),
                None => match worktree.which(&format!("norminette_lsp_{}", v)) {
                    Some(path) if fs::metadata(&path).map_or(false, |stat| stat.is_file()) => {
                        self.cache = Some(path.clone());
                        Ok(zed::Command {
                            command: path.clone(),
                            args: vec![],
                            env: vec![],
                        })
                    }
                    _ => self.cache_new(release, language_server_id),
                },
            },
            _ => self.cache_new(release, language_server_id),
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
