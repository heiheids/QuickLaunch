//! 校验 exe 内嵌的 RT_ICON 资源与 res\app.ico 是否逐字节一致。
//! 用法： cargo run --release --example verify_icon
//! 不经过 shell 图标缓存，直接读 PE 资源表，结论可信。
#![windows_subsystem = "console"]

use std::fs;

fn u16_at(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}
fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

struct IcoImage {
    id: u16,
    w: u8,
    h: u8,
    bytes: Vec<u8>,
}

fn main() {
    let exe_path = std::env::args().nth(1).unwrap_or_else(|| "target/release/QuickLaunch.exe".to_string());
    let ico_path = std::env::args().nth(2).unwrap_or_else(|| "res/app.ico".to_string());
    let exe = match fs::read(&exe_path) {
        Ok(v) => v,
        Err(e) => {
            println!("读取 {exe_path} 失败: {e}");
            return;
        }
    };
    let ico = fs::read(&ico_path).expect("读取 ico 失败");

    // ---- 解析 ico ----
    let n = u16_at(&ico, 4) as usize;
    let mut images = Vec::new();
    for i in 0..n {
        let o = 6 + i * 16;
        let sz = u32_at(&ico, o + 8) as usize;
        let off = u32_at(&ico, o + 12) as usize;
        images.push(IcoImage {
            id: (i + 1) as u16,
            w: ico[o],
            h: ico[o + 1],
            bytes: ico[off..off + sz].to_vec(),
        });
    }

    // ---- 解析 PE 头 ----
    let pe = u32_at(&exe, 0x3C) as usize;
    let nsec = u16_at(&exe, pe + 6) as usize;
    let size_opt = u16_at(&exe, pe + 20) as usize;
    let sec_off = pe + 24 + size_opt;
    let mut sections: Vec<(u32, u32, u32)> = Vec::new(); // (VA, VSize, RawPtr)
    for i in 0..nsec {
        let s = sec_off + i * 40;
        sections.push((u32_at(&exe, s + 12), u32_at(&exe, s + 8), u32_at(&exe, s + 20)));
    }
    let opt = pe + 24;
    let dd = opt + 112 + 16; // 资源表目录项
    let rsrc_rva = u32_at(&exe, dd);

    let rva2off = |rva: u32| -> Option<usize> {
        for (va, vsz, raw) in &sections {
            let span = (*vsz).max(1);
            if rva >= *va && rva < va + span {
                return Some((*raw + (rva - *va)) as usize);
            }
        }
        None
    };

    println!("exe = {exe_path}");
    println!("资源表 RVA = 0x{rsrc_rva:X}");
    if rsrc_rva == 0 {
        println!("结论：exe 里没有资源表");
        return;
    }
    let base = match rva2off(rsrc_rva) {
        Some(o) => o,
        None => {
            println!("结论：资源表 RVA 无法映射到文件偏移");
            return;
        }
    };
    let dir_entries = |off: usize| -> Vec<(u32, bool, u32)> {
        let named = u16_at(&exe, off + 12) as usize;
        let ids = u16_at(&exe, off + 14) as usize;
        let mut v = Vec::new();
        for i in 0..(named + ids) {
            let e = off + 16 + i * 8;
            let id = u32_at(&exe, e);
            let t = u32_at(&exe, e + 4);
            v.push((id, (t & 0x8000_0000) != 0, t & 0x7FFF_FFFF));
        }
        v
    };

    let root = dir_entries(base);
    if std::env::var("QL_DEBUG").is_ok() {
        let head: Vec<String> = exe[base..base + 48].iter().map(|b| format!("{b:02X}")).collect();
        println!("资源段前 48 字节: {}", head.join(" "));
        println!("根目录项: {:?}", root);
    }
    let icon_off = root.iter().find(|e| e.0 == 3 && e.1).map(|e| e.2);
    let group_off = root.iter().find(|e| e.0 == 14 && e.1).map(|e| e.2);
    let (icon_off, group_off) = match (icon_off, group_off) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            println!("结论：缺少 RT_ICON 或 RT_GROUP_ICON 目录");
            return;
        }
    };

    let icons = dir_entries(base + icon_off as usize);
    println!("RT_ICON 项数 = {}（ico 内图像数 = {}）", icons.len(), n);

    let mut ok = 0;
    for (id, is_dir, off) in &icons {
        if *is_dir {
            println!("  id={id} 意外是子目录");
            continue;
        }
        let de = base + *off as usize;
        let data_rva = u32_at(&exe, de);
        let data_sz = u32_at(&exe, de + 4) as usize;
        let expect = images.iter().find(|i| i.id as u32 == *id);
        let file_off = rva2off(data_rva);
        let mut line = format!("  id={id:<2} RVA=0x{data_rva:X} 大小={data_sz:<6}");
        match (expect, file_off) {
            (Some(exp), Some(fo)) if data_sz == exp.bytes.len() && fo + data_sz <= exe.len() => {
                let same = exe[fo..fo + data_sz] == exp.bytes[..];
                line.push_str(&format!(
                    " 源 {}x{} 一致={}",
                    if exp.w == 0 { 256 } else { exp.w as i32 },
                    if exp.h == 0 { 256 } else { exp.h as i32 },
                    same
                ));
                if same {
                    ok += 1;
                }
            }
            (Some(_), Some(_)) => line.push_str(" 大小或内容不符"),
            (_, None) => line.push_str(" RVA 映射失败"),
            (None, _) => line.push_str(" 源里找不到同 id 图像"),
        }
        println!("{line}");
    }

    // 组图标
    let groups = dir_entries(base + group_off as usize);
    if let Some((gid, _, goff)) = groups.first() {
        let de = base + *goff as usize;
        let rva = u32_at(&exe, de);
        let sz = u32_at(&exe, de + 4) as usize;
        if let Some(fo) = rva2off(rva) {
            let cnt = u16_at(&exe, fo + 4);
            println!("RT_GROUP_ICON id={gid} 大小={sz} 组内图像数={cnt}");
            if cnt as usize != n {
                println!("  注意：组内图像数与源 ico 不一致");
            }
        }
    }

    println!();
    if ok == n {
        println!("结论：图标资源正确（{ok}/{n} 张与源文件逐字节一致）");
    } else {
        println!("结论：图标资源不正确（{ok}/{n} 张一致）");
    }
}
