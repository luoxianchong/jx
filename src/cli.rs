use clap::{Parser, Subcommand};

use crate::commands::{
    add::AddCommand,
    build::BuildCommand,
    init::InitCommand,
    install::InstallCommand,
    remove::RemoveCommand,
    run::RunCommand,
    search::SearchCommand,
    test::TestCommand,
    tree::TreeCommand,
    update::UpdateCommand,
    venv::VenvCommand,
};

#[derive(Parser)]
#[clap(name = "jx", version, about = "A fast Java package manager written in Rust")]
pub struct Args {
    #[clap(global = true, short, long, help = "启用详细输出")]
    pub verbose: bool,

    #[clap(global = true, short, long, help = "静默模式")]
    pub quiet: bool,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[clap(about = "初始化新的Java项目")]
    Init(InitCommand),

    #[clap(about = "安装项目依赖")]
    Install(InstallCommand),

    #[clap(about = "添加新的依赖")]
    Add(AddCommand),

    #[clap(about = "移除依赖")]
    Remove(RemoveCommand),

    #[clap(about = "更新依赖")]
    Update(UpdateCommand),

    #[clap(about = "构建项目")]
    Build(BuildCommand),

    #[clap(about = "运行项目")]
    Run(RunCommand),

    #[clap(about = "运行测试")]
    Test(TestCommand),

    #[clap(about = "清理构建文件")]
    Clean,

    #[clap(about = "显示项目信息")]
    Info,

    #[clap(about = "显示依赖树")]
    Tree(TreeCommand),

    #[clap(about = "搜索依赖")]
    Search(SearchCommand),

    #[clap(about = "管理Java虚拟环境")]
    Venv(VenvCommand),
}