//! Shell 动作封装：打开属性窗口、定位文件。
//!
//! 踩坑记录：右键“属性”以前用的是
//!     ShellExecuteW(hwnd, "properties", path, ...)
//! 这个调用对 exe / 文件夹会返回 31（SE_ERR_NOASSOC）并且**静默失败**，
//! 用户看到的就是“点了没反应”。只有 .lnk 这类本身注册了 properties 动词的
//! 对象才碰巧能用。
//!
//! 正确做法是 ShellExecuteExW + SEE_MASK_INVOKEIDLIST：
//! 让 shell 通过 shell item 的方式去调动词，exe、文件夹、快捷方式都可靠。
#![allow(dead_code)]

use crate::win::*;

/// shell 的动作都成功/失败在返回值里体现，这里统一返回是否成功，便于上层提示
pub fn open_properties(hwnd: HWND, path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    if !path.starts_with("::") && !path.contains("://") && !std::path::Path::new(path).exists() {
        return false;
    }
    let verb = wide("properties");
    let file = wide(path);
    let mut sei = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_INVOKEIDLIST,
        hwnd,
        lpVerb: verb.as_ptr(),
        lpFile: file.as_ptr(),
        nShow: SW_SHOWNORMAL,
        ..Default::default()
    };
    unsafe { ShellExecuteExW(&mut sei) != 0 }
}

/// 在资源管理器里选中该文件 / 文件夹
pub fn open_location(hwnd: HWND, path: &str) {
    unsafe {
        let exe = wide("explorer.exe");
        let arg = wide(&format!("/select,\"{}\"", path));
        ShellExecuteW(hwnd, std::ptr::null(), exe.as_ptr(), arg.as_ptr(), std::ptr::null(), SW_SHOWNORMAL);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_path_rejected() {
        assert!(!open_properties(std::ptr::null_mut(), ""));
    }

    #[test]
    fn missing_path_rejected() {
        assert!(!open_properties(std::ptr::null_mut(), r"C:\这个路径肯定不存在\ql_none.exe"));
    }
}
