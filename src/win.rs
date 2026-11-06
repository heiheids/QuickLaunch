//! 纯手工 Win32 FFI 声明。整个工程不依赖任何第三方 crate。
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use std::ffi::c_void;

pub type BOOL = i32;
pub type BYTE = u8;
pub type WORD = u16;
pub type DWORD = u32;
pub type UINT = u32;
pub type INT = i32;
pub type LONG = i32;
pub type LRESULT = isize;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type HINSTANCE = *mut c_void;
pub type HWND = *mut c_void;
pub type HDC = *mut c_void;
pub type HBITMAP = *mut c_void;
pub type HICON = *mut c_void;
pub type HCURSOR = *mut c_void;
pub type HMENU = *mut c_void;
pub type HMODULE = *mut c_void;
pub type HANDLE = *mut c_void;
pub type HFONT = *mut c_void;
pub type HBRUSH = *mut c_void;
pub type HGDIOBJ = *mut c_void;
pub type COLORREF = u32;
pub type LPCWSTR = *const u16;
pub type LPWSTR = *mut u16;
pub type UINT_PTR = usize;
pub type ULONG_PTR = usize;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SIZE {
    pub cx: i32,
    pub cy: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RECTF {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl RECT {
    pub fn new(l: i32, t: i32, r: i32, b: i32) -> RECT {
        RECT { left: l, top: t, right: r, bottom: b }
    }
    pub fn w(&self) -> i32 {
        self.right - self.left
    }
    pub fn h(&self) -> i32 {
        self.bottom - self.top
    }
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }
    pub fn deflate(&self, n: i32) -> RECT {
        RECT::new(self.left + n, self.top + n, self.right - n, self.bottom - n)
    }
    pub fn offset(&self, dx: i32, dy: i32) -> RECT {
        RECT::new(self.left + dx, self.top + dy, self.right + dx, self.bottom + dy)
    }
}

#[repr(C)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: UINT,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: DWORD,
    pub pt: POINT,
    pub lPrivate: DWORD,
}

pub type WNDPROC = Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>;

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cbSize: UINT,
    pub style: UINT,
    pub lpfnWndProc: WNDPROC,
    pub cbClsExtra: INT,
    pub cbWndExtra: INT,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub hCursor: HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: LPCWSTR,
    pub lpszClassName: LPCWSTR,
    pub hIconSm: HICON,
}

#[repr(C)]
#[derive(Default)]
pub struct BITMAPINFOHEADER {
    pub biSize: DWORD,
    pub biWidth: LONG,
    pub biHeight: LONG,
    pub biPlanes: WORD,
    pub biBitCount: WORD,
    pub biCompression: DWORD,
    pub biSizeImage: DWORD,
    pub biXPelsPerMeter: LONG,
    pub biYPelsPerMeter: LONG,
    pub biClrUsed: DWORD,
    pub biClrImportant: DWORD,
}

#[repr(C)]
#[derive(Default)]
pub struct BITMAPINFO {
    pub bmiHeader: BITMAPINFOHEADER,
    pub bmiColors: [DWORD; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BLENDFUNCTION {
    pub BlendOp: BYTE,
    pub BlendFlags: BYTE,
    pub SourceConstantAlpha: BYTE,
    pub AlphaFormat: BYTE,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ICONINFO {
    pub fIcon: BOOL,
    pub xHotspot: DWORD,
    pub yHotspot: DWORD,
    pub hbmMask: HBITMAP,
    pub hbmColor: HBITMAP,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BITMAP {
    pub bmType: LONG,
    pub bmWidth: LONG,
    pub bmHeight: LONG,
    pub bmWidthBytes: LONG,
    pub bmPlanes: WORD,
    pub bmBitsPixel: WORD,
    pub bmBits: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NOTIFYICONDATAW {
    pub cbSize: DWORD,
    pub hWnd: HWND,
    pub uID: UINT,
    pub uFlags: UINT,
    pub uCallbackMessage: UINT,
    pub hIcon: HICON,
    pub szTip: [u16; 128],
    pub dwState: DWORD,
    pub dwStateMask: DWORD,
    pub szInfo: [u16; 256],
    pub uTimeoutOrVersion: UINT,
    pub szInfoTitle: [u16; 64],
    pub dwInfoFlags: DWORD,
    pub guidItem: [u8; 16],
    pub hBalloonIcon: HICON,
}

impl Default for NOTIFYICONDATAW {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
pub struct SHFILEINFOW {
    pub hIcon: HICON,
    pub iIcon: i32,
    pub dwAttributes: DWORD,
    pub szDisplayName: [u16; 260],
    pub szTypeName: [u16; 80],
}

impl Default for SHFILEINFOW {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct OPENFILENAMEW {
    pub lStructSize: DWORD,
    pub hwndOwner: HWND,
    pub hInstance: HINSTANCE,
    pub lpstrFilter: LPCWSTR,
    pub lpstrCustomFilter: LPWSTR,
    pub nMaxCustFilter: DWORD,
    pub nFilterIndex: DWORD,
    pub lpstrFile: LPWSTR,
    pub nMaxFile: DWORD,
    pub lpstrFileTitle: LPWSTR,
    pub nMaxFileTitle: DWORD,
    pub lpstrInitialDir: LPCWSTR,
    pub lpstrTitle: LPCWSTR,
    pub Flags: DWORD,
    pub nFileOffset: WORD,
    pub nFileExtension: WORD,
    pub lpstrDefExt: LPCWSTR,
    pub lCustData: LPARAM,
    pub lpfnHook: *mut c_void,
    pub lpTemplateName: LPCWSTR,
    pub pvReserved: *mut c_void,
    pub dwReserved: DWORD,
    pub FlagsEx: DWORD,
}

#[repr(C)]
#[derive(Default)]
pub struct BROWSEINFOW {
    pub hwndOwner: HWND,
    pub pidlRoot: *mut c_void,
    pub pszDisplayName: LPWSTR,
    pub lpszTitle: LPCWSTR,
    pub ulFlags: UINT,
    pub lpfn: *mut c_void,
    pub lParam: LPARAM,
    pub iImage: INT,
}

#[repr(C)]
#[derive(Default)]
pub struct SHELLEXECUTEINFOW {
    pub cbSize: DWORD,
    pub fMask: DWORD,
    pub hwnd: HWND,
    pub lpVerb: LPCWSTR,
    pub lpFile: LPCWSTR,
    pub lpParameters: LPCWSTR,
    pub lpDirectory: LPCWSTR,
    pub nShow: INT,
    pub hInstApp: HINSTANCE,
    pub lpIDList: *mut c_void,
    pub lpClass: LPCWSTR,
    pub hkeyClass: HANDLE,
    pub dwHotKey: DWORD,
    pub hIconOrMonitor: HANDLE,
    pub hProcess: HANDLE,
}

#[repr(C)]
#[derive(Default)]
pub struct CHOOSECOLORW {
    pub lStructSize: DWORD,
    pub hwndOwner: HWND,
    pub hInstance: HINSTANCE,
    pub rgbResult: COLORREF,
    pub lpCustColors: *mut COLORREF,
    pub Flags: DWORD,
    pub lCustData: LPARAM,
    pub lpfnHook: *mut c_void,
    pub lpTemplateName: LPCWSTR,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SYSTEMTIME {
    pub wYear: WORD,
    pub wMonth: WORD,
    pub wDayOfWeek: WORD,
    pub wDay: WORD,
    pub wHour: WORD,
    pub wMinute: WORD,
    pub wSecond: WORD,
    pub wMilliseconds: WORD,
}

#[repr(C)]
pub struct PAINTSTRUCT {
    pub hdc: HDC,
    pub fErase: BOOL,
    pub rcPaint: RECT,
    pub fRestore: BOOL,
    pub fIncUpdate: BOOL,
    pub rgbReserved: [u8; 32],
}

impl Default for PAINTSTRUCT {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct MONITORINFO {
    pub cbSize: DWORD,
    pub rcMonitor: RECT,
    pub rcWork: RECT,
    pub dwFlags: DWORD,
}

#[repr(C)]
#[derive(Default)]
pub struct DROPFILES {
    pub pFiles: DWORD,
    pub pt: POINT,
    pub fNC: BOOL,
    pub fWide: BOOL,
}

// ---------------- 常量 ----------------
pub const CW_USEDEFAULT: i32 = -2147483648;
pub const WS_POPUP: u32 = 0x8000_0000;
pub const WS_CHILD: u32 = 0x4000_0000;
pub const WS_VISIBLE: u32 = 0x1000_0000;
pub const WS_CLIPCHILDREN: u32 = 0x0200_0000;
pub const WS_BORDER: u32 = 0x0080_0000;
pub const WS_TABSTOP: u32 = 0x0001_0000;
pub const WS_EX_LAYERED: u32 = 0x0008_0000;
pub const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
pub const WS_EX_TOPMOST: u32 = 0x0000_0008;
pub const WS_EX_NOACTIVATE: u32 = 0x0800_0000;

pub const SW_HIDE: i32 = 0;
pub const SW_SHOW: i32 = 5;
pub const SW_SHOWNORMAL: i32 = 1;

pub const WM_CREATE: u32 = 0x0001;
pub const WM_DESTROY: u32 = 0x0002;
pub const WM_CLOSE: u32 = 0x0010;
pub const WM_PAINT: u32 = 0x000F;
pub const WM_ERASEBKGND: u32 = 0x0014;
pub const WM_SETCURSOR: u32 = 0x0020;
pub const WM_MOUSEMOVE: u32 = 0x0200;
pub const WM_LBUTTONDOWN: u32 = 0x0201;
pub const WM_LBUTTONUP: u32 = 0x0202;
pub const WM_LBUTTONDBLCLK: u32 = 0x0203;
pub const WM_RBUTTONDOWN: u32 = 0x0204;
pub const WM_RBUTTONUP: u32 = 0x0205;
pub const WM_MOUSEWHEEL: u32 = 0x020A;
pub const WM_KEYDOWN: u32 = 0x0100;
pub const WM_CHAR: u32 = 0x0102;
pub const WM_IME_CHAR: u32 = 0x0286;
pub const WM_TIMER: u32 = 0x0113;
pub const WM_NCHITTEST: u32 = 0x0084;
pub const WM_SETFOCUS: u32 = 0x0007;
pub const WM_KILLFOCUS: u32 = 0x0008;
pub const WM_ACTIVATE: u32 = 0x0006;
pub const WM_COMMAND: u32 = 0x0111;
pub const WM_CTLCOLOREDIT: u32 = 0x0133;
pub const WM_CTLCOLORBTN: u32 = 0x0135;
pub const WM_DROPFILES: u32 = 0x0233;
pub const WM_SETFONT: u32 = 0x0030;
pub const WM_APP: u32 = 0x8000;
pub const WM_HOTKEY: u32 = 0x0312;
pub const WM_SYSKEYDOWN: u32 = 0x0104;
pub const WM_DISPLAYCHANGE: u32 = 0x007E;
pub const WM_NCLBUTTONDOWN: u32 = 0x00A1;
pub const WM_CAPTURECHANGED: u32 = 0x0215;

pub const HTCLIENT: i32 = 1;
pub const HTCAPTION: i32 = 2;
pub const HTLEFT: i32 = 10;
pub const HTRIGHT: i32 = 11;
pub const HTTOP: i32 = 12;
pub const HTTOPLEFT: i32 = 13;
pub const HTTOPRIGHT: i32 = 14;
pub const HTBOTTOM: i32 = 15;
pub const HTBOTTOMLEFT: i32 = 16;
pub const HTBOTTOMRIGHT: i32 = 17;
pub const HTTRANSPARENT: i32 = -1;

pub const SWP_NOSIZE: u32 = 0x0001;
pub const SWP_NOMOVE: u32 = 0x0002;
pub const SWP_NOZORDER: u32 = 0x0004;
pub const SWP_NOACTIVATE: u32 = 0x0010;
pub const GWL_EXSTYLE: i32 = -20;
pub const GWLP_USERDATA: i32 = -21;
pub const GWLP_WNDPROC: i32 = -4;

pub const ULW_ALPHA: u32 = 0x0002;
pub const AC_SRC_OVER: u8 = 0;
pub const AC_SRC_ALPHA: u8 = 1;

pub const NIM_ADD: u32 = 0;
pub const NIM_MODIFY: u32 = 1;
pub const NIM_DELETE: u32 = 2;
pub const NIF_MESSAGE: u32 = 1;
pub const NIF_ICON: u32 = 2;
pub const NIF_TIP: u32 = 4;

pub const TPM_LEFTALIGN: u32 = 0x0000;
pub const TPM_TOPALIGN: u32 = 0x0000;
pub const TPM_RIGHTBUTTON: u32 = 0x0002;
pub const TPM_RETURNCMD: u32 = 0x0100;

pub const MF_STRING: u32 = 0x0000;
pub const MF_SEPARATOR: u32 = 0x0800;
pub const MF_GRAYED: u32 = 0x0001;
pub const MF_DISABLED: u32 = 0x0002;
pub const MF_CHECKED: u32 = 0x0008;
pub const MF_POPUP: u32 = 0x0010;
pub const MF_UNCHECKED: u32 = 0x0000;

pub const MB_OK: u32 = 0x0;
pub const MB_YESNO: u32 = 0x4;
pub const MB_ICONINFORMATION: u32 = 0x40;
pub const MB_ICONWARNING: u32 = 0x30;
pub const MB_ICONQUESTION: u32 = 0x20;
pub const IDYES: i32 = 6;

pub const SM_CXSCREEN: i32 = 0;
pub const SM_CXICON: i32 = 11;
pub const SM_CYICON: i32 = 12;
pub const SM_CXSMICON: i32 = 49;
pub const SM_CYSMICON: i32 = 50;
pub const IMAGE_ICON: u32 = 1;
pub const LR_DEFAULTCOLOR: u32 = 0;
pub const LR_SHARED: u32 = 0x8000;
pub const SM_CYSCREEN: i32 = 1;
pub const SM_CXDRAG: i32 = 68;
pub const SM_CYDRAG: i32 = 69;

pub const IDC_ARROW: usize = 32512;
pub const IDC_SIZEWE: usize = 32644;
pub const IDC_SIZENS: usize = 32645;
pub const IDC_SIZENWSE: usize = 32642;
pub const IDC_SIZENESW: usize = 32643;
pub const IDC_HAND: usize = 32649;
pub const IDC_IBEAM: usize = 32513;

pub const OFN_FILEMUSTEXIST: u32 = 0x0000_1000;
pub const OFN_PATHMUSTEXIST: u32 = 0x0000_0800;
pub const OFN_ALLOWMULTISELECT: u32 = 0x0000_0200;
pub const OFN_EXPLORER: u32 = 0x0008_0000;
pub const OFN_HIDEREADONLY: u32 = 0x0000_0004;

pub const BIF_RETURNONLYFSDIRS: u32 = 0x0001;
pub const BIF_NEWDIALOGSTYLE: u32 = 0x0040;
pub const BIF_EDITBOX: u32 = 0x0010;

pub const SHGFI_ICON: u32 = 0x0000_0100;
pub const SHGFI_LARGEICON: u32 = 0x0000_0000;
pub const SHGFI_SMALLICON: u32 = 0x0000_0001;
pub const SHGFI_USEFILEATTRIBUTES: u32 = 0x0000_0010;
pub const SHGFI_PIDL: u32 = 0x0000_0008;

pub const SEE_MASK_NOCLOSEPROCESS: u32 = 0x0000_0040;
pub const SEE_MASK_FLAG_NO_UI: u32 = 0x0000_0400;

pub const DIB_RGB_COLORS: u32 = 0;
pub const BI_RGB: u32 = 0;

pub const FW_NORMAL: i32 = 400;
pub const FW_BOLD: i32 = 700;
pub const DEFAULT_CHARSET: u32 = 1;
pub const CLEARTYPE_QUALITY: u32 = 5;
pub const OUT_DEFAULT_PRECIS: u32 = 0;
pub const CLIP_DEFAULT_PRECIS: u32 = 0;
pub const DEFAULT_PITCH: u32 = 0;
pub const DEFAULT_GUI_FONT: i32 = 17;

pub const ES_LEFT: u32 = 0x0000;
pub const ES_AUTOHSCROLL: u32 = 0x0080;
pub const ES_PASSWORD: u32 = 0x0020;

pub const CC_RGBINIT: u32 = 0x0000_0001;
pub const CC_FULLOPEN: u32 = 0x0000_0002;
pub const CC_ANYCOLOR: u32 = 0x0000_0100;

pub const GA_ROOT: u32 = 2;

pub const PROCESS_DPI_AWARENESS_UNAWARE: i32 = 0;

// ---------------- 导入函数 ----------------
#[link(name = "kernel32")]
extern "system" {
    pub fn GlobalLock(h: HANDLE) -> *mut c_void;
    pub fn GlobalUnlock(h: HANDLE) -> BOOL;
    pub fn GlobalSize(h: HANDLE) -> usize;
    pub fn GetModuleHandleW(name: LPCWSTR) -> HMODULE;
    pub fn LoadImageW(
        inst: HINSTANCE,
        name: LPCWSTR,
        typ: UINT,
        cx: INT,
        cy: INT,
        flags: UINT,
    ) -> HANDLE;
    pub fn GetTickCount64() -> u64;
    pub fn Sleep(ms: DWORD);
    pub fn CreateMutexW(attr: *mut c_void, owner: BOOL, name: LPCWSTR) -> HANDLE;
    pub fn GetLastError() -> DWORD;
    pub fn CloseHandle(h: HANDLE) -> BOOL;
    pub fn GetCurrentProcessId() -> DWORD;
    pub fn GetCurrentThreadId() -> DWORD;
    pub fn ExitProcess(code: UINT) -> !;
    pub fn GetModuleFileNameW(h: HMODULE, buf: LPWSTR, n: DWORD) -> DWORD;
    pub fn GetSystemTimeAsFileTime(t: *mut u64);
    pub fn GetLocalTime(t: *mut SYSTEMTIME);
    pub fn GetFileAttributesW(p: LPCWSTR) -> DWORD;
}

#[link(name = "user32")]
extern "system" {
    pub fn RegisterClassExW(c: *const WNDCLASSEXW) -> u16;
    pub fn CreateWindowExW(
        ex: DWORD,
        class: LPCWSTR,
        title: LPCWSTR,
        style: DWORD,
        x: INT,
        y: INT,
        w: INT,
        h: INT,
        parent: HWND,
        menu: HMENU,
        inst: HINSTANCE,
        param: *mut c_void,
    ) -> HWND;
    pub fn DefWindowProcW(h: HWND, m: UINT, w: WPARAM, l: LPARAM) -> LRESULT;
    pub fn DestroyWindow(h: HWND) -> BOOL;
    pub fn ShowWindow(h: HWND, cmd: INT) -> BOOL;
    pub fn SetWindowPos(h: HWND, after: HWND, x: INT, y: INT, cx: INT, cy: INT, flags: UINT) -> BOOL;
    pub fn GetWindowRect(h: HWND, r: *mut RECT) -> BOOL;
    pub fn GetClientRect(h: HWND, r: *mut RECT) -> BOOL;
    pub fn GetMessageW(m: *mut MSG, h: HWND, min: UINT, max: UINT) -> BOOL;
    pub fn TranslateMessage(m: *const MSG) -> BOOL;
    pub fn DispatchMessageW(m: *const MSG) -> LRESULT;
    pub fn PostQuitMessage(code: INT);
    pub fn PostMessageW(h: HWND, m: UINT, w: WPARAM, l: LPARAM) -> BOOL;
    pub fn SendMessageW(h: HWND, m: UINT, w: WPARAM, l: LPARAM) -> LRESULT;
    pub fn SetWindowLongPtrW(h: HWND, idx: INT, v: isize) -> isize;
    pub fn GetWindowLongPtrW(h: HWND, idx: INT) -> isize;
    pub fn CallWindowProcW(p: WNDPROC, h: HWND, m: UINT, w: WPARAM, l: LPARAM) -> LRESULT;
    pub fn SetFocus(h: HWND) -> HWND;
    pub fn GetFocus() -> HWND;
    pub fn SetCapture(h: HWND) -> HWND;
    pub fn ReleaseCapture() -> BOOL;
    pub fn GetCursorPos(p: *mut POINT) -> BOOL;
    pub fn SetCursorPos(x: INT, y: INT) -> BOOL;
    pub fn ScreenToClient(h: HWND, p: *mut POINT) -> BOOL;
    pub fn ClientToScreen(h: HWND, p: *mut POINT) -> BOOL;
    pub fn GetDC(h: HWND) -> HDC;
    pub fn ReleaseDC(h: HWND, dc: HDC) -> INT;
    pub fn InvalidateRect(h: HWND, r: *const RECT, erase: BOOL) -> BOOL;
    pub fn UpdateWindow(h: HWND) -> BOOL;
    pub fn LoadCursorW(h: HINSTANCE, name: LPCWSTR) -> HCURSOR;
    pub fn SetCursor(c: HCURSOR) -> HCURSOR;
    pub fn CreatePopupMenu() -> HMENU;
    pub fn AppendMenuW(m: HMENU, flags: UINT, id: usize, s: LPCWSTR) -> BOOL;
    pub fn DestroyMenu(m: HMENU) -> BOOL;
    pub fn TrackPopupMenuEx(m: HMENU, flags: UINT, x: INT, y: INT, h: HWND, p: *const c_void) -> INT;
    pub fn SetForegroundWindow(h: HWND) -> BOOL;
    pub fn RegisterHotKey(h: HWND, id: INT, mods: UINT, vk: UINT) -> BOOL;
    pub fn UnregisterHotKey(h: HWND, id: INT) -> BOOL;
    pub fn GetForegroundWindow() -> HWND;
    pub fn SetTimer(h: HWND, id: usize, ms: UINT, proc: *mut c_void) -> usize;
    pub fn KillTimer(h: HWND, id: usize) -> BOOL;
    pub fn MessageBoxW(h: HWND, text: LPCWSTR, cap: LPCWSTR, flags: UINT) -> INT;
    pub fn GetSystemMetrics(i: INT) -> INT;
    pub fn UpdateLayeredWindow(
        h: HWND,
        dst: HDC,
        pos: *const POINT,
        size: *const SIZE,
        src: HDC,
        srcPos: *const POINT,
        colorKey: COLORREF,
        blend: *const BLENDFUNCTION,
        flags: DWORD,
    ) -> BOOL;
    pub fn GetMonitorInfoW(h: *mut c_void, info: *mut MONITORINFO) -> BOOL;
    pub fn MonitorFromWindow(h: HWND, flags: DWORD) -> *mut c_void;
    pub fn MonitorFromPoint(p: POINT, flags: DWORD) -> *mut c_void;
    pub fn SetProcessDPIAware() -> BOOL;
    pub fn IsWindowVisible(h: HWND) -> BOOL;
    pub fn GetWindow(h: HWND, cmd: UINT) -> HWND;
    pub fn PrivateExtractIconsW(
        file: LPCWSTR,
        index: INT,
        cx: INT,
        cy: INT,
        icons: *mut HICON,
        ids: *mut UINT,
        n: UINT,
        flags: UINT,
    ) -> UINT;
    pub fn DestroyIcon(i: HICON) -> BOOL;
    pub fn GetIconInfo(i: HICON, info: *mut ICONINFO) -> BOOL;
    pub fn RegisterWindowMessageW(s: LPCWSTR) -> UINT;
    pub fn SetWindowTextW(h: HWND, s: LPCWSTR) -> BOOL;
    pub fn GetWindowTextW(h: HWND, s: LPWSTR, n: INT) -> INT;
    pub fn GetWindowTextLengthW(h: HWND) -> INT;
    pub fn GetClassNameW(h: HWND, s: LPWSTR, n: INT) -> INT;
    pub fn GetParent(h: HWND) -> HWND;
    pub fn IsWindow(h: HWND) -> BOOL;
    pub fn EnableWindow(h: HWND, e: BOOL) -> BOOL;
    pub fn GetKeyState(k: INT) -> i16;
    pub fn OpenClipboard(h: HWND) -> BOOL;
    pub fn CloseClipboard() -> BOOL;
    pub fn GetClipboardData(format: UINT) -> HANDLE;
    pub fn GetWindowThreadProcessId(h: HWND, pid: *mut DWORD) -> DWORD;
    pub fn AttachThreadInput(a: DWORD, b: DWORD, attach: BOOL) -> BOOL;
    pub fn BringWindowToTop(h: HWND) -> BOOL;
}

#[link(name = "gdi32")]
extern "system" {
    pub fn CreateDIBSection(
        dc: HDC,
        bi: *const BITMAPINFO,
        usage: UINT,
        bits: *mut *mut c_void,
        section: HANDLE,
        offset: DWORD,
    ) -> HBITMAP;
    pub fn CreateCompatibleDC(dc: HDC) -> HDC;
    pub fn SelectObject(dc: HDC, o: HGDIOBJ) -> HGDIOBJ;
    pub fn DeleteObject(o: HGDIOBJ) -> BOOL;
    pub fn DeleteDC(dc: HDC) -> BOOL;
    pub fn CreateFontW(
        h: INT,
        w: INT,
        esc: INT,
        orient: INT,
        weight: INT,
        italic: DWORD,
        underline: DWORD,
        strike: DWORD,
        charset: DWORD,
        outprec: DWORD,
        clipprec: DWORD,
        quality: DWORD,
        pitch: DWORD,
        face: LPCWSTR,
    ) -> HFONT;
    pub fn GetStockObject(i: INT) -> HGDIOBJ;
    pub fn GetObjectW(o: HGDIOBJ, n: INT, p: *mut c_void) -> INT;
    pub fn GetDIBits(
        dc: HDC,
        bmp: HBITMAP,
        start: UINT,
        lines: UINT,
        bits: *mut c_void,
        bi: *mut BITMAPINFO,
        usage: UINT,
    ) -> INT;
    pub fn SetBkMode(dc: HDC, mode: INT) -> INT;
    pub fn SetTextColor(dc: HDC, c: COLORREF) -> COLORREF;
    pub fn SetBkColor(dc: HDC, c: COLORREF) -> COLORREF;
    pub fn CreateSolidBrush(c: COLORREF) -> HBRUSH;
    pub fn BeginPaint(h: HWND, ps: *mut PAINTSTRUCT) -> HDC;
    pub fn EndPaint(h: HWND, ps: *const PAINTSTRUCT) -> BOOL;
    pub fn FillRect(dc: HDC, r: *const RECT, br: HBRUSH) -> INT;
    pub fn DrawTextW(dc: HDC, s: LPCWSTR, n: INT, r: *mut RECT, flags: UINT) -> INT;
}

#[link(name = "shell32")]
extern "system" {
    pub fn Shell_NotifyIconW(msg: DWORD, data: *mut NOTIFYICONDATAW) -> BOOL;
    pub fn ShellExecuteExW(info: *mut SHELLEXECUTEINFOW) -> BOOL;
    pub fn ShellExecuteW(
        h: HWND,
        verb: LPCWSTR,
        file: LPCWSTR,
        params: LPCWSTR,
        dir: LPCWSTR,
        show: INT,
    ) -> HINSTANCE;
    pub fn SHGetFileInfoW(
        path: LPCWSTR,
        attrs: DWORD,
        info: *mut SHFILEINFOW,
        size: UINT,
        flags: UINT,
    ) -> usize;
    pub fn SHBrowseForFolderW(bi: *mut BROWSEINFOW) -> *mut c_void;
    pub fn SHParseDisplayName(
        name: LPCWSTR,
        bind: *mut c_void,
        pidl: *mut *mut c_void,
        attrs_in: DWORD,
        attrs_out: *mut DWORD,
    ) -> i32;
    pub fn SHGetPathFromIDListW(pidl: *const c_void, path: LPWSTR) -> BOOL;
    pub fn DragAcceptFiles(h: HWND, accept: BOOL);
    pub fn DragQueryFileW(drop: HANDLE, idx: UINT, buf: LPWSTR, n: UINT) -> UINT;
    pub fn DragFinish(drop: HANDLE);
    pub fn DragQueryPoint(drop: HANDLE, pt: *mut POINT) -> BOOL;
    pub fn SHEmptyRecycleBinW(hwnd: HWND, root: LPCWSTR, flags: DWORD) -> i32;
}

pub const SHERB_NOCONFIRMATION: DWORD = 1;
pub const SHERB_NOPROGRESSUI: DWORD = 2;
pub const SHERB_NOSOUND: DWORD = 4;

#[link(name = "ole32")]
extern "system" {
    pub fn CoTaskMemFree(p: *mut c_void);
}

#[link(name = "comdlg32")]
extern "system" {
    pub fn GetOpenFileNameW(ofn: *mut OPENFILENAMEW) -> BOOL;
    pub fn ChooseColorW(cc: *mut CHOOSECOLORW) -> BOOL;
}

#[link(name = "advapi32")]
extern "system" {
    pub fn RegSetKeyValueW(
        key: *mut c_void,
        sub: LPCWSTR,
        name: LPCWSTR,
        typ: DWORD,
        data: *const c_void,
        len: DWORD,
    ) -> i32;
    pub fn RegDeleteKeyValueW(key: *mut c_void, sub: LPCWSTR, name: LPCWSTR) -> i32;
    pub fn RegGetValueW(
        key: *mut c_void,
        sub: LPCWSTR,
        name: LPCWSTR,
        flags: DWORD,
        typ: *mut DWORD,
        data: *mut c_void,
        len: *mut DWORD,
    ) -> i32;
}

pub const HKEY_CURRENT_USER: *mut c_void = 0x8000_0001u64 as *mut c_void;
pub const REG_SZ: DWORD = 1;
pub const RRF_RT_REG_SZ: DWORD = 2;

// ---------------- GDI+ ----------------
#[repr(C)]
pub struct GdiplusStartupInput {
    pub GdiplusVersion: u32,
    pub DebugEventCallback: *mut c_void,
    pub SuppressBackgroundThread: BOOL,
    pub SuppressExternalCodecs: BOOL,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct GpRect {
    pub X: i32,
    pub Y: i32,
    pub Width: i32,
    pub Height: i32,
}

#[repr(C)]
#[derive(Default)]
pub struct BitmapData {
    pub width: u32,
    pub height: u32,
    pub stride: i32,
    pub pixel_format: i32,
    pub scan0: *mut u8,
    pub reserved: usize,
}

pub type GpStatus = i32;
pub type GpGraphics = *mut c_void;
pub type GpImage = *mut c_void;
pub type GpBrush = *mut c_void;
pub type GpPen = *mut c_void;
pub type GpPath = *mut c_void;
pub type GpFont = *mut c_void;
pub type GpFontFamily = *mut c_void;
pub type GpStringFormat = *mut c_void;

extern "system" {
    pub fn GdiplusStartup(token: *mut usize, input: *const GdiplusStartupInput, output: *mut c_void) -> GpStatus;
    pub fn GdiplusShutdown(token: usize);
    pub fn GdipCreateBitmapFromScan0(
        w: INT,
        h: INT,
        stride: INT,
        format: INT,
        scan0: *mut u8,
        bmp: *mut GpImage,
    ) -> GpStatus;
    pub fn GdipCreateBitmapFromFile(name: LPCWSTR, bmp: *mut GpImage) -> GpStatus;
    pub fn GdipCreateBitmapFromHICON(icon: HICON, bmp: *mut GpImage) -> GpStatus;
    pub fn GdipCreateHICONFromBitmap(bmp: GpImage, icon: *mut HICON) -> GpStatus;
    pub fn GdipDisposeImage(bmp: GpImage) -> GpStatus;
    pub fn GdipGetImageGraphicsContext(img: GpImage, g: *mut GpGraphics) -> GpStatus;
    pub fn GdipGetImageWidth(img: GpImage, w: *mut u32) -> GpStatus;
    pub fn GdipGetImageHeight(img: GpImage, h: *mut u32) -> GpStatus;
    pub fn GdipDeleteGraphics(g: GpGraphics) -> GpStatus;
    pub fn GdipGraphicsClear(g: GpGraphics, color: u32) -> GpStatus;
    pub fn GdipSetSmoothingMode(g: GpGraphics, m: INT) -> GpStatus;
    pub fn GdipSetTextRenderingHint(g: GpGraphics, h: INT) -> GpStatus;
    pub fn GdipSetInterpolationMode(g: GpGraphics, m: INT) -> GpStatus;
    pub fn GdipSetPixelOffsetMode(g: GpGraphics, m: INT) -> GpStatus;
    pub fn GdipSetCompositingQuality(g: GpGraphics, q: INT) -> GpStatus;
    pub fn GdipSetCompositingMode(g: GpGraphics, m: INT) -> GpStatus;
    pub fn GdipSetClipRect(g: GpGraphics, x: f32, y: f32, w: f32, h: f32, combine: INT) -> GpStatus;
    pub fn GdipResetClip(g: GpGraphics) -> GpStatus;
    pub fn GdipCreateSolidFill(color: u32, brush: *mut GpBrush) -> GpStatus;
    pub fn GdipSetSolidFillColor(brush: GpBrush, color: u32) -> GpStatus;
    pub fn GdipDeleteBrush(brush: GpBrush) -> GpStatus;
    pub fn GdipCreatePen1(color: u32, width: f32, unit: INT, pen: *mut GpPen) -> GpStatus;
    pub fn GdipDeletePen(pen: GpPen) -> GpStatus;
    pub fn GdipSetPenColor(pen: GpPen, color: u32) -> GpStatus;
    pub fn GdipSetPenWidth(pen: GpPen, width: f32) -> GpStatus;
    pub fn GdipCreatePath(fill: INT, path: *mut GpPath) -> GpStatus;
    pub fn GdipDeletePath(path: GpPath) -> GpStatus;
    pub fn GdipResetPath(path: GpPath) -> GpStatus;
    pub fn GdipAddPathRectangle(path: GpPath, x: f32, y: f32, w: f32, h: f32) -> GpStatus;
    pub fn GdipAddPathArc(path: GpPath, x: f32, y: f32, w: f32, h: f32, start: f32, sweep: f32) -> GpStatus;
    pub fn GdipAddPathLine(path: GpPath, x1: f32, y1: f32, x2: f32, y2: f32) -> GpStatus;
    pub fn GdipClosePathFigure(path: GpPath) -> GpStatus;
    pub fn GdipStartPathFigure(path: GpPath) -> GpStatus;
    pub fn GdipFillPath(g: GpGraphics, brush: GpBrush, path: GpPath) -> GpStatus;
    pub fn GdipDrawPath(g: GpGraphics, pen: GpPen, path: GpPath) -> GpStatus;
    pub fn GdipFillRectangle(g: GpGraphics, brush: GpBrush, x: f32, y: f32, w: f32, h: f32) -> GpStatus;
    pub fn GdipDrawRectangle(g: GpGraphics, pen: GpPen, x: f32, y: f32, w: f32, h: f32) -> GpStatus;
    pub fn GdipDrawLine(g: GpGraphics, pen: GpPen, x1: f32, y1: f32, x2: f32, y2: f32) -> GpStatus;
    pub fn GdipCreateFontFamilyFromName(
        name: LPCWSTR,
        collection: *mut c_void,
        out: *mut GpFontFamily,
    ) -> GpStatus;
    pub fn GdipDeleteFontFamily(f: GpFontFamily) -> GpStatus;
    pub fn GdipCreateFont(family: GpFontFamily, size: f32, style: INT, unit: INT, out: *mut GpFont) -> GpStatus;
    pub fn GdipDeleteFont(f: GpFont) -> GpStatus;
    pub fn GdipCreateStringFormat(flags: INT, lang: u16, out: *mut GpStringFormat) -> GpStatus;
    pub fn GdipDeleteStringFormat(f: GpStringFormat) -> GpStatus;
    pub fn GdipSetStringFormatAlign(f: GpStringFormat, a: INT) -> GpStatus;
    pub fn GdipSetStringFormatLineAlign(f: GpStringFormat, a: INT) -> GpStatus;
    pub fn GdipSetStringFormatTrimming(f: GpStringFormat, t: INT) -> GpStatus;
    pub fn GdipSetStringFormatFlags(f: GpStringFormat, flags: INT) -> GpStatus;
    pub fn GdipDrawString(
        g: GpGraphics,
        s: *const u16,
        len: INT,
        font: GpFont,
        rect: *const RECTF,
        fmt: GpStringFormat,
        brush: GpBrush,
    ) -> GpStatus;
    pub fn GdipMeasureString(
        g: GpGraphics,
        s: *const u16,
        len: INT,
        font: GpFont,
        rect: *const RECTF,
        fmt: GpStringFormat,
        bounds: *mut RECTF,
        fitted: *mut INT,
        lines: *mut INT,
    ) -> GpStatus;
    pub fn GdipDrawImageRectRect(
        g: GpGraphics,
        img: GpImage,
        dx: f32,
        dy: f32,
        dw: f32,
        dh: f32,
        sx: f32,
        sy: f32,
        sw: f32,
        sh: f32,
        unit: INT,
        attrs: *mut c_void,
        cb: *mut c_void,
        data: *mut c_void,
    ) -> GpStatus;
    pub fn GdipSaveImageToFile(img: GpImage, name: LPCWSTR, clsid: *const [u8; 16], enc: *const c_void) -> GpStatus;
    pub fn GdipBitmapLockBits(
        bmp: GpImage,
        rc: *const GpRect,
        flags: u32,
        format: i32,
        data: *mut BitmapData,
    ) -> GpStatus;
    pub fn GdipBitmapUnlockBits(bmp: GpImage, data: *mut BitmapData) -> GpStatus;
}

// ---------------- 辅助 ----------------
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn to_wide_buf(s: &str, buf: &mut [u16]) {
    let v: Vec<u16> = s.encode_utf16().collect();
    let n = v.len().min(buf.len().saturating_sub(1));
    buf[..n].copy_from_slice(&v[..n]);
    buf[n] = 0;
}

pub fn from_wide(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}
