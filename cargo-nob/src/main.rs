use clap::Parser;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

const KERNEL_PATH: &'static str =
    "kernel/target/aarch64-unknown-lucy/debug/lucy";

#[derive(clap::Parser)]
struct Args {
    cargo_cmd: String,
    #[clap(subcommand)]
    cmd: Option<SubCmd>,
}

#[derive(clap::Subcommand, Default)]
enum SubCmd {
    #[default]
    Build,
    Run,
    Clean,
}

fn build_boot_code() {
    fs::create_dir_all("kernel/target/").unwrap();

    Command::new("aarch64-linux-gnu-as")
        .args(vec![
            "-march=armv8-a",
            "-W",
            "kernel/src/boot.S",
            "-o",
            "kernel/target/boot.o",
        ])
        .status()
        .unwrap();

    Command::new("ar")
        .args(vec![
            "rvs",
            "kernel/target/libboot.a",
            "kernel/target/boot.o",
        ])
        .status()
        .unwrap();
}

fn build_kernel() {
    Command::new("cargo")
        .args(vec![
            "+nightly",
            "-Z",
            "unstable-options",
            "-C",
            "kernel",
            "build",
        ])
        .status()
        .unwrap();
}

fn build() {
    build_boot_code();
    build_kernel();
}

fn run() {
    if !Path::new(KERNEL_PATH).exists() {
        build();
    }

    Command::new("qemu-system-aarch64")
        .args(vec![
            "-M",
            "virt",
            "-cpu",
            "cortex-a72",
            "-m",
            "1G",
            "-kernel",
            KERNEL_PATH,
            "-nographic",
            "-smp",
            "4",
            "-monitor",
            "tcp:localhost:8888,server,nowait",
            "-serial",
            "stdio",
            "-d",
            "guest_errors,unimp",
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .unwrap();
}

fn clean() {
    Command::new("cargo")
        .args(vec![
            "+nightly",
            "-Z",
            "unstable-options",
            "-C",
            "kernel",
            "clean",
        ])
        .status()
        .unwrap();
}

fn main() {
    let args = Args::parse();
    match args.cmd.unwrap_or_default() {
        SubCmd::Build => build(),
        SubCmd::Run => run(),
        SubCmd::Clean => clean(),
    }
}
