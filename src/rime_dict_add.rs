//! Rime 词库追加工具
//! 将「文字+制表符+编码」格式的输入插入到词库第一个非空行前（# 开头的行视作空行）
//! Ctrl+Cmd+F8 激活并置顶终端窗口

use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const DIM: &str = "\x1b[2m";

fn clear_screen() {
    print!("\x1b[2J\x1b[1;1H");
    let _ = io::stdout().flush();
}

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
    println!(
        "{}{}未找到配置文件{}，请输入 Rime 词库文件路径：{}",
        YELLOW, BOLD, RESET, RESET
    );
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

fn is_data_line(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty() && !trimmed.starts_with('#')
}

#[cfg(target_os = "macos")]
fn detect_terminal_app() -> Option<String> {
    let mut pid = std::process::id() as i32;
    let known = [
        ("Terminal", "Terminal"),
        ("iTerm2", "iTerm"),
        ("iTerm", "iTerm"),
        ("Cursor", "Cursor"),
        ("Code", "Visual Studio Code"),
        ("Warp", "Warp"),
        ("Alacritty", "Alacritty"),
    ];
    for _ in 0..20 {
        let out = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "ppid=,comm="])
            .output()
            .ok()?;
        let s = String::from_utf8_lossy(&out.stdout);
        let line = s.lines().next()?.trim();
        let mut iter = line.split_whitespace();
        let ppid: i32 = iter.next()?.parse().ok()?;
        let comm = iter.next().unwrap_or("");
        for (proc_name, app_name) in &known {
            if comm.contains(proc_name) {
                return Some((*app_name).to_string());
            }
        }
        pid = ppid;
        if ppid <= 1 {
            break;
        }
    }
    Some("Terminal".to_string())
}

#[cfg(target_os = "macos")]
fn focus_and_float_terminal() {
    let app = detect_terminal_app().unwrap_or_else(|| "Terminal".to_string());
    let activate = format!(r#"tell application "{}" to activate"#, app);
    let _ = Command::new("osascript").arg("-e").arg(&activate).output();
    std::thread::sleep(std::time::Duration::from_millis(200));
    let float = format!(
        r#"tell application "System Events" to tell process "{}"
            set frontmost to true
            try
                set value of attribute "AXFloating" of window 1 to true
            end try
        end tell"#,
        app
    );
    let _ = Command::new("osascript").arg("-e").arg(&float).output();
}

#[cfg(target_os = "macos")]
fn spawn_hotkey_listener() {
    static CTRL: AtomicBool = AtomicBool::new(false);
    static META: AtomicBool = AtomicBool::new(false);
    static DEBOUNCE: AtomicBool = AtomicBool::new(false);

    std::thread::spawn(|| {
        if let Err(e) = rdev::listen(move |event| {
            match &event.event_type {
                rdev::EventType::KeyPress(key) => {
                    match key {
                        rdev::Key::ControlLeft | rdev::Key::ControlRight => {
                            CTRL.store(true, Ordering::Relaxed);
                        }
                        rdev::Key::MetaLeft | rdev::Key::MetaRight => {
                            META.store(true, Ordering::Relaxed);
                        }
                        rdev::Key::F8 => {
                            if CTRL.load(Ordering::Relaxed)
                                && META.load(Ordering::Relaxed)
                                && !DEBOUNCE.swap(true, Ordering::Relaxed)
                            {
                                focus_and_float_terminal();
                            }
                        }
                        _ => {}
                    }
                }
                rdev::EventType::KeyRelease(key) => {
                    match key {
                        rdev::Key::ControlLeft | rdev::Key::ControlRight => {
                            CTRL.store(false, Ordering::Relaxed);
                            DEBOUNCE.store(false, Ordering::Relaxed);
                        }
                        rdev::Key::MetaLeft | rdev::Key::MetaRight => {
                            META.store(false, Ordering::Relaxed);
                            DEBOUNCE.store(false, Ordering::Relaxed);
                        }
                        rdev::Key::F8 => {
                            DEBOUNCE.store(false, Ordering::Relaxed);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }) {
            eprintln!("hotkey listener error: {:?}", e);
        }
    });
}

fn trigger_rime_deploy() {
    #[cfg(target_os = "macos")]
    {
        let squirrel =
            "/Library/Input Methods/Squirrel.app/Contents/MacOS/Squirrel";
        if std::path::Path::new(squirrel).exists() {
            let _ = Command::new(squirrel).arg("--reload").output();
        }
    }

    #[cfg(target_os = "windows")]
    {
        for path in [
            r"C:\Program Files (x86)\Rime\WeaselServer.exe",
            r"C:\Program Files\Rime\WeaselServer.exe",
        ] {
            if std::path::Path::new(path).exists() {
                let _ = Command::new(path).arg("/deploy").output();
                break;
            }
        }
    }
}

fn insert_to_dict(dict_path: &PathBuf, new_line: &str) -> io::Result<()> {
    let content = if dict_path.exists() {
        fs::read_to_string(dict_path)?
    } else {
        String::new()
    };

    let lines: Vec<&str> = content.lines().collect();
    let insert_pos = lines.iter().position(|l| is_data_line(l));

    let (before, after) = match insert_pos {
        Some(i) => (lines[..i].join("\n"), lines[i..].join("\n")),
        None => (content, String::new()),
    };

    let result = if after.is_empty() {
        format!(
            "{}{}{}",
            before,
            if before.is_empty() { "" } else { "\n" },
            new_line
        )
    } else {
        format!("{}\n{}\n{}", before, new_line, after)
    };

    fs::write(dict_path, result)
}

fn main() {
    let dict_path = match load_config() {
        Some(p) => p,
        None => {
            let path = match prompt_dict_path() {
                Some(p) => p,
                None => {
                    eprintln!("{}{}未输入有效路径，退出{}", RED, BOLD, RESET);
                    std::process::exit(1);
                }
            };
            let path_str = path.to_string_lossy().to_string();
            if let Err(e) = save_config(&path_str) {
                eprintln!("{}{}保存配置失败{}: {}", RED, BOLD, RESET, e);
                std::process::exit(1);
            }
            println!("{}{}✓ 配置已保存{}", GREEN, BOLD, RESET);
            path
        }
    };

    if !dict_path.exists() {
        if dict_path.parent().map(|p| p.exists()).unwrap_or(false) {
            println!(
                "{}{}词库文件将新建于{}: {}{}{}",
                YELLOW,
                BOLD,
                RESET,
                CYAN,
                dict_path.display(),
                RESET
            );
        } else {
            eprintln!(
                "{}{}路径无效，父目录不存在{}: {}",
                RED,
                BOLD,
                RESET,
                dict_path.display()
            );
            std::process::exit(1);
        }
    }

    let show_prompt = || {
        println!(
            "{}{}词库路径{}: {}{}{}",
            BOLD,
            CYAN,
            RESET,
            DIM,
            dict_path.display(),
            RESET
        );
        println!(
            "{}{}请输入{}「文字<Tab>编码」{}{}格式的内容，空行退出：{}",
            BOLD, GREEN, RESET, BOLD, GREEN, RESET
        );
        #[cfg(target_os = "macos")]
        println!("{}{}Ctrl+Cmd+F8{} 激活并置顶终端", DIM, BOLD, RESET);
    };
    show_prompt();

    #[cfg(target_os = "macos")]
    spawn_hotkey_listener();

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
                    if let Err(e) = insert_to_dict(&dict_path, line) {
                        eprintln!("{}{}写入失败{}: {}", RED, BOLD, RESET, e);
                    } else {
                        trigger_rime_deploy();
                        clear_screen();
                        println!("{}{}✓ 已插入{}: {}", GREEN, BOLD, RESET, line);
                        println!();
                        show_prompt();
                    }
                } else {
                    eprintln!("{}{}格式无效{}，应为：文字<Tab>编码", RED, BOLD, RESET);
                }
            }
            Err(e) => {
                eprintln!("{}{}读取输入错误{}: {}", RED, BOLD, RESET, e);
                break;
            }
        }
    }
}
