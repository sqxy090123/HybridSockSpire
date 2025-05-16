#[cfg(unix)]
mod unix_lib;

#[cfg(windows)]
mod win_lib;

#[cfg(windows)]
const VERSION: &str = "1.0.1.1";

#[cfg(unix)]
const VERSION: &str = "1.0.0.0";

#[cfg(unix)]
fn main() {
    println!("内部控制 v{} for linux.", VERSION);
    unix_lib::listen_on_port(13323);
}

#[cfg(windows)]
fn main() {
    println!("内部控制 v{} for windows.", VERSION);
    if ! win_lib::is_admin() {
        win_lib::rerun_as_admin();
        return;
    }
    win_lib::listen_on_port(13323);
}
