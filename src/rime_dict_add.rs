//! Rime 词库追加工具
//! 将「文字+制表符+编码」格式的输入追加到 Rime 词库文件末尾

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::PathBuf;

const CONFIG_FILENAME: &str = ".rime_dict_config";

fn script_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.canonicalize().ok().or(Some(p)))
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
}

fn config_path() -> Option<PathBuf> {
    script_dir().map(|d| d.join(CONFIG_FILENAME))
}

fn load_config() -> Option<PathBuf> {
    let path = config_path()?;
    if path.exists() {
        let content = fs::read_to_string(&path).ok()?;
        let dict_path = content.trim();
        if !dict_path.is_empty() {
            return Some(PathBuf::from(dict_path));
        }
    }
    None
}

fn save_config(dict_path: &str) -> io::Result<()> {
    let path = config_path()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "无法确定脚本所在目录"))?;
    fs::write(&path, dict_path.trim())
}

fn prompt_dict_path() -> Option<PathBuf> {
    println!("未找到配置文件，请输入 Rime 词库文件路径：");
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    let path_str = input.trim();
    if path_str.is_empty() {
        return None;
    }
    let path = PathBuf::from(path_str);
    let abs = if path.exists() {
        path.canonicalize().ok()
    } else if path.is_relative() {
        std::env::current_dir().ok().map(|cwd| cwd.join(&path))
    } else {
        Some(path)
    };
    abs
}

fn is_valid_entry(line: &str) -> bool {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 2 {
        return false;
    }
    let text = parts[0].trim();
    let code = parts[1].trim();
    !text.is_empty()
        && !code.is_empty()
        && code
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '\'' || c == ' ')
}

fn append_to_dict(dict_path: &PathBuf, line: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dict_path)?;
    writeln!(file, "{}", line)?;
    file.flush()?;
    Ok(())
}

fn main() {
    let dict_path = match load_config() {
        Some(p) => p,
        None => {
            let path = match prompt_dict_path() {
                Some(p) => p,
                None => {
                    eprintln!("未输入有效路径，退出");
                    std::process::exit(1);
                }
            };
            let path_str = path.to_string_lossy().to_string();
            if let Err(e) = save_config(&path_str) {
                eprintln!("保存配置失败: {}", e);
                std::process::exit(1);
            }
            println!("配置已保存");
            path
        }
    };

    if !dict_path.exists() {
        if dict_path.parent().map(|p| p.exists()).unwrap_or(false) {
            println!("词库文件将新建于: {}", dict_path.display());
        } else {
            eprintln!("路径无效，父目录不存在: {}", dict_path.display());
            std::process::exit(1);
        }
    }

    println!("词库路径: {}", dict_path.display());
    println!("请输入「文字<Tab>编码」格式的内容，空行退出：");

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let line = line.trim_end_matches(|c| c == '\n' || c == '\r');
                if line.is_empty() {
                    break;
                }
                if is_valid_entry(line) {
                    if let Err(e) = append_to_dict(&dict_path, line) {
                        eprintln!("写入失败: {}", e);
                    } else {
                        println!("已追加");
                    }
                } else {
                    eprintln!("格式无效，应为：文字<Tab>编码");
                }
            }
            Err(e) => {
                eprintln!("读取输入错误: {}", e);
                break;
            }
        }
    }
}
