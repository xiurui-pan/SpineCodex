use crate::config::ManagedFeatures;
use codex_config::spine_snapshot::SpineConfigLockToml;
use codex_config::spine_snapshot::SpineConfigSourceLockToml;
use codex_features::Feature as CodexFeature;
use codex_utils_absolute_path::AbsolutePathBuf;
use spine_core::host::RecordDigest;
use spine_core::host::SpineConfig;
use spine_core::host::SpineConfigLoader;
use spine_core::host::ToolCatalog;
use std::io;
use std::path::Path;

pub(crate) fn load(
    path: Option<&AbsolutePathBuf>,
    snapshot: Option<&SpineConfigLockToml>,
    working_directory: &Path,
    home_directory: Option<&Path>,
    enabled_features: &ManagedFeatures,
    project_config_trusted: bool,
) -> std::io::Result<(SpineConfig, ToolCatalog)> {
    let loader = loader(
        path,
        working_directory,
        home_directory,
        project_config_trusted,
    );
    let mut features = Vec::new();
    if enabled_features.enabled(CodexFeature::SpineJit) {
        features.push(spine_core::host::Feature::Jit);
    }
    if enabled_features.enabled(CodexFeature::SpineSpawn) {
        features.push(spine_core::host::Feature::Spawn);
    }
    let config = match snapshot {
        Some(snapshot) => {
            let effective = snapshot.effective_config.as_deref().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Spine snapshot is missing effective config"))?;
            SpineConfig::parse_toml(effective).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
        }
        None => loader.load().map_err(io::Error::from)?,
    }.with_features(features)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    let tools = ToolCatalog::new(&config)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    Ok((config, tools))
}

pub(crate) fn lock_snapshot(
    path: Option<&AbsolutePathBuf>,
    working_directory: &Path,
    home_directory: Option<&Path>,
    project_config_trusted: bool,
) -> io::Result<SpineConfigLockToml> {
    let loader = loader(
        path,
        working_directory,
        home_directory,
        project_config_trusted,
    );
    let mut sources = loader
        .optional_source_files()
        .into_iter()
        .map(|path| {
            let digest = match std::fs::read(&path) {
                Ok(contents) => Some(RecordDigest::digest(&contents).as_str().to_string()),
                Err(error) if error.kind() == io::ErrorKind::NotFound => None,
                Err(error) => {
                    return Err(io::Error::new(
                        error.kind(),
                        format!(
                            "failed to pin Spine config source {} in config lock: {error}",
                            path.display()
                        ),
                    ));
                }
            };
            Ok(SpineConfigSourceLockToml {
                path: AbsolutePathBuf::try_from(path)?,
                required: false,
                digest,
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    if let Some(path) = loader.required_source_file() {
        let contents = std::fs::read(&path).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!(
                    "failed to pin Spine config source {} in config lock: {error}",
                    path.display()
                ),
            )
        })?;
        sources.push(SpineConfigSourceLockToml {
            path: AbsolutePathBuf::try_from(path)?,
            required: true,
            digest: Some(RecordDigest::digest(&contents).as_str().to_string()),
        });
    }

    Ok(SpineConfigLockToml {
        schema_version: SpineConfig::v1().schema_version(),
        bundled_digest: RecordDigest::digest(spine_core::host::DEFAULT_CONFIG_TOML.as_bytes())
            .as_str()
            .to_string(),
        sources,
        effective_config: Some(loader.load().map_err(io::Error::from)?.snapshot_toml().map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?),
    })
}

fn loader(
    path: Option<&AbsolutePathBuf>,
    working_directory: &Path,
    home_directory: Option<&Path>,
    project_config_trusted: bool,
) -> SpineConfigLoader {
    let mut loader = SpineConfigLoader::new(working_directory);
    if !project_config_trusted {
        loader = loader.without_working_directory_layers();
    }
    if let Some(home_directory) = home_directory {
        loader = loader.with_home_directory(home_directory);
    }
    if let Some(path) = path {
        loader = loader.with_custom_path(path.as_path());
    }
    loader
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;

/// Reuse the SDK configuration captured at the effective transaction boundary on resume or fork.
pub(crate) fn restore_sampling_config(
    config: &mut crate::config::Config,
    history: &codex_history::InitialHistory,
) -> anyhow::Result<()> {
    let effective = super::effective_rollout(history.get_spine_rollout_items());
    let snapshot = effective.iter().rev().find_map(|(_, item)| match item {
        codex_history::RolloutItem::SpineSamplingStarted(started) => started.sdk_config.as_deref(),
        _ => None,
    });
    let sdk = match snapshot {
        Some(snapshot) => SpineConfig::parse_toml(snapshot)?,
        None => config.spine_config.clone(),
    };
    let mut features = Vec::new();
    if config.features.enabled(CodexFeature::SpineJit) { features.push(spine_core::host::Feature::Jit); }
    if config.features.enabled(CodexFeature::SpineSpawn) { features.push(spine_core::host::Feature::Spawn); }
    let sdk = sdk.with_features(features)?;
    config.spine_tools = ToolCatalog::new(&sdk)?;
    config.spine_config = sdk;
    Ok(())
}
