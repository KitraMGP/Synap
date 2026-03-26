//! Synap CLI - Your local immutable consciousness stream and topological network.
//!
//! This CLI transforms from "CRUD API wrapped in command line" to "mind extension prosthetic".
//!
//! Core Principles:
//! - **Zero-friction capture** - `synap "content"` captures directly, no explicit command needed
//! - **Symbol overloading** - `@ID` means "continue this thought"
//! - **Physical/flow verbs** - amend, void, trace, scrub instead of edit, delete, list, gc
//! - **Geek aesthetics** - Short ID prefix matching, ASCII art, immutable ledger philosophy

mod config;
mod commands;
mod net;
mod output;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use colored::Colorize;
use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
};
use synap_core::SynapService;

/// Synap - Your local immutable consciousness stream and topological network
#[derive(Parser)]
#[command(name = "synap")]
#[command(author = "Synap Contributors")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Synap - 你的本地不可变意识流与拓扑网络", long_about = None)]
struct Cli {
    /// 指定数据库路径 (覆盖默认位置)
    #[arg(short, long, global = true)]
    db: Option<String>,

    /// 显示详细输出
    #[arg(short, long, global = true)]
    verbose: bool,

    /// 隐式捕获：直接输入内容，不带任何命令
    /// 或者 @ID content 回复模式
    /// 或者 #tag 标签搜索
    /// 或者 ULID 前缀显示
    #[arg(group = "mode", value_name = "CONTENT", num_args = 1..)]
    capture: Option<Vec<String>>,
    #[command(subcommand)]
    command: Option<Commands>,
}

/// 检查是否为有效的 ULID 前缀 (不区分大小写)
fn is_valid_ulid_prefix(s: &str) -> bool {
    s.len() >= 3 && s.len() <= 26
        && s.chars().all(|c| {
            c.is_ascii_alphanumeric() && !matches!(c, 'i' | 'I' | 'l' | 'L' | 'o' | 'O' | 'u' | 'U')
        })
}

/// 子命令定义
#[derive(Subcommand)]
enum Commands {
    /// 进化：拉起编辑器，生成新区块并替代旧区块
    Amend {
        /// 目标区块的短 ID
        id: String,
    },

    /// 废弃：打上死神指针，在视图中抹除该区块
    Void {
        /// 目标区块的短 ID
        id: String,
    },

    /// 溯源：渲染图谱拓扑或显示最近的活跃流
    Trace {
        /// 目标区块的短 ID（可选）
        id: Option<String>,

        /// 流式显示所有思想（内存高效）
        #[arg(long)]
        stream: bool,
    },

    /// 检索：基于内容或 #标签 提取区块
    Search {
        /// 搜索内容或标签
        query: String,
    },

    /// 修剪：执行本地 GC，物理抹除无引用的墓碑区块
    Scrub,

    /// 详情：显示区块完整信息
    Show {
        /// 区块的短 ID
        id: String,
    },

    /// 统计：显示数据库统计信息
    Stats,

    /// 同步：P2P 同步操作
    Sync {
        #[command(subcommand)]
        action: SyncAction,
    },

    /// 列出所有区块
    List {
        /// 按标签过滤（可选）
        tag: Option<String>,
    },

    /// 标签管理
    Tag {
        /// 操作：add 或 remove
        action: String,

        /// 区块的短 ID
        id: String,

        /// 标签名
        tag: String,
    },

    /// 图谱可视化
    Graph {
        /// 根区块的短 ID（可选，默认从最新的开始）
        id: Option<String>,
    },

    /// 生成 shell 补全脚本
    Completions {
        /// Shell 类型 (bash, fish, zsh, powershell)
        shell: String,
    },

    /// 安装 shell 补全脚本
    InstallCompletions,
}

#[derive(Subcommand)]
enum SyncAction {
    /// 连接到对等节点并同步
    Init {
        /// 对等节点地址 (IP:PORT)
        addr: String,
    },

    /// 监听传入的同步连接
    Respond,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // 处理不需要数据库的命令
    if let Some(Commands::Completions { shell }) = &cli.command {
        handle_completions(shell)?;
        return Ok(());
    }

    if matches!(cli.command, Some(Commands::InstallCompletions)) {
        install_completions()?;
        return Ok(());
    }

    // 解析数据库路径
    let db_path = config::resolve_db_path(cli.db.clone());
    config::ensure_db_dir_exists_for(&db_path)?;

    // 打开数据库
    let service = SynapService::open(&db_path).map_err(|e| {
        format!(
            "{} 无法打开数据库 {}: {}",
            "[ERROR]".red(),
            db_path.display(),
            e.to_string()
        )
    })?;

    // 分发命令
    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Amend { id } => commands::amend::execute(&id, &service),
            Commands::Void { id } => commands::void::execute(&id, &service),
            Commands::Trace { id, stream } => {
                if stream {
                    commands::trace::execute_stream(&service)
                } else if let Some(id) = id {
                    let note = service.get_note_by_short_id(&id)?;
                    let graph = service.get_graph(note.id, 0)?;
                    commands::trace::render_graph_ascii(graph)
                } else {
                    commands::trace::execute_recent(&service)
                }
            }
            Commands::Search { query } => commands::search::execute(&query, &service),
            Commands::Scrub => commands::scrub::execute(&service),
            Commands::Show { id } => commands::show::execute(&id, &service),
            Commands::Stats => commands::stats::execute(&service),
            Commands::Sync { action } => match action {
                SyncAction::Init { addr } => commands::sync::execute(&["sync".to_string(), "init".to_string(), addr.clone()], &service),
                SyncAction::Respond => commands::sync::execute(&["sync".to_string(), "respond".to_string()], &service),
            },
            Commands::List { tag } => commands::list::execute(&service, tag),
            Commands::Tag { action, id, tag } => {
                match action.as_str() {
                    "add" => commands::tag::execute_add(&id, &tag, &service),
                    "remove" => commands::tag::execute_remove(&id, &tag, &service),
                    _ => Err("无效的 tag 操作: add 或 remove".into()),
                }
            }
            Commands::Graph { id } => {
                let note_id = if let Some(id) = id {
                    service.get_note_by_short_id(&id)?.id
                } else {
                    service.list_thoughts()?
                        .first()
                        .map(|n| n.id)
                        .ok_or("账本尚无思想")?
                };
                let graph = service.get_graph(note_id, 0)?;
                commands::trace::render_graph_ascii(graph)
            }
            Commands::Completions { .. } => unreachable!(),
            Commands::InstallCompletions => unreachable!(),
        }
    } else if let Some(args) = cli.capture {
        // 隐式模式：根据第一个参数的前缀决定行为
        if let Some(first) = args.first() {
            if first.starts_with('@') && args.len() >= 2 {
                // 回复模式: @ID content
                let id = first.trim_start_matches('@');
                let content = args[1..].join(" ");
                commands::reply::execute(id, &content, &service)
            } else if first.starts_with('#') {
                // 标签搜索: #tag
                let tag = first.trim_start_matches('#');
                commands::search::execute_tag(tag, &service)
            } else if is_valid_ulid_prefix(first) {
                // 显示模式: ULID前缀
                commands::show::execute(first, &service)
            } else {
                // 捕获模式: 内容
                let content = args.join(" ");
                commands::capture::execute(&content, &service)
            }
        } else {
            // 无参数：显示最近的活跃流
            commands::trace::execute_recent(&service)
        }
    } else {
        // 无参数：显示最近的活跃流
        commands::trace::execute_recent(&service)
    }
}

/// Handle shell completions
fn handle_completions(shell: &str) -> Result<(), Box<dyn std::error::Error>> {
    let shell = match shell {
        "bash" => Shell::Bash,
        "elvish" => Shell::Elvish,
        "fish" => Shell::Fish,
        "powershell" | "pwsh" => Shell::PowerShell,
        "zsh" => Shell::Zsh,
        _ => return Err(format!("不支持的 shell 类型: {}", shell).into()),
    };

    print_completions(shell, &mut Cli::command());
    Ok(())
}

/// Print shell completions
fn print_completions<G: Generator>(gen: G, cmd: &mut clap::Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

/// Install shell completions
fn install_completions() -> Result<(), Box<dyn std::error::Error>> {
    let shell = detect_shell()?;
    let (install_dir, filename, config_file, config_snippet) = match shell {
        Shell::Zsh => {
            let dir = env::var("HOME")
                .map(PathBuf::from)
                .map(|p| p.join(".zsh").join("completion"))
                .unwrap_or_else(|_| PathBuf::from("/usr/local/share/zsh/site-functions"));

            let rc_file = env::var("HOME")
                .map(PathBuf::from)
                .map(|p| p.join(".zshrc"))
                .unwrap();

            let snippet = format!(
                "\n# Synap completions\nfpath=($HOME/.zsh/completion $fpath)\n"
            );

            (dir, "_synap", Some(rc_file), Some(snippet))
        }
        Shell::Fish => {
            let dir = env::var("HOME")
                .map(PathBuf::from)
                .map(|p| p.join(".config").join("fish").join("completions"))
                .unwrap();

            (dir, "synap.fish", None, None)
        }
        Shell::Bash => {
            let dir = env::var("HOME")
                .map(PathBuf::from)
                .map(|p| p.join(".local").join("share").join("bash-completion").join("completions"))
                .unwrap();

            let rc_file = env::var("HOME")
                .map(PathBuf::from)
                .map(|p| p.join(".bashrc"))
                .unwrap();

            let snippet = format!(
                "\n# Synap completions\nsource ~/.local/share/bash-completion/completions/synap 2>/dev/null\n"
            );

            (dir, "synap", Some(rc_file), Some(snippet))
        }
        _ => {
            return Err(format!(
                "{}: {}",
                "不支持的 shell".red(),
                format!("{:?}", shell)
            )
            .into());
        }
    };

    // Create directory if it doesn't exist
    fs::create_dir_all(&install_dir)?;

    // Generate and write completion script
    let file_path = install_dir.join(filename);
    let mut file = File::create(&file_path)?;
    generate(shell, &mut Cli::command(), "synap".to_string(), &mut file);

    println!(
        "{} {}",
        "✓ 补全脚本已安装到:".green(),
        file_path.display().to_string().cyan()
    );

    // Add config snippet to rc file if needed
    if let (Some(rc_file), Some(snippet)) = (config_file, config_snippet) {
        if rc_file.exists() {
            let content = fs::read_to_string(&rc_file)?;
            if content.contains("Synap completions") {
                println!(
                    "{} {}",
                    "✓ 配置文件已包含 Synap 补全配置:".green(),
                    rc_file.display().to_string().cyan()
                );
            } else {
                let mut file = File::options().append(true).open(&rc_file)?;
                file.write_all(snippet.as_bytes())?;
                println!(
                    "{} {}",
                    "✓ 已添加补全配置到:".green(),
                    rc_file.display().to_string().cyan()
                );
            }
        } else {
            let mut file = File::create(&rc_file)?;
            file.write_all(snippet.as_bytes())?;
            println!(
                "{} {}",
                "✓ 已创建配置文件并添加补全配置:".green(),
                rc_file.display().to_string().cyan()
            );
        }
    }

    println!("\n{}", "重新加载配置或重启 shell 后即可使用自动补全".yellow());
    println!("{}", "运行: source ~/.zshrc (或 ~/.bashrc)".dimmed());

    Ok(())
}

/// Detect the current shell
fn detect_shell() -> Result<Shell, Box<dyn std::error::Error>> {
    if let Ok(shell_path) = env::var("SHELL") {
        if shell_path.contains("zsh") {
            return Ok(Shell::Zsh);
        } else if shell_path.contains("fish") {
            return Ok(Shell::Fish);
        } else if shell_path.contains("bash") {
            return Ok(Shell::Bash);
        }
    }

    let process_name = std::env::args()
        .next()
        .and_then(|p| {
            std::path::PathBuf::from(p)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
        });

    if let Some(name) = process_name {
        if name.contains("zsh") {
            return Ok(Shell::Zsh);
        } else if name.contains("fish") {
            return Ok(Shell::Fish);
        } else if name.contains("bash") {
            return Ok(Shell::Bash);
        }
    }

    Err(
        "无法检测到支持的 shell。请使用 --shell 参数手动指定支持的 shell (bash, fish, zsh)".into(),
    )
}
