//! Sync command implementation - P2P synchronization.
//!
//! This command provides P2P synchronization capabilities using the sync protocol.
//!
//! Subcommands:
//! - `sync init <ADDR>` - Connect to remote peer and sync as initiator
//! - `sync respond` - Listen for incoming sync connections (requires server mode)
//!
//! # Examples
//! ```bash
//! synap sync init 192.168.1.100:8080
//! synap sync respond
//! ```

use colored::Colorize;
use std::net::TcpListener;
use synap_core::SynapService;
use crate::net::TcpConn;

/// Execute the sync command.
///
/// # Arguments
/// * `args` - Command arguments (subcommand and parameters)
/// * `service` - The Synap service instance
///
/// # Example
/// ```bash
/// synap sync init 127.0.0.1:8080  # Connect and sync as initiator
/// synap sync respond              # Listen for incoming connections
/// ```
pub fn execute(args: &[String], service: &SynapService) -> Result<(), Box<dyn std::error::Error>> {
    let subcommand = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match subcommand {
        "init" => {
            let addr = args.get(2).ok_or("sync init 需要指定地址")?;
            execute_init(addr, service)
        }
        "respond" => {
            execute_respond(service)
        }
        _ => {
            print_sync_help();
            Err("未知的 sync 子命令".into())
        }
    }
}

/// Execute sync as initiator (connect to remote peer).
///
/// # Arguments
/// * `addr` - The address of the remote peer (e.g., "127.0.0.1:8080")
/// * `service` - The Synap service instance
fn execute_init(addr: &str, service: &SynapService) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("{} 正在连接到 {}...", "[→]".dimmed(), addr.cyan());

    let conn = TcpConn::connect(addr).map_err(|e| {
        format!("连接失败: {}", e)
    })?;

    eprintln!("{} 已连接，开始同步...", "[✓]".green());

    let stats = service.sync_with(conn)?;

    println!();
    println!("{} 同步完成", "[✓]".green());
    println!("  接收区块: {}", stats.blocks_received.to_string().cyan());
    println!("  接收指针: {}", stats.pointers_received.to_string().cyan());

    if stats.blocks_duplicate > 0 || stats.pointers_duplicate > 0 {
        println!("  跳过重复区块: {}", stats.blocks_duplicate.to_string().dimmed());
        println!("  跳过重复指针: {}", stats.pointers_duplicate.to_string().dimmed());
    }

    println!("  耗时: {} ms", stats.duration_ms.to_string().dimmed());
    println!("  发送: {} bytes", stats.bytes_sent.to_string().dimmed());
    println!("  接收: {} bytes", stats.bytes_received.to_string().dimmed());
    println!();

    Ok(())
}

/// Execute sync as responder (listen for incoming connections).
///
/// # Arguments
/// * `service` - The Synap service instance
///
/// This will listen on 0.0.0.0:8080 by default and accept one sync connection.
fn execute_respond(service: &SynapService) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:8080";

    eprintln!("{} 正在监听 {}...",("[*]").blue(), addr.cyan());
    eprintln!("{} 按 Ctrl+C 停止监听", "[i]".dimmed());

    let listener = TcpListener::bind(addr).map_err(|e| {
        format!("绑定端口失败: {}", e)
    })?;

    eprintln!("{} 等待连接...", "[*]".blue());

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let peer_addr = stream.peer_addr().map_err(|e| format!("获取对端地址失败: {}", e))?;
                eprintln!("{} 连接来自: {}", "[+]".green(), peer_addr.to_string().cyan());

                // Wrap the TcpStream in our TcpConn
                let conn = TcpConn::from_stream(stream);

                eprintln!("{} 正在同步...", "[→]".dimmed());

                match service.sync_respond(conn) {
                    Ok(stats) => {
                        println!();
                        println!("{} 同步完成", "[✓]".green());
                        println!("  接收区块: {}", stats.blocks_received.to_string().cyan());
                        println!("  接收指针: {}", stats.pointers_received.to_string().cyan());

                        if stats.blocks_duplicate > 0 || stats.pointers_duplicate > 0 {
                            println!("  跳过重复区块: {}", stats.blocks_duplicate.to_string().dimmed());
                            println!("  跳过重复指针: {}", stats.pointers_duplicate.to_string().dimmed());
                        }

                        println!("  耗时: {} ms", stats.duration_ms.to_string().dimmed());
                        println!("  发送: {} bytes", stats.bytes_sent.to_string().dimmed());
                        println!("  接收: {} bytes", stats.bytes_received.to_string().dimmed());
                        println!();
                    }
                    Err(e) => {
                        eprintln!("{} 同步失败: {}", "[!]".red(), e);
                    }
                }

                eprintln!("{} 等待下一个连接...", "[*]".blue());
            }
            Err(e) => {
                eprintln!("{} 连接失败: {}", "[!]".red(), e);
            }
        }
    }

    Ok(())
}

/// Print sync command help.
fn print_sync_help() {
    println!();
    println!("Sync 命令:");
    println!("  synap sync init <ADDR>     连接到对等节点并同步");
    println!("  synap sync respond         监听传入的同步连接");
    println!();
    println!("示例:");
    println!("  synap sync init 127.0.0.1:8080");
    println!("  synap sync respond");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_invalid_subcommand() {
        let service = SynapService::open_memory().unwrap();
        let result = execute(&["sync".to_string(), "invalid".to_string()], &service);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_init_missing_addr() {
        let service = SynapService::open_memory().unwrap();
        let result = execute(&["sync".to_string(), "init".to_string()], &service);
        assert!(result.is_err());
    }
}
