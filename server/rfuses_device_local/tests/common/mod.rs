use assert_fs::{fixture::ChildPath, prelude::PathChild};
use etcetera::BaseStrategy;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use regex::Regex;
use std::{
    borrow::BorrowMut,
    path::PathBuf,
    process::{ExitStatus, Output, Stdio},
};
use tokio::{process::Command, sync::mpsc};

#[allow(dead_code)] // Macro and test context only, don't use directly.
pub const INSTA_FILTERS: &[(&str, &str)] = &[
    // Operation times
    (r"(\s|\()(\d+m )?(\d+\.)?\d+(ms|s)", "$1[TIME]"),
    // File sizes
    (r"(\s|\()(\d+\.)?\d+([KM]i)?B", "$1[SIZE]"),
];

pub struct TestContext {
    #[allow(dead_code)]
    pub mount_dir: ChildPath,
    #[allow(dead_code)]
    pub origin_dir: ChildPath,
    // pub workspace_root: PathBuf,
    #[allow(dead_code)]
    _root: tempfile::TempDir,
}

impl TestContext {
    pub fn new() -> Self {
        let bucket = etcetera::base_strategy::choose_base_strategy()
            .expect("Failed to find base strategy")
            .data_dir()
            .join("rfuses")
            .join("tests");
        fs_err::create_dir_all(&bucket).expect("Failed to create test bucket");
        let root = tempfile::TempDir::new_in(bucket).expect("Failed to create test root directory");

        let mount_dir = ChildPath::new(root.path()).child("mount_dir");
        fs_err::create_dir_all(&mount_dir).expect("Failed to create mount_dir working directory");

        let origin_dir = ChildPath::new(root.path()).child("origin_dir");
        fs_err::create_dir_all(&origin_dir).expect("Failed to create origin_dir cache directory");

        Self {
            mount_dir,
            origin_dir,
            // workspace_root: todo!(),
            _root: root,
        }
    }

    #[allow(dead_code)]
    /// Create a `rfusers_device_local help` command with options shared across scenarios.
    pub fn help(&self) -> Command {
        let mut command = Command::new(get_bin());
        command.arg("help");
        command
    }

    #[allow(dead_code)]
    /// Create a rfusers_device_local command for testing.
    pub fn command(&self) -> Command {
        Command::new(get_bin())
    }

    #[allow(dead_code)]
    /// Create a `rfusers_device_local link` command with options shared across scenarios.
    pub fn link(&self) -> Command {
        let mut command = Command::new(get_bin());
        command.arg("link");
        command
    }
}

/// Returns the rfusers_device_local binary that cargo built before launching the tests.
///
/// <https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates>
/// <https://github.com/nextest-rs/nextest/issues/917>
pub fn get_bin() -> PathBuf {
    let my_app = std::env::var("OVERRIDE_CARGO_BIN_EXE_rfuses_device_local")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rfuses_device_local").to_owned());
    PathBuf::from(my_app)
}

#[allow(dead_code)]
/// Execute the command and format its output status, stdout and stderr into a snapshot string.
pub async fn run_command<T: AsRef<str>>(
    command: impl BorrowMut<Command>,
    filters: impl AsRef<[(T, T)]>,
    rx: Option<mpsc::Receiver<()>>,
) -> (String, Output) {
    let (snapshot, output, _) = run_command_with_status(command, filters, rx).await;
    (snapshot, output)
}

pub async fn run_command_with_status<T: AsRef<str>>(
    mut command: impl BorrowMut<Command>,
    filters: impl AsRef<[(T, T)]>,
    rx: Option<mpsc::Receiver<()>>,
) -> (String, Output, ExitStatus) {
    // TODO: add tracing-durations-export
    let program = command
        .borrow_mut()
        .as_std()
        .get_program()
        .to_string_lossy()
        .to_string();

    let child = command
        .borrow_mut()
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .unwrap_or_else(|err| panic!("Failed to spawn {program}: {err}"));

    if let Some(mut rx) = rx {
        rx.recv().await;
        // 获取子进程的 PID
        let pid = child.id().expect("Failed to get child PID");
        // 向子进程发送 SIGINT (Ctrl+C)
        signal::kill(Pid::from_raw(pid as i32), Signal::SIGINT).expect("Failed to send SIGINT");
    }
    let output = child.wait_with_output().await.unwrap_or_else(|err| {
        panic!("Failed to wait for {program} to finish: {err}");
    });

    let mut snapshot = format!(
        "success: {:?}\nexit_code: {}\n----- stdout -----\n{}\n----- stderr -----\n{}",
        output.status.success(),
        output.status.code().unwrap_or(!0),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    for (matcher, replacement) in filters.as_ref() {
        let re = Regex::new(matcher.as_ref()).expect("Do you need to regex::escape your filter?");
        if re.is_match(&snapshot) {
            snapshot = re.replace_all(&snapshot, replacement.as_ref()).to_string();
        }
    }

    let status = output.status;
    (snapshot, output, status)
}

/// Run [`assert_cmd_snapshot!`], with default filters or with custom filters.
///
/// filter them out and decrease the package counts by one for each match.
#[allow(unused_macros)]
macro_rules! rfuses_snapshot {
    ($spawnable:expr, @$snapshot:literal) => {{
        rfuses_snapshot!($crate::common::INSTA_FILTERS.to_vec(), $spawnable, @$snapshot)
    }};
    ($filters:expr, $spawnable:expr, @$snapshot:literal) => {{
        let (snapshot, output) = $crate::common::run_command($spawnable, $filters, None).await;
        ::insta::assert_snapshot!(snapshot, @$snapshot);
        output
    }};
    ($filters:expr,$spawnable:expr, $rx:expr, @$snapshot:literal) => {{
        let (snapshot, output) = $crate::common::run_command($spawnable, $filters, rx).await;
        ::insta::assert_snapshot!(snapshot, @$snapshot);
        output
    }};
}

#[allow(unused_macros)]
macro_rules! rfuses_spawn_run {
    ($cmd:expr, $func:expr) => {{
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let handle = tokio::spawn(async move {
            let (_, _, status) =
                run_command_with_status($cmd, Vec::<(String, String)>::new(), Some(rx)).await;
            assert!(status.success());
        });

        // 等待fuse完全启动
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // 运行测试代码
        $func();

        // 通知子进程发送 SIGINT 信号
        tx.send(()).await.unwrap();
        handle.await.unwrap();
    }};
}

/// <https://stackoverflow.com/a/31749071/3549270>
#[allow(unused_imports)]
pub(crate) use rfuses_snapshot;
#[allow(unused_imports)]
pub(crate) use rfuses_spawn_run;
