use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use anyhow::{Context, Result};

const CGROUP_ROOT: &str = "/sys/fs/cgroup";

/// Resource limits for a container
#[derive(Debug, Default, Clone)]
pub struct ResourceLimits {
    /// Memory limit in bytes
    pub memory_limit: Option<u64>,
    /// CPU weight (1-10000, default 100)
    pub cpu_weight: Option<u32>,
    /// Maximum number of PIDs
    pub pids_max: Option<u64>,
}

/// Cgroup controller for managing container resources
pub struct Cgroup {
    name: String,
    path: PathBuf,
}

impl Cgroup {
    /// Create a new cgroup with the given name
    pub fn new(name: &str) -> Result<Self> {
        let path = PathBuf::from(CGROUP_ROOT).join(name);
        
        if !path.exists() {
            fs::create_dir_all(&path)
                .context("Failed to create cgroup directory")?;
        }

        Ok(Self {
            name: name.to_string(),
            path,
        })
    }

    /// Add a process to this cgroup
    pub fn add_process(&self, pid: u32) -> Result<()> {
        let procs_file = self.path.join("cgroup.procs");
        let mut file = File::create(&procs_file)
            .context("Failed to open cgroup.procs")?;
        
        writeln!(file, "{}", pid)
            .context("Failed to write PID to cgroup")?;
        
        Ok(())
    }

    /// Set memory limit for this cgroup
    pub fn set_memory_limit(&self, bytes: u64) -> Result<()> {
        let memory_max = self.path.join("memory.max");
        fs::write(&memory_max, bytes.to_string())
            .context("Failed to set memory limit")?;
        
        Ok(())
    }

    /// Set CPU weight for this cgroup (1-10000)
    pub fn set_cpu_weight(&self, weight: u32) -> Result<()> {
        let cpu_weight = self.path.join("cpu.weight");
        fs::write(&cpu_weight, weight.to_string())
            .context("Failed to set CPU weight")?;
        
        Ok(())
    }

    /// Set maximum number of PIDs
    pub fn set_pids_max(&self, max: u64) -> Result<()> {
        let pids_max = self.path.join("pids.max");
        fs::write(&pids_max, max.to_string())
            .context("Failed to set pids limit")?;
        
        Ok(())
    }

    /// Apply resource limits
    pub fn apply_limits(&self, limits: &ResourceLimits) -> Result<()> {
        if let Some(mem) = limits.memory_limit {
            self.set_memory_limit(mem)?;
        }
        if let Some(weight) = limits.cpu_weight {
            self.set_cpu_weight(weight)?;
        }
        if let Some(pids) = limits.pids_max {
            self.set_pids_max(pids)?;
        }
        Ok(())
    }

    /// Get the path of this cgroup
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl Drop for Cgroup {
    fn drop(&mut self) {
        // Clean up cgroup on drop
        let _ = fs::remove_dir(&self.path);
    }
}
