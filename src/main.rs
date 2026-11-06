#![windows_subsystem = "windows"]

mod app;
mod data;
mod drop;
mod gfx;
mod icons;
mod win;

use app::App;
use std::path::PathBuf;
use win::*;

unsafe extern "system" fn wndproc(h: HWND, m: UINT, w: WPARAM, l: LPARAM) -> LRESULT {
    if m == WM_DESTROY {
        PostQuitMessage(0);
        return 0;
    }
    let p = GetWindowLongPtrW(h, GWLP_USERDATA) as *mut App;
    if p.is_null() {
        return DefWindowProcW(h, m, w, l);
    }
    let a = &mut *p;
    match a.on_message(m, w, l) {
        Some(r) => r,
        None => DefWindowProcW(h, m, w, l),
    }
}

fn exe_dir() -> PathBuf {
    let mut buf = [0u16; 1024];
    let n = unsafe { GetModuleFileNameW(std::ptr::null_mut(), buf.as_mut_ptr(), 1024) };
    let s = String::from_utf16_lossy(&buf[..n as usize]);
    let p = PathBuf::from(s);
    p.parent().map(|x| x.to_path_buf()).unwrap_or_else(|| PathBuf::from("."))
}

fn main() {
    unsafe {
        SetProcessDPIAware();

        let mut token: usize = 0;
        let input = GdiplusStartupInput {
            GdiplusVersion: 1,
            DebugEventCallback: std::ptr::null_mut(),
            SuppressBackgroundThread: 0,
            SuppressExternalCodecs: 0,
        };
        if GdiplusStartup(&mut token, &input, std::ptr::null_mut()) != 0 {
            return;
        }

        // 单实例
        let mname = wide("QuickLaunch_SingleInstance_9F2A");
        let _mutex = CreateMutexW(std::ptr::null_mut(), 0, mname.as_ptr());
        if GetLastError() == 183 {
            ExitProcess(0);
        }

        let dir = exe_dir();
        let inst = GetModuleHandleW(std::ptr::null());
        let cls_name = wide("QLMainWnd");
        let icx = GetSystemMetrics(SM_CXICON);
        let icy = GetSystemMetrics(SM_CYICON);
        let scx = GetSystemMetrics(SM_CXSMICON);
        let scy = GetSystemMetrics(SM_CYSMICON);
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: inst,
            hIcon: app::app_icon(icx, icy),
            hCursor: LoadCursorW(std::ptr::null_mut(), 32512 as LPCWSTR),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: cls_name.as_ptr(),
            hIconSm: app::app_icon(scx, scy),
        };
        if RegisterClassExW(&wc) == 0 {
            return;
        }

        let conf = data::Conf::load(&dir);
        let mut x = conf.x;
        let mut y = conf.y;
        if x < 0 || y < 0 {
            let sw = GetSystemMetrics(SM_CXSCREEN);
            x = (sw - conf.w - 40).max(0);
            y = 80;
        }
        let ex_style = WS_EX_LAYERED | WS_EX_TOOLWINDOW | if conf.topmost { WS_EX_TOPMOST } else { 0 };
        let title = wide("QuickLaunch");
        let hwnd = CreateWindowExW(
            ex_style,
            cls_name.as_ptr(),
            title.as_ptr(),
            WS_POPUP,
            x,
            y,
            conf.w,
            conf.h,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            inst,
            std::ptr::null_mut(),
        );
        if hwnd.is_null() {
            return;
        }

        let boxed = Box::new(App::new(hwnd, dir));
        let ptr = Box::into_raw(boxed);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr as isize);
        let app = &mut *ptr;
        DragAcceptFiles(hwnd, 1);
        let sink = drop::register(hwnd);
        let _ = sink;
        app.init_tray();
        app.register_hotkey();
        app.show();

        if std::env::var_os("QL_BENCH").is_some() {
            app.bench();
            ExitProcess(0);
        }
        if std::env::var_os("QL_TEST_UI").is_some() {
            app.test_open_settings();
        }

        let mut msg: MSG = std::mem::zeroed();
        loop {
            let r = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
            if r == 0 || r == -1 {
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        GdiplusShutdown(token);
    }
}
