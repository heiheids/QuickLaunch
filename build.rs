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
            None => fallback_rsrc(),
        },
        None => fallback_rsrc(),
    }
}

fn fallback_rsrc() {
    match gen_rsrc_object() {
        Some(obj) => println!("cargo:rustc-link-arg={}", obj.display()),
        None => println!("cargo:warning=图标资源生成失败，程序将使用内置绘制图标"),
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

// ---------------- 无 windres 时的纯 Rust 图标资源生成 ----------------
//
// windres 的产物本质就是一个只含 .rsrc 段的 COFF 目标文件：
//   段内布局 = PE 资源目录树（根 -> RT_ICON / RT_GROUP_ICON -> 数据项）
// 有两个容易写错的点：
//
// 1) 目录项里的 OffsetToSubdir 是“段内相对偏移”，而数据项里的
//    OffsetToData 是 **RVA（相对镜像基址）**。段最终落在哪个 RVA 由链接器决定，
//    所以数据项这个字段必须配一条 IMAGE_REL_AMD64_ADDR32NB 重定位，
//    让链接器把“段内偏移”换算成“段 RVA + 偏移”。少了重定位，
//    资源表指向垃圾数据，Explorer 里显示的图标就是错的（甚至变成通用图标）。
// 2) 重定位要引用一个 .rsrc 段符号（带 1 个 aux 段定义记录），
//    链接器才知道加的是本段的基址。

fn push_u16(v: &mut Vec<u8>, x: u16) {
    v.extend_from_slice(&x.to_le_bytes());
}
fn push_u32(v: &mut Vec<u8>, x: u32) {
    v.extend_from_slice(&x.to_le_bytes());
}

struct IcoImage {
    width: u8,
    height: u8,
    colors: u8,
    planes: u16,
    bits: u16,
    bytes: Vec<u8>,
}

fn parse_ico(data: &[u8]) -> Option<Vec<IcoImage>> {
    if data.len() < 6 || data[0] != 0 || data[1] != 0 || data[2] != 1 {
        return None;
    }
    let count = u16::from_le_bytes([data[4], data[5]]) as usize;
    if count == 0 || data.len() < 6 + count * 16 {
        return None;
    }
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let b = 6 + i * 16;
        let bytes_in_res =
            u32::from_le_bytes([data[b + 8], data[b + 9], data[b + 10], data[b + 11]]) as usize;
        let off = u32::from_le_bytes([data[b + 12], data[b + 13], data[b + 14], data[b + 15]]) as usize;
        if off + bytes_in_res > data.len() {
            return None;
        }
        out.push(IcoImage {
            width: data[b],
            height: data[b + 1],
            colors: data[b + 2],
            planes: u16::from_le_bytes([data[b + 4], data[b + 5]]),
            bits: u16::from_le_bytes([data[b + 6], data[b + 7]]),
            bytes: data[off..off + bytes_in_res].to_vec(),
        });
    }
    Some(out)
}

fn align4(n: usize) -> usize {
    (n + 3) & !3
}

/// 返回（.rsrc 段内容, 需要 ADDR32NB 重定位的字段偏移）
fn build_rsrc(images: &[IcoImage]) -> (Vec<u8>, Vec<usize>) {
    let n = images.len();
    let root_off = 16usize;
    let icon_dir_off = root_off + 2 * 8;
    let group_dir_off = icon_dir_off + 16 + n * 8;
    let mut cur = align4(group_dir_off + 16 + 8);

    let mut icon_entry_off = Vec::with_capacity(n);
    let mut icon_data_off = Vec::with_capacity(n);
    for im in images {
        icon_entry_off.push(cur);
        cur += 16;
        icon_data_off.push(cur);
        cur += align4(im.bytes.len());
    }
    let group_entry_off = cur;
    let group_data_off = group_entry_off + 16;

    let mut v: Vec<u8> = Vec::new();
    let mut relocs: Vec<usize> = Vec::new();

    // ---- 根目录（RT_ICON=3, RT_GROUP_ICON=14）----
    push_u32(&mut v, 0); // characteristics
    push_u32(&mut v, 0); // timestamp
    push_u16(&mut v, 0);
    push_u16(&mut v, 0); // 版本
    push_u16(&mut v, 0); // 具名项
    push_u16(&mut v, 2); // ID 项
    push_u32(&mut v, 3);
    // 指向子目录的项，偏移最高位必须置 1，否则会被当成数据项
    push_u32(&mut v, 0x8000_0000 | icon_dir_off as u32);
    push_u32(&mut v, 14);
    push_u32(&mut v, 0x8000_0000 | group_dir_off as u32);

    // ---- RT_ICON 目录 ----
    push_u32(&mut v, 0);
    push_u32(&mut v, 0);
    push_u16(&mut v, 0);
    push_u16(&mut v, 0);
    push_u16(&mut v, 0);
    push_u16(&mut v, n as u16);
    for i in 0..n {
        push_u32(&mut v, (i + 1) as u32);
        push_u32(&mut v, icon_entry_off[i] as u32);
    }

    // ---- RT_GROUP_ICON 目录 ----
    push_u32(&mut v, 0);
    push_u32(&mut v, 0);
    push_u16(&mut v, 0);
    push_u16(&mut v, 0);
    push_u16(&mut v, 0);
    push_u16(&mut v, 1);
    push_u32(&mut v, 1); // 组图标 ID = 1
    push_u32(&mut v, group_entry_off as u32);

    // ---- RT_ICON 数据项 + 图像数据 ----
    for i in 0..n {
        relocs.push(v.len()); // OffsetToData 字段，需要在链接时改成 RVA
        push_u32(&mut v, icon_data_off[i] as u32);
        push_u32(&mut v, images[i].bytes.len() as u32);
        push_u32(&mut v, 0); // codepage
        push_u32(&mut v, 0);
        v.resize(icon_data_off[i], 0);
        v.extend_from_slice(&images[i].bytes);
        let pad = align4(v.len()) - v.len();
        v.resize(v.len() + pad, 0);
    }

    // ---- RT_GROUP_ICON 数据项 + GRPICONDIR ----
    relocs.push(v.len());
    push_u32(&mut v, group_data_off as u32);
    push_u32(&mut v, (6 + 14 * n) as u32);
    push_u32(&mut v, 0);
    push_u32(&mut v, 0);
    v.resize(group_data_off, 0);
    push_u16(&mut v, 0);
    push_u16(&mut v, 1);
    push_u16(&mut v, n as u16);
    for (i, im) in images.iter().enumerate() {
        v.push(im.width);
        v.push(im.height);
        v.push(im.colors);
        v.push(0);
        push_u16(&mut v, im.planes);
        push_u16(&mut v, im.bits);
        push_u32(&mut v, im.bytes.len() as u32);
        push_u16(&mut v, (i + 1) as u16);
    }
    (v, relocs)
}

fn gen_rsrc_object() -> Option<PathBuf> {
    let ico = std::fs::read(Path::new("res").join("app.ico")).ok()?;
    let images = parse_ico(&ico)?;
    let (data, relocs) = build_rsrc(&images);
    let tmp = env::temp_dir().join("ql_icon_build");
    if !is_ascii_path(&tmp) {
        return None;
    }
    std::fs::create_dir_all(&tmp).ok()?;
    let obj = tmp.join("app_rsrc.o");

    const HDR: usize = 20;
    const SEC: usize = 40;
    const RAW: usize = HDR + SEC; // 60
    let reloc_off = align4(RAW + data.len());
    let reloc_len = relocs.len() * 10;
    let sym_off = reloc_off + reloc_len;
    // 1 个段符号 + 1 个 aux 段定义记录
    let nsyms: u32 = 2;

    let mut out: Vec<u8> = Vec::with_capacity(sym_off + 64);
    // ---- COFF 文件头 ----
    push_u16(&mut out, 0x8664); // AMD64
    push_u16(&mut out, 1); // 段数
    push_u32(&mut out, 0); // 时间戳
    push_u32(&mut out, sym_off as u32); // 符号表偏移
    push_u32(&mut out, nsyms); // 符号数
    push_u16(&mut out, 0); // 可选头长度
    push_u16(&mut out, 0); // 特征
    // ---- 段头 ----
    out.extend_from_slice(b".rsrc\0\0\0");
    push_u32(&mut out, 0); // 虚拟大小
    push_u32(&mut out, 0); // 虚拟地址
    push_u32(&mut out, data.len() as u32); // 原始数据大小
    push_u32(&mut out, RAW as u32); // 原始数据偏移
    push_u32(&mut out, reloc_off as u32); // 重定位表偏移
    push_u32(&mut out, 0); // 行号表偏移
    push_u16(&mut out, relocs.len() as u16); // 重定位数
    push_u16(&mut out, 0); // 行号数
    push_u32(&mut out, 0x4000_0040); // 已初始化数据 | 可读
    // ---- 段数据 ----
    out.extend_from_slice(&data);
    while out.len() < reloc_off {
        out.push(0);
    }
    // ---- 重定位：IMAGE_REL_AMD64_ADDR32NB(0x0003) -> 段符号(索引 0) ----
    for r in &relocs {
        push_u32(&mut out, *r as u32);
        push_u32(&mut out, 0); // 符号表索引 0 = .rsrc 段符号
        push_u16(&mut out, 0x0003);
    }
    // ---- 符号表：段符号 + aux 段定义 ----
    out.extend_from_slice(&[0u8; 8]); // 名字（段符号留空）
    push_u32(&mut out, 0); // Value
    push_u16(&mut out, 1); // SectionNumber = 1
    push_u16(&mut out, 0); // Type
    out.push(3); // StorageClass = IMAGE_SYM_CLASS_STATIC
    out.push(1); // 1 个 aux 记录
    // aux：段定义
    push_u32(&mut out, data.len() as u32); // Length
    push_u16(&mut out, relocs.len() as u16); // NumberOfRelocations
    push_u16(&mut out, 0); // NumberOfLinenumbers
    push_u32(&mut out, 0); // CheckSum
    push_u16(&mut out, 0); // Number
    out.push(0); // Selection
    out.extend_from_slice(&[0u8; 3]); // 保留

    std::fs::write(&obj, out).ok()?;
    Some(obj)
}
