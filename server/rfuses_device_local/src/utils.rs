use std::time::{SystemTime, UNIX_EPOCH};

use nix::sys::time::{TimeSpec, TimeValLike};

pub fn system_time_to_timespec(system_time: SystemTime) -> TimeSpec {
    let duration_since_epoch = system_time.duration_since(UNIX_EPOCH).unwrap();
    TimeSpec::nanoseconds(duration_since_epoch.as_nanos() as i64)
}
