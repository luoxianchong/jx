mod cli;
mod commands;
mod config;
mod dependency;
mod download;
mod install;
mod lock;
mod project;
mod registry;
mod resolve;
mod utils;

use clap::Parser;
use log::error;
use std::process;

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = cli::Args::parse();

    // 设置日志级别
    if args.verbose {
        std::env::set_var("RUST_LOG", "debug");
    } else if args.quiet {
        std::env::set_var("RUST_LOG", "error");
    }

    // 显示欢迎信息
    if !args.quiet {
        println!("🚀 jx - Fast Java Package Manager");
        println!("Built with Rust for speed and reliability");
        println!();
    }

    // 执行命令
    let result = execute_command(&args).await;

    // 处理结果
    match result {
        Ok(_) => {
            if !args.quiet {
                println!("✅ 操作完成");
            }
            process::exit(0);
        }
        Err(e) => {
            error!("操作失败: {}", e);
            if !args.quiet {
                eprintln!("❌ 错误: {}", e);
            }
            process::exit(1);
        }
    }
}

async fn execute_command(args: &cli::Args) -> anyhow::Result<()> {
    match &args.command {
        cli::Commands::Init(cmd) => cmd.execute(),
        cli::Commands::Install(cmd) => cmd.execute(),
        cli::Commands::Add(cmd) => cmd.execute(),
        cli::Commands::Remove(cmd) => cmd.execute(),
        cli::Commands::Update(cmd) => cmd.execute(),
        cli::Commands::Build(cmd) => cmd.execute(),
        cli::Commands::Run(cmd) => cmd.execute(),
        cli::Commands::Test(cmd) => cmd.execute(),
        cli::Commands::Clean => commands::clean::execute(),
        cli::Commands::Info => commands::info::execute(),
        cli::Commands::Tree(cmd) => cmd.execute(),
        cli::Commands::Search(cmd) => cmd.execute(),
        cli::Commands::Venv(cmd) => cmd.execute(args.verbose).await,
    }
}