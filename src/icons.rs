//! 图标提取与磁盘缓存：HICON -> 预乘 ARGB 原始像素 -> .icn 文件。
use crate::gfx::*;
use crate::win::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Entry {
    pub img: GpImage,
    _buf: Box<[u32]>,
}

pub struct IconStore {
    dir: PathBuf,
    map: HashMap<u64, Entry>,
}

fn fnv(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    h
}

fn fnv2(a: &str, b: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for s in [a, b] {
        for x in s.as_bytes() {
            h ^= *x as u64;
            h = h.wrapping_mul(0x1000_0000_01b3);
        }
        h ^= 0x7c;
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    h
}

impl IconStore {
    pub fn new(dir: PathBuf) -> IconStore {
        let _ = fs::create_dir_all(&dir);
        IconStore { dir, map: HashMap::new() }
    }

    pub fn clear_disk(&mut self) -> usize {
        let mut n = 0;
        if let Ok(rd) = fs::read_dir(&self.dir) {
            for e in rd.flatten() {
                if fs::remove_file(e.path()).is_ok() {
                    n += 1;
                }
            }
        }
        self.clear_mem();
        n
    }

    pub fn clear_mem(&mut self) {
        for (_, v) in self.map.drain() {
            gp_dispose(v.img);
        }
    }

    /// 根据（解析后的）图标来源路径取图标；返回 GDI+ 位图句柄（内部持有，勿释放）
    pub fn get(&mut self, src: &str, hint_file: &str) -> GpImage {
        let key = if !hint_file.is_empty() {
            fnv2(src, hint_file)
        } else {
            fnv(src)
        };
        if let Some(e) = self.map.get(&key) {
            return e.img;
        }
        // 1) 显式图标文件
        if !hint_file.is_empty() && Path::new(hint_file).is_file() {
            let img = gp_image_from_file(hint_file);
            if !img.is_null() {
                let e = Entry { img, _buf: Box::new([]) };
                self.map.insert(key, e);
                return img;
            }
        }
        // 2) 磁盘缓存
        let cache = self.dir.join(format!("{:016x}.icn", key));
        if let Some((img, buf)) = load_icn(&cache) {
            let e = Entry { img, _buf: buf };
            self.map.insert(key, e);
            return img;
        }
        // 3) 现场提取
        let mut pixels: Vec<u32> = Vec::new();
        let mut w = 0;
        let mut h = 0;
        extract_icon(src, &mut w, &mut h, &mut pixels);
        if w <= 0 || h <= 0 {
            let e = Entry { img: std::ptr::null_mut(), _buf: Box::new([]) };
            self.map.insert(key, e);
            return std::ptr::null_mut();
        }
        save_icn(&cache, w, h, &pixels);
        let mut buf: Box<[u32]> = pixels.into_boxed_slice();
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, buf.len() * 4)
        };
        let img = gp_image_from_pixels(w, h, bytes);
        let e = Entry { img, _buf: buf };
        self.map.insert(key, e);
        img
    }
}

fn load_icn(p: &Path) -> Option<(GpImage, Box<[u32]>)> {
    let data = fs::read(p).ok()?;
    if data.len() < 13 || &data[0..5] != b"ICON3" {
        return None;
    }
    let w = u32::from_le_bytes([data[5], data[6], data[7], data[8]]) as i32;
    let h = u32::from_le_bytes([data[9], data[10], data[11], data[12]]) as i32;
    let need = (w as usize) * (h as usize) * 4;
    if w <= 0 || h <= 0 || data.len() < 13 + need {
        return None;
    }
    let mut buf: Box<[u32]> = vec![0u32; w as usize * h as usize].into_boxed_slice();
    {
        let dst = unsafe { std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, need) };
        dst.copy_from_slice(&data[13..13 + need]);
    }
    let bytes = unsafe { std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, need) };
    let img = gp_image_from_pixels(w, h, bytes);
    Some((img, buf))
}

fn save_icn(p: &Path, w: i32, h: i32, px: &[u32]) {
    let mut out = Vec::with_capacity(13 + px.len() * 4);
    out.extend_from_slice(b"ICON3");
    out.extend_from_slice(&(w as u32).to_le_bytes());
    out.extend_from_slice(&(h as u32).to_le_bytes());
    for v in px {
        out.extend_from_slice(&v.to_le_bytes());
    }
    let _ = fs::write(p, out);
}

fn extract_icon(src: &str, w: &mut i32, h: &mut i32, out: &mut Vec<u32>) {
    unsafe {
        let mut hicon: HICON = std::ptr::null_mut();
        let mut id: u32 = 0;
        let ws = wide(src);
        // shell 命名空间（此电脑 / 回收站）：先解析成 PIDL 再取图标
        if src.starts_with("::") {
            let mut pidl: *mut std::ffi::c_void = std::ptr::null_mut();
            if SHParseDisplayName(ws.as_ptr(), std::ptr::null_mut(), &mut pidl, 0, std::ptr::null_mut()) == 0
                && !pidl.is_null()
            {
                let mut info: SHFILEINFOW = std::mem::zeroed();
                SHGetFileInfoW(
                    pidl as LPCWSTR,
                    0,
                    &mut info,
                    std::mem::size_of::<SHFILEINFOW>() as u32,
                    SHGFI_ICON | SHGFI_LARGEICON | SHGFI_PIDL,
                );
                hicon = info.hIcon;
                CoTaskMemFree(pidl);
            }
        }
        let n = if hicon.is_null() {
            PrivateExtractIconsW(ws.as_ptr(), 0, 48, 48, &mut hicon, &mut id, 1, 0)
        } else {
            1
        };
        if n == 0 || hicon.is_null() {
            let mut info: SHFILEINFOW = std::mem::zeroed();
            SHGetFileInfoW(
                ws.as_ptr(),
                0,
                &mut info,
                std::mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_LARGEICON,
            );
            hicon = info.hIcon;
        }
        if hicon.is_null() {
            // 用扩展名取通用图标
            let ext = Path::new(src)
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_else(|| "exe".to_string());
            let dummy = format!("x.{}", ext);
            let wd = wide(&dummy);
            let mut info: SHFILEINFOW = std::mem::zeroed();
            SHGetFileInfoW(
                wd.as_ptr(),
                0x80,
                &mut info,
                std::mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_LARGEICON | SHGFI_USEFILEATTRIBUTES,
            );
            hicon = info.hIcon;
        }
        if hicon.is_null() {
            *w = 0;
            *h = 0;
            return;
        }
        // 首选：直接读 HICON 的颜色位图 + 掩码。
        // GdipCreateBitmapFromHICON 对"有掩码但颜色位图 alpha 全 255"的图标
        // 会返回整块不透明的黑色背景，所以不能只靠它。
        if let Some((rw, rh, px)) = hicon_to_pixels(hicon) {
            DestroyIcon(hicon);
            *w = rw;
            *h = rh;
            *out = px;
            return;
        }
        // 兜底：GDI+
        let mut bmp: GpImage = std::ptr::null_mut();
        let st = GdipCreateBitmapFromHICON(hicon, &mut bmp);
        DestroyIcon(hicon);
        if st != 0 || bmp.is_null() {
            *w = 0;
            *h = 0;
            return;
        }
        let mut iw: u32 = 0;
        let mut ih: u32 = 0;
        GdipGetImageWidth(bmp, &mut iw);
        GdipGetImageHeight(bmp, &mut ih);
        let mut data: BitmapData = std::mem::zeroed();
        let rc = GpRect { X: 0, Y: 0, Width: iw as i32, Height: ih as i32 };
        let st2 = GdipBitmapLockBits(
            bmp,
            &rc,
            1, // ImageLockModeRead
            PIXEL_FORMAT_32BPP_PARGB_PUB,
            &mut data,
        );
        if st2 == 0 && !data.scan0.is_null() {
            *w = iw as i32;
            *h = ih as i32;
            out.clear();
            out.reserve((iw * ih) as usize);
            for y in 0..ih as i32 {
                let row = data.scan0.offset((y * data.stride) as isize) as *const u32;
                for x in 0..iw as i32 {
                    let v = *row.offset(x as isize);
                    // GDI+ 给的是预乘 ARGB：必须原样保留半透明，不能强行置 1
                    out.push(if (v >> 24) == 0 { 0 } else { v });
                }
            }
            GdipBitmapUnlockBits(bmp, &mut data);
        } else {
            *w = 0;
            *h = 0;
        }
        GdipDisposeImage(bmp);
    }
}

/// 直接读取 HICON 的 32bpp 颜色位图与 1bpp 掩码，自行合成预乘 ARGB。
/// 这样能正确保留半透明边缘，也不会出现"透明区域变黑底"的问题。
unsafe fn hicon_to_pixels(hicon: HICON) -> Option<(i32, i32, Vec<u32>)> {
    let mut ii: ICONINFO = std::mem::zeroed();
    if GetIconInfo(hicon, &mut ii) == 0 {
        return None;
    }
    let mut bm: BITMAP = std::mem::zeroed();
    let ok = GetObjectW(
        ii.hbmColor,
        std::mem::size_of::<BITMAP>() as i32,
        &mut bm as *mut BITMAP as *mut std::ffi::c_void,
    );
    let (cw, ch) = (bm.bmWidth, bm.bmHeight);
    if ok == 0 || cw <= 0 || ch <= 0 || bm.bmBitsPixel != 32 {
        DeleteObject(ii.hbmMask);
        DeleteObject(ii.hbmColor);
        return None;
    }

    // 颜色位图 -> 自上而下的 BGRA
    let mut bi: BITMAPINFO = std::mem::zeroed();
    bi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bi.bmiHeader.biWidth = cw;
    bi.bmiHeader.biHeight = -ch;
    bi.bmiHeader.biPlanes = 1;
    bi.bmiHeader.biBitCount = 32;
    bi.bmiHeader.biCompression = 0; // BI_RGB
    let mut buf = vec![0u8; (cw * ch * 4) as usize];
    let dc = CreateCompatibleDC(std::ptr::null_mut());
    let got = GetDIBits(
        dc,
        ii.hbmColor,
        0,
        ch as u32,
        buf.as_mut_ptr() as *mut std::ffi::c_void,
        &mut bi,
        0,
    );
    DeleteDC(dc);
    if got == 0 {
        DeleteObject(ii.hbmMask);
        DeleteObject(ii.hbmColor);
        return None;
    }

    // 掩码位图 -> 每像素 1 bit（1 = 透明）
    let mut mask: Vec<u8> = Vec::new();
    let mut mstride = 0usize;
    let mut mh = 0i32;
    let mut mbm: BITMAP = std::mem::zeroed();
    if !ii.hbmMask.is_null()
        && GetObjectW(
            ii.hbmMask,
            std::mem::size_of::<BITMAP>() as i32,
            &mut mbm as *mut BITMAP as *mut std::ffi::c_void,
        ) != 0
        && mbm.bmWidth > 0
        && mbm.bmHeight >= ch
    {
        mh = mbm.bmHeight;
        mstride = (((mbm.bmWidth + 31) / 32) * 4) as usize;
        mask = vec![0u8; mstride * mh as usize];
        let mut mbi: BITMAPINFO = std::mem::zeroed();
        mbi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        mbi.bmiHeader.biWidth = mbm.bmWidth;
        mbi.bmiHeader.biHeight = -mh;
        mbi.bmiHeader.biPlanes = 1;
        mbi.bmiHeader.biBitCount = 1;
        mbi.bmiHeader.biCompression = 0;
        mbi.bmiHeader.biClrUsed = 2;
        let mdc = CreateCompatibleDC(std::ptr::null_mut());
        let mg = GetDIBits(
            mdc,
            ii.hbmMask,
            0,
            mh as u32,
            mask.as_mut_ptr() as *mut std::ffi::c_void,
            &mut mbi,
            0,
        );
        DeleteDC(mdc);
        if mg == 0 {
            mask.clear();
        }
    }
    DeleteObject(ii.hbmMask);
    DeleteObject(ii.hbmColor);

    // 颜色位图的 alpha 全 0 或全 255 时说明它没有可用的 alpha 通道，改用掩码
    let n = (cw * ch) as usize;
    let mut all_full = true;
    let mut all_zero = true;
    for i in 0..n {
        let a = buf[i * 4 + 3];
        if a != 255 {
            all_full = false;
        }
        if a != 0 {
            all_zero = false;
        }
        if !all_full && !all_zero {
            break;
        }
    }
    let use_mask = all_full || all_zero;

    let mut out: Vec<u32> = Vec::with_capacity(n);
    for y in 0..ch {
        for x in 0..cw {
            let i = ((y * cw + x) * 4) as usize;
            let (b, g, r, a0) = (buf[i], buf[i + 1], buf[i + 2], buf[i + 3]);
            let mbit = if !mask.is_empty() && y < mh {
                let row = y as usize * mstride;
                let byte = mask[row + x as usize / 8];
                (byte >> (7 - (x as usize % 8))) & 1
            } else {
                0
            };
            let a = if mbit == 1 {
                0
            } else if use_mask {
                255
            } else {
                a0
            };
            if a == 0 {
                out.push(0);
                continue;
            }
            let (pr, pg, pb) = if a == 255 {
                (r, g, b)
            } else {
                let af = a as u32;
                (
                    ((r as u32 * af + 127) / 255) as u8,
                    ((g as u32 * af + 127) / 255) as u8,
                    ((b as u32 * af + 127) / 255) as u8,
                )
            };
            out.push(((a as u32) << 24) | ((pr as u32) << 16) | ((pg as u32) << 8) | (pb as u32));
        }
    }
    Some((cw, ch, out))
}
