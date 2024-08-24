use common::{rfuses_snapshot, TestContext};

mod common;

#[test]
fn help() {
    let context = TestContext::new();

    rfuses_snapshot!(context.help(), @r###"
success: true
exit_code: 0
----- stdout -----
rfuse: A fuse server.

Usage: rfuses_device_local [OPTIONS] <COMMAND>

Commands:
  link  A fuse server, similar to link.
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

Log levels:
  -v, --verbose  Enable verbose logging
  -q, --quiet    Print diagnostics, but nothing else
  -s, --silent   Disable all logging (but still exit with status code "1" upon detecting diagnostics)

For help with a specific command, see: `rfuses help <command>`.

----- stderr -----"###);
}

#[test]
fn help_link() {
    let context = TestContext::new();

    rfuses_snapshot!(context.help().arg("link"), @r###"
success: true
exit_code: 0
----- stdout -----
A fuse server, similar to link.

Usage: rfuses_device_local link [OPTIONS] <ORIGIN> <MOUNT> [FS_NAME]

Arguments:
  <ORIGIN>   Origin file address [default: .]
  <MOUNT>    Mount point address [default: .]
  [FS_NAME]  Set the name of the source in mtab. [default: rfuses]

Options:
  -r, --read-only  Read only
  -h, --help       Print help
  -V, --version    Print version

Disk types:
  -l, --local  Use local disk
  -m, --mem    Use memory as disk

Log levels:
  -v, --verbose  Enable verbose logging
  -q, --quiet    Print diagnostics, but nothing else
  -s, --silent   Disable all logging (but still exit with status code "1" upon detecting diagnostics)

----- stderr -----"###);
}
