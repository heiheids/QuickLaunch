//! 网站图标（favicon）抓取。
//!
//! 设计要点：
//! 1. 只用系统自带的 winhttp.dll（工具链自带 libwinhttp.a），不引入任何第三方依赖。
//! 2. 网络请求一律放在后台工作线程，界面线程只通过 mpsc 投递任务、靠
//!    PostMessageW 唤醒，绝不阻塞主循环——否则点一下“添加网址”窗口就卡住。
//! 3. 抓到的字节直接落盘成 .ico，交给 icons.rs 现有的图标管线
//!    （GDI+ 能读 ico，也能自动识别里面其实是 PNG 的情况）。
#![allow(dead_code)]

use std::collections::HashSet;
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};

/// 抓取完成后通知主窗口的消息（lParam 未用）
pub const WM_FAVICON_READY: u32 = crate::win::WM_APP + 11;

const MAX_BYTES: usize = 512 * 1024;
const TIMEOUT_MS: i32 = 3000;

/// 主机名规范化：小写、去掉 www. 前缀、去掉端口
pub fn host_of(url: &str) -> Option<String> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let host = rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .to_string();
    // 去掉 userinfo 与端口
    let host = host.rsplit('@').next().unwrap_or(&host).to_string();
    let host = host.split(':').next().unwrap_or("").to_string();
    if host.is_empty() {
        return None;
    }
    Some(host)
}

/// 显示用名称：去掉 www. 前缀，保留其余部分
/// 只影响名字，不改真实 URL（有些站点必须带 www 才能访问）
pub fn display_name(url: &str) -> String {
    let no_scheme = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let trimmed = no_scheme.trim_end_matches('/');
    if trimmed.len() >= 4 && trimmed[..4].eq_ignore_ascii_case("www.") {
        // 用原串对应位置切片，保留大小写
        return trimmed[4..].to_string();
    }
    trimmed.to_string()
}

fn fnv(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    h
}

/// 图标缓存文件路径（与 URL 一一对应，方便完成后再去找）
pub fn path_for(cache_dir: &Path, url: &str) -> PathBuf {
    cache_dir.join(format!("fav_{:016x}.ico", fnv(url)))
}

/// 按内容决定落盘路径。
/// 为什么不能一律用 .ico：icons.rs 对 `.ico` 走 PrivateExtractIconsW，
/// 而不少站点的 favicon.ico 其实是 PNG，那样会提取失败（实测 rust-lang、baidu 如此）。
/// PNG/JPEG 存成真实扩展名，就能走“图片文件”分支正常显示。
fn path_for_bytes(cache_dir: &Path, url: &str, b: &[u8]) -> PathBuf {
    let base = format!("fav_{:016x}", fnv(url));
    let ext = if b.starts_with(&[0x89, b'P', b'N', b'G']) {
        "png"
    } else if b.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "jpg"
    } else if b.starts_with(b"GIF8") {
        "gif"
    } else if b.starts_with(b"BM") {
        "bmp"
    } else {
        "ico"
    };
    cache_dir.join(format!("{base}.{ext}"))
}

/// 找出该 URL 已缓存的图标文件（按候选扩展名依次查找）
fn cached_for(cache_dir: &Path, url: &str) -> Option<PathBuf> {
    for ext in ["ico", "png", "jpg", "gif", "bmp"] {
        let p = cache_dir.join(format!("fav_{:016x}.{ext}", fnv(url)));
        if p.is_file() && p.metadata().map(|m| m.len() > 64).unwrap_or(false) {
            return Some(p);
        }
    }
    None
}

/// 后台抓取器：持有一个工作线程与任务队列
pub struct Fetcher {
    tx: Sender<String>,
    queue: Vec<String>,
    cache: PathBuf,
}

impl Fetcher {
    /// hwnd 用 usize 传（HWND 不是 Send），完成后向它发 WM_FAVICON_READY
    pub fn new(cache: PathBuf, hwnd: usize) -> Fetcher {
        let (tx, rx) = channel::<String>();
        let dir = cache.clone();
        std::thread::spawn(move || work_loop(rx, dir, hwnd));
        Fetcher { tx, queue: Vec::new(), cache }
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache
    }

    /// 排队抓取；同一 URL 在一次运行内只抓一次
    pub fn request(&mut self, url: &str) {
        if url.is_empty() || self.queue.iter().any(|u| u == url) {
            return;
        }
        self.queue.push(url.to_string());
        let _ = self.tx.send(url.to_string());
    }

    /// 若已抓到，返回可直接作为 Link.icon 的路径
    pub fn ready_path(&self, url: &str) -> Option<String> {
        let p = path_for(&self.cache, url);
        if p.is_file() && p.metadata().map(|m| m.len() > 64).unwrap_or(false) {
            Some(p.to_string_lossy().to_string())
        } else {
            None
        }
    }
}

fn work_loop(rx: Receiver<String>, cache: PathBuf, hwnd: usize) {
    use std::sync::mpsc::RecvTimeoutError;
    let mut done: HashSet<String> = HashSet::new();
    let _ = std::fs::create_dir_all(&cache);
    loop {
        match rx.recv_timeout(std::time::Duration::from_secs(600)) {
            Ok(url) => {
                if done.contains(&url) {
                    continue;
                }
                done.insert(url.clone());
                let saved = fetch_and_save(&url, &cache);
                if saved.is_some() && hwnd != 0 {
                    unsafe {
                        crate::win::PostMessageW(hwnd as crate::win::HWND, WM_FAVICON_READY, 0, 0);
                    }
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                if done.is_empty() {
                    // 长时间没任务就退出，不占资源
                    return;
                }
            }
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}

/// 开发期验证用：把 WinHTTP 抓取单独暴露出来，
/// 让 examples/probe_favicon.rs 走同一套代码路径做真实联网测试。
pub fn debug_http_get(host: &str, path: &str, secure: bool) -> Option<Vec<u8>> {
    http_get(host, path, secure)
}

pub fn debug_ready(cache: &std::path::Path, url: &str) -> Option<String> {
    let p = path_for(cache, url);
    if p.is_file() && p.metadata().map(|m| m.len() > 64).unwrap_or(false) {
        Some(p.to_string_lossy().to_string())
    } else {
        None
    }
}

pub fn debug_fetch_and_save(url: &str, cache: &std::path::Path) -> Option<std::path::PathBuf> {
    fetch_and_save(url, cache)
}

/// 抓 favicon，按可靠性从高到低的顺序尝试：
///   1. <协议>://<host>/favicon.ico       （最常见，一次请求搞定）
///   2. 首页 HTML 里的 <link rel="icon">  （rust-lang.org、example.com 这类没有
///      根 favicon.ico 的站点靠它；data: 内联图标要跳过）
/// 每个候选都按 URL 自身协议先试，https 站点失败再退回 http。
fn fetch_and_save(url: &str, cache: &Path) -> Option<PathBuf> {
    let host = host_of(url)?;
    if let Some(hit) = cached_for(cache, url) {
        return Some(hit);
    }
    let _ = std::fs::create_dir_all(cache);

    let is_https = url.to_ascii_lowercase().starts_with("https://");
    let mut schemes: Vec<bool> = vec![is_https];
    if is_https {
        schemes.push(false);
    }

    // 1) 根目录 favicon.ico：只用 URL 自身协议试一次，
    //    不做 http 回退（多等一轮要好几秒，收益极低，直接进第 2 步更划算）
    if let Some(bytes) = http_get(&host, "/favicon.ico", is_https) {
        if looks_like_icon(&bytes) {
            let p = path_for_bytes(cache, url, &bytes);
            if std::fs::write(&p, &bytes).is_ok() {
                return Some(p);
            }
        }
    }

    // 2) 首页 HTML 里声明的图标
    for &secure in &schemes {
        let path = page_path(url);
        let Some(html) = http_get(&host, &path, secure) else { continue };
        let text = String::from_utf8_lossy(&html);
        for icon_path in declare_icons(&text) {
            let abs = resolve_href(&host, &icon_path);
            if let Some(bytes) = http_get(&host, &abs, secure) {
                if looks_like_icon(&bytes) {
                    let p = path_for_bytes(cache, url, &bytes);
                    if std::fs::write(&p, &bytes).is_ok() {
                        return Some(p);
                    }
                }
            }
        }
    }
    None
}

/// 首页路径：把 URL 的 path+query 取出来，没有就用 "/"
fn page_path(url: &str) -> String {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    match rest.find('/') {
        Some(i) if i + 1 < rest.len() => rest[i..].to_string(),
        _ => "/".to_string(),
    }
}

/// 从 <link rel="...icon..." href="..."> 里抽出图标地址；跳过 data: 内联与 svg
fn declare_icons(html: &str) -> Vec<String> {
    // (优先级, href)：0 = 普通 icon，1 = apple-touch-icon（应用图标，兜底用）
    let mut out: Vec<(u8, String)> = Vec::new();
    let lower = html.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut i = 0usize;
    while let Some(pos) = lower[i..].find("<link") {
        let start = i + pos;
        let end = match lower[start..].find('>') {
            Some(e) => start + e,
            None => break,
        };
        let tag = &html[start..=end];
        let tag_l = &lower[start..=end];
        i = end + 1;
        // 必须含 rel 且 rel 值里带 icon
        let Some(rel_pos) = tag_l.find("rel") else { continue };
        let rel_seg = &tag_l[rel_pos..(rel_pos + 60).min(tag_l.len())];
        let Some(eq) = rel_seg.find('=') else { continue };
        let val = rel_seg[eq + 1..].trim_start();
        let quote = val.chars().next().unwrap_or(' ');
        let rel_val = if quote == '"' || quote == '\'' {
            let body = &val[1..];
            match body.find(quote) {
                Some(e) => body[..e].to_string(),
                None => continue,
            }
        } else {
            val.split_whitespace().next().unwrap_or("").to_string()
        };
        if !rel_val.contains("icon") {
            continue;
        }
        // 取 href
        let Some(href_pos) = tag_l.find("href") else { continue };
        let href_seg = &tag[href_pos + 4..];
        let href_seg = href_seg.trim_start();
        let Some(eq2) = href_seg.find('=') else { continue };
        let v = href_seg[eq2 + 1..].trim_start();
        let q = v.chars().next().unwrap_or(' ');
        let href = if q == '"' || q == '\'' {
            match v[1..].find(q) {
                Some(e) => v[1..1 + e].to_string(),
                None => continue,
            }
        } else {
            v.split_whitespace().next().unwrap_or("").to_string()
        };
        let href = href.trim().to_string();
        if href.is_empty() || href.starts_with("data:") || href.to_ascii_lowercase().ends_with(".svg") {
            continue;
        }
        let prio: u8 = if rel_val.contains("apple-touch") { 1 } else { 0 };
        if !out.iter().any(|(_, h)| h == &href) {
            out.push((prio, href));
        }
        let _ = bytes;
    }
    // 普通 icon 排在 apple-touch-icon 前面：后者是“添加到主屏幕”用的应用图标，
    // 不一定是网站标识，作为兜底候选更合适
    out.sort_by_key(|(p, _)| *p);
    out.into_iter().map(|(_, h)| h).collect()
}

/// 把 href 变成可请求的绝对路径（只保留 path+query，host 复用当前站点）
fn resolve_href(host: &str, href: &str) -> String {
    let h = href.trim();
    if let Some(rest) = h.strip_prefix("https://").or_else(|| h.strip_prefix("http://")) {
        // 绝对地址：取出路径部分
        let after_host = match rest.find('/') {
            Some(i) => rest[i..].to_string(),
            None => "/".to_string(),
        };
        return after_host;
    }
    if h.starts_with("//") {
        let after = &h[2..];
        return match after.find('/') {
            Some(i) => after[i..].to_string(),
            None => "/".to_string(),
        };
    }
    if h.starts_with('/') {
        return h.to_string();
    }
    // 相对路径：相对站点根（够用，favicon 极少深链相对路径）
    let _ = host;
    format!("/{}", h)
}


/// 粗略判断是不是图标数据：ICO(00 00 01 00) 或 PNG/JPEG/GIF/BMP 头
fn looks_like_icon(b: &[u8]) -> bool {
    if b.len() < 64 {
        return false;
    }
    if b.starts_with(&[0, 0, 1, 0]) {
        return true;
    }
    if b.starts_with(&[0x89, b'P', b'N', b'G']) {
        return true;
    }
    if b.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return true;
    }
    if b.starts_with(b"GIF8") || b.starts_with(b"BM") {
        return true;
    }
    false
}

// ---------------- WinHTTP ----------------

#[link(name = "winhttp")]
extern "system" {
    fn WinHttpOpen(
        agent: *const u16,
        access_type: u32,
        proxy: *const u16,
        bypass: *const u16,
        flags: u32,
    ) -> *mut c_void;
    fn WinHttpConnect(session: *mut c_void, host: *const u16, port: u16, reserved: u32) -> *mut c_void;
    fn WinHttpOpenRequest(
        connect: *mut c_void,
        verb: *const u16,
        path: *const u16,
        version: *const u16,
        referrer: *const u16,
        accept_types: *const *const u16,
        flags: u32,
    ) -> *mut c_void;
    fn WinHttpSendRequest(
        request: *mut c_void,
        headers: *const u16,
        headers_len: u32,
        optional: *mut c_void,
        optional_len: u32,
        total_len: u32,
        context: usize,
    ) -> i32;
    fn WinHttpReceiveResponse(request: *mut c_void, reserved: *mut c_void) -> i32;
    fn WinHttpQueryHeaders(
        request: *mut c_void,
        info_level: u32,
        name: *const u16,
        buffer: *mut c_void,
        buffer_len: *mut u32,
        index: *mut u32,
    ) -> i32;
    fn WinHttpReadData(request: *mut c_void, buffer: *mut c_void, to_read: u32, read: *mut u32) -> i32;
    fn WinHttpCloseHandle(handle: *mut c_void) -> i32;
    fn WinHttpSetTimeouts(
        handle: *mut c_void,
        resolve: i32,
        connect: i32,
        send: i32,
        receive: i32,
    ) -> i32;
}

const WINHTTP_ACCESS_TYPE_DEFAULT_PROXY: u32 = 0;
const WINHTTP_FLAG_SECURE: u32 = 0x0080_0000;
const WINHTTP_QUERY_STATUS_CODE: u32 = 19;
const WINHTTP_QUERY_FLAG_NUMBER: u32 = 0x2000_0000;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn agent_name() -> Vec<u16> {
    wide("QuickLaunch/1.0 (favicon)")
}

/// 取一个 URL 的原始字节；失败返回 None
fn http_get(host: &str, path: &str, secure: bool) -> Option<Vec<u8>> {
    unsafe {
        let agent = agent_name();
        let session = WinHttpOpen(
            agent.as_ptr(),
            WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
            std::ptr::null(),
            std::ptr::null(),
            0,
        );
        if session.is_null() {
            return None;
        }
        WinHttpSetTimeouts(session, TIMEOUT_MS, TIMEOUT_MS, TIMEOUT_MS, TIMEOUT_MS);

        let whost = wide(host);
        let port: u16 = if secure { 443 } else { 80 };
        let connect = WinHttpConnect(session, whost.as_ptr(), port, 0);
        if connect.is_null() {
            WinHttpCloseHandle(session);
            return None;
        }

        let verb = wide("GET");
        let wpath = wide(path);
        let flags = if secure { WINHTTP_FLAG_SECURE } else { 0 };
        let req = WinHttpOpenRequest(
            connect,
            verb.as_ptr(),
            wpath.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            flags,
        );
        if req.is_null() {
            WinHttpCloseHandle(connect);
            WinHttpCloseHandle(session);
            return None;
        }
        WinHttpSetTimeouts(req, TIMEOUT_MS, TIMEOUT_MS, TIMEOUT_MS, TIMEOUT_MS);

        let ok = WinHttpSendRequest(req, std::ptr::null(), 0, std::ptr::null_mut(), 0, 0, 0);
        if ok == 0 || WinHttpReceiveResponse(req, std::ptr::null_mut()) == 0 {
            WinHttpCloseHandle(req);
            WinHttpCloseHandle(connect);
            WinHttpCloseHandle(session);
            return None;
        }

        // 状态码必须是 200
        let mut status: u32 = 0;
        let mut len: u32 = 4;
        let got = WinHttpQueryHeaders(
            req,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            std::ptr::null(),
            &mut status as *mut u32 as *mut c_void,
            &mut len,
            std::ptr::null_mut(),
        );
        if got == 0 || status != 200 {
            WinHttpCloseHandle(req);
            WinHttpCloseHandle(connect);
            WinHttpCloseHandle(session);
            return None;
        }

        let mut out: Vec<u8> = Vec::new();
        let mut buf = vec![0u8; 16 * 1024];
        loop {
            let mut read: u32 = 0;
            let r = WinHttpReadData(
                req,
                buf.as_mut_ptr() as *mut c_void,
                buf.len() as u32,
                &mut read,
            );
            if r == 0 || read == 0 {
                break;
            }
            out.extend_from_slice(&buf[..read as usize]);
            if out.len() > MAX_BYTES {
                break;
            }
        }
        WinHttpCloseHandle(req);
        WinHttpCloseHandle(connect);
        WinHttpCloseHandle(session);
        if out.is_empty() || out.len() > MAX_BYTES {
            None
        } else {
            Some(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_parsing() {
        assert_eq!(host_of("https://www.bilibili.com/video/1").as_deref(), Some("www.bilibili.com"));
        assert_eq!(host_of("http://example.com:8080/a").as_deref(), Some("example.com"));
        assert_eq!(host_of("bilibili.com"), Some("bilibili.com".to_string()));
        assert_eq!(host_of(""), None);
    }

    #[test]
    fn strip_www_in_display_name() {
        assert_eq!(display_name("https://www.bilibili.com/"), "bilibili.com");
        assert_eq!(display_name("https://www2.example.com/"), "www2.example.com");
        assert_eq!(display_name("https://example.com/path/"), "example.com/path");
        assert_eq!(display_name("http://WWW.Example.com"), "Example.com");
        assert_eq!(display_name("example.com"), "example.com");
    }

    #[test]
    fn icon_sniffing() {
        let mut ico = vec![0u8, 0, 1, 0];
        ico.resize(100, 0);
        assert!(looks_like_icon(&ico));
        let mut png = vec![0x89u8, b'P', b'N', b'G'];
        png.resize(100, 0);
        assert!(looks_like_icon(&png));
        assert!(!looks_like_icon(&vec![0u8; 100]));
        assert!(!looks_like_icon(b"short"));
    }
}

#[cfg(test)]
mod parse_tests {
    use super::*;

    const SAMPLE: &str = r#"<html><head>
<link rel="apple-touch-icon" sizes="180x180" href="/apple.png">
<link rel="icon" sizes="16x16" type="image/png" href="/static/images/favicon-16x16.png">
<link rel="icon" type="image/svg+xml" href="/fav.svg">
<link rel=icon href=data:,>
</head></html>"#;

    #[test]
    fn finds_icon_links() {
        let v = declare_icons(SAMPLE);
        assert!(v.contains(&"/static/images/favicon-16x16.png".to_string()), "解析结果: {:?}", v);
        // svg 与 data: 应当被跳过
        assert!(!v.iter().any(|h| h.ends_with(".svg")), "不该包含 svg: {:?}", v);
        assert!(!v.iter().any(|h| h.starts_with("data:")), "不该包含 data: {:?}", v);
        // apple-touch-icon 允许收录，但必须排在普通 icon 之后
        assert!(!v.is_empty());
        assert_eq!(v[0], "/static/images/favicon-16x16.png", "普通 icon 应排第一: {:?}", v);
    }

    #[test]
    fn resolve_variants() {
        assert_eq!(resolve_href("h", "/a/b.png"), "/a/b.png");
        assert_eq!(resolve_href("h", "a/b.png"), "/a/b.png");
        assert_eq!(resolve_href("h", "https://other.com/x.png"), "/x.png");
        assert_eq!(resolve_href("h", "//cdn.com/y.png"), "/y.png");
    }

    #[test]
    fn page_path_variants() {
        assert_eq!(page_path("https://a.com"), "/");
        assert_eq!(page_path("https://a.com/"), "/");
        assert_eq!(page_path("https://a.com/x/y?z=1"), "/x/y?z=1");
    }
}