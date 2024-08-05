#[cfg(debug_assertions)]
pub fn init_log() {
    use env_logger;
    env_logger::init();
}

#[cfg(not(debug_assertions))]
pub fn init_log() {
    use flexi_logger::{Cleanup, Criterion, FileSpec, Logger, Naming, WriteMode};
    Logger::try_with_str("info")
        .unwrap()
        .log_to_file(FileSpec::default().directory("./logs").suppress_timestamp()) // 日志文件目录
        .write_mode(WriteMode::BufferAndFlush) // 有缓存的写入模式
        // .write_mode(WriteMode::Direct) // 直接写入模式
        .rotate(
            Criterion::Size(10 * 1024 * 1024), // 每个文件 10M
            Naming::Numbers,
            Cleanup::KeepLogFiles(7),
        )
        .start()
        .unwrap();
}
