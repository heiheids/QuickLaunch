use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 找一个可用的 windres（rustup 的 GNU 工具链不自带，常见于独立 MinGW / MSYS2）
fn find_windres() -> Option<PathBuf> {
    if let Ok(p) = env::var("WINDRES") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }
    let mut cands: Vec<PathBuf> = vec![
        PathBuf::from("C:\\ff-tools\\mingw64\\bin\\windres.exe"),
        PathBuf::from("C:\\msys64\\mingw64\\bin\\windres.exe"),
        PathBuf::from("C:\\mingw64\\bin\\windres.exe"),
    ];
    if let Ok(pf) = env::var("ProgramFiles") {
        cands.push(Path::new(&pf).join("mingw64\\bin\\windres.exe"));
    }
    for c in cands {
        if c.is_file() {
            return Some(c);
        }
    }
    // PATH 里的
    let name = if cfg!(windows) { "windres.exe" } else { "windres" };
    if let Ok(path) = env::var("PATH") {
        for dir in env::split_paths(&path) {
            let p = dir.join(name);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

fn main() {
    // GDI+ 的导入库不在 Rust GNU 自带的 MinGW 里，GNU ld 支持直接链接 DLL，
    // 因此这里把系统 DLL 作为链接入参传入（无需任何第三方 crate）。
    let env_target = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if env_target == "gnu" {
        let root = env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
        println!("cargo:rustc-link-arg={}\\System32\\gdiplus.dll", root);
    } else {
        println!("cargo:rustc-link-lib=gdiplus");
    }

    // 程序图标：把 res\app.ico 编进 PE 资源段
    // 注意：这台机器上的 windres 是 ANSI 版，遇到中文路径会失败，
    // 所以统一放到 ASCII 的临时目录里编译，再把 .o 交给链接器。
    println!("cargo:rerun-if-changed=res/app.rc");
    println!("cargo:rerun-if-changed=res/app.ico");
    match find_windres() {
        Some(wr) => match compile_resource(&wr) {
            Some(obj) => println!("cargo:rustc-link-arg={}", obj.display()),
            None => println!("cargo:warning=图标资源编译失败，程序将使用内置绘制图标"),
        },
        None => {
            println!("cargo:warning=未找到 windres，跳过图标资源（可设置 WINDRES 环境变量）");
        }
    }
}

fn is_ascii_path(p: &Path) -> bool {
    p.to_string_lossy().is_ascii()
}

fn compile_resource(wr: &Path) -> Option<PathBuf> {
    let tmp = env::temp_dir().join("ql_icon_build");
    if !is_ascii_path(&tmp) {
        return None;
    }
    std::fs::create_dir_all(&tmp).ok()?;
    let rc_dst = tmp.join("app.rc");
    let ico_dst = tmp.join("app.ico");
    std::fs::copy(Path::new("res").join("app.ico"), &ico_dst).ok()?;
    std::fs::write(&rc_dst, "1 ICON \"app.ico\"\n").ok()?;

    let obj = tmp.join("app_res.o");
    let ok = Command::new(wr)
        .current_dir(&tmp)
        .arg("-I")
        .arg(&tmp)
        .arg("-O")
        .arg("coff")
        .arg("-o")
        .arg(&obj)
        .arg(&rc_dst)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if ok && obj.is_file() {
        Some(obj)
    } else {
        None
    }
}
