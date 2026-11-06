//! OLE 拖放目标。
//!
//! WM_DROPFILES 只处理文件系统里的东西（CF_HDROP），桌面上的"此电脑""回收站"
//! 这类虚拟项永远不会触发它。要接收它们必须注册一个 IDropTarget。
#![allow(non_snake_case, dead_code)]
use crate::win::*;
use std::ffi::c_void;

/// 拖放结果通过这个消息投递给窗口（lParam 是 Box<DropPayload> 的裸指针）
pub const WM_DROP_ITEMS: u32 = WM_APP + 10;

pub enum DropItem {
    Path(String),
    Sys(&'static str),
}

pub struct DropPayload {
    pub items: Vec<DropItem>,
}

#[repr(C)]
pub struct GUID {
    d1: u32,
    d2: u16,
    d3: u16,
    d4: [u8; 8],
}

const IID_IUNKNOWN: GUID = GUID { d1: 0, d2: 0, d3: 0, d4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46] };
const IID_IDROP_TARGET: GUID = GUID { d1: 0x122, d2: 0, d3: 0, d4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46] };

#[repr(C)]
pub struct POINTL {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
pub struct FORMATETC {
    pub cfFormat: u16,
    pub ptd: *mut c_void,
    pub dwAspect: u32,
    pub lindex: i32,
    pub tymed: u32,
}

#[repr(C)]
pub struct STGMEDIUM {
    pub tymed: u32,
    pub hGlobal: *mut c_void,
    pub pUnkForRelease: *mut c_void,
}

#[repr(C)]
struct IDataObjectVtbl {
    QueryInterface: *const c_void,
    AddRef: *const c_void,
    Release: *const c_void,
    GetData: unsafe extern "system" fn(*mut c_void, *const FORMATETC, *mut STGMEDIUM) -> i32,
}

#[repr(C)]
struct IDropTargetVtbl {
    QueryInterface: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32,
    AddRef: unsafe extern "system" fn(*mut c_void) -> u32,
    Release: unsafe extern "system" fn(*mut c_void) -> u32,
    DragEnter: unsafe extern "system" fn(*mut c_void, *mut c_void, u32, POINTL, *mut u32) -> i32,
    DragOver: unsafe extern "system" fn(*mut c_void, u32, POINTL, *mut u32) -> i32,
    DragLeave: unsafe extern "system" fn(*mut c_void) -> i32,
    Drop: unsafe extern "system" fn(*mut c_void, *mut c_void, u32, POINTL, *mut u32) -> i32,
}

#[repr(C)]
struct DropSink {
    vtbl: *const IDropTargetVtbl,
    refs: u32,
    hwnd: HWND,
}

static VTABLE: IDropTargetVtbl = IDropTargetVtbl {
    QueryInterface: qi,
    AddRef: add_ref,
    Release: release,
    DragEnter: drag_enter,
    DragOver: drag_over,
    DragLeave: drag_leave,
    Drop: drop_item,
};

const S_OK: i32 = 0;
const E_NOINTERFACE: i32 = 0x8000_4002u32 as i32;
const DVASPECT_CONTENT: u32 = 1;
const TYMED_HGLOBAL: u32 = 1;
const CF_HDROP: u16 = 15;
const SIGDN_DESKTOPABSOLUTEPARSING: i32 = 0x8002_8000u32 as i32;
const DROPEFFECT_COPY: u32 = 1;
const DROPEFFECT_NONE: u32 = 0;

unsafe extern "system" fn qi(this: *mut c_void, riid: *const GUID, out: *mut *mut c_void) -> i32 {
    let r = &*riid;
    let same = |a: &GUID| {
        a.d1 == r.d1 && a.d2 == r.d2 && a.d3 == r.d3 && a.d4 == r.d4
    };
    if same(&IID_IUNKNOWN) || same(&IID_IDROP_TARGET) {
        *out = this;
        add_ref(this);
        S_OK
    } else {
        *out = std::ptr::null_mut();
        E_NOINTERFACE
    }
}

unsafe extern "system" fn add_ref(this: *mut c_void) -> u32 {
    let s = &mut *(this as *mut DropSink);
    s.refs += 1;
    s.refs
}

unsafe extern "system" fn release(this: *mut c_void) -> u32 {
    let s = &mut *(this as *mut DropSink);
    s.refs = s.refs.saturating_sub(1);
    s.refs
}

unsafe extern "system" fn drag_enter(
    _this: *mut c_void,
    _data: *mut c_void,
    _keys: u32,
    _pt: POINTL,
    effect: *mut u32,
) -> i32 {
    if !effect.is_null() {
        *effect = DROPEFFECT_COPY;
    }
    S_OK
}

unsafe extern "system" fn drag_over(
    _this: *mut c_void,
    _keys: u32,
    _pt: POINTL,
    effect: *mut u32,
) -> i32 {
    if !effect.is_null() {
        *effect = DROPEFFECT_COPY;
    }
    S_OK
}

unsafe extern "system" fn drag_leave(_this: *mut c_void) -> i32 {
    S_OK
}

unsafe extern "system" fn drop_item(
    this: *mut c_void,
    data: *mut c_void,
    _keys: u32,
    _pt: POINTL,
    effect: *mut u32,
) -> i32 {
    let s = &*(this as *const DropSink);
    let items = parse(data);
    if !items.is_empty() {
        let boxed = Box::into_raw(Box::new(DropPayload { items }));
        PostMessageW(s.hwnd, WM_DROP_ITEMS, 0, boxed as isize);
    }
    if !effect.is_null() {
        *effect = DROPEFFECT_COPY;
    }
    S_OK
}

/// 注册拖放目标；失败返回 null（此时仍可用 WM_DROPFILES 接收普通文件）
pub unsafe fn register(hwnd: HWND) -> *mut c_void {
    OleInitialize(std::ptr::null_mut());
    let sink = Box::into_raw(Box::new(DropSink { vtbl: &VTABLE, refs: 1, hwnd }));
    if RegisterDragDrop(hwnd, sink as *mut c_void) != S_OK {
        drop(Box::from_raw(sink));
        return std::ptr::null_mut();
    }
    sink as *mut c_void
}

pub unsafe fn unregister(hwnd: HWND, sink: *mut c_void) {
    if sink.is_null() {
        return;
    }
    RevokeDragDrop(hwnd);
    drop(Box::from_raw(sink as *mut DropSink));
}

unsafe fn get_data(data: *mut c_void, fmt: *const FORMATETC, med: *mut STGMEDIUM) -> i32 {
    let vtbl = *(data as *const *const IDataObjectVtbl);
    ((*vtbl).GetData)(data, fmt, med)
}

unsafe fn parse(data: *mut c_void) -> Vec<DropItem> {
    let mut items = Vec::new();
    if data.is_null() {
        return items;
    }
    // 1) 普通文件 / 文件夹
    let fmt = FORMATETC {
        cfFormat: CF_HDROP,
        ptd: std::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT,
        lindex: -1,
        tymed: TYMED_HGLOBAL,
    };
    let mut med: STGMEDIUM = std::mem::zeroed();
    if get_data(data, &fmt, &mut med) == S_OK && !med.hGlobal.is_null() {
        let drop = med.hGlobal as HANDLE;
        let n = DragQueryFileW(drop, 0xFFFF_FFFF, std::ptr::null_mut(), 0);
        for i in 0..n {
            let len = DragQueryFileW(drop, i, std::ptr::null_mut(), 0) as usize;
            if len == 0 {
                continue;
            }
            let mut buf = vec![0u16; len + 1];
            DragQueryFileW(drop, i, buf.as_mut_ptr(), (len + 1) as u32);
            items.push(DropItem::Path(String::from_utf16_lossy(&buf[..len])));
        }
        ReleaseStgMedium(&mut med);
        if !items.is_empty() {
            return items;
        }
    }
    // 2) shell 命名空间项（此电脑 / 回收站 …）
    let name = wide("Shell IDList Array");
    let cf = RegisterClipboardFormatW(name.as_ptr());
    if cf != 0 {
        let fmt = FORMATETC {
            cfFormat: cf as u16,
            ptd: std::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT,
            lindex: -1,
            tymed: TYMED_HGLOBAL,
        };
        let mut med: STGMEDIUM = std::mem::zeroed();
        if get_data(data, &fmt, &mut med) == S_OK && !med.hGlobal.is_null() {
            if let Some(key) = cida_to_sys(med.hGlobal) {
                items.push(DropItem::Sys(key));
            }
            ReleaseStgMedium(&mut med);
        }
    }
    items
}

/// 把 CIDA（Shell IDList Array）里的第一项转成内置功能 key
unsafe fn cida_to_sys(hglobal: *mut c_void) -> Option<&'static str> {
    let size = GlobalSize(hglobal) as usize;
    if size < 12 {
        return None;
    }
    let base = GlobalLock(hglobal) as *const u8;
    if base.is_null() {
        return None;
    }
    let out = cida_inner(base, size);
    GlobalUnlock(hglobal);
    out
}

unsafe fn cida_inner(base: *const u8, size: usize) -> Option<&'static str> {
    let cidl = std::ptr::read_unaligned(base as *const u32) as usize;
    if cidl == 0 || size < 8 + 4 * cidl {
        return None;
    }
    let aoff = base.add(4) as *const u32;
    let parent_off = std::ptr::read_unaligned(aoff) as usize;
    let item_off = std::ptr::read_unaligned(aoff.add(1)) as usize;
    if parent_off >= size {
        return None;
    }
    let parent = base.add(parent_off);
    let parent_limit = size - parent_off;
    // 两种约定都试：项目偏移相对 CIDA 起点，或相对父 PIDL 起点
    let mut cands = [std::ptr::null(), std::ptr::null()];
    if item_off < size {
        cands[0] = base.add(item_off);
    }
    if parent_off + item_off < size {
        cands[1] = parent.add(item_off);
    }
    for c in cands {
        if c.is_null() {
            continue;
        }
        let item_limit = size - (c as usize - base as usize);
        if let Some(name) = full_name(parent, parent_limit, c, item_limit) {
            if let Some(k) = name_to_sys(&name) {
                return Some(k);
            }
        }
    }
    None
}

unsafe fn pidl_len(p: *const u8, limit: usize) -> Option<usize> {
    let mut i = 0usize;
    loop {
        if i + 2 > limit {
            return None;
        }
        let cb = std::ptr::read_unaligned(p.add(i) as *const u16) as usize;
        i += 2;
        if cb == 0 {
            return Some(i);
        }
        if cb < 2 || i + (cb - 2) > limit {
            return None;
        }
        i += cb - 2;
    }
}

unsafe fn full_name(
    parent: *const u8,
    parent_limit: usize,
    item: *const u8,
    item_limit: usize,
) -> Option<String> {
    let pl = pidl_len(parent, parent_limit)?;
    let il = pidl_len(item, item_limit)?;
    let total = (pl - 2) + il;
    // 用 u16 缓冲保证 2 字节对齐
    let mut buf = vec![0u16; total / 2 + 1];
    let dst = buf.as_mut_ptr() as *mut u8;
    std::ptr::copy_nonoverlapping(parent, dst, pl - 2);
    std::ptr::copy_nonoverlapping(item, dst.add(pl - 2), il);
    let mut name: *mut u16 = std::ptr::null_mut();
    let hr = SHGetNameFromIDList(buf.as_ptr() as *const c_void, SIGDN_DESKTOPABSOLUTEPARSING, &mut name);
    if hr != 0 || name.is_null() {
        return None;
    }
    let s = {
        let mut n = 0usize;
        while *name.add(n) != 0 {
            n += 1;
        }
        String::from_utf16_lossy(std::slice::from_raw_parts(name, n))
    };
    CoTaskMemFree(name as *mut c_void);
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn name_to_sys(name: &str) -> Option<&'static str> {
    let u = name.to_uppercase();
    if u.contains("20D04FE0-3AEA-1069-A2D8-08002B30309D") {
        Some("thispc")
    } else if u.contains("645FF040-5081-101B-9F08-00AA002F954E") {
        Some("recyclebin")
    } else {
        None
    }
}

#[link(name = "ole32")]
extern "system" {
    fn OleInitialize(p: *mut c_void) -> i32;
    fn RegisterDragDrop(h: HWND, t: *mut c_void) -> i32;
    fn RevokeDragDrop(h: HWND) -> i32;
    fn ReleaseStgMedium(m: *mut STGMEDIUM);
}

#[link(name = "shell32")]
extern "system" {
    fn RegisterClipboardFormatW(name: LPCWSTR) -> UINT;
    fn SHGetNameFromIDList(pidl: *const c_void, sigdn: i32, name: *mut *mut u16) -> i32;
    fn SHParseDisplayName(
        name: LPCWSTR,
        bind: *mut c_void,
        pidl: *mut *mut c_void,
        attrs_in: u32,
        attrs_out: *mut u32,
    ) -> i32;
    fn CoInitializeEx(reserved: *mut c_void, coinit: u32) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_com() {
        unsafe {
            CoInitializeEx(std::ptr::null_mut(), 2); // COINIT_APARTMENTTHREADED
        }
    }

    fn pidl_of(clsid: &str) -> Vec<u8> {
        init_com();
        let w = wide(clsid);
        let mut pidl: *mut c_void = std::ptr::null_mut();
        let mut attrs: u32 = 0;
        let hr = unsafe { SHParseDisplayName(w.as_ptr(), std::ptr::null_mut(), &mut pidl, 0, &mut attrs) };
        assert_eq!(hr, 0, "SHParseDisplayName 失败");
        assert!(!pidl.is_null());
        // 读出一个完整的 ITEMIDLIST（以 0 结尾）
        let mut n = 0usize;
        unsafe {
            loop {
                let cb = std::ptr::read_unaligned((pidl as *const u8).add(n) as *const u16) as usize;
                n += 2;
                if cb == 0 {
                    break;
                }
                n += cb - 2;
            }
            let v = std::slice::from_raw_parts(pidl as *const u8, n).to_vec();
            CoTaskMemFree(pidl);
            v
        }
    }

    /// 按 Explorer 的 CIDA 布局拼一个「桌面拖出单项」的数据块
    fn make_cida(item: &[u8]) -> Vec<u8> {
        let parent: [u8; 2] = [0, 0]; // 桌面根
        let mut v = Vec::new();
        v.extend_from_slice(&1u32.to_le_bytes()); // cidl
        v.extend_from_slice(&12u32.to_le_bytes()); // aoffset[0] -> 父 PIDL
        v.extend_from_slice(&14u32.to_le_bytes()); // aoffset[1] -> 项目 PIDL
        v.extend_from_slice(&parent);
        v.extend_from_slice(item);
        v
    }

    #[test]
    fn cida_this_pc() {
        let raw = pidl_of("::{20D04FE0-3AEA-1069-A2D8-08002B30309D}");
        // 先确认直接解析原 PIDL 能拿到名字
        unsafe {
            let mut name: *mut u16 = std::ptr::null_mut();
            let hr = SHGetNameFromIDList(raw.as_ptr() as *const c_void, SIGDN_DESKTOPABSOLUTEPARSING, &mut name);
            assert_eq!(hr, 0, "SHGetNameFromIDList 失败");
            let mut n = 0usize;
            while *name.add(n) != 0 {
                n += 1;
            }
            let s = String::from_utf16_lossy(std::slice::from_raw_parts(name, n));
            CoTaskMemFree(name as *mut c_void);
            assert_eq!(name_to_sys(&s), Some("thispc"), "名字是 {}", s);
        }
        let blob = make_cida(&raw);
        let k = unsafe { cida_inner(blob.as_ptr(), blob.len()) };
        assert_eq!(k, Some("thispc"));
    }

    #[test]
    fn cida_recycle_bin() {
        let blob = make_cida(&pidl_of("::{645FF040-5081-101B-9F08-00AA002F954E}"));
        let k = unsafe { cida_inner(blob.as_ptr(), blob.len()) };
        assert_eq!(k, Some("recyclebin"));
    }
}
