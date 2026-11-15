//! 开发期验证：沿正式代码路径调用 shell::open_properties，确认属性窗口真的会弹出。
//! 用法： cargo run --release --example probe_props -- <路径>
//! 带 --self-test 时只调用、不真的弹窗（用于 CI）。
#![windows_subsystem = "console"]

#[path = "../src/win.rs"]
#[allow(dead_code)]
mod win;

#[path = "../src/shell.rs"]
#[allow(dead_code)]
mod shell;

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let self_test = args.iter().any(|a| a == "--self-test");
    args.retain(|a| !a.starts_with("--"));
    if args.is_empty() {
        // 现场造一个 .lnk 用来测
        let tmp = std::env::temp_dir().join("ql_props_probe");
        let _ = std::fs::create_dir_all(&tmp);
        let lnk = tmp.join("probe.lnk");
        let _ = std::fs::remove_file(&lnk);
        if make_lnk(&lnk, r"C:\Windows\system32\notepad.exe") {
            args.push(lnk.to_string_lossy().to_string());
        }
        args.push(r"C:\Windows\system32\notepad.exe".to_string());
        args.push(r"C:\Windows".to_string());
    }
    for p in &args {
        let ok = shell::open_properties(std::ptr::null_mut(), p);
        println!("open_properties({}) = {}", p, ok);
        if !self_test {
            std::thread::sleep(std::time::Duration::from_millis(800));
        }
    }
}

/// 用系统 COM 造一个 .lnk（与 shortcut.rs 测试里的做法一致）
fn make_lnk(dst: &std::path::Path, target: &str) -> bool {
    #[repr(C)]
    struct GUID { d1: u32, d2: u16, d3: u16, d4: [u8; 8] }
    #[repr(C)]
    struct Vtbl {
        query: unsafe extern "system" fn(*mut std::ffi::c_void, *const GUID, *mut *mut std::ffi::c_void) -> i32,
        addref: *const std::ffi::c_void,
        release: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
        get_path: *const std::ffi::c_void,
        get_idlist: *const std::ffi::c_void,
        set_idlist: *const std::ffi::c_void,
        get_description: *const std::ffi::c_void,
        set_description: *const std::ffi::c_void,
        get_workdir: *const std::ffi::c_void,
        set_workdir: *const std::ffi::c_void,
        get_arguments: *const std::ffi::c_void,
        set_arguments: *const std::ffi::c_void,
        get_hotkey: *const std::ffi::c_void,
        set_hotkey: *const std::ffi::c_void,
        get_showcmd: *const std::ffi::c_void,
        set_showcmd: *const std::ffi::c_void,
        get_icon: *const std::ffi::c_void,
        set_icon: *const std::ffi::c_void,
        set_relative_path: *const std::ffi::c_void,
        resolve: *const std::ffi::c_void,
        set_path: unsafe extern "system" fn(*mut std::ffi::c_void, *const u16) -> i32,
    }
    #[repr(C)]
    struct PVtbl {
        query: *const std::ffi::c_void,
        addref: *const std::ffi::c_void,
        release: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
        get_class_id: *const std::ffi::c_void,
        is_dirty: *const std::ffi::c_void,
        load: *const std::ffi::c_void,
        save: unsafe extern "system" fn(*mut std::ffi::c_void, *const u16, i32) -> i32,
    }
    #[link(name = "ole32")]
    extern "system" {
        fn CoInitializeEx(p: *mut std::ffi::c_void, c: u32) -> i32;
        fn CoCreateInstance(
            cls: *const GUID,
            outer: *mut std::ffi::c_void,
            ctx: u32,
            iid: *const GUID,
            out: *mut *mut std::ffi::c_void,
        ) -> i32;
    }
    const CLSID: GUID = GUID { d1: 0x0002_1401, d2: 0, d3: 0, d4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46] };
    const IID_LINK: GUID = GUID { d1: 0x0002_14F9, d2: 0, d3: 0, d4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46] };
    const IID_PF: GUID = GUID { d1: 0x0000_010B, d2: 0, d3: 0, d4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46] };
    unsafe {
        CoInitializeEx(std::ptr::null_mut(), 2);
        let mut link: *mut std::ffi::c_void = std::ptr::null_mut();
        if CoCreateInstance(&CLSID, std::ptr::null_mut(), 1, &IID_LINK, &mut link) != 0 {
            return false;
        }
        let vt = *(link as *const *const Vtbl);
        let tw = win::wide(target);
        ((*vt).set_path)(link, tw.as_ptr());
        let mut pf: *mut std::ffi::c_void = std::ptr::null_mut();
        if ((*vt).query)(link, &IID_PF, &mut pf) != 0 {
            ((*vt).release)(link);
            return false;
        }
        let pv = *(pf as *const *const PVtbl);
        let dw = win::wide(dst.to_str().unwrap_or("probe.lnk"));
        let hr = ((*pv).save)(pf, dw.as_ptr(), 1);
        ((*pv).release)(pf);
        ((*vt).release)(link);
        hr == 0
    }
}
