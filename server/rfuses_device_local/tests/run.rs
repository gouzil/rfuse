use common::{rfuses_spawn_run, run_command_with_status, TestContext};

mod common;

#[tokio::test]
async fn test_run_link() {
    let context = TestContext::new();
    let closure = || {};
    rfuses_spawn_run!(
        {
            context
                .link()
                .arg(context.origin_dir.path())
                .arg(context.mount_dir.path())
        },
        closure
    );
}
