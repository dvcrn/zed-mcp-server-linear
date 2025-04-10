use serde::Deserialize;
use std::env;
use zed::settings::ContextServerSettings;
use zed_extension_api::{self as zed, serde_json, Command, ContextServerId, Project, Result};

// use my mcp at dvcrn/mcp-server-linear
const PACKAGE_NAME: &str = "mcp-server-linear";
const SERVER_PATH: &str = "node_modules/mcp-server-linear/build/index.js";

#[derive(Debug, Deserialize)]
struct LinearContextServerSettings {
    linear_api_key: String,
    #[serde(default)]
    tool_prefix: Option<String>,
}

struct LinearModelContextExtension;

impl zed::Extension for LinearModelContextExtension {
    fn new() -> Self {
        Self
    }

    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        project: &Project,
    ) -> Result<Command> {
        let latest_version = zed::npm_package_latest_version(PACKAGE_NAME)?;
        let version = zed::npm_package_installed_version(PACKAGE_NAME)?;
        if version.as_deref() != Some(latest_version.as_ref()) {
            zed::npm_install_package(PACKAGE_NAME, &latest_version)?;
        }

        // using the same key as the other mcp since it expects the same API key
        // decided for this to not have conflicting settings
        let settings = ContextServerSettings::for_project("mcp-server-linear", project)?;
        let Some(settings) = settings.settings else {
            return Err("missing `linear_api_key` setting".into());
        };
        let settings: LinearContextServerSettings =
            serde_json::from_value(settings).map_err(|e| e.to_string())?;

        if settings.linear_api_key.is_empty() {
            return Err("missing `linear_api_key` setting".into());
        }

        // If tool_prefix is not set, default to empty string
        let tool_prefix = settings.tool_prefix.unwrap_or_default();

        Ok(Command {
            command: zed::node_binary_path()?,
            args: vec![env::current_dir()
                .unwrap()
                .join(SERVER_PATH)
                .to_string_lossy()
                .to_string()],
            env: vec![
                ("LINEAR_ACCESS_TOKEN".into(), settings.linear_api_key),
                ("TOOL_PREFIX".into(), tool_prefix),
            ],
        })
    }
}

zed::register_extension!(LinearModelContextExtension);
