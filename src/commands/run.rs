use anyhow::Result;
use clap::Parser;
use std::process::Command;
use tracing::debug;

#[cfg(target_os = "linux")]
use nix::mount::{mount, umount, MsFlags};
#[cfg(target_os = "linux")]
use nix::sched::{unshare, CloneFlags};
#[cfg(unix)]
use nix::{sys, unistd};
#[cfg(unix)]
use std::os::unix::process::CommandExt;

#[derive(Parser, Debug)]
pub struct RunOpts {
    /// Specify the root directory path
    #[arg(long, short)]
    pub fsroot: String,
    /// Specify the path to the application to run
    pub app: String,
    /// Arguments to be passed to the app
    pub arguments: Vec<String>,
}

#[cfg(target_os = "linux")]
pub fn run(opts: RunOpts) -> Result<()> {
    debug!("host nodename = {:?}", sys::utsname::uname()?.nodename());

    unshare(CloneFlags::CLONE_NEWNS)?;
    unshare(CloneFlags::CLONE_NEWPID)?;
    unshare(CloneFlags::CLONE_NEWUTS)?;
    unistd::sethostname("container")?;

    debug!(
        "container nodename = {:?}",
        sys::utsname::uname()?.nodename()
    );

    let mut command = Command::new("/proc/self/exe");
    command.arg0("init").arg("start");
    command.args(["--fsroot", &opts.fsroot]);
    command.arg(&opts.app).args(opts.arguments);
    command.env("PATH", "/bin");

    let status = command.spawn()?.wait()?;
    debug!("container exited with status {:?}", status);

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn start(opts: RunOpts) -> Result<()> {
    unistd::chroot(&opts.fsroot[..])?;
    unistd::chdir("/")?;

    mount(
        None::<&str>,
        "/proc",
        Some("proc"),
        MsFlags::empty(),
        None::<&str>,
    )?;
    mount(
        None::<&str>,
        "/tmp",
        Some("tmpfs"),
        MsFlags::empty(),
        None::<&str>,
    )?;

    let mut command = Command::new(&opts.app);
    command.args(&opts.arguments);

    let status = command.spawn()?.wait()?;
    debug!("application exited with status {:?}", status);

    umount("/proc")?;
    umount("/tmp")?;

    Ok(())
}

// Unix fallback (macOS, BSD) - limited container support
#[cfg(all(unix, not(target_os = "linux")))]
pub fn run(opts: RunOpts) -> Result<()> {
    debug!("host nodename = {:?}", sys::utsname::uname()?.nodename());
    debug!("Running in limited mode (non-Linux Unix)");

    // On non-Linux Unix, we can only use chroot
    let mut command = Command::new("/proc/self/exe");
    command.arg0("init").arg("start");
    command.args(["--fsroot", &opts.fsroot]);
    command.arg(&opts.app).args(opts.arguments);
    command.env("PATH", "/bin");

    let status = command.spawn()?.wait()?;
    debug!("container exited with status {:?}", status);

    Ok(())
}

#[cfg(all(unix, not(target_os = "linux")))]
pub fn start(opts: RunOpts) -> Result<()> {
    unistd::chroot(&opts.fsroot[..])?;
    unistd::chdir("/")?;

    let mut command = Command::new(&opts.app);
    command.args(&opts.arguments);

    let status = command.spawn()?.wait()?;
    debug!("application exited with status {:?}", status);

    Ok(())
}

