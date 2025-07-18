#[cfg(unix)]
mod unix_lib;
#[cfg(unix)]
mod pre_unix_lib;

#[cfg(windows)]
mod win_lib;
#[cfg(windows)]
mod pre_win_lib;

#[cfg(windows)]
const VERSION: &str = "1.0.0.2";

#[cfg(unix)]
const VERSION: &str = "1.0.0.1";

const DEBUG: bool = false;

// Windows 服务实现
#[cfg(windows)]
mod service {
    use windows_service::{
        service::{ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType},
        service_manager::{ServiceManager, ServiceManagerAccess},
        service_dispatcher,
    };
    use std::ffi::OsString;
    use std::time::Duration;
    use winapi::shared::winerror::ERROR_SERVICE_SPECIFIC_ERROR;
    use windows_service::define_windows_service;
    use log::{error, info};
    use crate::win_lib;

    const SERVICE_NAME: &str = "LLLLLLLLLLL";
    const SERVICE_DISPLAY_NAME: &str = "LLLLLLLLLLL";

    // 服务状态句柄
    static mut SERVICE_STATUS_HANDLE: Option<windows_service::service::ServiceStatusHandle> = None;

    // 注册服务分发器
    pub fn run_service() -> windows_service::Result<()> {
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)
    }

    // FFI服务主函数
    define_windows_service!(ffi_service_main, service_main);

    // 实际服务主函数
    fn service_main(_arguments: Vec<OsString>) {
        if let Err(e) = run_service_logic() {
            error!("Service failed: {:?}", e);
            report_service_status(ServiceState::Stopped, Some(1));
        }
    }

    // 服务逻辑
    fn run_service_logic() -> Result<(), Box<dyn std::error::Error>> {
        // 设置服务状态为启动中
        report_service_status(ServiceState::StartPending, None);
        
        // 初始化日志系统（服务模式下）
        init_service_logger();
        info!("Service starting...");
        
        // 设置服务状态为运行中
        report_service_status(ServiceState::Running, None);
        info!("Service started successfully");
        
        // 启动业务逻辑
        win_lib::listen_on_port(13323);
        
        Ok(())
    }

    // 报告服务状态
    fn report_service_status(state: ServiceState, exit_code: Option<u32>) {
        let status_handle = unsafe { SERVICE_STATUS_HANDLE.as_ref().unwrap() };
        
        let status = ServiceStatus {
            service_type: ServiceType::OwnProcess,
            current_state: state,
            controls_accepted: windows_service::service::ServiceControlAccept::STOP,
            exit_code: exit_code
                .map(|code| windows_service::service::ServiceExitCode::Win32(code))
                .unwrap_or(windows_service::service::ServiceExitCode::NoError),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        };
        
        status_handle.set_service_status(status).unwrap();
    }

    // 初始化服务日志系统
    fn init_service_logger() {
        // 这里使用简单的文件日志，实际中可使用log4rs等更强大的日志库
        let log_file = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("service.log");
        
        let logger = simple_logger::SimpleLogger::new()
            .with_level(log::LevelFilter::Info)
            .with_utc_timestamps()
            .to_file(log_file)
            .unwrap();
        
        log::set_boxed_logger(Box::new(logger)).unwrap();
        log::set_max_level(log::LevelFilter::Info);
    }

    // 安装服务
    pub fn install_service() -> Result<(), windows_service::Error> {
        let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
        let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;
        
        let current_exe = std::env::current_exe()?;
        
        let service_info = ServiceInfo {
            name: OsString::from(SERVICE_NAME),
            display_name: OsString::from(SERVICE_DISPLAY_NAME),
            service_type: ServiceType::OwnProcess,
            start_type: ServiceStartType::AutoStart,
            error_control: ServiceErrorControl::Normal,
            executable_path: current_exe.clone(),
            launch_arguments: vec![OsString::from("--service")],
            dependencies: vec![],
            account_name: None, // 使用 LocalSystem
            account_password: None,
        };
        
        let _service = service_manager.create_service(
            &service_info,
            ServiceAccess::CHANGE_CONFIG | ServiceAccess::START
        )?;
        
        info!("Service installed successfully: {:?}", current_exe);
        Ok(())
    }
}

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
    use crate::service;
    use crate::win_lib;
    use log::info;

    // 解析命令行参数
    let args: Vec<String> = std::env::args().collect();
    
    // 服务模式运行
    if args.iter().any(|arg| arg == "--service") {
        if let Err(e) = service::run_service() {
            eprintln!("Service failed: {:?}", e);
            std::process::exit(1);
        }
        return;
    }
    
    // 普通控制台模式
    println!("内部控制 v{} for windows.", VERSION);
    
    // 安装服务命令
    if args.len() > 1 && args[1] == "install" {
        if !win_lib::is_admin() {
            win_lib::rerun_as_admin();
            return;
        }
        match service::install_service() {
            Ok(_) => println!("Service installed successfully"),
            Err(e) => eprintln!("Failed to install service: {:?}", e),
        }
        return;
    }
    
    // 调试模式
    if DEBUG {
        println!("DEBUG:");
        pre_win_lib::listen_on_port(13323);
    } else {
        win_lib::listen_on_port(13323);
    }
}