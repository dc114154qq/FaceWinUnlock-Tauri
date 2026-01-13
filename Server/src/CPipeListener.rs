use std::{sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, thread};
use windows::Win32::{
    Foundation::CloseHandle, 
    Storage::FileSystem::{ReadFile, PIPE_ACCESS_INBOUND}, 
    System::
        Pipes::{ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_MESSAGE, PIPE_TYPE_MESSAGE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT}
    , UI::Shell::ICredentialProviderEvents
};

use crate::SharedCredentials;

// 包装 COM 接口，使其可以跨线程传输
#[derive(Clone)]
struct SendableEvents(pub ICredentialProviderEvents);
// 声明这是安全的
unsafe impl Send for SendableEvents {}
unsafe impl Sync for SendableEvents {}

pub struct CPipeListener {
    pub is_unlocked: AtomicBool,
    pub running: Arc<AtomicBool>,
}

impl CPipeListener {
    pub fn start(provider_events: ICredentialProviderEvents, advise_context: usize, shared_creds_clone: Arc<Mutex<SharedCredentials>>) -> Arc<Self> {
        info!("CPipeListener::start - 启动管道监听");

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let listener = Arc::new(Self {
            is_unlocked: AtomicBool::new(false),
            running: running_clone.clone(),
        });

        let sendable_events = SendableEvents(provider_events);
        let listener_clone = listener.clone();

        thread::spawn(move || {
            info!("CPipeListener::start - 进入管道监听线程");
            let events_wrapper = sendable_events;
            let pipe_name = windows_core::w!(r"\\.\pipe\MansonWindowsUnlockRust");
            unsafe {
                while running_clone.load(Ordering::SeqCst) {
                    // 创建命名管道 
                    let h_pipe = CreateNamedPipeW(
                        pipe_name,
                        PIPE_ACCESS_INBOUND,
                        PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                        PIPE_UNLIMITED_INSTANCES,
                        512, 512, 0,
                        None // 先使用默认权限，如果创建失败，则使用 create_everyone_full_access_sa
                    );

                    if h_pipe.is_invalid() {
                        error!("创建管道失败");
                        break;
                    }

                    // 使命名管道服务器进程能够等待客户端进程连接到命名管道的实例
                    let f_connected = ConnectNamedPipe(
                        h_pipe, 
                        None // 创建管道时未指定FILE_FLAG_OVERLAPPED，使用同步模式，函数会阻塞线程，直到客户端连接成功或发生错误才返回
                    );

                    if f_connected.is_err() {
                        let _ = CloseHandle(h_pipe);
                        error!("管道连接失败：{:?}", f_connected.err());
                        break;
                    }

                    if !running_clone.load(Ordering::SeqCst) {
                        // 防止在退出时误读数据
                        let _ = CloseHandle(h_pipe);
                        break; 
                    }

                    let mut buf = [0u16; 256];
                    let mut read = 0;

                    // 获取 buf 的字节切片视图 (&mut [u8])
                    // buf 的字节长度是 256 * 2 (每个 u16 占两个字节)
                    let byte_slice = std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, buf.len() * 2);
                    
                    // 读取用户名
                    if ReadFile(h_pipe, Some(byte_slice), Some(&mut read), None).is_ok() {
                        let user = String::from_utf16_lossy(&buf[.. (read as usize / 2)]);
                        let mut creds = shared_creds_clone.lock().unwrap();
                        creds.username = user.trim_matches('\0').to_string();
                    }

                    // 读取密码前重置一下缓冲区
                    read = 0;

                    // 读取密码
                    if ReadFile(h_pipe, Some(byte_slice), Some(&mut read), None).is_ok() {
                        let pass = String::from_utf16_lossy(&buf[.. (read as usize / 2)]);
                        let mut creds = shared_creds_clone.lock().unwrap();
                        creds.password = pass.trim_matches('\0').to_string();
                    }

                    // 准备就绪
                    {
                        let mut creds = shared_creds_clone.lock().unwrap();
                        creds.is_ready = true;
                    }

                    running_clone.store(false, Ordering::SeqCst);
                    listener_clone.is_unlocked.store(true, Ordering::SeqCst);
                    
                    // 通知 UI 刷新，触发 GetCredentialCount
                    let _ = events_wrapper.0.CredentialsChanged(advise_context);
                    
                    let _ = CloseHandle(h_pipe);

                    break;
                }
            }
            info!("CPipeListener 线程已彻底退出");
        });
        listener
    }
}

impl Drop for CPipeListener {
    fn drop(&mut self) {
        info!("销毁一个 CPipeListener");
    }
}