use common::{rfuses_spawn_run, run_command_with_status, TestContext};

mod common;

#[tokio::test]
async fn test_logging_verbose() {
    let context = TestContext::new();
    let closure = || {};
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
