use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

use clap::Parser;

const KERNEL_PATH: &'static str =
    "kernel/target/aarch64-unknown-lucy/debug/lucy";

#[derive(clap::Parser)]
struct Args {
    cargo_cmd: String,
    #[clap(subcommand)]
    cmd: Option<SubCmd>,
}

#[derive(clap::Subcommand)]
enum SubCmd {
    Build {
        #[clap(long, action)]
        hardware: bool,
    },
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

    Command::new("aarch64-linux-gnu-ar")
        .args(vec![
            "rvs",
            "kernel/target/libboot.a",
            "kernel/target/boot.o",
        ])
        .status()
        .unwrap();
}

fn build_kernel(hardware: bool) {
    let mut cmd = Command::new("cargo");
    cmd.args(vec![
        "+nightly",
        "-Z",
        "unstable-options",
        "-C",
        "kernel",
        "build",
    ]);
    if hardware {
        cmd.arg("--features hardware");
    }
    cmd.status().unwrap();
}

fn build(hardware: bool) {
    build_boot_code();
    build_kernel(hardware);
}

fn run() {
    if !Path::new(KERNEL_PATH).exists() {
        build(false);
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
            "-serial",
            "telnet:localhost:5000,server",
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
    match args.cmd.unwrap_or(SubCmd::Build { hardware: false }) {
        SubCmd::Build { hardware } => build(hardware),
        SubCmd::Run => run(),
        SubCmd::Clean => clean(),
    }
}
