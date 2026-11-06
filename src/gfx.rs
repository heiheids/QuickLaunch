//! GDI+ 自绘画布：32 位预乘 ARGB DIB + UpdateLayeredWindow（逐像素透明）。
use crate::win::*;
use std::collections::HashMap;

pub const UNIT_PIXEL: i32 = 2;
const PIXEL_FORMAT_32BPP_PARGB: i32 = 0x000E_200B;
pub const PIXEL_FORMAT_32BPP_PARGB_PUB: i32 = PIXEL_FORMAT_32BPP_PARGB;

pub const ALIGN_NEAR: i32 = 0;
pub const ALIGN_CENTER: i32 = 1;
pub const ALIGN_FAR: i32 = 2;

pub const TRIM_NONE: i32 = 0;
pub const TRIM_ELLIPSIS_CHAR: i32 = 3;

pub fn argb(a: u8, r: u8, g: u8, b: u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

pub fn rgb(r: u8, g: u8, b: u8) -> u32 {
    argb(255, r, g, b)
}

pub fn with_alpha(c: u32, a: u8) -> u32 {
    (c & 0x00FF_FFFF) | ((a as u32) << 24)
}

pub fn luminance(c: u32) -> f32 {
    let r = ((c >> 16) & 0xff) as f32;
    let g = ((c >> 8) & 0xff) as f32;
    let b = (c & 0xff) as f32;
    0.299 * r + 0.587 * g + 0.114 * b
}

pub fn mix(c1: u32, c2: u32, t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let ch = |sh: u32| -> u8 {
        let a = ((c1 >> sh) & 0xff) as f32;
        let b = ((c2 >> sh) & 0xff) as f32;
        (a + (b - a) * t) as u8
    };
    argb(ch(24), ch(16), ch(8), ch(0))
}

/// 依据底色明暗，生成"选中块"的配色（浅色主题偏白，深色主题偏亮）
pub fn panel_hl(c: u32) -> u32 {
    let r = ((c >> 16) & 0xff) as f32;
    let g = ((c >> 8) & 0xff) as f32;
    let b = (c & 0xff) as f32;
    let lum = 0.299 * r + 0.587 * g + 0.114 * b;
    if lum < 120.0 {
        mix(c, rgb(255, 255, 255), 0.14)
    } else {
        mix(c, rgb(255, 255, 255), 0.78)
    }
}

pub fn parse_color(s: &str) -> Option<u32> {
    let t = s.trim().trim_start_matches('#');
    let v = u32::from_str_radix(t, 16).ok()?;
    Some(match t.len() {
        6 => 0xFF00_0000 | v,
        8 => v,
        _ => return None,
    })
}

pub fn color_to_hex(c: u32) -> String {
    let a = (c >> 24) & 0xff;
    if a == 0xff {
        format!("#{:02X}{:02X}{:02X}", (c >> 16) & 0xff, (c >> 8) & 0xff, c & 0xff)
    } else {
        format!("#{:02X}{:02X}{:02X}{:02X}", a, (c >> 16) & 0xff, (c >> 8) & 0xff, c & 0xff)
    }
}

/// COLORREF (0x00BBGGRR) 与 ARGB 互转
pub fn colorref_to_argb(c: COLORREF) -> u32 {
    argb(255, (c & 0xff) as u8, ((c >> 8) & 0xff) as u8, ((c >> 16) & 0xff) as u8)
}
pub fn argb_to_colorref(c: u32) -> COLORREF {
    ((c & 0xff) << 16) | (((c >> 8) & 0xff) << 8) | ((c >> 16) & 0xff)
}

pub struct Fonts {
    family: GpFontFamily,
    cache: HashMap<(i32, i32), GpFont>,
}

impl Fonts {
    pub fn new() -> Fonts {
        let mut family: GpFontFamily = std::ptr::null_mut();
        for name in ["Microsoft YaHei UI", "Microsoft YaHei", "SimSun", "Arial"] {
            let w = wide(name);
            let st = unsafe { GdipCreateFontFamilyFromName(w.as_ptr(), std::ptr::null_mut(), &mut family) };
            if st == 0 && !family.is_null() {
                break;
            }
        }
        Fonts { family, cache: HashMap::new() }
    }

    pub fn get(&mut self, size: i32, bold: bool) -> GpFont {
        let key = (size, bold as i32);
        if let Some(&f) = self.cache.get(&key) {
            return f;
        }
        let mut font: GpFont = std::ptr::null_mut();
        let style = if bold { 1 } else { 0 };
        let st = unsafe { GdipCreateFont(self.family, size as f32, style, UNIT_PIXEL, &mut font) };
        if st != 0 || font.is_null() {
            font = std::ptr::null_mut();
        }
        self.cache.insert(key, font);
        font
    }
}

impl Drop for Fonts {
    fn drop(&mut self) {
        for (_, f) in self.cache.drain() {
            if !f.is_null() {
                unsafe { GdipDeleteFont(f) };
            }
        }
        if !self.family.is_null() {
            unsafe { GdipDeleteFontFamily(self.family) };
        }
    }
}

pub struct Canvas {
    pub w: i32,
    pub h: i32,
    pub hdc: HDC,
    hbmp: HBITMAP,
    pub bits: *mut u32,
    pub g: GpGraphics,
    bmp: GpImage,
    pub fonts: Fonts,
    brush: GpBrush,
    pen: GpPen,
    fmts: HashMap<u32, GpStringFormat>,
}

impl Canvas {
    pub fn new(w: i32, h: i32) -> Canvas {
        let mut c = Canvas {
            w: 0,
            h: 0,
            hdc: std::ptr::null_mut(),
            hbmp: std::ptr::null_mut(),
            bits: std::ptr::null_mut(),
            g: std::ptr::null_mut(),
            bmp: std::ptr::null_mut(),
            fonts: Fonts::new(),
            brush: std::ptr::null_mut(),
            pen: std::ptr::null_mut(),
            fmts: HashMap::new(),
        };
        c.resize(w, h);
        c
    }

    /// 供"静态层缓存"使用：把本画布内容作为一张位图交给别的画布合成
    pub fn image(&self) -> GpImage {
        self.bmp
    }

    pub fn resize(&mut self, w: i32, h: i32) {
        let w = w.max(1);
        let h = h.max(1);
        if w == self.w && h == self.h && !self.bits.is_null() {
            return;
        }
        self.release();
        unsafe {
            let screen = GetDC(std::ptr::null_mut());
            self.hdc = CreateCompatibleDC(screen);
            let mut bi: BITMAPINFO = std::mem::zeroed();
            bi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            bi.bmiHeader.biWidth = w;
            bi.bmiHeader.biHeight = -h; // 自上而下
            bi.bmiHeader.biPlanes = 1;
            bi.bmiHeader.biBitCount = 32;
            bi.bmiHeader.biCompression = BI_RGB;
            let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
            self.hbmp = CreateDIBSection(self.hdc, &bi, DIB_RGB_COLORS, &mut bits, std::ptr::null_mut(), 0);
            SelectObject(self.hdc, self.hbmp);
            ReleaseDC(std::ptr::null_mut(), screen);
            self.bits = bits as *mut u32;
            let mut bmp: GpImage = std::ptr::null_mut();
            GdipCreateBitmapFromScan0(
                w,
                h,
                w * 4,
                PIXEL_FORMAT_32BPP_PARGB,
                bits as *mut u8,
                &mut bmp,
            );
            self.bmp = bmp;
            let mut g: GpGraphics = std::ptr::null_mut();
            GdipGetImageGraphicsContext(bmp, &mut g);
            self.g = g;
            GdipSetSmoothingMode(g, 4); // AntiAlias
            GdipSetTextRenderingHint(g, 4); // AntiAlias
            GdipSetInterpolationMode(g, 7); // HighQualityBicubic
            GdipSetPixelOffsetMode(g, 4); // Half
            GdipSetCompositingQuality(g, 0); // Default（HighQuality 的 gamma 混合慢一个数量级）
            GdipGraphicsClear(g, 0);
            GdipCreateSolidFill(0xFFFF_FFFF, &mut self.brush);
            GdipCreatePen1(0xFFFF_FFFF, 1.0, UNIT_PIXEL, &mut self.pen);
        }
        self.w = w;
        self.h = h;
    }

    fn release(&mut self) {
        unsafe {
            for (_, f) in self.fmts.drain() {
                if !f.is_null() {
                    GdipDeleteStringFormat(f);
                }
            }
            if !self.brush.is_null() {
                GdipDeleteBrush(self.brush);
                self.brush = std::ptr::null_mut();
            }
            if !self.pen.is_null() {
                GdipDeletePen(self.pen);
                self.pen = std::ptr::null_mut();
            }
            if !self.g.is_null() {
                GdipDeleteGraphics(self.g);
                self.g = std::ptr::null_mut();
            }
            if !self.bmp.is_null() {
                GdipDisposeImage(self.bmp);
                self.bmp = std::ptr::null_mut();
            }
            if !self.hbmp.is_null() {
                DeleteObject(self.hbmp);
                self.hbmp = std::ptr::null_mut();
            }
            if !self.hdc.is_null() {
                DeleteDC(self.hdc);
                self.hdc = std::ptr::null_mut();
            }
            self.bits = std::ptr::null_mut();
            self.w = 0;
            self.h = 0;
        }
    }

    pub fn present(&self, hwnd: HWND, alpha: u8) {
        unsafe {
            let screen = GetDC(std::ptr::null_mut());
            let mut pt = POINT { x: 0, y: 0 };
            let mut size = SIZE { cx: self.w, cy: self.h };
            let mut src = POINT { x: 0, y: 0 };
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER,
                BlendFlags: 0,
                SourceConstantAlpha: alpha,
                AlphaFormat: AC_SRC_ALPHA,
            };
            let mut wr: RECT = std::mem::zeroed();
            GetWindowRect(hwnd, &mut wr);
            pt.x = wr.left;
            pt.y = wr.top;
            size.cx = wr.w();
            size.cy = wr.h();
            UpdateLayeredWindow(
                hwnd,
                screen,
                &pt,
                &size,
                self.hdc,
                &mut src,
                0,
                &blend,
                ULW_ALPHA,
            );
            ReleaseDC(std::ptr::null_mut(), screen);
        }
    }

    pub fn clear(&self, color: u32) {
        unsafe {
            GdipGraphicsClear(self.g, color);
        }
    }

    pub fn clip(&self, r: RECT) {
        unsafe {
            GdipSetClipRect(self.g, r.left as f32, r.top as f32, r.w() as f32, r.h() as f32, 1);
        }
    }

    pub fn unclip(&self) {
        unsafe {
            GdipResetClip(self.g);
        }
    }

    pub fn fill_rect(&self, x: f32, y: f32, w: f32, h: f32, color: u32) {
        unsafe {
            if !self.brush.is_null() {
                GdipSetSolidFillColor(self.brush, color);
                GdipFillRectangle(self.g, self.brush, x, y, w, h);
            } else {
                let mut br: GpBrush = std::ptr::null_mut();
                GdipCreateSolidFill(color, &mut br);
                GdipFillRectangle(self.g, br, x, y, w, h);
                GdipDeleteBrush(br);
            }
        }
    }

    pub fn fill_round(&self, r: RECTF, rad: [f32; 4], color: u32) {
        if r.w <= 0.0 || r.h <= 0.0 {
            return;
        }
        // 直角时走矩形快速路径（GDI+ 的 AA 路径填充要慢一个数量级）
        if rad.iter().all(|&v| v <= 0.0) {
            return self.fill_rect(r.x, r.y, r.w, r.h, color);
        }
        unsafe {
            let mut path: GpPath = std::ptr::null_mut();
            GdipCreatePath(0, &mut path);
            self.round_path(path, r, rad);
            if !self.brush.is_null() {
                GdipSetSolidFillColor(self.brush, color);
                GdipFillPath(self.g, self.brush, path);
            } else {
                let mut br: GpBrush = std::ptr::null_mut();
                GdipCreateSolidFill(color, &mut br);
                GdipFillPath(self.g, br, path);
                GdipDeleteBrush(br);
            }
            GdipDeletePath(path);
        }
    }

    pub fn stroke_round(&self, r: RECTF, rad: [f32; 4], width: f32, color: u32) {
        if r.w <= 0.0 || r.h <= 0.0 {
            return;
        }
        if rad.iter().all(|&v| v <= 0.0) {
            unsafe {
                if !self.pen.is_null() {
                    GdipSetPenColor(self.pen, color);
                    GdipSetPenWidth(self.pen, width);
                    GdipDrawRectangle(self.g, self.pen, r.x, r.y, r.w, r.h);
                } else {
                    let mut pen: GpPen = std::ptr::null_mut();
                    GdipCreatePen1(color, width, UNIT_PIXEL, &mut pen);
                    GdipDrawRectangle(self.g, pen, r.x, r.y, r.w, r.h);
                    GdipDeletePen(pen);
                }
            }
            return;
        }
        unsafe {
            let mut path: GpPath = std::ptr::null_mut();
            GdipCreatePath(0, &mut path);
            self.round_path(path, r, rad);
            if !self.pen.is_null() {
                GdipSetPenColor(self.pen, color);
                GdipSetPenWidth(self.pen, width);
                GdipDrawPath(self.g, self.pen, path);
            } else {
                let mut pen: GpPen = std::ptr::null_mut();
                GdipCreatePen1(color, width, UNIT_PIXEL, &mut pen);
                GdipDrawPath(self.g, pen, path);
                GdipDeletePen(pen);
            }
            GdipDeletePath(path);
        }
    }

    pub fn line(&self, x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: u32) {
        unsafe {
            if !self.pen.is_null() {
                GdipSetPenColor(self.pen, color);
                GdipSetPenWidth(self.pen, width);
                GdipDrawLine(self.g, self.pen, x1, y1, x2, y2);
            } else {
                let mut pen: GpPen = std::ptr::null_mut();
                GdipCreatePen1(color, width, UNIT_PIXEL, &mut pen);
                GdipDrawLine(self.g, pen, x1, y1, x2, y2);
                GdipDeletePen(pen);
            }
        }
    }

    unsafe fn round_path(&self, path: GpPath, r: RECTF, rad: [f32; 4]) {
        let maxr = (r.w.min(r.h) / 2.0).max(0.0);
        let tl = rad[0].clamp(0.0, maxr);
        let tr = rad[1].clamp(0.0, maxr);
        let br_ = rad[2].clamp(0.0, maxr);
        let bl = rad[3].clamp(0.0, maxr);
        let x0 = r.x;
        let y0 = r.y;
        let x1 = r.x + r.w;
        let y1 = r.y + r.h;

        // 从左上角起始点开始，顺时针
        GdipStartPathFigure(path);
        GdipAddPathLine(path, x0 + tl, y0, x1 - tr, y0);
        if tr > 0.0 {
            GdipAddPathArc(path, x1 - tr * 2.0, y0, tr * 2.0, tr * 2.0, 270.0, 90.0);
        }
        GdipAddPathLine(path, x1, y0 + tr, x1, y1 - br_);
        if br_ > 0.0 {
            GdipAddPathArc(path, x1 - br_ * 2.0, y1 - br_ * 2.0, br_ * 2.0, br_ * 2.0, 0.0, 90.0);
        }
        GdipAddPathLine(path, x1 - br_, y1, x0 + bl, y1);
        if bl > 0.0 {
            GdipAddPathArc(path, x0, y1 - bl * 2.0, bl * 2.0, bl * 2.0, 90.0, 90.0);
        }
        GdipAddPathLine(path, x0, y1 - bl, x0, y0 + tl);
        if tl > 0.0 {
            GdipAddPathArc(path, x0, y0, tl * 2.0, tl * 2.0, 180.0, 90.0);
        }
        GdipClosePathFigure(path);
    }

    pub fn draw_text(
        &mut self,
        txt: &str,
        r: RECTF,
        size: i32,
        bold: bool,
        color: u32,
        align: i32,
        wrap: bool,
        trim: i32,
    ) {
        if txt.is_empty() || r.w <= 0.0 || r.h <= 0.0 {
            return;
        }
        let font = self.fonts.get(size, bold);
        if font.is_null() {
            return;
        }
        let u: Vec<u16> = txt.encode_utf16().collect();
        let key = (align as u32 & 3) | ((wrap as u32) << 2) | (((trim as u32) & 15) << 3);
        let fmt = match self.fmts.get(&key) {
            Some(&f) => f,
            None => {
                let mut f: GpStringFormat = std::ptr::null_mut();
                unsafe {
                    GdipCreateStringFormat(0, 0, &mut f);
                    GdipSetStringFormatAlign(f, align);
                    GdipSetStringFormatLineAlign(f, if wrap { 0 } else { 1 });
                    if trim != TRIM_NONE {
                        GdipSetStringFormatTrimming(f, trim);
                    }
                    if !wrap {
                        GdipSetStringFormatFlags(f, 0x1000);
                    }
                }
                self.fmts.insert(key, f);
                f
            }
        };
        unsafe {
            if !self.brush.is_null() {
                GdipSetSolidFillColor(self.brush, color);
                GdipDrawString(self.g, u.as_ptr(), u.len() as i32, font, &r, fmt, self.brush);
            } else {
                let mut br: GpBrush = std::ptr::null_mut();
                GdipCreateSolidFill(color, &mut br);
                GdipDrawString(self.g, u.as_ptr(), u.len() as i32, font, &r, fmt, br);
                GdipDeleteBrush(br);
            }
        }
    }

    pub fn measure(&mut self, txt: &str, size: i32, bold: bool, maxw: f32, wrap: bool) -> (f32, f32) {
        if txt.is_empty() {
            return (0.0, 0.0);
        }
        let font = self.fonts.get(size, bold);
        if font.is_null() {
            return (0.0, 0.0);
        }
        let u: Vec<u16> = txt.encode_utf16().collect();
        unsafe {
            let mut fmt: GpStringFormat = std::ptr::null_mut();
            GdipCreateStringFormat(0, 0, &mut fmt);
            GdipSetStringFormatAlign(fmt, ALIGN_NEAR);
            if !wrap {
                GdipSetStringFormatFlags(fmt, 0x1000);
            }
            let layout = RECTF { x: 0.0, y: 0.0, w: maxw.max(1.0), h: 100000.0 };
            let mut bounds = RECTF { x: 0.0, y: 0.0, w: 0.0, h: 0.0 };
            let mut fitted: i32 = 0;
            let mut lines: i32 = 0;
            GdipMeasureString(
                self.g,
                u.as_ptr(),
                u.len() as i32,
                font,
                &layout,
                fmt,
                &mut bounds,
                &mut fitted,
                &mut lines,
            );
            GdipDeleteStringFormat(fmt);
            (bounds.w, bounds.h)
        }
    }

    pub fn draw_image(&self, img: GpImage, x: f32, y: f32, w: f32, h: f32) {
        if img.is_null() {
            return;
        }
        unsafe {
            let mut iw: u32 = 0;
            let mut ih: u32 = 0;
            GdipGetImageWidth(img, &mut iw);
            GdipGetImageHeight(img, &mut ih);
            GdipDrawImageRectRect(
                self.g,
                img,
                x,
                y,
                w,
                h,
                0.0,
                0.0,
                iw as f32,
                ih as f32,
                UNIT_PIXEL,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
        }
    }

    /// 1:1 快速贴图（用于把静态层缓存合成到当前帧），跳过双三次插值
    pub fn blit(&self, img: GpImage, w: f32, h: f32) {
        if img.is_null() {
            return;
        }
        unsafe {
            GdipSetInterpolationMode(self.g, 1); // NearestNeighbor
            GdipDrawImageRectRect(
                self.g,
                img,
                0.0,
                0.0,
                w,
                h,
                0.0,
                0.0,
                w,
                h,
                UNIT_PIXEL,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            GdipSetInterpolationMode(self.g, 7); // HighQualityBicubic
        }
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        self.release();
    }
}

pub fn gp_image_from_file(path: &str) -> GpImage {
    let w = wide(path);
    let mut bmp: GpImage = std::ptr::null_mut();
    let st = unsafe { GdipCreateBitmapFromFile(w.as_ptr(), &mut bmp) };
    if st != 0 {
        return std::ptr::null_mut();
    }
    bmp
}

pub fn gp_image_from_pixels(w: i32, h: i32, pixels: &mut [u8]) -> GpImage {
    let mut bmp: GpImage = std::ptr::null_mut();
    let st = unsafe {
        GdipCreateBitmapFromScan0(w, h, w * 4, PIXEL_FORMAT_32BPP_PARGB, pixels.as_mut_ptr(), &mut bmp)
    };
    if st != 0 {
        return std::ptr::null_mut();
    }
    bmp
}

pub fn gp_dispose(img: GpImage) {
    if !img.is_null() {
        unsafe { GdipDisposeImage(img) };
    }
}
