//! 开发期验证：真实联网抓 favicon，走的是产品代码里的同一个 fetch_and_save，
//! 不另外复刻一套，避免“测试通过但产品路径有问题”。
//! 用法： cargo run --release --example probe_favicon [url ...]
#![windows_subsystem = "console"]

#[path = "../src/win.rs"]
#[allow(dead_code)]
mod win;

#[path = "../src/favicon.rs"]
#[allow(dead_code)]
mod favicon;

fn main() {
    let mut urls: Vec<String> = std::env::args().skip(1).collect();
    if urls.is_empty() {
        urls = vec![
            "https://www.bilibili.com/".to_string(),
            "https://www.baidu.com/".to_string(),
            "https://www.zhihu.com/".to_string(),
            "https://www.qq.com/".to_string(),
            "https://www.rust-lang.org/".to_string(),
        ];
    }
    let cache = std::env::temp_dir().join("ql_favicon_probe");
    let _ = std::fs::create_dir_all(&cache);
    let _ = std::fs::remove_dir_all(&cache);
    let _ = std::fs::create_dir_all(&cache);

    let mut ok = 0;
    for u in &urls {
        let host = favicon::host_of(u).unwrap_or_default();
        let name = favicon::display_name(u);
        let t0 = std::time::Instant::now();
        let got = favicon::debug_fetch_and_save(u, &cache);
        let ms = t0.elapsed().as_millis();
        match got {
            Some(p) => {
                ok += 1;
                let sz = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                println!("[OK]   {:<34} host={:<22} 名称={:<22} {} 字节 {} ms", u, host, name, sz, ms);
            }
            None => println!("[FAIL] {:<34} host={:<22} 名称={:<22} 没抓到 {} ms", u, host, name, ms),
        }
    }
    println!();
    println!("结果：{}/{} 成功", ok, urls.len());
}
