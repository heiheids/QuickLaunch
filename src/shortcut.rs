//! 快捷方式解析：把桌面上的 .lnk / .url 还原成它指向的真实目标。
//!
//! 背景：拖进来的“桌面快捷方式”只是一个中间层。若把 .lnk 自身的路径存进
//! 数据库，用户一删桌面上的图标，条目就失效。因此入库前先解析出源程序
//! 路径（以及参数、工作目录、图标），数据库里保存的始终是源程序位置。
//!
//! 只用到系统自带的 shell32 / ole32，没有任何第三方依赖。
#![allow(non_snake_case, dead_code)]

use crate::data::{Db, KIND_FOLDER, KIND_PROGRAM, KIND_URL};
use crate::win::wide;
use std::ffi::c_void;
use std::path::Path;

#[repr(C)]
struct GUID {
    d1: u32,
    d2: u16,
    d3: u16,
    d4: [u8; 8],
}

const CLSID_SHELL_LINK: GUID =
    GUID { d1: 0x0002_1401, d2: 0, d3: 0, d4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46] };
const IID_ISHELL_LINK_W: GUID =
    GUID { d1: 0x0002_14F9, d2: 0, d3: 0, d4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46] };
const IID_IPERSIST_FILE: GUID =
    GUID { d1: 0x0000_010B, d2: 0, d3: 0, d4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46] };

const CLSCTX_INPROC_SERVER: u32 = 1;
const SLGP_DEFAULT: u32 = 0;
const S_OK: i32 = 0;
const MAX_LNK_DEPTH: u32 = 4;

#[repr(C)]
struct ShellLinkVtbl {
    query: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32,
    addref: *const c_void,
    release: unsafe extern "system" fn(*mut c_void) -> u32,
    get_path: unsafe extern "system" fn(*mut c_void, *mut u16, i32, *mut c_void, u32) -> i32,
    get_idlist: *const c_void,
    set_idlist: *const c_void,
    get_description: unsafe extern "system" fn(*mut c_void, *mut u16, i32) -> i32,
    set_description: *const c_void,
    get_workdir: unsafe extern "system" fn(*mut c_void, *mut u16, i32) -> i32,
    set_workdir: unsafe extern "system" fn(*mut c_void, *const u16) -> i32,
    get_arguments: unsafe extern "system" fn(*mut c_void, *mut u16, i32) -> i32,
    set_arguments: unsafe extern "system" fn(*mut c_void, *const u16) -> i32,
    get_hotkey: *const c_void,
    set_hotkey: *const c_void,
    get_showcmd: *const c_void,
    set_showcmd: *const c_void,
    get_icon: unsafe extern "system" fn(*mut c_void, *mut u16, i32, *mut i32) -> i32,
    set_icon: *const c_void,
    set_relative_path: *const c_void,
    resolve: *const c_void,
    set_path: unsafe extern "system" fn(*mut c_void, *const u16) -> i32,
}

#[repr(C)]
struct PersistFileVtbl {
    query: *const c_void,
    addref: *const c_void,
    release: unsafe extern "system" fn(*mut c_void) -> u32,
    get_class_id: *const c_void,
    is_dirty: *const c_void,
    load: unsafe extern "system" fn(*mut c_void, *const u16, u32) -> i32,
    save: unsafe extern "system" fn(*mut c_void, *const u16, i32) -> i32,
}

/// 解析结果：可以直接写进数据库的链接字段
#[derive(Clone, Default)]
pub struct Resolved {
    pub path: String,
    pub param: String,
    pub workdir: String,
    pub icon: String,
    /// 目标类型：KIND_PROGRAM / KIND_FOLDER / KIND_URL
    pub kind: i32,
}

/// 扩展名（大小写无关）是否为 .lnk
pub fn is_lnk(path: &str) -> bool {
    Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().eq_ignore_ascii_case("lnk"))
        .unwrap_or(false)
}

/// 扩展名（大小写无关）是否为 .url（Internet 快捷方式）
pub fn is_url_file(path: &str) -> bool {
    Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().eq_ignore_ascii_case("url"))
        .unwrap_or(false)
}

/// 是否是本模块能解析的快捷方式
pub fn is_shortcut(path: &str) -> bool {
    is_lnk(path) || is_url_file(path)
}

/// 解析快捷方式。不是快捷方式、或解析不到真实目标（如 UWP 应用）时返回 None，
/// 调用方应当原样保留传入的路径。
pub fn resolve(path: &str) -> Option<Resolved> {
    if is_url_file(path) {
        return resolve_url_file(path);
    }
    if !is_lnk(path) {
        return None;
    }
    let mut cur = path.to_string();
    let mut param = String::new();
    let mut workdir = String::new();
    let mut icon = String::new();
    let mut hit = false;
    for _ in 0..MAX_LNK_DEPTH {
        if !is_lnk(&cur) {
            break;
        }
        let r = unsafe { resolve_lnk_raw(&cur) }?;
        if r.path.trim().is_empty() {
            // UWP / 应用商店应用：保留 .lnk 本身
            return None;
        }
        let mut target = r.path.trim().to_string();
        if !Path::new(&target).is_absolute() {
            if let Some(d) = Path::new(&cur).parent() {
                target = d.join(&target).to_string_lossy().to_string();
            }
        }
        if param.is_empty() && !r.param.trim().is_empty() {
            param = r.param.trim().to_string();
        }
        if workdir.is_empty() && !r.workdir.trim().is_empty() {
            workdir = r.workdir.trim().to_string();
        }
        if icon.is_empty() && !r.icon.trim().is_empty() {
            icon = r.icon.trim().to_string();
        }
        cur = target;
        hit = true;
    }
    if !hit {
        return None;
    }
    // 工作目录缺失时用目标所在目录（比让系统从本程序目录启动靠谱）
    if workdir.is_empty() {
        if let Some(p) = Path::new(&cur).parent() {
            if p.to_string_lossy().len() > 1 {
                workdir = p.to_string_lossy().to_string();
            }
        }
    }
    let kind = if Path::new(&cur).is_dir() { KIND_FOLDER } else { KIND_PROGRAM };
    Some(Resolved { path: cur, param, workdir, icon, kind })
}

/// 读 .url（INI 格式）里的 URL=
fn resolve_url_file(path: &str) -> Option<Resolved> {
    let txt = std::fs::read_to_string(path).ok()?;
    for line in txt.lines() {
        let l = line.trim();
        if let Some(rest) = l.strip_prefix("URL=").or_else(|| {
            // 兼容大小写与前置空格
            let (k, v) = l.split_once('=')?;
            if k.trim().eq_ignore_ascii_case("URL") {
                Some(v)
            } else {
                None
            }
        }) {
            let u = rest.trim().trim_matches('"').trim();
            if !u.is_empty() {
                return Some(Resolved {
                    path: u.to_string(),
                    param: String::new(),
                    workdir: String::new(),
                    icon: String::new(),
                    kind: KIND_URL,
                });
            }
        }
    }
    None
}

unsafe fn resolve_lnk_raw(path: &str) -> Option<RawLink> {
    let mut link: *mut c_void = std::ptr::null_mut();
    let hr = CoCreateInstance(
        &CLSID_SHELL_LINK,
        std::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        &IID_ISHELL_LINK_W,
        &mut link,
    );
    if hr != S_OK || link.is_null() {
        return None;
    }
    let vtbl = *(link as *const *const ShellLinkVtbl);
    let mut persist: *mut c_void = std::ptr::null_mut();
    let hr = ((*vtbl).query)(link, &IID_IPERSIST_FILE, &mut persist);
    if hr != S_OK || persist.is_null() {
        ((*vtbl).release)(link);
        return None;
    }
    let pvtbl = *(persist as *const *const PersistFileVtbl);
    let w = wide(path);
    let hr = ((*pvtbl).load)(persist, w.as_ptr(), 0);
    // IPersistFile 只是同一对象上的另一个接口，用完即可放掉
    ((*pvtbl).release)(persist);
    if hr != S_OK {
        ((*vtbl).release)(link);
        return None;
    }
    let get_str = |f: unsafe extern "system" fn(*mut c_void, *mut u16, i32) -> i32| -> String {
        let mut buf = vec![0u16; 2048];
        let hr = f(link, buf.as_mut_ptr(), buf.len() as i32);
        if hr < 0 {
            return String::new();
        }
        let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        String::from_utf16_lossy(&buf[..n])
    };
    let mut pbuf = vec![0u16; 2048];
    let p_hr = ((*vtbl).get_path)(link, pbuf.as_mut_ptr(), pbuf.len() as i32, std::ptr::null_mut(), SLGP_DEFAULT);
    let path_s = if p_hr < 0 {
        String::new()
    } else {
        let n = pbuf.iter().position(|&c| c == 0).unwrap_or(pbuf.len());
        String::from_utf16_lossy(&pbuf[..n])
    };
    let param = get_str((*vtbl).get_arguments);
    let workdir = get_str((*vtbl).get_workdir);
    let mut ibuf = vec![0u16; 1024];
    let mut idx: i32 = 0;
    let icon_hr = ((*vtbl).get_icon)(link, ibuf.as_mut_ptr(), ibuf.len() as i32, &mut idx);
    let mut icon = String::new();
    if icon_hr >= 0 {
        let n = ibuf.iter().position(|&c| c == 0).unwrap_or(ibuf.len());
        let s = String::from_utf16_lossy(&ibuf[..n]);
        if !s.trim().is_empty() {
            icon = format!("{},{}", s.trim(), idx);
        }
    }
    ((*vtbl).release)(link);
    Some(RawLink { path: path_s, param, workdir, icon })
}

struct RawLink {
    path: String,
    param: String,
    workdir: String,
    icon: String,
}

/// 把数据库里历史遗留的快捷方式条目迁移成真实目标；返回是否发生了改动。
/// 只处理 .lnk 仍然存在的情况，避免把“快捷方式已被删掉”的条目误伤。
pub fn migrate(db: &mut Db) -> bool {
    let mut changed = false;
    for l in db.links.iter_mut() {
        if !is_shortcut(&l.path) {
            continue;
        }
        if !Path::new(&l.path).exists() {
            continue;
        }
        let Some(r) = resolve(&l.path) else { continue };
        if r.path == l.path {
            continue;
        }
        // 原始快捷方式路径先留下来，“属性”要用它打开快捷方式自己的属性窗口
        if l.src.is_empty() {
            l.src = l.path.clone();
        }
        l.path = r.path;
        if l.param.is_empty() {
            l.param = r.param;
        }
        if l.workdir.is_empty() {
            l.workdir = r.workdir;
        }
        if l.icon.is_empty() {
            l.icon = r.icon;
        }
        l.kind = r.kind;
        changed = true;
    }
    changed
}

#[link(name = "ole32")]
extern "system" {
    fn CoCreateInstance(
        rclsid: *const GUID,
        outer: *mut c_void,
        clsctx: u32,
        riid: *const GUID,
        out: *mut *mut c_void,
    ) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ext_detect() {
        assert!(is_lnk(r"C:\Users\x\Desktop\Chrome.LNK"));
        assert!(!is_lnk(r"C:\Windows\notepad.exe"));
        assert!(is_url_file(r"C:\Users\x\Desktop\GitHub.url"));
        assert!(is_shortcut(r"C:\a\b.lnk"));
    }

    #[test]
    fn url_file_parse() {
        let p = std::env::temp_dir().join("ql_test_shortcut.url");
        std::fs::write(&p, "[InternetShortcut]\r\nURL=https://example.com/x\r\n").unwrap();
        let r = resolve_url_file(p.to_str().unwrap()).expect("应当解析出 URL");
        assert_eq!(r.path, "https://example.com/x");
        assert_eq!(r.kind, KIND_URL);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn plain_file_untouched() {
        assert!(resolve(r"C:\Windows\notepad.exe").is_none());
    }

    /// 端到端：用系统 COM 现场生成一个 .lnk，再解析回来。
    /// 这一条才是本功能的核心验证——快捷方式必须解析成真实目标。
    #[test]
    fn resolve_real_lnk() {
        unsafe {
            CoInitializeEx(std::ptr::null_mut(), 2);
        }
        let dir = std::env::temp_dir().join("ql_lnk_test");
        let _ = std::fs::create_dir_all(&dir);
        let lnk = dir.join("ql_test.lnk");
        let _ = std::fs::remove_file(&lnk);
        let target = r"C:\Windows\notepad.exe";
        if !Path::new(target).exists() {
            return; // 极端环境没有记事本，跳过
        }
        unsafe {
            make_lnk(&lnk, target, "--ql-test", r"C:\Windows");
        }
        let r = resolve(lnk.to_str().unwrap()).expect("解析真实 .lnk 应当成功");
        assert_eq!(r.path.to_lowercase(), target.to_lowercase(), "目标路径不对");
        assert_eq!(r.param, "--ql-test");
        assert_eq!(r.workdir.to_lowercase(), r"c:\windows");
        assert_eq!(r.kind, KIND_PROGRAM);
        let _ = std::fs::remove_file(&lnk);
    }

    #[test]
    fn migrate_rewrites_lnk_entry() {
        unsafe {
            CoInitializeEx(std::ptr::null_mut(), 2);
        }
        let dir = std::env::temp_dir().join("ql_lnk_migrate");
        let _ = std::fs::create_dir_all(&dir);
        let lnk = dir.join("ql_mig.lnk");
        let _ = std::fs::remove_file(&lnk);
        let target = r"C:\Windows\notepad.exe";
        if !Path::new(target).exists() {
            return;
        }
        unsafe {
            make_lnk(&lnk, target, "", "");
        }
        let mut db = Db::empty();
        let g = db.groups[0].id;
        let t = db.ensure_tag(g);
        db.add_link(t, "旧条目", KIND_PROGRAM, lnk.to_str().unwrap(), "", "", "", "", false);
        assert!(migrate(&mut db), "应当发生迁移");
        assert_eq!(db.links[0].path.to_lowercase(), target.to_lowercase());
        assert_eq!(db.links[0].src, lnk.to_str().unwrap(), "原快捷方式路径要留档");
        let _ = std::fs::remove_file(&lnk);
    }

    unsafe fn make_lnk(dst: &Path, target: &str, args: &str, workdir: &str) {
        let mut link: *mut c_void = std::ptr::null_mut();
        let hr = CoCreateInstance(
            &CLSID_SHELL_LINK,
            std::ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            &IID_ISHELL_LINK_W,
            &mut link,
        );
        assert_eq!(hr, S_OK, "CoCreateInstance 失败");
        let vtbl = *(link as *const *const ShellLinkVtbl);
        let tw = wide(target);
        assert_eq!(((*vtbl).set_path)(link, tw.as_ptr()), S_OK);
        if !args.is_empty() {
            let aw = wide(args);
            assert_eq!(((*vtbl).set_arguments)(link, aw.as_ptr()), S_OK);
        }
        if !workdir.is_empty() {
            let ww = wide(workdir);
            assert_eq!(((*vtbl).set_workdir)(link, ww.as_ptr()), S_OK);
        }
        let mut persist: *mut c_void = std::ptr::null_mut();
        assert_eq!(((*vtbl).query)(link, &IID_IPERSIST_FILE, &mut persist), S_OK);
        let pvtbl = *(persist as *const *const PersistFileVtbl);
        let dw = wide(dst.to_str().unwrap());
        let hr = ((*pvtbl).save)(persist, dw.as_ptr(), 1);
        ((*pvtbl).release)(persist);
        ((*vtbl).release)(link);
        assert_eq!(hr, S_OK, "保存 .lnk 失败");
        assert!(dst.exists(), "生成的 .lnk 不存在");
    }

    #[link(name = "ole32")]
    extern "system" {
        fn CoInitializeEx(reserved: *mut c_void, coinit: u32) -> i32;
    }
}
