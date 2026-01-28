// 引入必要的同步原语和Win32 API
use std::sync::{Arc, Mutex};
use windows::Win32::{
    Foundation::{ERROR_NOT_READY, E_NOTIMPL, STATUS_SUCCESS}, Graphics::Gdi::HBITMAP, Security::Credentials::{CredPackAuthenticationBufferW, CRED_PACK_FLAGS}, System::Com::CoTaskMemAlloc, UI::Shell::{
        ICredentialProviderCredential, ICredentialProviderCredentialEvents, ICredentialProviderCredential_Impl, CPFIS_NONE, CPFS_DISPLAY_IN_BOTH, CPGSR_RETURN_CREDENTIAL_FINISHED, CPSI_ERROR, CPSI_NONE, CREDENTIAL_PROVIDER_CREDENTIAL_SERIALIZATION, CREDENTIAL_PROVIDER_FIELD_INTERACTIVE_STATE, CREDENTIAL_PROVIDER_FIELD_STATE, CREDENTIAL_PROVIDER_GET_SERIALIZATION_RESPONSE, CREDENTIAL_PROVIDER_STATUS_ICON
    }
};
use windows_core::{implement, BOOL, PCWSTR, PWSTR};
use crate::{dll_add_ref, dll_release, CLSID_SampleProvider, SharedCredentials};

/// 凭据实现类，代表登录界面上的一个磁贴
/// 每个凭据对应一个可选择的登录选项
#[implement(ICredentialProviderCredential)]
pub struct SampleCredential {
    // 用于接收系统事件通知的接口（互斥锁保护线程安全）
    events: Mutex<Option<ICredentialProviderCredentialEvents>>,
    shared_creds: Arc<Mutex<SharedCredentials>>,
    auth_package_id: u32
}

impl SampleCredential {
    /// 创建新的凭据实例
    pub fn new(shared_creds: Arc<Mutex<SharedCredentials>>, auth_package_id: u32) -> Self {
        info!("SampleCredential::new - 创建凭据实例");
        dll_add_ref(); // 增加DLL引用计数
        Self { 
            events: Mutex::new(None),
            shared_creds: shared_creds,
            auth_package_id: auth_package_id
        }
    }
}

impl Drop for SampleCredential {
    fn drop(&mut self) {
        info!("SampleCredential::drop - 销毁凭据实例");
        dll_release(); // 减少DLL引用计数，与new中的dll_add_ref()对应
    }
}

impl ICredentialProviderCredential_Impl for SampleCredential_Impl {
    /// 设置事件通知接口，用于向系统发送状态变化
    /// pcpce: 系统提供的事件接口
    fn Advise(&self, pcpce: windows_core::Ref<ICredentialProviderCredentialEvents>) -> windows_core::Result<()> {
        info!("SampleCredential::Advise - 注册事件通知");
        let mut events = self.events.lock().unwrap();
        *events = pcpce.clone(); // 保存事件接口
        Ok(())
    }

    /// 取消事件通知
    fn UnAdvise(&self) -> windows_core::Result<()> {
        info!("SampleCredential::UnAdvise - 取消事件通知");
        let mut events = self.events.lock().unwrap();
        *events = None; // 清除事件接口
        Ok(())
    }

    /// 当凭据磁贴被选中时调用
    fn SetSelected(&self) -> windows_core::Result<BOOL> {
        info!("SampleCredential::SetSelected - 磁贴被选中");
        Ok(true.into()) // 返回true表示处理成功
    }

    /// 当凭据磁贴被取消选中时调用
    fn SetDeselected(&self) -> windows_core::Result<()> {
        info!("SampleCredential::SetDeselected - 磁贴被取消选中");
        Ok(())
    }

    /// 获取字段的状态（可见性和交互性）
    /// dwfieldid: 字段ID
    /// pcpfs: 输出参数，字段的显示状态
    /// pcpfis: 输出参数，字段的交互状态
    fn GetFieldState(
        &self, 
        dwfieldid: u32, 
        pcpfs: *mut CREDENTIAL_PROVIDER_FIELD_STATE, 
        pcpfis: *mut CREDENTIAL_PROVIDER_FIELD_INTERACTIVE_STATE
    ) -> windows_core::Result<()> {
        info!("SampleCredential::GetFieldState - 获取字段 {} 的状态", dwfieldid);
        unsafe {
            match dwfieldid {
                // 字段0: 图标，字段1: 文本
                0 | 1 => {  
                    *pcpfs = CPFS_DISPLAY_IN_BOTH; // 在磁贴和详细视图中都显示
                    *pcpfis = CPFIS_NONE;          // 非交互元素（不能点击或编辑）
                }
                _ => {
                    error!("SampleCredential::GetFieldState - 无效的字段ID: {}", dwfieldid);
                    return Err(windows::Win32::Foundation::E_INVALIDARG.into());
                }
            }
        }
        Ok(())
    }

    /// 获取文本字段的内容
    /// dwfieldid: 字段ID
    fn GetStringValue(&self, dwfieldid: u32) -> windows_core::Result<PWSTR> {
        info!("SampleCredential::GetStringValue - 获取字段 {} 的文本内容", dwfieldid);
        let val = match dwfieldid {
            1 => "FaceWinUnlock-Tauri-请勿点击此磁贴",  // 字段1的文本内容
            _ => {
                warn!("SampleCredential::GetStringValue - 字段 {} 无文本内容", dwfieldid);
                ""
            }
        };
        
        // 分配COM可释放的内存（使用CoTaskMemAlloc）
        unsafe {
            let utf16: Vec<u16> = val.encode_utf16().chain(Some(0)).collect(); // 转换为UTF-16并添加终止符
            let ptr = windows::Win32::System::Com::CoTaskMemAlloc(utf16.len() * 2); // 分配内存
            if ptr.is_null() {
                error!("SampleCredential::GetStringValue - 内存分配失败");
                return Err(windows::Win32::Foundation::E_OUTOFMEMORY.into());
            }
            // 复制数据到分配的内存
            std::ptr::copy_nonoverlapping(utf16.as_ptr(), ptr as *mut u16, utf16.len());
            Ok(PWSTR(ptr as *mut _))
        }
    }

    /// 获取图标字段的位图
    /// _dwfieldid: 字段ID（这里是0）
    fn GetBitmapValue(&self, _dwfieldid: u32) -> windows_core::Result<HBITMAP> {
        info!("SampleCredential::GetBitmapValue - 获取图标字段的位图");
        Ok(HBITMAP::default())  // 返回默认图标
    }

    /// 获取复选框字段的值（未实现）
    fn GetCheckboxValue(&self, _dwfieldid: u32, _pbchecked: *mut BOOL, _ppszlabel: *mut PWSTR) -> windows_core::Result<()> {
        info!("SampleCredential::GetCheckboxValue - 未实现的接口");
        Err(E_NOTIMPL.into())
    }

    /// 获取提交按钮字段的值（未实现）
    fn GetSubmitButtonValue(&self, _dwfieldid: u32) -> windows_core::Result<u32> {
        info!("SampleCredential::GetSubmitButtonValue - 未实现的接口");
        Err(E_NOTIMPL.into())
    }

    /// 获取下拉框字段的选项数量（未实现）
    fn GetComboBoxValueCount(&self, _dwfieldid: u32, _pcitems: *mut u32, _pdwselecteditem: *mut u32) -> windows_core::Result<()> {
        info!("SampleCredential::GetComboBoxValueCount - 未实现的接口");
        Err(E_NOTIMPL.into())
    }

    /// 获取下拉框指定选项的文本（未实现）
    fn GetComboBoxValueAt(&self, _dwfieldid: u32, _dwitem: u32) -> windows_core::Result<PWSTR> {
        info!("SampleCredential::GetComboBoxValueAt - 未实现的接口");
        Err(E_NOTIMPL.into())
    }

    /// 设置文本字段的值（未实现）
    fn SetStringValue(&self, _dwfieldid: u32, _psz: &windows_core::PCWSTR) -> windows_core::Result<()> {
        info!("SampleCredential::SetStringValue - 未实现的接口");
        Err(E_NOTIMPL.into())
    }

    /// 设置复选框字段的值（未实现）
    fn SetCheckboxValue(&self, _dwfieldid: u32, _bchecked: BOOL) -> windows_core::Result<()> {
        info!("SampleCredential::SetCheckboxValue - 未实现的接口");
        Err(E_NOTIMPL.into())
    }

    /// 设置下拉框选中项（未实现）
    fn SetComboBoxSelectedValue(&self, _dwfieldid: u32, _dwselecteditem: u32) -> windows_core::Result<()> {
        info!("SampleCredential::SetComboBoxSelectedValue - 未实现的接口");
        Err(E_NOTIMPL.into())
    }

    /// 命令链接被点击（未实现）
    fn CommandLinkClicked(&self, _dwfieldid: u32) -> windows_core::Result<()> {
        info!("SampleCredential::CommandLinkClicked - 未实现的接口");
        Err(E_NOTIMPL.into())
    }

    /// 序列化凭据信息（登录时调用）
    fn GetSerialization(
        &self, 
        pcpgsr: *mut CREDENTIAL_PROVIDER_GET_SERIALIZATION_RESPONSE, 
        pcpcs: *mut CREDENTIAL_PROVIDER_CREDENTIAL_SERIALIZATION, 
        _ppszoptionalstatustext: *mut PWSTR, 
        _pcpsioptionalstatusicon: *mut CREDENTIAL_PROVIDER_STATUS_ICON
    ) -> windows_core::Result<()> {
        info!("SampleCredential::GetSerialization - 序列化凭据");
        unsafe {
            let creds = self.shared_creds.lock().unwrap();
            if !creds.is_ready {
                error!("SampleCredential::GetSerialization - 凭据未就绪");
                return Err(ERROR_NOT_READY.into());
            }

            // 设置响应为：成功，准备好序列化数据了
            *pcpgsr = CPGSR_RETURN_CREDENTIAL_FINISHED;

            // 准备拼接后的用户名 (Domain\User)
            let full_username = if creds.domain.is_empty() || creds.domain == "." {
                creds.username.clone()
            } else {
                format!("{}\\{}", creds.domain, creds.username)
            };

            // 获取管道收到的用户名和密码
            let v_username = to_wide_vec(&full_username);
            let v_password = to_wide_vec(&creds.password);

            // 转换成 PCWSTR (指向 u16 数组开头的指针)
            let pwz_username = PCWSTR(v_username.as_ptr());
            let pwz_password = PCWSTR(v_password.as_ptr());

            // 调用 LSA 序列化开始

            let mut auth_buffer_size: u32 = 0;

            // 使用系统 API 打包 Kerberos 凭据
            // 第一次调用获取长度
            let _ = CredPackAuthenticationBufferW(
                CRED_PACK_FLAGS(0), // 默认传 0
                pwz_username,
                pwz_password,
                None, // 第一次传 None
                &mut auth_buffer_size
            );

            // 分配 COM 内存，系统会自动释放这块内存
            let out_buf = CoTaskMemAlloc(auth_buffer_size as usize) as *mut u8;

            // 第二次调用真正打包
            CredPackAuthenticationBufferW(
                CRED_PACK_FLAGS(0),
                pwz_username,
                pwz_password,
                Some(out_buf), // 传入分配好的指针
                &mut auth_buffer_size
            )?;

            // 填充返回给 Windows 的结构体
            *pcpgsr = CPGSR_RETURN_CREDENTIAL_FINISHED;
            (*pcpcs).clsidCredentialProvider = CLSID_SampleProvider; 
            (*pcpcs).cbSerialization = auth_buffer_size;
            (*pcpcs).rgbSerialization = out_buf;

            // 重点：AuthenticationPackage 需要通过 LsaLookupAuthenticationPackage 获取
            // 通常在 Provider 初始化时获取一次。
            (*pcpcs).ulAuthenticationPackage = self.auth_package_id;
        }
        Ok(())
    }

    /// 报告登录结果
    fn ReportResult(
        &self, 
        ntsstatus: windows::Win32::Foundation::NTSTATUS, 
        _ntssubstatus: windows::Win32::Foundation::NTSTATUS, 
        ppszoptionalstatustext: *mut PWSTR, 
        pcpsioptionalstatusicon: *mut CREDENTIAL_PROVIDER_STATUS_ICON
    ) -> windows_core::Result<()> {
        info!("SampleCredential::ReportResult - 报告登录结果");
        unsafe {
            if ntsstatus != STATUS_SUCCESS {
                // 如果登录失败
                let mut creds = self.shared_creds.lock().unwrap();
                // 清空错误凭据
                creds.username.clear();
                creds.password.clear();
                creds.is_ready = false;

                // 设置错误提示文本
                let error_text = "用户名或密码错误，请点击自己账户，手动输入密码进入系统。";
                let utf16: Vec<u16> = error_text.encode_utf16().chain(Some(0)).collect();
                let ptr = windows::Win32::System::Com::CoTaskMemAlloc(utf16.len() * 2);
                if !ptr.is_null() {
                    std::ptr::copy_nonoverlapping(utf16.as_ptr(), ptr as *mut u16, utf16.len());
                    *ppszoptionalstatustext = PWSTR(ptr as *mut _);
                }

                *pcpsioptionalstatusicon = CPSI_ERROR;
            } else {
                // 登录成功
                *ppszoptionalstatustext = PWSTR(std::ptr::null_mut());
                *pcpsioptionalstatusicon = CPSI_NONE;
            }
        }
        Ok(())
    }
}

// 将 String 转换为符合 Win32 要求的 UTF-16 向量（带 null 结尾）
fn to_wide_vec(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}