mod base_fn;

use base_fn::self_criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkId, Criterion, Throughput,
};
use common::{rfuses_spawn_run, run_command_with_status, TestContext};
use rand::Rng;
use std::{
    fs::{self, File},
    io::{Read, Write},
};
use tokio::runtime::Runtime;
#[path = "../tests/common/mod.rs"]
mod common;

// const FILE_SIZE: usize = 1024 * 1024 * 10; // 10 MB
const FILE_SIZE: usize = 1024 * 1024;

fn benchmark_file_continuous_current_thread(c: &mut Criterion<WallTime>) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    benchmark_file_continuous(c, rt);
}

fn benchmark_file_continuous(c: &mut Criterion<WallTime>, rt: Runtime) {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let mut group = c.benchmark_group("file_continuous");

    let closure = || {
        // 创建文件
        let mut test_file_mount = mount_path.clone();
        test_file_mount.push(format!(
            "benchmark_test_file_{}.txt",
            rand::thread_rng().gen::<u64>()
        ));
        File::create(&test_file_mount).unwrap();
        // 创建随机数据
        let mut data = vec![0u8; FILE_SIZE];
        rand::thread_rng().fill(&mut data[..]);

        group.throughput(Throughput::Bytes(data.len() as u64));
        group.bench_function(BenchmarkId::from_parameter("write"), |b| {
            b.iter(|| {
                let mut f = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(&test_file_mount)
                    .unwrap();
                f.write_all(&data).unwrap();
                f.sync_data().unwrap();
            });
        });

        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&test_file_mount)
            .unwrap();
        file.write_all(&data).unwrap();
        file.flush().unwrap();

        group.bench_function(BenchmarkId::from_parameter("read"), |b| {
            b.iter(|| {
                let mut f = fs::OpenOptions::new()
                    .read(true)
                    .open(&test_file_mount)
                    .unwrap();
                f.read_exact(&mut data[..]).unwrap();
            });
        });
        group.finish();
    };

    rt.block_on(async {
        rfuses_spawn_run!(
            {
                context
                    .link()
                    .arg(context.origin_dir.path())
                    .arg(context.mount_dir.path())
            },
            closure
        );
    });
}

criterion_group!(
    continuous_operation,
    benchmark_file_continuous_current_thread
);
criterion_main!(continuous_operation);
