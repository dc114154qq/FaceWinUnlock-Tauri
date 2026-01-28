use std::{ffi::OsStr, os::windows::ffi::OsStrExt};
use windows::Win32::{
    Foundation::{CloseHandle, GetLastError, E_UNEXPECTED, GENERIC_WRITE, HANDLE}, 
    Storage::FileSystem::{CreateFileW, ReadFile, WriteFile, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_MODE, OPEN_EXISTING, PIPE_ACCESS_DUPLEX}, 
    System::
        Pipes::{ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, WaitNamedPipeW, PIPE_READMODE_MESSAGE, PIPE_TYPE_MESSAGE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT}
};
use windows_core::{Error, Result, HSTRING};

pub fn read(handle: HANDLE) -> Result<String> {
    let mut buf = [0u16; 256];
    let mut read = 0;

    let byte_slice = unsafe { std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, buf.len() * 2) };
    
    unsafe { ReadFile(handle, Some(byte_slice), Some(&mut read), None) }?;

    let content = String::from_utf16_lossy(&buf[.. (read as usize / 2)]);
    
    Ok(content.trim_matches('\0').to_string())
}

pub fn write(handle: HANDLE, content: String) -> Result<()> { 
    // 转 UTF-16 含 \0
    let wide_chars: Vec<u16> = OsStr::new(&content).encode_wide().chain(std::iter::once(0)).collect();
    // 转 &[u8] 切片
    let write_buf = unsafe { std::slice::from_raw_parts(
        wide_chars.as_ptr() as *const u8,
        wide_chars.len() * 2,
    ) };
    // 准备字节数（可变引用，匹配 Option<&mut u32>）
    let mut total_bytes = write_buf.len() as u32;

    unsafe { WriteFile(
        handle,
        Some(write_buf),
        Some(&mut total_bytes),
        None
    ) }
}

pub struct Server {
    pub handle: HANDLE,
    pipe_name: HSTRING,
    is_connected: bool,
}

impl Server {
    pub fn new(pipe_name: HSTRING) -> Self {
        // 创建命名管道 
        let h_pipe = unsafe { CreateNamedPipeW(
            &pipe_name,
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
            PIPE_UNLIMITED_INSTANCES,
            512, 512, 0,
            None
        ) };

        Self { handle: h_pipe, pipe_name, is_connected: false }
    }

    pub fn connect(&mut self) -> Result<()> {
        // 等待管道连接
        unsafe { ConnectNamedPipe(self.handle, None) }?;
        self.is_connected = true;

        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<()> {
        // 断开管道连接
        unsafe { DisconnectNamedPipe(self.handle) }?;
        self.is_connected = false;

        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.handle) };
    }
}

pub struct Client {
    pub handle: HANDLE,
    pipe_name: HSTRING,
}

impl Client {
    pub fn new(pipe_name: HSTRING) -> Result<Self> {
        let result = unsafe { WaitNamedPipeW(&pipe_name, 5000) };
        if !result.as_bool() {
            return Err(Error::new(E_UNEXPECTED, "管道不存在"));
        }

        // 打开管道
        let handle = unsafe { CreateFileW(
            &pipe_name, // 管道名称
            GENERIC_WRITE.0, // 对文件的操作模式，只写
            FILE_SHARE_MODE(0), // 阻止对管道的后续打开操作，在我主动关闭之前
            None,
            OPEN_EXISTING, // 只在文件存在时才打开，否则返回错误
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None
        ) };

        if handle.is_err() {
            return Err(Error::new(E_UNEXPECTED, format!("打开管道失败: {:?}, 扩展信息: {:?}", handle.err(), unsafe {GetLastError()})));
        }
        let handle = handle.unwrap();
        
        Ok(Client { handle: handle, pipe_name: pipe_name })
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}
