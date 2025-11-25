#[cfg(target_os = "linux")]
use nix::sched::{unshare, CloneFlags};
use anyhow::Result;

/// Available namespace types for container isolation
#[derive(Debug, Clone, Copy)]
pub enum NamespaceType {
    /// Mount namespace - isolate filesystem mount points
    Mount,
    /// UTS namespace - isolate hostname and domain name
    Uts,
    /// IPC namespace - isolate System V IPC and POSIX message queues
    Ipc,
    /// PID namespace - isolate process IDs
    Pid,
    /// Network namespace - isolate network devices, stacks, ports
    Network,
    /// User namespace - isolate user and group IDs
    User,
    /// Cgroup namespace - isolate cgroup root directory
    Cgroup,
}

#[cfg(target_os = "linux")]
impl NamespaceType {
    /// Convert to nix CloneFlags
    pub fn to_clone_flag(&self) -> CloneFlags {
        match self {
            NamespaceType::Mount => CloneFlags::CLONE_NEWNS,
            NamespaceType::Uts => CloneFlags::CLONE_NEWUTS,
            NamespaceType::Ipc => CloneFlags::CLONE_NEWIPC,
            NamespaceType::Pid => CloneFlags::CLONE_NEWPID,
            NamespaceType::Network => CloneFlags::CLONE_NEWNET,
            NamespaceType::User => CloneFlags::CLONE_NEWUSER,
            NamespaceType::Cgroup => CloneFlags::CLONE_NEWCGROUP,
        }
    }
}

/// Container namespace configuration
#[derive(Debug, Default)]
pub struct NamespaceConfig {
    pub mount: bool,
    pub uts: bool,
    pub ipc: bool,
    pub pid: bool,
    pub network: bool,
    pub user: bool,
    pub cgroup: bool,
}

impl NamespaceConfig {
    /// Create a new namespace config with common container defaults
    pub fn container_default() -> Self {
        Self {
            mount: true,
            uts: true,
            ipc: true,
            pid: true,
            network: false, // Often shared with host
            user: false,    // Requires additional setup
            cgroup: false,  // Optional
        }
    }

    /// Get list of enabled namespaces
    pub fn enabled_namespaces(&self) -> Vec<NamespaceType> {
        let mut ns = Vec::new();
        if self.mount { ns.push(NamespaceType::Mount); }
        if self.uts { ns.push(NamespaceType::Uts); }
        if self.ipc { ns.push(NamespaceType::Ipc); }
        if self.pid { ns.push(NamespaceType::Pid); }
        if self.network { ns.push(NamespaceType::Network); }
        if self.user { ns.push(NamespaceType::User); }
        if self.cgroup { ns.push(NamespaceType::Cgroup); }
        ns
    }
}

/// Unshare namespaces based on configuration
#[cfg(target_os = "linux")]
pub fn setup_namespaces(config: &NamespaceConfig) -> Result<()> {
    for ns in config.enabled_namespaces() {
        unshare(ns.to_clone_flag())?;
    }
    Ok(())
}

/// Non-Linux Unix systems have limited namespace support
#[cfg(all(unix, not(target_os = "linux")))]
pub fn setup_namespaces(_config: &NamespaceConfig) -> Result<()> {
    // macOS and BSD don't have Linux namespaces
    // Container isolation will be limited to chroot
    tracing::warn!("Namespace isolation not available on this platform");
    Ok(())
}
