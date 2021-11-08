#![deny(missing_docs)]

//! Create [`Command`]s for running Docker or Docker-compatible clients.
//!
//! Rather than speaking directly to the Docker daemon, this library
//! produces commands that can be run in a subprocess to invoke the
//! Docker client (or a compatible client such as Podman).
//!
//! [`Command`]: https://docs.rs/command-run/latest/command_run/struct.Command.html

pub use command_run;

use command_run::Command;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::{env, fmt};

/// Preset base commands that a [`Launcher`] can be constructed from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaseCommand {
    /// Docker without sudo.
    Docker,

    /// Docker with sudo.
    SudoDocker,

    /// Podman.
    Podman,
}

// TODO: is there a good existing crate for this? I found a few on
// crates.io that didn't look quite right.
fn is_exe_in_path(exe_name: &OsStr) -> bool {
    let paths = if let Some(paths) = env::var_os("PATH") {
        paths
    } else {
        return false;
    };

    env::split_paths(&paths).any(|path| path.join(exe_name).exists())
}

// TODO: consider using nix or some other crate.
fn is_user_in_group(target_group: &str) -> bool {
    let mut cmd = Command::new("groups");
    cmd.log_command = false;
    cmd.capture = true;
    cmd.log_output_on_error = true;
    let output = if let Ok(output) = cmd.run() {
        output
    } else {
        return false;
    };
    let stdout = output.stdout_string_lossy();
    stdout.split_whitespace().any(|group| group == target_group)
}

/// Base container command used for building and running containers.
///
/// This allows variations such as "docker", "sudo docker", and
/// "podman".
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Launcher {
    /// If true, run the command with `sudo`. Defaults to false.
    pub sudo: bool,

    /// The container command. The default is "docker", but can be
    /// changed to another compatible program such as "podman".
    pub program: PathBuf,
}

impl Launcher {
    /// Create a new Docker instance with the default values set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Automatically choose a base command.
    ///
    /// * Chooses `podman` if is in the `$PATH`.
    /// * Otherwise chooses `docker` if it is in the `$PATH`.
    ///   * If the current user is not in a `docker` group, `sudo` is added.
    ///
    /// If neither command is in the `$PATH`, returns `None`.
    pub fn auto() -> Option<Self> {
        let docker = OsStr::new("docker");
        let podman = OsStr::new("podman");
        if is_exe_in_path(podman) {
            Some(BaseCommand::Podman.into())
        } else if is_exe_in_path(docker) {
            Some(if is_user_in_group("docker") {
                BaseCommand::Docker.into()
            } else {
                BaseCommand::SudoDocker.into()
            })
        } else {
            None
        }
    }

    /// Create the base [`Command`] for running Docker.
    pub fn command(&self) -> Command {
        if self.sudo {
            Command::with_args("sudo", &[&self.program])
        } else {
            Command::new(&self.program)
        }
    }

    /// Create a [`Command`] for building a container.
    pub fn build(&self, opt: BuildOpt) -> Command {
        let mut cmd = self.command();
        cmd.add_arg("build");

        // --build-arg
        for (key, value) in opt.build_args {
            cmd.add_arg_pair("--build-arg", format!("{}={}", key, value));
        }

        // --file
        if let Some(dockerfile) = &opt.dockerfile {
            cmd.add_arg_pair("--file", dockerfile);
        }

        // --no-cache
        if opt.no_cache {
            cmd.add_arg("--no-cache");
        }

        // --pull
        if opt.pull {
            cmd.add_arg("--pull");
        }

        // --quiet
        if opt.quiet {
            cmd.add_arg("--quiet");
        }

        // --tag
        if let Some(tag) = &opt.tag {
            cmd.add_arg_pair("--tag", tag);
        }

        cmd.add_arg(opt.context);
        cmd
    }

    /// Create a [`Command`] for creating a network.
    pub fn create_network(&self, opt: CreateNetworkOpt) -> Command {
        let mut cmd = self.command();
        cmd.add_arg_pair("network", "create");
        cmd.add_arg(opt.name);

        cmd
    }

    /// Create a [`Command`] for removing a network.
    pub fn remove_network(&self, name: &str) -> Command {
        let mut cmd = self.command();
        cmd.add_arg_pair("network", "rm");
        cmd.add_arg(name);

        cmd
    }

    /// Create a [`Command`] for running a container.
    pub fn run(&self, opt: RunOpt) -> Command {
        let mut cmd = self.command();
        cmd.add_arg("run");

        // --detach
        if opt.detach {
            cmd.add_arg("--detach");
        }

        // --env
        for (key, value) in &opt.env {
            let mut arg = OsString::new();
            arg.push(key);
            arg.push("=");
            arg.push(value);
            cmd.add_arg_pair("--env", arg);
        }

        // --init
        if opt.init {
            cmd.add_arg("--init");
        }

        // --name
        if let Some(name) = &opt.name {
            cmd.add_arg_pair("--name", name);
        }

        // --network
        if let Some(network) = &opt.network {
            cmd.add_arg_pair("--network", network);
        }

        // --read-only
        if opt.read_only {
            cmd.add_arg("--read-only");
        }

        // --rm
        if opt.remove {
            cmd.add_arg("--rm");
        }

        // --user
        if let Some(user) = &opt.user {
            cmd.add_arg_pair("--user", user.arg());
        }

        // --volume
        for vol in &opt.volumes {
            cmd.add_arg_pair("--volume", vol.arg());
        }

        // Add image and command+args
        cmd.add_arg(opt.image);
        if let Some(command) = &opt.command {
            cmd.add_arg(command);
        }
        cmd.add_args(&opt.args);
        cmd
    }
}

impl From<BaseCommand> for Launcher {
    fn from(bc: BaseCommand) -> Launcher {
        let docker = "docker";
        let podman = "podman";
        match bc {
            BaseCommand::Docker => Self {
                sudo: false,
                program: docker.into(),
            },
            BaseCommand::SudoDocker => Self {
                sudo: true,
                program: docker.into(),
            },
            BaseCommand::Podman => Self {
                sudo: false,
                program: podman.into(),
            },
        }
    }
}

impl Default for Launcher {
    fn default() -> Self {
        Self {
            sudo: false,
            program: Path::new("docker").into(),
        }
    }
}

/// Options for building a container.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BuildOpt {
    /// Build-time variables.
    pub build_args: Vec<(String, String)>,

    /// Root directory containing files that can be pulled into the
    /// container.
    pub context: PathBuf,

    /// Dockerfile to build. This must be somewhere in the `context`
    /// directory. If not set (the default) then
    /// `<context>/Dockerfile` is used.
    pub dockerfile: Option<PathBuf>,

    /// Do not use cache when building the image.
    pub no_cache: bool,

    /// Always attempt to pull a newer version of the image.
    pub pull: bool,

    /// Suppress the build output and print image ID on success.
    pub quiet: bool,

    /// If set, the image will be tagged with this name.
    pub tag: Option<String>,
}

/// Options for creating a network.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CreateNetworkOpt {
    /// Network name.
    pub name: String,
}

/// Name or numeric ID for a user or group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NameOrId {
    /// Name or the user or group.
    Name(String),

    /// Numeric ID of the user or group.
    Id(u32),
}

impl fmt::Display for NameOrId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Name(name) => write!(f, "{}", name),
            Self::Id(id) => write!(f, "{}", id),
        }
    }
}

impl From<String> for NameOrId {
    fn from(name: String) -> Self {
        Self::Name(name)
    }
}

impl From<u32> for NameOrId {
    fn from(id: u32) -> Self {
        Self::Id(id)
    }
}

/// User and (optionally) group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserAndGroup {
    /// User or UID.
    pub user: NameOrId,

    /// Group or GID.
    pub group: Option<NameOrId>,
}

impl UserAndGroup {
    /// Get a `UserAndGroup` with the current UID and GID set.
    pub fn current() -> Self {
        Self {
            user: users::get_current_uid().into(),
            group: Some(users::get_current_gid().into()),
        }
    }

    /// Get a `UserAndGroup` with UID and GID set to zero.
    pub fn root() -> Self {
        Self {
            user: 0.into(),
            group: Some(0.into()),
        }
    }

    /// Format as an argument. If `group` is set, the format is
    /// `<user>:<group>`, otherwise just `<user>`.
    pub fn arg(&self) -> String {
        let mut out = self.user.to_string();
        if let Some(group) = &self.group {
            out.push(':');
            out.push_str(&group.to_string());
        }
        out
    }
}

/// Volume specification used when running a container.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Volume {
    /// Either a path on the host (if an absolute path) or the name of
    /// a volume.
    pub src: PathBuf,

    /// Absolute path in the container where the volume will be
    /// mounted.
    pub dst: PathBuf,

    /// If true, mount the volume read-write. Defaults to `false`.
    pub read_write: bool,

    /// Additional options to set on the volume.
    pub options: Vec<String>,
}

impl Volume {
    /// Format as an argument.
    pub fn arg(&self) -> OsString {
        let mut out = OsString::new();
        out.push(&self.src);
        out.push(":");
        out.push(&self.dst);
        if self.read_write {
            out.push(":rw");
        } else {
            out.push(":ro");
        }
        for opt in &self.options {
            out.push(",");
            out.push(opt);
        }
        out
    }
}

/// Options for running a container.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RunOpt {
    /// Container image to run.
    pub image: String,

    /// Set environment variables.
    pub env: Vec<(OsString, OsString)>,

    /// If true, run the container in the background and print
    /// container ID. Defaults to `false`.
    pub detach: bool,

    /// Run an init inside the container that forwards signals and
    /// reaps processes.
    pub init: bool,

    /// Optional name to give the container.
    pub name: Option<String>,

    /// Connect a container to a network.
    pub network: Option<String>,

    /// User (and optionally) group to use inside the container.
    pub user: Option<UserAndGroup>,

    /// Mount the container's root filesystem as read only.
    pub read_only: bool,

    /// If true, automatically remove the container when it
    /// exits. Defaults to `false`.
    pub remove: bool,

    /// Volumes to mount in the container.
    pub volumes: Vec<Volume>,

    /// Optional command to run.
    pub command: Option<PathBuf>,

    /// Optional arguments to pass to the command.
    pub args: Vec<OsString>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command() {
        let mut launcher = Launcher::new();
        assert_eq!(launcher.command().command_line_lossy(), "docker");

        launcher.sudo = true;
        assert_eq!(launcher.command().command_line_lossy(), "sudo docker");

        launcher.program = "myCommand".into();
        assert_eq!(launcher.command().command_line_lossy(), "sudo myCommand");
    }
}
