use chrono::Utc;
use common::{rfuses_spawn_run, run_command_with_status, TestContext};
use directories::ProjectDirs;

mod common;

#[tokio::test]
async fn test_logging_verbose() {
    let context = TestContext::new();
    let closure = || {
        if cfg!(debug_assertions) {
            let base_path = ProjectDirs::from("", "", "rfuse").unwrap();
            let mut log_dir_path = base_path.runtime_dir().unwrap().to_owned();
            log_dir_path.push("logs/");
            let log_file =
                log_dir_path.join(format!("{}-rfuses.log", Utc::now().format("%Y-%m-%d")));
            assert!(log_dir_path.is_dir());
            assert!(log_file.is_file());

            let log_content = std::fs::read_to_string(log_file).unwrap();
            assert!(log_content.contains("INFO"));
            assert!(log_content.contains("DEBUG"));
            assert!(log_content.contains("File system init success"));
        }
    };
    rfuses_spawn_run!(
        {
            context
                .link()
                .arg(context.origin_dir.path())
                .arg(context.mount_dir.path())
                .arg("-v")
        },
        closure
    );
}

#[tokio::test]
async fn test_logging_silent() {
    let context = TestContext::new();
    let closure = || {};
    rfuses_spawn_run!(
        {
            context
                .link()
                .arg(context.origin_dir.path())
                .arg(context.mount_dir.path())
                .arg("-s")
        },
        closure
    );
}

#[tokio::test]
async fn test_logging_quiet() {
    let context = TestContext::new();
    let closure = || {};
    rfuses_spawn_run!(
        {
            context
                .link()
                .arg(context.origin_dir.path())
                .arg(context.mount_dir.path())
                .arg("-q")
        },
        closure
    );
}
