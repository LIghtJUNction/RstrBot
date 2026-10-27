use crate::PluginError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// An administrator-installed package. A workspace instance is a separate object.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub id: String,
    pub version: String,
    pub executable: PathBuf,
    #[serde(default)]
    pub args: Vec<String>,
    /// Protocol domains, such as `model`, `tools`, `platform`, or `events`.
    pub capabilities: Vec<String>,
    /// Explicit disclosure, not an OS sandbox or an authorization substitute.
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub timeouts: Timeouts,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Timeouts {
    pub handshake_ms: u64,
    pub call_ms: u64,
}

impl Default for Timeouts {
    fn default() -> Self {
        Self {
            handshake_ms: 5_000,
            call_ms: 120_000,
        }
    }
}

/// Validate a manifest before it is persisted or used to start an executable.
/// Registration still requires host administrator approval in the core.
pub fn validate_manifest(manifest: &Manifest) -> Result<(), PluginError> {
    let valid_id = !manifest.id.is_empty()
        && manifest.id.len() <= 100
        && manifest
            .id
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"-_.".contains(&c));
    if !valid_id || manifest.version.is_empty() || manifest.version.len() > 100 {
        return Err(PluginError::InvalidManifest("invalid id or version".into()));
    }
    if !manifest.executable.is_absolute()
        || manifest.args.len() > 32
        || manifest
            .args
            .iter()
            .any(|a| a.len() > 4096 || a.contains('\0'))
    {
        return Err(PluginError::InvalidManifest(
            "executable must be absolute; arguments must be bounded".into(),
        ));
    }
    const CAPABILITIES: &[&str] = &[
        "model",
        "tools",
        "platform",
        "events",
        "tasks",
        "knowledge",
        "runtime",
    ];
    if manifest.capabilities.is_empty()
        || manifest.capabilities.len() > 16
        || manifest
            .capabilities
            .iter()
            .any(|c| !CAPABILITIES.contains(&c.as_str()))
    {
        return Err(PluginError::InvalidManifest(
            "unknown or missing capability".into(),
        ));
    }
    if manifest.permissions.len() > 32 || manifest.permissions.iter().any(|p| p.len() > 200) {
        return Err(PluginError::InvalidManifest(
            "permission disclosure exceeds limits".into(),
        ));
    }
    if let Some(digest) = &manifest.sha256
        && (digest.len() != 64 || !digest.bytes().all(|c| c.is_ascii_hexdigit()))
    {
        return Err(PluginError::InvalidManifest(
            "sha256 must contain 64 hexadecimal digits".into(),
        ));
    }
    if !(100..=30_000).contains(&manifest.timeouts.handshake_ms)
        || !(100..=300_000).contains(&manifest.timeouts.call_ms)
    {
        return Err(PluginError::InvalidManifest(
            "timeouts outside supported bounds".into(),
        ));
    }
    Ok(())
}

impl Manifest {
    /// Check the declared domain before any child process is started.
    pub fn permits(&self, method: &str) -> bool {
        method.split_once('.').is_some_and(|(domain, action)| {
            !action.is_empty()
                && action.len() < 100
                && action
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'_')
                && self.capabilities.iter().any(|c| c == domain)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_relative_executables_and_unknown_capabilities() {
        let mut manifest = Manifest {
            id: "test".into(),
            version: "1".into(),
            executable: "./evil".into(),
            args: vec![],
            capabilities: vec!["model".into()],
            permissions: vec![],
            sha256: None,
            timeouts: Timeouts::default(),
        };
        assert!(validate_manifest(&manifest).is_err());
        manifest.executable = std::env::current_exe().expect("test executable");
        assert!(validate_manifest(&manifest).is_ok());
        assert!(manifest.permits("model.generate"));
        assert!(!manifest.permits("tools.call"));
        manifest.capabilities.push("core.admin".into());
        assert!(validate_manifest(&manifest).is_err());
    }
}
