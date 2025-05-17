#[cfg(unix)]
mod unix_lib;
#[cfg(unix)]
mod pre_unix_lib;

#[cfg(windows)]
mod win_lib;
#[cfg(windows)]
mod pre_win_lib;

#[cfg(windows)]
const VERSION: &str = "1.0.1.1";

#[cfg(unix)]
const VERSION: &str = "1.0.0.0";

const DEBUG: bool = true;

#[cfg(unix)]
fn main() {
    
    println!("内部控制 v{} for linux.", VERSION);
    if DEBUG {
        println!("DEBUGGING");
        pre_unix_lib::listen_on_port(13323);
    } else {
        unix_lib::listen_on_port(13323);
    }
}

#[cfg(windows)]
fn main() {

    println!("内部控制 v{} for windows.", VERSION);
    if ! win_lib::is_admin() {
        win_lib::rerun_as_admin();
        return;
    }
    if DEBUG {
        println!("DEBUG:");
        pre_win_lib::listen_on_port(13323);
    } else {
        win_lib::listen_on_port(13323);
    }
}
