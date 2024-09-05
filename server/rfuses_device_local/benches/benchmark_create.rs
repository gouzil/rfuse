// pub mod criterion {
//     //! This module re-exports the criterion API but picks the right backend depending on whether
//     //! the benchmarks are built to run locally or with codspeed

//     #[cfg(not(feature = "codspeed"))]
//     pub use criterion::*;

//     #[cfg(feature = "codspeed")]
//     pub use codspeed_criterion_compat::*;
// }

// criterion
use codspeed_criterion_compat::{criterion_group, criterion_main, Criterion};
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

fn benchmark_file_continuous_current_thread(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    benchmark_file_continuous(c, rt);
}

fn benchmark_file_continuous(c: &mut Criterion, rt: Runtime) {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let mut closure = || {
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

        let mut group = c.benchmark_group("file continuous operation");
        group.throughput(criterion::Throughput::Bytes(data.len() as u64));
        group.bench_function("write", |b| {
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

        group.bench_function("read", |b| {
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

criterion_group!(benches_create, benchmark_file_continuous_current_thread);
criterion_main!(benches_create);
