use clap::{App, Arg, SubCommand};
use log::error;
use std::process;

mod commands;
mod utils;

fn main() {
    // 初始化日志
    env_logger::init();
    
    let matches = App::new("jx")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A fast Java package manager written in Rust")
        .arg(
            Arg::with_name("verbose")
                .short('v')
                .long("verbose")
                .help("启用详细输出")
        )
        .arg(
            Arg::with_name("quiet")
                .short('q')
                .long("quiet")
                .help("静默模式")
        )
        .subcommand(
            SubCommand::with_name("init")
                .about("初始化新的Java项目")
                .arg(
                    Arg::with_name("NAME")
                        .help("项目名称")
                        .index(1)
                )
                .arg(
                    Arg::with_name("template")
                        .short('t')
                        .long("template")
                        .help("项目类型 (maven, gradle)")
                        .default_value("maven")
                        .possible_values(&["maven", "gradle"])
                )
        )
        .subcommand(
            SubCommand::with_name("install")
                .about("安装项目依赖")
                .arg(
                    Arg::with_name("file")
                        .short('f')
                        .long("file")
                        .help("指定依赖文件")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("production")
                        .long("production")
                        .help("仅安装生产依赖")
                )
                .arg(
                    Arg::with_name("force")
                        .long("force")
                        .help("强制重新安装")
                )
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("添加新的依赖")
                .arg(
                    Arg::with_name("DEPENDENCY")
                        .help("依赖坐标 (groupId:artifactId:version)")
                        .required(true)
                        .index(1)
                )
                .arg(
                    Arg::with_name("scope")
                        .short('s')
                        .long("scope")
                        .help("依赖类型 (compile, runtime, test, provided)")
                        .default_value("compile")
                        .possible_values(&["compile", "runtime", "test", "provided"])
                )
        )
        .subcommand(
            SubCommand::with_name("remove")
                .about("移除依赖")
                .arg(
                    Arg::with_name("DEPENDENCY")
                        .help("依赖坐标 (groupId:artifactId)")
                        .required(true)
                        .index(1)
                )
        )
        .subcommand(
            SubCommand::with_name("update")
                .about("更新依赖")
                .arg(
                    Arg::with_name("DEPENDENCY")
                        .help("依赖坐标 (groupId:artifactId)")
                        .index(1)
                )
                .arg(
                    Arg::with_name("latest")
                        .long("latest")
                        .help("更新到最新版本")
                )
        )
        .subcommand(
            SubCommand::with_name("build")
                .about("构建项目")
                .arg(
                    Arg::with_name("mode")
                        .short('m')
                        .long("mode")
                        .help("构建模式 (debug, release)")
                        .default_value("debug")
                        .possible_values(&["debug", "release"])
                )
                .arg(
                    Arg::with_name("no-test")
                        .long("no-test")
                        .help("跳过测试")
                )
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("运行项目")
                .arg(
                    Arg::with_name("MAIN_CLASS")
                        .help("主类名")
                        .index(1)
                )
                .arg(
                    Arg::with_name("ARGS")
                        .help("程序参数")
                        .multiple(true)
                        .index(2)
                )
        )
        .subcommand(
            SubCommand::with_name("test")
                .about("运行测试")
                .arg(
                    Arg::with_name("TEST_CLASS")
                        .help("测试类名")
                        .index(1)
                )
                .arg(
                    Arg::with_name("method")
                        .long("method")
                        .help("测试方法名")
                        .takes_value(true)
                )
        )
        .subcommand(
            SubCommand::with_name("clean")
                .about("清理构建文件")
        )
        .subcommand(
            SubCommand::with_name("info")
                .about("显示项目信息")
        )
        .subcommand(
            SubCommand::with_name("tree")
                .about("显示依赖树")
                .arg(
                    Arg::with_name("transitive")
                        .long("transitive")
                        .help("显示传递依赖")
                )
        )
        .subcommand(
            SubCommand::with_name("search")
                .about("搜索依赖")
                .arg(
                    Arg::with_name("QUERY")
                        .help("搜索关键词")
                        .required(true)
                        .index(1)
                )
                .arg(
                    Arg::with_name("limit")
                        .short('l')
                        .long("limit")
                        .help("最大结果数")
                        .default_value("20")
                        .takes_value(true)
                )
        )
        .subcommand(
            SubCommand::with_name("venv")
                .about("管理Java虚拟环境")
                .subcommand(
                    SubCommand::with_name("create")
                        .about("创建虚拟环境")
                        .arg(
                            Arg::with_name("NAME")
                                .help("虚拟环境名称")
                                .index(1)
                        )
                        .arg(
                            Arg::with_name("java-version")
                                .long("java-version")
                                .help("Java版本 (8, 11, 17, 21)")
                                .default_value("11")
                                .takes_value(true)
                        )
                        .arg(
                            Arg::with_name("maven-version")
                                .long("maven-version")
                                .help("Maven版本")
                                .default_value("3.9.5")
                                .takes_value(true)
                        )
                        .arg(
                            Arg::with_name("gradle-version")
                                .long("gradle-version")
                                .help("Gradle版本")
                                .default_value("8.5")
                                .takes_value(true)
                        )
                )
                .subcommand(
                    SubCommand::with_name("activate")
                        .about("激活虚拟环境")
                        .arg(
                            Arg::with_name("NAME")
                                .help("虚拟环境名称")
                                .index(1)
                        )
                )
                .subcommand(
                    SubCommand::with_name("deactivate")
                        .about("停用虚拟环境")
                )
                .subcommand(
                    SubCommand::with_name("list")
                        .about("列出所有虚拟环境")
                )
                .subcommand(
                    SubCommand::with_name("remove")
                        .about("删除虚拟环境")
                        .arg(
                            Arg::with_name("NAME")
                                .help("虚拟环境名称")
                                .required(true)
                                .index(1)
                        )
                )
                .subcommand(
                    SubCommand::with_name("info")
                        .about("显示虚拟环境信息")
                        .arg(
                            Arg::with_name("NAME")
                                .help("虚拟环境名称")
                                .index(1)
                        )
                )
        )
        .get_matches();

    let verbose = matches.is_present("verbose");
    let quiet = matches.is_present("quiet");
    
    // 设置日志级别
    if verbose {
        std::env::set_var("RUST_LOG", "debug");
    } else if quiet {
        std::env::set_var("RUST_LOG", "error");
    }
    
    // 显示欢迎信息
    if !quiet {
        println!("🚀 jx - Fast Java Package Manager");
        println!("Built with Rust for speed and reliability");
        println!();
    }
    
    // 执行命令
    let result = match matches.subcommand() {
        Some(("init", init_matches)) => {
            let name = init_matches.value_of("NAME").map(|s| s.to_string());
            let template = init_matches.value_of("template").unwrap_or("maven").to_string();
            commands::init::execute(name, template)
        }
        Some(("install", install_matches)) => {
            let file = install_matches.value_of("file").map(|s| s.to_string());
            let production = install_matches.is_present("production");
            let force = install_matches.is_present("force");
            commands::install::execute(file, production, force)
        }
        Some(("add", add_matches)) => {
            let dependency = add_matches.value_of("DEPENDENCY").unwrap().to_string();
            let scope = add_matches.value_of("scope").unwrap_or("compile").to_string();
            commands::add::execute(dependency, scope)
        }
        Some(("remove", remove_matches)) => {
            let dependency = remove_matches.value_of("DEPENDENCY").unwrap().to_string();
            commands::remove::execute(dependency)
        }
        Some(("update", update_matches)) => {
            let dependency = update_matches.value_of("DEPENDENCY").map(|s| s.to_string());
            let latest = update_matches.is_present("latest");
            commands::update::execute(dependency, latest)
        }
        Some(("build", build_matches)) => {
            let mode = build_matches.value_of("mode").unwrap_or("debug").to_string();
            let no_test = build_matches.is_present("no-test");
            commands::build::execute(mode, no_test)
        }
        Some(("run", run_matches)) => {
            let main_class = run_matches.value_of("MAIN_CLASS").map(|s| s.to_string());
            let args: Vec<String> = run_matches.values_of("ARGS")
                .unwrap_or_default()
                .map(|s| s.to_string())
                .collect();
            commands::run::execute(main_class, args)
        }
        Some(("test", test_matches)) => {
            let test_class = test_matches.value_of("TEST_CLASS").map(|s| s.to_string());
            let method = test_matches.value_of("method").map(|s| s.to_string());
            commands::test::execute(test_class, method)
        }
        Some(("clean", _)) => {
            commands::clean::execute()
        }
        Some(("info", _)) => {
            commands::info::execute()
        }
        Some(("tree", tree_matches)) => {
            let transitive = tree_matches.is_present("transitive");
            commands::tree::execute(transitive)
        }
        Some(("search", search_matches)) => {
            let query = search_matches.value_of("QUERY").unwrap().to_string();
            let limit = search_matches.value_of("limit")
                .unwrap_or("20")
                .parse()
                .unwrap_or(20);
            commands::search::execute(query, limit)
        }
        Some(("venv", venv_matches)) => {
            match venv_matches.subcommand() {
                Some(("create", create_matches)) => {
                    let name = create_matches.value_of("NAME").map(|s| s.to_string());
                    let java_version = create_matches.value_of("java-version").unwrap_or("11").to_string();
                    let maven_version = create_matches.value_of("maven-version").unwrap_or("3.9.5").to_string();
                    let gradle_version = create_matches.value_of("gradle-version").unwrap_or("8.5").to_string();
                    commands::venv::create(name, java_version, maven_version, gradle_version)
                }
                Some(("activate", activate_matches)) => {
                    let name = activate_matches.value_of("NAME").map(|s| s.to_string());
                    commands::venv::activate(name)
                }
                Some(("deactivate", _)) => {
                    commands::venv::deactivate()
                }
                Some(("list", _)) => {
                    commands::venv::list()
                }
                Some(("remove", remove_matches)) => {
                    let name = remove_matches.value_of("NAME").unwrap().to_string();
                    commands::venv::remove(name)
                }
                Some(("info", info_matches)) => {
                    let name = info_matches.value_of("NAME").map(|s| s.to_string());
                    commands::venv::info(name)
                }
                _ => {
                    println!("jx venv - Java虚拟环境管理");
                    println!("");
                    println!("使用方法:");
                    println!("  jx venv create [NAME] [--java-version VERSION] [--maven-version VERSION] [--gradle-version VERSION]");
                    println!("  jx venv activate [NAME]");
                    println!("  jx venv deactivate");
                    println!("  jx venv list");
                    println!("  jx venv remove <NAME>");
                    println!("  jx venv info [NAME]");
                    Ok(())
                }
            }
        }
        _ => {
            println!("jx - Fast Java Package Manager");
            println!("");
            println!("使用方法:");
            println!("  jx init [NAME] --template <maven|gradle>  # 初始化新项目");
            println!("  jx install [--production] [--force]       # 安装依赖");
            println!("  jx add <DEPENDENCY> [--scope SCOPE]       # 添加依赖");
            println!("  jx remove <DEPENDENCY>                     # 移除依赖");
            println!("  jx update [DEPENDENCY] [--latest]          # 更新依赖");
            println!("  jx build [--mode <debug|release>]          # 构建项目");
            println!("  jx run [MAIN_CLASS] [ARGS...]             # 运行项目");
            println!("  jx test [TEST_CLASS] [--method METHOD]     # 运行测试");
            println!("  jx clean                                  # 清理构建文件");
            println!("  jx info                                   # 显示项目信息");
            println!("  jx tree [--transitive]                     # 显示依赖树");
            println!("  jx search <QUERY> [--limit N]              # 搜索依赖");
            println!("  jx venv <COMMAND>                          # 管理虚拟环境");
            println!("  jx --help                                 # 查看详细帮助");
            Ok(())
        }
    };
    
    // 处理结果
    match result {
        Ok(_) => {
            if !quiet {
                println!("✅ 操作完成");
            }
            process::exit(0);
        }
        Err(e) => {
            error!("操作失败: {}", e);
            if !quiet {
                eprintln!("❌ 错误: {}", e.to_string());
            }
            process::exit(1);
        }
    }
}
