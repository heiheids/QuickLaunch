//! 主窗口：布局、自绘、命中测试、右键菜单、托盘、设置、启动逻辑。
use crate::data::*;
use crate::drop;
use crate::gfx::*;
use crate::icons::IconStore;
use crate::win::*;
use std::path::{Path, PathBuf};

pub const WM_TRAY: u32 = WM_APP + 1;
pub const WM_INPUT_OK: u32 = WM_APP + 2;
pub const WM_INPUT_CANCEL: u32 = WM_APP + 3;
const WM_NULL: u32 = 0;

// ---- 菜单命令 ----
const ID_NEW_GROUP: usize = 1;
const ID_REN_GROUP: usize = 2;
const ID_DEL_GROUP: usize = 3;
const ID_GROUP_UP: usize = 4;
const ID_GROUP_DOWN: usize = 5;
const ID_ICON_S: usize = 20;
const ID_ICON_M: usize = 21;
const ID_ICON_L: usize = 22;
const ID_VIEW_TILE: usize = 23;
const ID_VIEW_LIST: usize = 24;
const ID_ADD_FILE: usize = 30;
const ID_ADD_DIR: usize = 31;
const ID_ADD_URL: usize = 32;
const ID_REFRESH_ALL: usize = 34;
const ID_CLEAR_CACHE: usize = 35;
const ID_RUN: usize = 40;
const ID_RUN_ADMIN: usize = 41;
const ID_OPEN_LOC: usize = 42;
const ID_EDIT: usize = 43;
const ID_DEL_LINK: usize = 44;
const ID_PROPS: usize = 45;
const ID_MANAGE: usize = 46;
const ID_EMPTY_BIN: usize = 47;
const ID_SEL_CLEAR: usize = 48;
const ID_MOVE_BASE: usize = 400;
const ID_SETTINGS: usize = 50;
const ID_TOG_TITLE: usize = 51;
const ID_TOG_DATE: usize = 52;
const ID_TOG_TOP: usize = 53;
const ID_ABOUT: usize = 54;
const ID_EXIT: usize = 55;
const ID_HIDE: usize = 56;
const ID_SYS_BASE: usize = 200;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hit {
    None,
    Shadow,
    Title,
    Search,
    Menu,
    Close,
    GroupBar,
    Group(usize),
    Body,
    Link(usize),
    Edge(i32),
    Row(usize),
    OverlayBtn(usize),
    InputOk,
    InputCancel,
    InputField,
    FilterChip,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Overlay {
    None,
    Settings,
    Input,
    Hotkey,
}

#[derive(Clone)]
enum InputAction {
    NewGroup,
    RenameGroup(u32),
    AddUrl,
    RenameLink(u32),
    Search,
    SetTitle,
}

enum BtnKind {
    Close,
    Menu,
    Search,
}

pub struct SysItem {
    pub name: &'static str,
    pub key: &'static str,
    pub cmd: &'static str,
    pub args: &'static str,
    pub icon: &'static str,
    /// "管理" 命令（空 = 没有）
    pub manage: &'static str,
    /// "属性" 用 ShellExecute 的 properties 动词打开的目标（空 = 用路径本身）
    pub props: &'static str,
}

const CLSID_THIS_PC: &str = "::{20D04FE0-3AEA-1069-A2D8-08002B30309D}";
const CLSID_RECYCLE: &str = "::{645FF040-5081-101B-9F08-00AA002F954E}";

pub const SYS_ITEMS: &[SysItem] = &[
    SysItem { name: "记事本", key: "notepad", cmd: "notepad.exe", args: "", icon: "notepad.exe", manage: "", props: "" },
    SysItem { name: "计算器", key: "calc", cmd: "calc.exe", args: "", icon: "calc.exe", manage: "", props: "" },
    SysItem { name: "命令行", key: "cmd", cmd: "cmd.exe", args: "", icon: "cmd.exe", manage: "", props: "" },
    SysItem { name: "任务管理器", key: "taskmgr", cmd: "taskmgr.exe", args: "", icon: "taskmgr.exe", manage: "", props: "" },
    SysItem { name: "控制面板", key: "control", cmd: "control.exe", args: "", icon: "control.exe", manage: "", props: "" },
    SysItem { name: "此电脑", key: "thispc", cmd: CLSID_THIS_PC, args: "", icon: CLSID_THIS_PC, manage: "compmgmt.msc", props: CLSID_THIS_PC },
    SysItem { name: "回收站", key: "recyclebin", cmd: CLSID_RECYCLE, args: "", icon: CLSID_RECYCLE, manage: "", props: CLSID_RECYCLE },
    SysItem { name: "资源管理器", key: "explorer", cmd: "explorer.exe", args: "", icon: "explorer.exe", manage: "", props: "" },
    SysItem { name: "系统设置", key: "settings", cmd: "ms-settings:", args: "", icon: "explorer.exe", manage: "", props: "" },
    SysItem { name: "画图", key: "mspaint", cmd: "mspaint.exe", args: "", icon: "mspaint.exe", manage: "", props: "" },
    SysItem { name: "注册表", key: "regedit", cmd: "regedit.exe", args: "", icon: "regedit.exe", manage: "", props: "" },
    SysItem { name: "服务", key: "services", cmd: "services.msc", args: "", icon: "services.msc", manage: "", props: "" },
    SysItem { name: "设备管理器", key: "devmgmt", cmd: "devmgmt.msc", args: "", icon: "devmgmt.msc", manage: "", props: "" },
    SysItem { name: "网络连接", key: "ncpa", cmd: "ncpa.cpl", args: "", icon: "ncpa.cpl", manage: "", props: "" },
    SysItem { name: "任务计划", key: "taskschd", cmd: "taskschd.msc", args: "", icon: "taskschd.msc", manage: "", props: "" },
    SysItem { name: "关机", key: "shutdown", cmd: "shutdown.exe", args: "/s /t 0", icon: "shutdown.exe", manage: "", props: "" },
];

pub fn sys_item(key: &str) -> Option<&'static SysItem> {
    SYS_ITEMS.iter().find(|s| s.key == key)
}

#[derive(Clone)]
struct Layout {
    panel: RECT,
    title: RECT,
    bsearch: RECT,
    bmenu: RECT,
    bclose: RECT,
    group: RECT,
    body: RECT,
    gitems: Vec<RECT>,
    litems: Vec<RECT>,
    content_h: i32,
    chip: Option<RECT>,
}

impl Layout {
    fn empty() -> Layout {
        let z = RECT::new(0, 0, 0, 0);
        Layout {
            panel: z,
            title: z,
            bsearch: z,
            bmenu: z,
            bclose: z,
            group: z,
            body: z,
            gitems: Vec::new(),
            litems: Vec::new(),
            content_h: 0,
            chip: None,
        }
    }
}

pub struct App {
    pub hwnd: HWND,
    pub dir: PathBuf,
    pub conf: Conf,
    pub db: Db,
    canvas: Canvas,
    icons: IconStore,
    layout: Layout,
    lkey: u64,
    bg: Canvas,
    bgkey: u64,
    hover: Hit,
    pressed: Option<Hit>,
    drag_start: Option<(i32, i32, i32, i32)>,
    resize_dir: Option<i32>,
    overlay: Overlay,
    input_title: String,
    input_action: InputAction,
    input_text: String,
    input_caret: usize,
    scroll: i32,
    tray_ok: bool,
    tray_icon: HICON,
    filter: String,
    /// 多选：选中的链接 ID（按 ID 存，排序/刷新后仍有效）
    sel: Vec<u32>,
    last_w: i32,
    last_h: i32,
}

impl App {
    pub fn new(hwnd: HWND, dir: PathBuf) -> App {
        let conf = Conf::load(&dir);
        let db = match Db::load(&dir) {
            Some(d) => d,
            None => {
                let d = Db::default_data();
                d.save(&dir);
                d
            }
        };
        let canvas = Canvas::new(conf.w, conf.h);
        let icons = IconStore::new(dir.join("cache").join("icons"));
        let mut app = App {
            hwnd,
            dir,
            conf,
            db,
            canvas,
            icons,
            layout: Layout::empty(),
            lkey: 0,
            bg: Canvas::new(1, 1),
            bgkey: 0,
            hover: Hit::None,
            pressed: None,
            drag_start: None,
            resize_dir: None,
            overlay: Overlay::None,
            input_title: String::new(),
            input_action: InputAction::NewGroup,
            input_text: String::new(),
            input_caret: 0,
            scroll: 0,
            tray_ok: false,
            tray_icon: std::ptr::null_mut(),
            filter: String::new(),
            sel: Vec::new(),
            last_w: 0,
            last_h: 0,
        };
        if app.db.group_index(app.conf.cur_group).is_none() {
            app.conf.cur_group = app.db.groups.first().map(|g| g.id).unwrap_or(0);
        }
        app.last_w = app.conf.w;
        app.last_h = app.conf.h;
        app
    }

    // ---------------- 数据 ----------------
    fn cur_links(&self) -> Vec<usize> {
        let mut v = self.db.links_of_group(self.conf.cur_group);
        if !self.filter.is_empty() {
            let f = self.filter.to_lowercase();
            v.retain(|&i| {
                let l = &self.db.links[i];
                l.name.to_lowercase().contains(&f)
                    || l.path.to_lowercase().contains(&f)
                    || l.remark.to_lowercase().contains(&f)
            });
        }
        v
    }

    // ---------------- 多选 ----------------
    fn is_sel(&self, id: u32) -> bool {
        self.sel.contains(&id)
    }

    fn sel_toggle(&mut self, id: u32) {
        if let Some(p) = self.sel.iter().position(|&x| x == id) {
            self.sel.remove(p);
        } else {
            self.sel.push(id);
        }
    }

    fn move_links(&mut self, ids: &[u32], group: u32) {
        let tag = self.db.ensure_tag(group);
        for id in ids {
            if let Some(i) = self.db.link_index(*id) {
                self.db.links[i].tag = tag;
            }
        }
        self.sel.clear();
        self.db.save(&self.dir);
        self.repaint();
    }

    fn delete_links(&mut self, ids: &[u32]) {
        for id in ids {
            self.db.remove_link(*id);
        }
        self.sel.clear();
        self.db.save(&self.dir);
        self.repaint();
    }

    fn icon_source(&self, l: &Link) -> String {
        if l.kind == KIND_SYS {
            match sys_item(&l.path) {
                Some(s) => sys_dir(s.icon),
                None => String::new(),
            }
        } else {
            l.path.clone()
        }
    }

    // ---------------- 布局 ----------------
    fn layout_key(&self) -> u64 {
        let vals: [i64; 14] = [
            self.conf.w as i64,
            self.conf.h as i64,
            self.conf.shadow as i64,
            self.conf.show_title as i64,
            self.conf.show_group as i64,
            self.conf.group_w as i64,
            self.conf.font_size as i64,
            self.conf.icon_size as i64,
            self.conf.view as i64,
            self.scroll as i64,
            self.conf.cur_group as i64,
            self.db.groups.len() as i64,
            self.db.links.len() as i64,
            match self.overlay {
                Overlay::None => 0,
                Overlay::Settings => 1,
                Overlay::Input => 2,
                Overlay::Hotkey => 3,
            },
        ];
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for v in vals {
            h ^= v as u64;
            h = h.wrapping_mul(0x1000_0000_01b3);
        }
        for b in self.filter.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x1000_0000_01b3);
        }
        h
    }

    /// 布局只在几何相关参数变化时重建（鼠标移动、悬停重绘时直接复用）
    fn ensure_layout(&mut self) {
        let k = self.layout_key();
        if k != self.lkey {
            self.layout = self.build_layout();
            self.lkey = k;
        }
    }

    fn build_layout(&mut self) -> Layout {
        let s = if self.conf.shadow { 12 } else { 0 };
        let panel = RECT::new(s, s, self.conf.w - s, self.conf.h - s);
        let title_h = if self.conf.show_title { 40 } else { 0 };
        let title = RECT::new(panel.left, panel.top, panel.right, panel.top + title_h);
        let gw = if self.conf.show_group {
            self.conf.group_w.max(self.conf.font_size * 4 + 36)
        } else {
            0
        };
        let group = RECT::new(panel.left, title.bottom, panel.left + gw, panel.bottom);
        let body = RECT::new(group.right, title.bottom, panel.right, panel.bottom);

        let bs = 36;
        let bclose = RECT::new(title.right - bs, title.top, title.right, title.bottom);
        let bmenu = RECT::new(bclose.left - bs, title.top, bclose.left, title.bottom);
        let bsearch = RECT::new(bmenu.left - bs, title.top, bmenu.left, title.bottom);

        let mut gitems = Vec::new();
        if gw > 0 {
            let ih = self.conf.font_size + 18;
            let mut y = group.top + 8;
            for _ in 0..self.db.groups.len() {
                gitems.push(RECT::new(group.left + 4, y, group.right - 4, y + ih));
                y += ih + 2;
            }
        }

        let mut litems: Vec<RECT> = Vec::new();
        let mut content_h = 0;
        if body.w() > 0 && body.h() > 0 {
            let n = self.cur_links().len();
            if self.conf.view == 0 {
                let cw = self.conf.icon_size + 44;
                let ch = self.conf.icon_size + 52;
                let cols = ((body.w() - 16) / cw).max(1);
                let rows = (n as i32 + cols - 1) / cols;
                content_h = rows * ch + 16;
                let base_y = body.top + 8 - self.scroll;
                for i in 0..n {
                    let cx = body.left + 8 + (i as i32 % cols) * cw;
                    let cy = base_y + (i as i32 / cols) * ch;
                    litems.push(RECT::new(cx, cy, cx + cw, cy + ch));
                }
            } else {
                let rh = (self.conf.icon_size + 20).max(self.conf.font_size * 2 + 10);
                content_h = n as i32 * rh + 12;
                let base_y = body.top + 6 - self.scroll;
                for i in 0..n {
                    let cy = base_y + i as i32 * rh;
                    litems.push(RECT::new(body.left + 6, cy, body.right - 6, cy + rh));
                }
            }
        }

        let chip = if self.filter.is_empty() || body.w() <= 0 {
            None
        } else {
            let (tw, _) = self.canvas.measure(&self.filter, 12, false, 400.0, false);
            let w = tw as i32 + 44;
            Some(RECT::new(body.right - w - 10, body.top + 8, body.right - 10, body.top + 32))
        };

        Layout { panel, title, bsearch, bmenu, bclose, group, body, gitems, litems, content_h, chip }
    }

    fn clamp_scroll(&mut self) {
        self.ensure_layout();
        let m = (self.layout.content_h - self.layout.body.h()).max(0);
        self.scroll = self.scroll.clamp(0, m);
    }

    fn filter_chip_rect(&mut self) -> Option<RECT> {
        self.ensure_layout();
        self.layout.chip
    }

    fn settings_geom(&self) -> (RECT, RECT, RECT) {
        let rows = 13;
        let w = (self.conf.w - 40).clamp(300, 430);
        let h = 48 + rows * 34 + 50;
        let x = (self.conf.w - w) / 2;
        let y = (self.conf.h - h) / 2;
        let panel = RECT::new(x, y, x + w, y + h);
        let reset = RECT::new(panel.left + 14, panel.bottom - 40, panel.left + 104, panel.bottom - 12);
        let close = RECT::new(panel.right - 84, panel.bottom - 40, panel.right - 14, panel.bottom - 12);
        (panel, reset, close)
    }

    fn settings_row(&self, i: usize) -> RECT {
        let (panel, _, _) = self.settings_geom();
        let y = panel.top + 48 + i as i32 * 34;
        RECT::new(panel.left, y, panel.right, y + 34)
    }

    fn input_geom(&self) -> (RECT, RECT, RECT, RECT) {
        let w = (self.conf.w - 80).clamp(250, 380);
        let h = 158;
        let x = (self.conf.w - w) / 2;
        let y = (self.conf.h - h) / 2;
        let panel = RECT::new(x, y, x + w, y + h);
        let field = RECT::new(panel.left + 20, panel.top + 56, panel.right - 20, panel.top + 94);
        let ok = RECT::new(panel.right - 100, panel.bottom - 46, panel.right - 20, panel.bottom - 14);
        let cancel = RECT::new(panel.right - 186, panel.bottom - 46, panel.right - 106, panel.bottom - 14);
        (panel, field, ok, cancel)
    }

    // ---------------- 命中测试 ----------------
    fn hit_at(&mut self, x: i32, y: i32) -> Hit {
        match self.overlay {
            Overlay::Input => {
                let (_, field, ok, cancel) = self.input_geom();
                if ok.contains(x, y) {
                    return Hit::InputOk;
                }
                if cancel.contains(x, y) {
                    return Hit::InputCancel;
                }
                if field.contains(x, y) {
                    return Hit::InputField;
                }
                return Hit::None;
            }
            Overlay::Hotkey => {
                return Hit::None;
            }
            Overlay::Settings => {
                let (_, reset, close) = self.settings_geom();
                for i in 0..13usize {
                    if self.settings_row(i).contains(x, y) {
                        return Hit::Row(i);
                    }
                }
                if reset.contains(x, y) {
                    return Hit::OverlayBtn(0);
                }
                if close.contains(x, y) {
                    return Hit::Row(100);
                }
                return Hit::None;
            }
            Overlay::None => {}
        }
        self.ensure_layout();
        let l = &self.layout;
        let p = l.panel;
        let e = 6;
        if x < p.left || x >= p.right || y < p.top || y >= p.bottom {
            return Hit::Shadow;
        }
        if x < p.left + e && y < p.top + e {
            return Hit::Edge(HTTOPLEFT);
        }
        if x > p.right - e && y < p.top + e {
            return Hit::Edge(HTTOPRIGHT);
        }
        if x < p.left + e && y > p.bottom - e {
            return Hit::Edge(HTBOTTOMLEFT);
        }
        if x > p.right - e && y > p.bottom - e {
            return Hit::Edge(HTBOTTOMRIGHT);
        }
        if x < p.left + e {
            return Hit::Edge(HTLEFT);
        }
        if x > p.right - e {
            return Hit::Edge(HTRIGHT);
        }
        if y < p.top + e {
            return Hit::Edge(HTTOP);
        }
        if y > p.bottom - e {
            return Hit::Edge(HTBOTTOM);
        }
        if self.conf.show_title {
            if l.bclose.contains(x, y) {
                return Hit::Close;
            }
            if l.bmenu.contains(x, y) {
                return Hit::Menu;
            }
            if l.bsearch.contains(x, y) {
                return Hit::Search;
            }
            if l.title.contains(x, y) {
                return Hit::Title;
            }
        }
        if let Some(chip) = l.chip {
            if chip.contains(x, y) {
                return Hit::FilterChip;
            }
        }
        if self.conf.show_group {
            for (i, r) in l.gitems.iter().enumerate() {
                if r.contains(x, y) {
                    return Hit::Group(i);
                }
            }
            if l.group.contains(x, y) {
                return Hit::GroupBar;
            }
        }
        if l.body.contains(x, y) {
            for (i, r) in l.litems.iter().enumerate() {
                if r.contains(x, y) {
                    return Hit::Link(i);
                }
            }
            return Hit::Body;
        }
        Hit::None
    }

    // ---------------- 绘制 ----------------
    pub fn repaint(&mut self) {
        if self.conf.w != self.last_w || self.conf.h != self.last_h {
            let mut wr: RECT = RECT::new(0, 0, 0, 0);
            unsafe {
                GetWindowRect(self.hwnd, &mut wr);
                SetWindowPos(
                    self.hwnd,
                    std::ptr::null_mut(),
                    wr.left,
                    wr.top,
                    self.conf.w,
                    self.conf.h,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            self.last_w = self.conf.w;
            self.last_h = self.conf.h;
        }
        self.canvas.resize(self.conf.w, self.conf.h);
        self.draw();
        self.canvas.present(self.hwnd, self.conf.opacity);
    }

    /// 静态层缓存：阴影 + 面板 + 标题栏 + 分组栏底 + 内容区底
    /// 只在尺寸/配色/标题/日期变化时重绘，鼠标悬停时直接复用，避免每帧重画阴影
    fn ensure_bg(&mut self, l: &Layout, c: &Conf, date: &str) {
        let mut k: u64 = 0xcbf2_9ce4_8422_2325;
        let vals: [u64; 16] = [
            c.w as u64,
            c.h as u64,
            c.shadow as u64,
            c.show_title as u64,
            c.show_group as u64,
            c.show_date as u64,
            c.font_size as u64,
            c.group_w as u64,
            c.c_bg as u64,
            c.c_title as u64,
            c.c_title_text as u64,
            c.c_text as u64,
            c.c_border as u64,
            c.c_group as u64,
            c.c_accent as u64,
            c.c_hover as u64,
        ];
        for v in vals {
            k ^= v;
            k = k.wrapping_mul(0x1000_0000_01b3);
        }
        for b in c.title_text.as_bytes() {
            k ^= *b as u64;
            k = k.wrapping_mul(0x1000_0000_01b3);
        }
        for b in date.as_bytes() {
            k ^= *b as u64;
            k = k.wrapping_mul(0x1000_0000_01b3);
        }
        if k == self.bgkey && self.bg.w == c.w && self.bg.h == c.h {
            return;
        }
        self.bgkey = k;
        self.bg.resize(c.w, c.h);
        let radius = [0.0, 0.0, 0.0, 0.0];
        let fs = c.font_size;
        self.bg.clear(0);
        if c.shadow {
            let s = 12;
            for i in 0..8 {
                let a = (18 - i * 2).max(2) as u8;
                let off = (9 - i) as f32 * 0.8;
                self.bg.fill_round(
                    RECTF {
                        x: s as f32 - off,
                        y: s as f32 - off + 2.0,
                        w: l.panel.w() as f32 + off * 2.0,
                        h: l.panel.h() as f32 + off * 2.0,
                    },
                    radius,
                    argb(a, 0, 0, 0),
                );
            }
        }
        self.bg.fill_round(
            RECTF { x: l.panel.left as f32, y: l.panel.top as f32, w: l.panel.w() as f32, h: l.panel.h() as f32 },
            radius,
            c.c_bg,
        );

        // 标题栏
        if c.show_title {
            self.bg.fill_round(
                RECTF { x: l.title.left as f32, y: l.title.top as f32, w: l.title.w() as f32, h: l.title.h() as f32 },
                [0.0, 0.0, 0.0, 0.0],
                c.c_title,
            );
            let name = c.title_text.clone();
            let (nw, _) = self.bg.measure(&name, fs + 2, true, 600.0, false);
            let tr = RECTF {
                x: (l.title.left + 16) as f32,
                y: l.title.top as f32,
                w: (nw + 6.0).min((l.bsearch.left - l.title.left - 20) as f32),
                h: l.title.h() as f32,
            };
            self.bg.draw_text(&name, tr, fs + 2, true, c.c_title_text, ALIGN_NEAR, false, TRIM_ELLIPSIS_CHAR);
            if c.show_date && !date.is_empty() {
                let dx = l.title.left + 24 + nw as i32;
                let dr = RECTF {
                    x: dx as f32,
                    y: l.title.top as f32,
                    w: (l.bsearch.left - dx - 6).max(10) as f32,
                    h: l.title.h() as f32,
                };
                self.bg.draw_text(date, dr, (fs - 2).max(10), false, with_alpha(c.c_text, 150), ALIGN_NEAR, false, TRIM_ELLIPSIS_CHAR);
            }
            self.bg.line(
                l.title.left as f32 + 1.0,
                (l.title.bottom - 1) as f32 + 0.5,
                (l.title.right - 1) as f32,
                (l.title.bottom - 1) as f32 + 0.5,
                1.0,
                sep_color(c),
            );
        }

        // 分组栏底色 + 右边线
        if c.show_group {
            self.bg.fill_rect(l.group.left as f32, l.group.top as f32, l.group.w() as f32, l.group.h() as f32, c.c_group);
            let gx = (l.group.right - 1) as f32 + 0.5;
            self.bg.line(gx, l.group.top as f32, gx, l.group.bottom as f32, 1.0, sep_color(c));
        }

        // 内容区底色
        self.bg.fill_rect(l.body.left as f32, l.body.top as f32, l.body.w() as f32, l.body.h() as f32, c.c_bg);

        // 外边框：和内部分割线同一套推导，只是再深一档
        let bd = border_color(c);
        self.bg.stroke_round(
            RECTF {
                x: l.panel.left as f32 + 0.5,
                y: l.panel.top as f32 + 0.5,
                w: l.panel.w() as f32 - 1.0,
                h: l.panel.h() as f32 - 1.0,
            },
            radius,
            1.0,
            bd,
        );
    }

    /// 性能基准（设置环境变量 QL_BENCH=1 启动，结果写入 bench.txt 后退出）
    #[allow(dead_code)]
    pub fn test_open_settings(&mut self) {
        self.overlay = Overlay::Settings;
        self.repaint();
    }

    pub fn bench(&mut self) {
        let mut out = String::new();
        let t = std::time::Instant::now();
        let mut acc = 0u32;
        for i in 0..100_000i32 {
            if self.hit_at(i % 400, i % 700) != Hit::None {
                acc += 1;
            }
        }
        out.push_str(&format!(
            "hit_at x100000: {:.1} ms (non-none {})\n",
            t.elapsed().as_secs_f64() * 1000.0,
            acc
        ));

        let old_shadow = self.conf.shadow;
        for shadow in [false, true] {
            self.conf.shadow = shadow;
            self.bgkey = 0;
            self.repaint();
            let t = std::time::Instant::now();
            self.bgkey = 0;
            self.repaint();
            let rebuild = t.elapsed().as_secs_f64() * 1000.0;
            for _ in 0..10 {
                self.repaint();
            }
            let n = 100i32;
            let t = std::time::Instant::now();
            for i in 0..n {
                self.hover = if i % 2 == 0 { Hit::Link(0) } else { Hit::None };
                self.repaint();
            }
            let avg = t.elapsed().as_secs_f64() * 1000.0 / n as f64;
            out.push_str(&format!(
                "shadow={:<5} 静态层重建 {:.2} ms, 悬停重绘 {:.3} ms/帧\n",
                shadow, rebuild, avg
            ));
        }
        self.conf.shadow = old_shadow;
        self.hover = Hit::None;
        self.bgkey = 0;
        self.repaint();
        let _ = std::fs::write(self.dir.join("bench.txt"), out);
    }

    fn draw(&mut self) {
        self.ensure_layout();
        let l = self.layout.clone();
        let c = self.conf.clone();
        let date = if c.show_date { today_text() } else { String::new() };
        self.ensure_bg(&l, &c, &date);
        self.canvas.clear(0);
        self.canvas.blit(self.bg.image(), c.w as f32, c.h as f32);
        let fs = c.font_size;

        // 标题栏按钮（悬停态，逐帧）
        if c.show_title {
            self.draw_title_btn(l.bclose, BtnKind::Close, c.c_text);
            self.draw_title_btn(l.bmenu, BtnKind::Menu, c.c_text);
            if c.show_search {
                self.draw_title_btn(l.bsearch, BtnKind::Search, c.c_text);
            }
        }

        // 分组栏（底色已在静态层）
        if c.show_group {
            let groups: Vec<(u32, String)> = self.db.groups.iter().map(|g| (g.id, g.name.clone())).collect();
            for (i, (gid, name)) in groups.iter().enumerate() {
                if i >= l.gitems.len() {
                    break;
                }
                let r = l.gitems[i];
                let selected = *gid == c.cur_group;
                let hot = self.hover == Hit::Group(i);
                if selected || hot {
                    self.canvas.fill_round(
                        RECTF { x: r.left as f32, y: r.top as f32, w: r.w() as f32, h: r.h() as f32 },
                        [0.0, 0.0, 0.0, 0.0],
                        if selected { panel_hl(c.c_group) } else { c.c_hover },
                    );
                }
                if selected {
                    self.canvas.fill_rect(r.left as f32, (r.top + 6) as f32, 3.0, (r.h() - 12) as f32, c.c_accent);
                }
                let col = if selected { c.c_accent } else { c.c_text };
                let tr = RECTF { x: (r.left + 6) as f32, y: (r.top + 2) as f32, w: (r.w() - 10) as f32, h: (r.h() - 4) as f32 };
                self.canvas.draw_text(name, tr, fs, selected, col, ALIGN_CENTER, false, TRIM_ELLIPSIS_CHAR);
            }
        }

        // 内容（底色已在静态层）
        self.canvas.clip(l.body);
        let idxs = self.cur_links();
        let mut imgs: Vec<GpImage> = Vec::with_capacity(idxs.len());
        for &li in &idxs {
            let lk = &self.db.links[li];
            let src = self.icon_source(lk);
            let hint = if lk.icon.is_empty() || lk.icon.contains('*') { String::new() } else { lk.icon.clone() };
            imgs.push(self.icons.get(&src, &hint, c.icon_size));
        }
        let n = idxs.len().min(l.litems.len());
        let name_font = if c.view == 0 { (fs - 1).max(10) } else { fs };
        for i in 0..n {
            let li = idxs[i];
            let lk = self.db.links[li].clone();
            let r = l.litems[i];
            let hot = self.hover == Hit::Link(i);
            let selected = self.is_sel(lk.id);
            if selected {
                self.canvas.fill_round(
                    RECTF { x: r.left as f32 + 1.0, y: r.top as f32 + 1.0, w: r.w() as f32 - 2.0, h: r.h() as f32 - 2.0 },
                    [0.0, 0.0, 0.0, 0.0],
                    with_alpha(c.c_accent, 40),
                );
            }
            if c.view == 0 {
                if hot {
                    self.canvas.fill_round(
                        RECTF { x: r.left as f32 + 2.0, y: r.top as f32 + 2.0, w: r.w() as f32 - 4.0, h: r.h() as f32 - 4.0 },
                        [0.0, 0.0, 0.0, 0.0],
                        c.c_hover,
                    );
                }
                let ix = r.left + (r.w() - c.icon_size) / 2;
                let iy = r.top + 8;
                self.canvas.draw_image(imgs[i], ix as f32, iy as f32, c.icon_size as f32, c.icon_size as f32);
                let tr = RECTF {
                    x: (r.left + 4) as f32,
                    y: (iy + c.icon_size + 4) as f32,
                    w: (r.w() - 8) as f32,
                    h: (r.bottom - (iy + c.icon_size + 4) - 2) as f32,
                };
                self.canvas.draw_text(&lk.name, tr, name_font, false, c.c_text, ALIGN_CENTER, true, TRIM_ELLIPSIS_CHAR);
            } else {
                if hot {
                    self.canvas.fill_round(
                        RECTF { x: r.left as f32, y: r.top as f32 + 1.0, w: r.w() as f32, h: r.h() as f32 - 2.0 },
                        [0.0, 0.0, 0.0, 0.0],
                        c.c_hover,
                    );
                }
                let ix = r.left + 10;
                let iy = r.top + (r.h() - c.icon_size) / 2;
                self.canvas.draw_image(imgs[i], ix as f32, iy as f32, c.icon_size as f32, c.icon_size as f32);
                let tx = ix + c.icon_size + 10;
                let name_w = (r.right - tx - 90) as f32;
                let tr = RECTF { x: tx as f32, y: r.top as f32, w: name_w.max(20.0), h: r.h() as f32 };
                self.canvas.draw_text(&lk.name, tr, name_font, false, c.c_text, ALIGN_NEAR, false, TRIM_ELLIPSIS_CHAR);
                if !lk.remark.is_empty() {
                    let rr = RECTF { x: (r.right - 100) as f32, y: r.top as f32, w: 90.0, h: r.h() as f32 };
                    self.canvas.draw_text(&lk.remark, rr, (fs - 3).max(9), false, with_alpha(c.c_text, 140), ALIGN_FAR, false, TRIM_ELLIPSIS_CHAR);
                }
            }
            if selected {
                self.canvas.stroke_round(
                    RECTF { x: r.left as f32 + 1.0, y: r.top as f32 + 1.0, w: r.w() as f32 - 2.0, h: r.h() as f32 - 2.0 },
                    [0.0, 0.0, 0.0, 0.0],
                    2.0,
                    c.c_accent,
                );
            }
        }
        self.canvas.unclip();
        if idxs.is_empty() {
            let tr = RECTF { x: l.body.left as f32, y: l.body.top as f32, w: l.body.w() as f32, h: l.body.h() as f32 };
            self.canvas.draw_text("右键添加程序 / 文件夹 / 网址，或把文件拖进来", tr, fs, false, with_alpha(c.c_text, 110), ALIGN_CENTER, false, TRIM_NONE);
        }
        if let Some(chip) = self.filter_chip_rect() {
            let hot = self.hover == Hit::FilterChip;
            self.canvas.fill_round(
                RECTF { x: chip.left as f32, y: chip.top as f32, w: chip.w() as f32, h: chip.h() as f32 },
                [0.0, 0.0, 0.0, 0.0],
                if hot { c.c_accent } else { with_alpha(c.c_accent, 210) },
            );
            let mut txt = self.filter.clone();
            txt.push_str("  ×");
            let tr = RECTF { x: (chip.left + 8) as f32, y: chip.top as f32, w: (chip.w() - 14) as f32, h: chip.h() as f32 };
            self.canvas.draw_text(&txt, tr, (fs - 2).max(10), false, rgb(255, 255, 255), ALIGN_CENTER, false, TRIM_ELLIPSIS_CHAR);
        }

        match self.overlay {
            Overlay::Settings => self.draw_settings(),
            Overlay::Input => self.draw_input(),
            Overlay::Hotkey => self.draw_hotkey(),
            Overlay::None => {}
        }
    }

    fn draw_title_btn(&mut self, r: RECT, kind: BtnKind, col: u32) {
        let hot = match kind {
            BtnKind::Close => self.hover == Hit::Close,
            BtnKind::Menu => self.hover == Hit::Menu,
            BtnKind::Search => self.hover == Hit::Search,
        };
        if hot {
            let bg = if matches!(kind, BtnKind::Close) { argb(235, 222, 80, 80) } else { self.conf.c_hover };
            self.canvas.fill_round(
                RECTF { x: r.left as f32 + 4.0, y: r.top as f32 + 4.0, w: r.w() as f32 - 8.0, h: r.h() as f32 - 8.0 },
                [0.0, 0.0, 0.0, 0.0],
                bg,
            );
        }
        let cx = (r.left + r.right) as f32 / 2.0;
        let cy = (r.top + r.bottom) as f32 / 2.0;
        let k = if hot && matches!(kind, BtnKind::Close) { rgb(255, 255, 255) } else { col };
        match kind {
            BtnKind::Close => {
                self.canvas.line(cx - 5.0, cy - 5.0, cx + 5.0, cy + 5.0, 1.6, k);
                self.canvas.line(cx + 5.0, cy - 5.0, cx - 5.0, cy + 5.0, 1.6, k);
            }
            BtnKind::Menu => {
                for i in -1..=1 {
                    let y = cy + i as f32 * 4.5;
                    self.canvas.line(cx - 6.0, y, cx + 6.0, y, 1.6, k);
                }
            }
            BtnKind::Search => {
                self.canvas.stroke_round(RECTF { x: cx - 6.5, y: cy - 6.5, w: 9.0, h: 9.0 }, [0.0, 0.0, 0.0, 0.0], 1.5, k);
                self.canvas.line(cx + 1.5, cy + 1.5, cx + 6.0, cy + 6.0, 1.6, k);
            }
        }
    }

    /// 记录一次快捷键按键
    fn capture_hotkey(&mut self, vk: u32) {
        if vk == 27 {
            self.overlay = Overlay::None;
            self.repaint();
            return;
        }
        if vk == 46 {
            self.conf.hk_mod = 0;
            self.conf.hk_vk = 0;
            self.save_conf();
            self.register_hotkey();
            self.overlay = Overlay::None;
            self.repaint();
            return;
        }
        // 只按修饰键本身不算
        if matches!(vk, 0x10 | 0x11 | 0x12 | 0x5B | 0x5C | 0xA0..=0xA5) {
            return;
        }
        let st = |k: i32| unsafe { GetKeyState(k) } < 0;
        let mut mods = 0u32;
        if st(0x11) {
            mods |= MOD_CONTROL;
        }
        if st(0x12) {
            mods |= MOD_ALT;
        }
        if st(0x10) {
            mods |= MOD_SHIFT;
        }
        if st(0x5B) || st(0x5C) {
            mods |= MOD_WIN;
        }
        if mods == 0 {
            // 裸键注册成全局热键会抢走全系统的按键，要求带修饰键
            self.repaint();
            return;
        }
        self.conf.hk_mod = mods;
        self.conf.hk_vk = vk;
        self.save_conf();
        if !self.register_hotkey() {
            let t = wide(&format!("快捷键 {} 已被其他程序占用", hotkey_text(mods, vk)));            let cap = wide("提示");
            unsafe {
                MessageBoxW(self.hwnd, t.as_ptr(), cap.as_ptr(), MB_OK | MB_ICONWARNING);
            }
        }
        self.overlay = Overlay::None;
        self.repaint();
    }

    /// 注册 / 重新注册全局呼出快捷键；返回是否成功
    pub fn register_hotkey(&self) -> bool {
        unsafe {
            UnregisterHotKey(self.hwnd, 1);
            if self.conf.hk_vk != 0 && self.conf.hk_mod != 0 {
                return RegisterHotKey(self.hwnd, 1, self.conf.hk_mod | MOD_NOREPEAT, self.conf.hk_vk) != 0;
            }
            true
        }
    }

    /// 覆盖层的压暗层：只压暗窗口内容区，不含四周的阴影留白，
    /// 否则阴影会被一起涂黑，看起来像"阴影突然变大"。
    fn dim_layer(&mut self) {
        self.ensure_layout();
        let p = self.layout.panel;
        self.canvas.fill_rect(p.left as f32, p.top as f32, p.w() as f32, p.h() as f32, argb(96, 0, 0, 0));
    }

    fn draw_hotkey(&mut self) {
        let c = self.conf.clone();
        let w = (c.w - 60).clamp(280, 380);
        let h = 132;
        let x = (c.w - w) / 2;
        let y = (c.h - h) / 2;
        let panel = RECT::new(x, y, x + w, y + h);
        self.dim_layer();

        self.canvas.fill_round(
            RECTF { x: panel.left as f32, y: panel.top as f32, w: panel.w() as f32, h: panel.h() as f32 },
            [0.0, 0.0, 0.0, 0.0],
            c.c_bg,
        );
        self.canvas.stroke_round(
            RECTF { x: panel.left as f32, y: panel.top as f32, w: panel.w() as f32, h: panel.h() as f32 },
            [0.0, 0.0, 0.0, 0.0],
            1.0,
            c.c_accent,
        );
        let t1 = RECTF { x: (panel.left + 20) as f32, y: (panel.top + 18) as f32, w: (panel.w() - 40) as f32, h: 30.0 };
        self.canvas.draw_text("请按下呼出快捷键", t1, 15, true, c.c_text, ALIGN_CENTER, false, TRIM_NONE);
        let t2 = RECTF { x: (panel.left + 20) as f32, y: (panel.top + 54) as f32, w: (panel.w() - 40) as f32, h: 26.0 };
        self.canvas.draw_text(
            "需含 Ctrl / Alt / Shift / Win，Esc 取消，Delete 清除",
            t2,
            12,
            false,
            with_alpha(c.c_text, 150),
            ALIGN_CENTER,
            false,
            TRIM_NONE,
        );
        let t3 = RECTF { x: (panel.left + 20) as f32, y: (panel.top + 86) as f32, w: (panel.w() - 40) as f32, h: 30.0 };
        self.canvas.draw_text(&hotkey_text(c.hk_mod, c.hk_vk), t3, 14, false, c.c_accent, ALIGN_CENTER, false, TRIM_NONE);
    }

    fn draw_settings(&mut self) {
        let c = self.conf.clone();
        let (panel, reset, close) = self.settings_geom();
        let fs = c.font_size.min(14);
        self.dim_layer();

        self.canvas.fill_round(
            RECTF { x: panel.left as f32, y: panel.top as f32, w: panel.w() as f32, h: panel.h() as f32 },
            [0.0, 0.0, 0.0, 0.0],
            c.c_bg,
        );
        self.canvas.stroke_round(
            RECTF { x: panel.left as f32, y: panel.top as f32, w: panel.w() as f32, h: panel.h() as f32 },
            [0.0, 0.0, 0.0, 0.0],
            1.0,
            c.c_border,
        );
        let hr = RECTF { x: (panel.left + 16) as f32, y: panel.top as f32, w: (panel.w() - 32) as f32, h: 48.0 };
        self.canvas.draw_text("设置", hr, 15, true, c.c_text, ALIGN_NEAR, false, TRIM_NONE);

        let labels = [
            "程序名称", "主题色", "不透明度", "字体大小", "图标大小", "视图方式",
            "显示日期", "显示标题栏", "窗口置顶", "窗口阴影", "开机启动", "呼出快捷键",
            "Ctrl+滚轮可直接缩放图标",
        ];
        for i in 0..labels.len() {
            let r = self.settings_row(i);
            let hot = self.hover == Hit::Row(i);
            if hot && i < 12 {
                self.canvas.fill_rect(r.left as f32, r.top as f32, r.w() as f32, r.h() as f32, with_alpha(c.c_hover, 150));
            }
            let dim = if i == 12 { with_alpha(c.c_text, 110) } else { c.c_text };
            let tr = RECTF { x: (r.left + 16) as f32, y: r.top as f32, w: (r.w() - 200) as f32, h: r.h() as f32 };
            self.canvas.draw_text(labels[i], tr, fs, false, dim, ALIGN_NEAR, false, TRIM_NONE);
            if i >= 12 {
                continue;
            }
            let cw = 134;
            let cx = r.right - 16 - cw;
            match i {
                0 => {
                    let s = c.title_text.clone();
                    let vr = RECTF { x: cx as f32, y: r.top as f32, w: cw as f32, h: r.h() as f32 };
                    self.canvas.draw_text(&s, vr, fs, false, with_alpha(c.c_text, 190), ALIGN_FAR, false, TRIM_ELLIPSIS_CHAR);
                }
                1 => {
                    let sw = RECTF { x: (cx + 14) as f32, y: (r.top + 7) as f32, w: 90.0, h: 20.0 };
                    self.canvas.fill_round(sw, [0.0, 0.0, 0.0, 0.0], c.c_base | 0xFF00_0000);
                    self.canvas.stroke_round(sw, [0.0, 0.0, 0.0, 0.0], 1.0, with_alpha(c.c_border, 220));
                }
                2 | 3 | 4 => {
                    let minus = RECT::new(cx, r.top + 6, cx + 28, r.top + 28);
                    let plus = RECT::new(r.right - 44, r.top + 6, r.right - 16, r.top + 28);
                    for (b, sym) in [(minus, "－"), (plus, "＋")] {
                        self.canvas.fill_round(
                            RECTF { x: b.left as f32, y: b.top as f32, w: b.w() as f32, h: b.h() as f32 },
                            [0.0, 0.0, 0.0, 0.0],
                            with_alpha(c.c_hover, 210),
                        );
                        let br = RECTF { x: b.left as f32, y: b.top as f32, w: b.w() as f32, h: b.h() as f32 };
                        self.canvas.draw_text(sym, br, fs, false, c.c_text, ALIGN_CENTER, false, TRIM_NONE);
                    }
                    let v = match i {
                        2 => format!("{}", c.opacity),
                        3 => format!("{}", c.font_size),
                        _ => format!("{}", c.icon_size),
                    };
                    let vr = RECTF { x: (minus.right + 4) as f32, y: r.top as f32, w: (plus.left - minus.right - 8) as f32, h: r.h() as f32 };
                    self.canvas.draw_text(&v, vr, fs, false, c.c_text, ALIGN_CENTER, false, TRIM_NONE);
                }
                5 => {
                    let tile = RECT::new(cx + 4, r.top + 6, cx + 68, r.top + 28);
                    let list = RECT::new(cx + 72, r.top + 6, cx + 130, r.top + 28);
                    for (b, name, on) in [(tile, "磁贴", c.view == 0), (list, "列表", c.view == 1)] {
                        self.canvas.fill_round(
                            RECTF { x: b.left as f32, y: b.top as f32, w: b.w() as f32, h: b.h() as f32 },
                            [0.0, 0.0, 0.0, 0.0],
                            if on { c.c_accent } else { with_alpha(c.c_hover, 200) },
                        );
                        let br = RECTF { x: b.left as f32, y: b.top as f32, w: b.w() as f32, h: b.h() as f32 };
                        self.canvas.draw_text(name, br, fs - 1, false, if on { rgb(255, 255, 255) } else { c.c_text }, ALIGN_CENTER, false, TRIM_NONE);
                    }
                }
                11 => {
                    let s = hotkey_text(c.hk_mod, c.hk_vk);
                    let vr = RECTF { x: cx as f32, y: r.top as f32, w: cw as f32, h: r.h() as f32 };
                    self.canvas.draw_text(&s, vr, fs, false, with_alpha(c.c_text, 190), ALIGN_FAR, false, TRIM_ELLIPSIS_CHAR);
                }
                _ => {
                    let on = match i {
                        6 => c.show_date,
                        7 => c.show_title,
                        8 => c.topmost,
                        10 => autostart_get(),
                        _ => c.shadow,
                    };
                    let track = RECT::new(r.right - 60, r.top + 6, r.right - 16, r.top + 28);
                    self.canvas.fill_round(
                        RECTF { x: track.left as f32, y: track.top as f32, w: track.w() as f32, h: track.h() as f32 },
                        [0.0, 0.0, 0.0, 0.0],
                        if on { c.c_accent } else { with_alpha(c.c_text, 70) },
                    );
                    let kx = if on { track.right - 20 } else { track.left + 2 };
                    self.canvas.fill_round(
                        RECTF { x: kx as f32, y: (track.top + 2) as f32, w: 16.0, h: 16.0 },
                        [0.0, 0.0, 0.0, 0.0],
                        rgb(255, 255, 255),
                    );
                }
            }
        }
        for (r, label, primary, hot) in [
            (reset, "恢复默认", false, self.hover == Hit::OverlayBtn(0)),
            (close, "关闭", true, self.hover == Hit::Row(100)),
        ] {
            let bg = if primary {
                if hot { mix(c.c_accent, rgb(255, 255, 255), 0.2) } else { c.c_accent }
            } else if hot {
                c.c_hover
            } else {
                with_alpha(c.c_hover, 160)
            };
            self.canvas.fill_round(
                RECTF { x: r.left as f32, y: r.top as f32, w: r.w() as f32, h: r.h() as f32 },
                [0.0, 0.0, 0.0, 0.0],
                bg,
            );
            let tr = RECTF { x: r.left as f32, y: r.top as f32, w: r.w() as f32, h: r.h() as f32 };
            let col = if primary { rgb(255, 255, 255) } else { c.c_text };
            self.canvas.draw_text(label, tr, fs - 1, false, col, ALIGN_CENTER, false, TRIM_NONE);
        }
    }

    fn draw_input(&mut self) {
        let c = self.conf.clone();
        let (panel, field, ok, cancel) = self.input_geom();
        self.dim_layer();

        self.canvas.fill_round(
            RECTF { x: panel.left as f32, y: panel.top as f32, w: panel.w() as f32, h: panel.h() as f32 },
            [0.0, 0.0, 0.0, 0.0],
            c.c_bg,
        );
        self.canvas.stroke_round(
            RECTF { x: panel.left as f32, y: panel.top as f32, w: panel.w() as f32, h: panel.h() as f32 },
            [0.0, 0.0, 0.0, 0.0],
            1.0,
            c.c_border,
        );
        let tr = RECTF { x: (panel.left + 20) as f32, y: (panel.top + 12) as f32, w: (panel.w() - 40) as f32, h: 32.0 };
        let title = self.input_title.clone();
        self.canvas.draw_text(&title, tr, 14, true, c.c_text, ALIGN_NEAR, false, TRIM_NONE);
        self.canvas.fill_rect(field.left as f32, field.top as f32, field.w() as f32, field.h() as f32, mix(c.c_bg, rgb(255, 255, 255), 0.6));
        self.canvas.stroke_round(
            RECTF { x: field.left as f32, y: field.top as f32, w: field.w() as f32, h: field.h() as f32 },
            [0.0, 0.0, 0.0, 0.0],
            1.0,
            c.c_accent,
        );
        let txt = self.input_text.clone();
        let tx = (field.left + 8) as f32;
        let tarea = RECTF { x: tx, y: field.top as f32, w: (field.w() - 16) as f32, h: field.h() as f32 };
        self.canvas.draw_text(&txt, tarea, 13, false, c.c_text, ALIGN_NEAR, false, TRIM_NONE);
        let (tw, _) = self.canvas.measure(&txt, 13, false, 4000.0, false);
        let cx = (tx + tw + 1.0).min((field.right - 4) as f32);
        self.canvas.line(cx, (field.top + 6) as f32, cx, (field.bottom - 6) as f32, 1.2, c.c_accent);
        for (r, label, primary, h) in [
            (ok, "确定", true, self.hover == Hit::InputOk),
            (cancel, "取消", false, self.hover == Hit::InputCancel),
        ] {
            let bg = if primary {
                if h { mix(c.c_accent, rgb(255, 255, 255), 0.2) } else { c.c_accent }
            } else if h {
                c.c_hover
            } else {
                with_alpha(c.c_hover, 180)
            };
            self.canvas.fill_round(
                RECTF { x: r.left as f32, y: r.top as f32, w: r.w() as f32, h: r.h() as f32 },
                [0.0, 0.0, 0.0, 0.0],
                bg,
            );
            let tr = RECTF { x: r.left as f32, y: r.top as f32, w: r.w() as f32, h: r.h() as f32 };
            self.canvas.draw_text(label, tr, 13, false, if primary { rgb(255, 255, 255) } else { c.c_text }, ALIGN_CENTER, false, TRIM_NONE);
        }
    }

    // ---------------- 事件 ----------------
    pub fn on_message(&mut self, msg: u32, w: WPARAM, l: LPARAM) -> Option<LRESULT> {
        match msg {
            WM_MOUSEMOVE => {
                let x = (l & 0xffff) as i16 as i32;
                let y = ((l >> 16) & 0xffff) as i16 as i32;
                if let Some((sx, sy, ox, oy)) = self.drag_start {
                    let mut pt = POINT { x: sx, y: sy };
                    unsafe {
                        GetCursorPos(&mut pt);
                    }
                    let nx = ox + pt.x - sx;
                    let ny = oy + pt.y - sy;
                    unsafe {
                        SetWindowPos(self.hwnd, std::ptr::null_mut(), nx, ny, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
                    }
                    self.conf.x = nx;
                    self.conf.y = ny;
                    return Some(0);
                }
                if let Some(dir) = self.resize_dir {
                    self.do_resize(dir);
                    return Some(0);
                }
                let h = self.hit_at(x, y);
                if h != self.hover {
                    self.hover = h;
                    self.repaint();
                }
                Some(0)
            }
            WM_LBUTTONDOWN => {
                let x = (l & 0xffff) as i16 as i32;
                let y = ((l >> 16) & 0xffff) as i16 as i32;
                let h = self.hit_at(x, y);
                self.pressed = Some(h);
                match h {
                    Hit::Title | Hit::Body | Hit::GroupBar => self.start_drag(),
                    Hit::Edge(dir) => {
                        self.resize_dir = Some(dir);
                        unsafe {
                            SetCapture(self.hwnd);
                        }
                    }
                    _ => unsafe {
                        SetCapture(self.hwnd);
                    },
                }
                Some(0)
            }
            WM_LBUTTONUP => {
                let x = (l & 0xffff) as i16 as i32;
                let y = ((l >> 16) & 0xffff) as i16 as i32;
                unsafe {
                    ReleaseCapture();
                }
                let was_drag = self.drag_start.take().is_some();
                let was_resize = self.resize_dir.take().is_some();
                let h = self.hit_at(x, y);
                let pressed = self.pressed.take();
                if was_resize || was_drag {
                    self.save_conf();
                    return Some(0);
                }
                if let Some(p) = pressed {
                    if p == h {
                        self.activate(h);
                    }
                }
                Some(0)
            }
            WM_RBUTTONUP => {
                let x = (l & 0xffff) as i16 as i32;
                let y = ((l >> 16) & 0xffff) as i16 as i32;
                if self.overlay == Overlay::None {
                    let h = self.hit_at(x, y);
                    self.context_menu(h, x, y);
                }
                Some(0)
            }
            WM_MOUSEWHEEL => {
                let delta = ((w >> 16) as i16) as i32;
                let ctrl = unsafe { GetKeyState(0x11) } < 0;
                if ctrl {
                    const SIZES: [i32; 9] = [16, 24, 32, 40, 48, 56, 64, 80, 96];
                    let pos = SIZES.iter().position(|&s| s >= self.conf.icon_size).unwrap_or(4);
                    let np = if delta > 0 { (pos + 1).min(SIZES.len() - 1) } else { pos.saturating_sub(1) };
                    self.conf.icon_size = SIZES[np];
                    self.save_conf();
                } else {
                    self.scroll += if delta > 0 { -60 } else { 60 };
                    self.clamp_scroll();
                }
                self.repaint();
                Some(0)
            }
            WM_SETCURSOR => {
                let mut pt = POINT { x: 0, y: 0 };
                unsafe {
                    GetCursorPos(&mut pt);
                    ScreenToClient(self.hwnd, &mut pt);
                }
                let h = self.hit_at(pt.x, pt.y);
                let cur = match h {
                    Hit::Edge(HTLEFT) | Hit::Edge(HTRIGHT) => IDC_SIZEWE,
                    Hit::Edge(HTTOP) | Hit::Edge(HTBOTTOM) => IDC_SIZENS,
                    Hit::Edge(HTTOPLEFT) | Hit::Edge(HTBOTTOMRIGHT) => IDC_SIZENWSE,
                    Hit::Edge(_) => IDC_SIZENESW,
                    Hit::Link(_) | Hit::Group(_) | Hit::Close | Hit::Menu | Hit::Search | Hit::Row(_) | Hit::InputOk | Hit::InputCancel | Hit::FilterChip | Hit::OverlayBtn(_) => IDC_HAND,
                    Hit::InputField => IDC_IBEAM,
                    _ => IDC_ARROW,
                };
                unsafe {
                    let c = LoadCursorW(std::ptr::null_mut(), int_resource(cur));
                    SetCursor(c);
                }
                Some(1)
            }
            WM_HOTKEY => {
                if w == 1 {
                    self.toggle_show();
                }
                Some(0)
            }
            WM_DROPFILES => {
                self.drop_files(w as HANDLE);
                Some(0)
            }
            drop::WM_DROP_ITEMS => {
                let payload = unsafe { Box::from_raw(l as *mut drop::DropPayload) };
                let mut added = false;
                for it in payload.items {
                    match it {
                        drop::DropItem::Path(p) => {
                            self.add_path(&p);
                            added = true;
                        }
                        drop::DropItem::Sys(key) => {
                            let name = sys_item(key).map(|s| s.name).unwrap_or(key);
                            self.add_link(name, KIND_SYS, key);
                            added = true;
                        }
                    }
                }
                if added {
                    self.db.save(&self.dir);
                    self.repaint();
                }
                Some(0)
            }
            WM_TRAY => {
                let ev = (l & 0xffff) as u32;
                if ev == WM_LBUTTONUP || ev == WM_LBUTTONDBLCLK {
                    self.toggle_show();
                } else if ev == WM_RBUTTONUP {
                    self.tray_menu();
                }
                Some(0)
            }
            WM_INPUT_OK => {
                self.confirm_input();
                Some(0)
            }
            WM_INPUT_CANCEL => {
                self.cancel_input();
                Some(0)
            }
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                if self.overlay == Overlay::Hotkey {
                    self.capture_hotkey(w as u32);
                    return Some(0);
                }
                if self.overlay == Overlay::Input {
                    let ctrl = unsafe { GetKeyState(0x11) } < 0;
                    match w {
                        27 => self.cancel_input(),
                        13 => self.confirm_input(),
                        8 => {
                            self.input_backspace();
                            self.repaint();
                        }
                        46 => {
                            if self.input_caret < self.input_text.len() {
                                let save = self.input_caret;
                                self.input_right();
                                self.input_backspace();
                                self.input_caret = save;
                                self.repaint();
                            }
                        }
                        37 => {
                            self.input_left();
                            self.repaint();
                        }
                        39 => {
                            self.input_right();
                            self.repaint();
                        }
                        36 => {
                            self.input_caret = 0;
                            self.repaint();
                        }
                        35 => {
                            self.input_caret = self.input_text.len();
                            self.repaint();
                        }
                        86 if ctrl => {
                            self.paste_clipboard();
                            self.repaint();
                        }
                        _ => {}
                    }
                    return Some(0);
                }
                if self.overlay == Overlay::Settings && w == 27 {
                    self.overlay = Overlay::None;
                    self.repaint();
                    return Some(0);
                }
                if w == 27 {
                    if !self.filter.is_empty() {
                        self.filter.clear();
                        self.repaint();
                    } else {
                        self.hide();
                    }
                }
                Some(0)
            }
            WM_CHAR => {
                if self.overlay == Overlay::Input {
                    if let Some(ch) = char::from_u32(w as u32) {
                        if ch >= ' ' && ch != '\u{7f}' {
                            let mut b = [0u8; 4];
                            let s = ch.encode_utf8(&mut b).to_string();
                            self.input_insert(&s);
                            self.repaint();
                        }
                    }
                    return Some(0);
                }
                Some(0)
            }
            WM_IME_CHAR => {
                if self.overlay == Overlay::Input {
                    if let Some(ch) = char::from_u32(w as u32) {
                        if ch >= ' ' {
                            let mut b = [0u8; 4];
                            let s = ch.encode_utf8(&mut b).to_string();
                            self.input_insert(&s);
                            self.repaint();
                        }
                    }
                    return Some(0);
                }
                None
            }
            WM_PAINT => {
                self.repaint();
                Some(0)
            }
            _ => None,
        }
    }

    fn start_drag(&mut self) {
        let mut pt = POINT { x: 0, y: 0 };
        let mut wr: RECT = RECT::new(0, 0, 0, 0);
        unsafe {
            GetCursorPos(&mut pt);
            GetWindowRect(self.hwnd, &mut wr);
        }
        self.drag_start = Some((pt.x, pt.y, wr.left, wr.top));
        unsafe {
            SetCapture(self.hwnd);
        }
    }

    fn do_resize(&mut self, dir: i32) {
        let mut pt = POINT { x: 0, y: 0 };
        unsafe {
            GetCursorPos(&mut pt);
        }
        let mut wr: RECT = RECT::new(0, 0, 0, 0);
        unsafe {
            GetWindowRect(self.hwnd, &mut wr);
        }
        let mut x = wr.left;
        let mut y = wr.top;
        let mut cw = wr.w();
        let mut ch = wr.h();
        let min_w = 280;
        let min_h = 220;
        match dir {
            HTLEFT => {
                cw = (wr.right - pt.x).max(min_w);
                x = wr.right - cw;
            }
            HTRIGHT => cw = (pt.x - wr.left).max(min_w),
            HTTOP => {
                ch = (wr.bottom - pt.y).max(min_h);
                y = wr.bottom - ch;
            }
            HTBOTTOM => ch = (pt.y - wr.top).max(min_h),
            HTTOPLEFT => {
                cw = (wr.right - pt.x).max(min_w);
                x = wr.right - cw;
                ch = (wr.bottom - pt.y).max(min_h);
                y = wr.bottom - ch;
            }
            HTTOPRIGHT => {
                cw = (pt.x - wr.left).max(min_w);
                ch = (wr.bottom - pt.y).max(min_h);
                y = wr.bottom - ch;
            }
            HTBOTTOMLEFT => {
                cw = (wr.right - pt.x).max(min_w);
                x = wr.right - cw;
                ch = (pt.y - wr.top).max(min_h);
            }
            HTBOTTOMRIGHT => {
                cw = (pt.x - wr.left).max(min_w);
                ch = (pt.y - wr.top).max(min_h);
            }
            _ => {}
        }
        self.conf.w = cw;
        self.conf.h = ch;
        self.conf.x = x;
        self.conf.y = y;
        self.last_w = cw;
        self.last_h = ch;
        unsafe {
            SetWindowPos(self.hwnd, std::ptr::null_mut(), x, y, cw, ch, SWP_NOZORDER | SWP_NOACTIVATE);
        }
        self.repaint();
    }

    fn activate(&mut self, h: Hit) {
        match h {
            Hit::Close => self.hide(),
            Hit::Menu => self.main_menu(),
            Hit::Search => self.open_search(),
            Hit::FilterChip => {
                self.filter.clear();
                self.repaint();
            }
            Hit::Group(i) => {
                if let Some(g) = self.db.groups.get(i) {
                    self.conf.cur_group = g.id;
                    self.conf.cur_tag = self.db.ensure_tag(g.id);
                    self.scroll = 0;
                    self.filter.clear();
                    self.sel.clear();
                    self.save_conf();
                    self.repaint();
                }
            }
            Hit::Link(i) => {
                let links = self.cur_links();
                if let Some(&li) = links.get(i) {
                    let id = self.db.links[li].id;
                    let ctrl = unsafe { GetKeyState(0x11) } < 0;
                    if ctrl {
                        self.sel_toggle(id);
                        self.repaint();
                    } else {
                        // 普通点击：清空多选，只启动这一个
                        if !self.sel.is_empty() {
                            self.sel.clear();
                            self.repaint();
                        }
                        self.launch(li, false);
                    }
                }
            }
            Hit::Body => {
                if !self.sel.is_empty() {
                    self.sel.clear();
                    self.repaint();
                }
            }
            Hit::Row(100) => {
                self.overlay = Overlay::None;
                self.repaint();
            }
            Hit::Row(i) => self.on_settings_row(i),
            Hit::OverlayBtn(0) => {
                let def = Conf::default();
                self.conf.c_base = def.c_base;
                self.conf.apply_base();
                self.conf.opacity = def.opacity;
                self.conf.font_size = def.font_size;
                self.conf.icon_size = def.icon_size;
                self.conf.view = def.view;
                self.conf.show_date = def.show_date;
                self.conf.show_title = def.show_title;
                self.conf.topmost = def.topmost;
                self.conf.shadow = def.shadow;
                self.conf.title_text = def.title_text;
                self.save_conf();
                self.repaint();
            }
            Hit::InputOk => self.confirm_input(),
            Hit::InputCancel => self.cancel_input(),
            _ => {}
        }
    }

    fn on_settings_row(&mut self, i: usize) {
        if i >= 12 {
            return;
        }
        let mut pt = POINT { x: 0, y: 0 };
        unsafe {
            GetCursorPos(&mut pt);
            ScreenToClient(self.hwnd, &mut pt);
        }
        let r = self.settings_row(i);
        let cx = r.right - 16 - 134;
        match i {
            0 => {
                let cur = self.conf.title_text.clone();
                self.open_input_pre("程序名称", InputAction::SetTitle, &cur);
                return;
            }
            1 => {
                let mut cust: [COLORREF; 16] = [0; 16];
                let mut cc = CHOOSECOLORW {
                    lStructSize: std::mem::size_of::<CHOOSECOLORW>() as u32,
                    hwndOwner: self.hwnd,
                    hInstance: std::ptr::null_mut(),
                    rgbResult: argb_to_colorref(self.conf.c_base),
                    lpCustColors: cust.as_mut_ptr(),
                    Flags: CC_RGBINIT | CC_FULLOPEN | CC_ANYCOLOR,
                    ..Default::default()
                };
                if unsafe { ChooseColorW(&mut cc) } != 0 {
                    self.conf.c_base = colorref_to_argb(cc.rgbResult);
                    self.conf.apply_base();
                }
            }
            2 => {
                if pt.x < cx + 28 {
                    self.conf.opacity = (self.conf.opacity as i32 - 20).max(60) as u8;
                } else if pt.x > r.right - 44 {
                    self.conf.opacity = (self.conf.opacity as i32 + 20).min(255) as u8;
                }
            }
            3 => {
                if pt.x < cx + 28 {
                    self.conf.font_size = (self.conf.font_size - 1).clamp(11, 20);
                } else if pt.x > r.right - 44 {
                    self.conf.font_size = (self.conf.font_size + 1).clamp(11, 20);
                }
            }
            4 => {
                const SIZES: [i32; 9] = [16, 24, 32, 40, 48, 56, 64, 80, 96];
                let pos = SIZES.iter().position(|&s| s >= self.conf.icon_size).unwrap_or(4);
                if pt.x < cx + 28 {
                    self.conf.icon_size = SIZES[pos.saturating_sub(1)];
                } else if pt.x > r.right - 44 {
                    self.conf.icon_size = SIZES[(pos + 1).min(SIZES.len() - 1)];
                }
            }
            5 => {
                self.conf.view = if pt.x < cx + 70 { 0 } else { 1 };
            }
            6 => self.conf.show_date = !self.conf.show_date,
            7 => self.conf.show_title = !self.conf.show_title,
            8 => {
                self.conf.topmost = !self.conf.topmost;
                let ex = WS_EX_LAYERED | WS_EX_TOOLWINDOW | if self.conf.topmost { WS_EX_TOPMOST } else { 0 };
                unsafe {
                    SetWindowLongPtrW(self.hwnd, GWL_EXSTYLE, ex as isize);
                    let after: HWND = if self.conf.topmost { (-1isize) as HWND } else { (-2isize) as HWND };
                    SetWindowPos(self.hwnd, after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
                }
            }
            9 => self.conf.shadow = !self.conf.shadow,
            10 => {
                let on = autostart_get();
                autostart_set(!on);
            }
            11 => {
                self.overlay = Overlay::Hotkey;
                unsafe {
                    SetForegroundWindow(self.hwnd);
                }
            }
            _ => {}
        }
        self.save_conf();
        self.repaint();
    }

    // ---------------- 输入 ----------------
    fn open_search(&mut self) {
        let cur = self.filter.clone();
        self.open_input_pre("搜索（当前分组内筛选）", InputAction::Search, &cur);
    }

    fn open_input(&mut self, title: &str, action: InputAction) {
        self.open_input_pre(title, action, "");
    }

    fn open_input_pre(&mut self, title: &str, action: InputAction, text: &str) {
        self.input_title = title.to_string();
        self.input_action = action;
        self.input_text = text.to_string();
        self.input_caret = self.input_text.len();
        self.overlay = Overlay::Input;
        unsafe {
            SetForegroundWindow(self.hwnd);
            SetFocus(self.hwnd);
        }
        self.repaint();
    }

    fn input_insert(&mut self, s: &str) {
        if self.input_caret > self.input_text.len() || !self.input_text.is_char_boundary(self.input_caret) {
            self.input_caret = self.input_text.len();
        }
        self.input_text.insert_str(self.input_caret, s);
        self.input_caret += s.len();
    }

    fn input_backspace(&mut self) {
        if self.input_caret == 0 {
            return;
        }
        let mut i = self.input_caret - 1;
        while i > 0 && !self.input_text.is_char_boundary(i) {
            i -= 1;
        }
        self.input_text.replace_range(i..self.input_caret, "");
        self.input_caret = i;
    }

    fn input_left(&mut self) {
        if self.input_caret == 0 {
            return;
        }
        let mut i = self.input_caret - 1;
        while i > 0 && !self.input_text.is_char_boundary(i) {
            i -= 1;
        }
        self.input_caret = i;
    }

    fn input_right(&mut self) {
        if self.input_caret >= self.input_text.len() {
            return;
        }
        let mut i = self.input_caret + 1;
        while i < self.input_text.len() && !self.input_text.is_char_boundary(i) {
            i += 1;
        }
        self.input_caret = i;
    }

    fn paste_clipboard(&mut self) {
        unsafe {
            if OpenClipboard(self.hwnd) == 0 {
                return;
            }
            let h = GetClipboardData(13);
            if !h.is_null() {
                let p = GlobalLock(h) as *const u16;
                if !p.is_null() {
                    let mut n = 0usize;
                    while *p.add(n) != 0 && n < 8192 {
                        n += 1;
                    }
                    let s = String::from_utf16_lossy(std::slice::from_raw_parts(p, n));
                    GlobalUnlock(h);
                    let s: String = s.chars().filter(|c| *c != '\r' && *c != '\n').collect();
                    self.input_insert(&s);
                }
            }
            CloseClipboard();
        }
    }

    fn cancel_input(&mut self) {
        self.overlay = Overlay::None;
        self.input_text.clear();
        self.repaint();
    }

    fn confirm_input(&mut self) {
        let text = self.input_text.trim().to_string();
        let action = self.input_action.clone();
        self.overlay = Overlay::None;
        self.input_text.clear();
        match action {
            InputAction::Search => {
                self.filter = text;
            }
            InputAction::SetTitle => {
                if !text.is_empty() {
                    self.conf.title_text = text;
                    self.conf.save(&self.dir);
                }
            }
            InputAction::NewGroup => {
                if !text.is_empty() {
                    let g = self.db.add_group(&text);
                    let t = self.db.ensure_tag(g);
                    self.conf.cur_group = g;
                    self.conf.cur_tag = t;
                    self.db.save(&self.dir);
                    self.save_conf();
                }
            }
            InputAction::RenameGroup(id) => {
                if !text.is_empty() {
                    if let Some(i) = self.db.group_index(id) {
                        self.db.groups[i].name = text;
                    }
                    self.db.save(&self.dir);
                }
            }
            InputAction::RenameLink(id) => {
                if !text.is_empty() {
                    if let Some(i) = self.db.link_index(id) {
                        self.db.links[i].name = text;
                    }
                    self.db.save(&self.dir);
                }
            }
            InputAction::AddUrl => {
                if !text.is_empty() {
                    let url = if text.contains("://") { text.clone() } else { format!("https://{}", text) };
                    let name = url
                        .trim_start_matches("https://")
                        .trim_start_matches("http://")
                        .trim_end_matches('/')
                        .to_string();
                    self.add_link(&name, KIND_URL, &url);
                }
            }
        }
        self.repaint();
    }

    // ---------------- 启动 ----------------
    fn launch(&mut self, idx: usize, admin: bool) {
        let l = self.db.links[idx].clone();
        let mut file = l.path.clone();
        let mut params = l.param.clone();
        let dir = l.workdir.clone();
        let verb = if admin || l.admin { "runas" } else { "open" };
        if l.kind == KIND_SYS {
            if let Some(s) = sys_item(&l.path) {
                file = s.cmd.to_string();
                if params.is_empty() {
                    params = s.args.to_string();
                }
            }
        }
        if file.is_empty() {
            return;
        }
        let wf = wide(&file);
        let wv = wide(verb);
        let wp = wide(&params);
        let wd = wide(&dir);
        let mut sei = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_NOCLOSEPROCESS,
            hwnd: self.hwnd,
            lpVerb: wv.as_ptr(),
            lpFile: wf.as_ptr(),
            lpParameters: if params.is_empty() { std::ptr::null() } else { wp.as_ptr() },
            lpDirectory: if dir.is_empty() { std::ptr::null() } else { wd.as_ptr() },
            nShow: SW_SHOWNORMAL,
            ..Default::default()
        };
        let ok = unsafe { ShellExecuteExW(&mut sei) };
        if ok == 0 {
            unsafe {
                ShellExecuteW(
                    self.hwnd,
                    std::ptr::null(),
                    wf.as_ptr(),
                    if params.is_empty() { std::ptr::null() } else { wp.as_ptr() },
                    if dir.is_empty() { std::ptr::null() } else { wd.as_ptr() },
                    SW_SHOWNORMAL,
                );
            }
        } else if !sei.hProcess.is_null() {
            unsafe {
                CloseHandle(sei.hProcess);
            }
        }
        self.db.links[idx].run_count = self.db.links[idx].run_count.saturating_add(1);
        self.db.save(&self.dir);
    }

    // ---------------- 菜单 ----------------
    fn popup(&mut self, hmenu: HMENU, x: i32, y: i32) -> usize {
        unsafe {
            let fg = GetForegroundWindow();
            let fg_tid = if fg.is_null() { 0 } else { GetWindowThreadProcessId(fg, std::ptr::null_mut()) };
            let my_tid = GetCurrentThreadId();
            if fg_tid != 0 && fg_tid != my_tid {
                AttachThreadInput(my_tid, fg_tid, 1);
            }
            SetForegroundWindow(self.hwnd);
            BringWindowToTop(self.hwnd);
            if fg_tid != 0 && fg_tid != my_tid {
                AttachThreadInput(my_tid, fg_tid, 0);
            }
            let mut pt = POINT { x: 0, y: 0 };
            GetCursorPos(&mut pt);
            let cmd = TrackPopupMenuEx(
                hmenu,
                TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_LEFTALIGN | TPM_TOPALIGN,
                if x < 0 { pt.x } else { x },
                if y < 0 { pt.y } else { y },
                self.hwnd,
                std::ptr::null(),
            );
            PostMessageW(self.hwnd, WM_NULL, 0, 0);
            DestroyMenu(hmenu);
            cmd as usize
        }
    }

    fn context_menu(&mut self, h: Hit, x: i32, y: i32) {
        let mut pt = POINT { x: 0, y: 0 };
        unsafe {
            ClientToScreen(self.hwnd, &mut pt);
        }
        let sx = pt.x + x;
        let sy = pt.y + y;
        match h {
            Hit::Link(i) => {
                let links = self.cur_links();
                if let Some(&li) = links.get(i) {
                    let id = self.db.links[li].id;
                    // 右键点在没选中的项上：选区只留它
                    if !self.is_sel(id) {
                        self.sel.clear();
                        self.sel.push(id);
                        self.repaint();
                    }
                    let targets = self.sel.clone();
                    let n = targets.len();
                    let lk = self.db.links[li].clone();
                    let sys = if lk.kind == KIND_SYS { sys_item(&lk.path) } else { None };
                    let m = unsafe { CreatePopupMenu() };
                    if n > 1 {
                        menu_str(m, ID_RUN, &format!("打开 {} 项", n));
                        menu_move(m, &self.db.groups, self.conf.cur_group);
                        menu_sep(m);
                        menu_str(m, ID_DEL_LINK, &format!("删除 {} 项", n));
                        menu_str(m, ID_SEL_CLEAR, "取消选择");
                    } else {
                        menu_str(m, ID_RUN, "打开");
                        match sys {
                            Some(s) => {
                                if !s.manage.is_empty() {
                                    menu_str(m, ID_MANAGE, "管理");
                                }
                                if !s.props.is_empty() {
                                    menu_str(m, ID_PROPS, "属性");
                                }
                                if s.key == "recyclebin" {
                                    menu_sep(m);
                                    menu_str(m, ID_EMPTY_BIN, "清空回收站");
                                }
                            }
                            None => {
                                menu_str(m, ID_RUN_ADMIN, "以管理员身份运行");
                                menu_sep(m);
                                menu_str(m, ID_OPEN_LOC, "打开文件位置");
                                menu_str(m, ID_PROPS, "属性");
                            }
                        }
                        menu_sep(m);
                        menu_move(m, &self.db.groups, self.conf.cur_group);
                        menu_sep(m);
                        menu_str(m, ID_EDIT, "重命名");
                        menu_str(m, ID_DEL_LINK, "删除");
                    }
                    let c = self.popup(m, sx, sy);
                    self.on_link_cmd(c, targets);
                }
            }
            Hit::Group(i) => {
                if let Some(g) = self.db.groups.get(i) {
                    let gid = g.id;
                    let m = unsafe { CreatePopupMenu() };
                    menu_str(m, ID_NEW_GROUP, "新建分组");
                    menu_str(m, ID_REN_GROUP, "重命名");
                    menu_str(m, ID_DEL_GROUP, "删除分组");
                    menu_sep(m);
                    menu_str(m, ID_GROUP_UP, "上移");
                    menu_str(m, ID_GROUP_DOWN, "下移");
                    menu_sep(m);
                    menu_str(m, ID_ADD_FILE, "添加程序…");
                    menu_str(m, ID_ADD_DIR, "添加文件夹…");
                    menu_str(m, ID_ADD_URL, "添加网址…");
                    menu_add_sys(m);
                    menu_sep(m);
                    menu_str(m, ID_ICON_S, "小图标");
                    menu_str(m, ID_ICON_M, "中图标");
                    menu_str(m, ID_ICON_L, "大图标");
                    menu_str(m, ID_VIEW_TILE, "磁贴视图");
                    menu_str(m, ID_VIEW_LIST, "列表视图");
                    menu_sep(m);
                    menu_str(m, ID_SETTINGS, "设置…");
                    let c = self.popup(m, sx, sy);
                    self.on_group_cmd(c, gid);
                }
            }
            _ => {
                let m = unsafe { CreatePopupMenu() };
                menu_str(m, ID_ADD_FILE, "添加程序…");
                menu_str(m, ID_ADD_DIR, "添加文件夹…");
                menu_str(m, ID_ADD_URL, "添加网址…");
                menu_add_sys(m);
                menu_sep(m);
                menu_str(m, ID_NEW_GROUP, "新建分组");
                menu_sep(m);
                menu_str(m, ID_ICON_S, "小图标");
                menu_str(m, ID_ICON_M, "中图标");
                menu_str(m, ID_ICON_L, "大图标");
                menu_str(m, ID_VIEW_TILE, "磁贴视图");
                menu_str(m, ID_VIEW_LIST, "列表视图");
                menu_sep(m);
                menu_str(m, ID_REFRESH_ALL, "重建所有图标");
                menu_str(m, ID_CLEAR_CACHE, "清除图标缓存");
                menu_sep(m);
                menu_str(m, ID_SETTINGS, "设置…");
                let c = self.popup(m, sx, sy);
                self.on_main_cmd(c);
            }
        }
    }

    fn main_menu(&mut self) {
        let mut pt = POINT { x: 0, y: 0 };
        unsafe {
            ClientToScreen(self.hwnd, &mut pt);
        }
        self.ensure_layout();
        let l = self.layout.clone();
        let sx = pt.x + l.bmenu.left;
        let sy = pt.y + l.bmenu.bottom;
        let m = unsafe { CreatePopupMenu() };
        menu_str(m, ID_SETTINGS, "设置…");
        menu_sep(m);
        menu_check(m, ID_TOG_TITLE, self.conf.show_title, "显示标题栏");
        menu_check(m, ID_TOG_DATE, self.conf.show_date, "显示日期");
        menu_check(m, ID_TOG_TOP, self.conf.topmost, "窗口置顶");
        menu_sep(m);
        menu_str(m, ID_ADD_FILE, "添加程序…");
        menu_str(m, ID_ADD_DIR, "添加文件夹…");
        menu_str(m, ID_ADD_URL, "添加网址…");
        menu_add_sys(m);
        menu_sep(m);
        menu_str(m, ID_NEW_GROUP, "新建分组");
        menu_sep(m);
        menu_str(m, ID_ICON_S, "小图标");
        menu_str(m, ID_ICON_M, "中图标");
        menu_str(m, ID_ICON_L, "大图标");
        menu_str(m, ID_VIEW_TILE, "磁贴视图");
        menu_str(m, ID_VIEW_LIST, "列表视图");
        menu_sep(m);
        menu_str(m, ID_REFRESH_ALL, "重建所有图标");
        menu_str(m, ID_CLEAR_CACHE, "清除图标缓存");
        menu_sep(m);
        menu_str(m, ID_HIDE, "隐藏到托盘");
        menu_str(m, ID_ABOUT, "关于");
        menu_str(m, ID_EXIT, "退出");
        let c = self.popup(m, sx, sy);
        self.on_main_cmd(c);
    }

    fn tray_menu(&mut self) {
        let m = unsafe { CreatePopupMenu() };
        menu_str(m, ID_HIDE, "显示 / 隐藏主界面");
        menu_str(m, ID_SETTINGS, "设置…");
        menu_sep(m);
        menu_str(m, ID_EXIT, "退出");
        let c = self.popup(m, -1, -1);
        self.on_main_cmd(c);
    }

    fn on_main_cmd(&mut self, c: usize) {
        match c {
            ID_SETTINGS => {
                self.overlay = Overlay::Settings;
                self.repaint();
            }
            ID_TOG_TITLE => {
                self.conf.show_title = !self.conf.show_title;
                self.save_conf();
                self.repaint();
            }
            ID_TOG_DATE => {
                self.conf.show_date = !self.conf.show_date;
                self.save_conf();
                self.repaint();
            }
            ID_TOG_TOP => {
                self.conf.topmost = !self.conf.topmost;
                let ex = WS_EX_LAYERED | WS_EX_TOOLWINDOW | if self.conf.topmost { WS_EX_TOPMOST } else { 0 };
                unsafe {
                    SetWindowLongPtrW(self.hwnd, GWL_EXSTYLE, ex as isize);
                    let after: HWND = if self.conf.topmost { (-1isize) as HWND } else { (-2isize) as HWND };
                    SetWindowPos(self.hwnd, after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
                }
                self.save_conf();
            }
            ID_HIDE => self.toggle_show(),
            ID_ABOUT => unsafe {
                let t = wide(
                    "QuickLaunch 1.0.2\n\n\
                     作者：黑山大叔\n\
                     主页：https://space.bilibili.com/19428259\n\
                     GitHub：https://github.com/heiheids\n\n\
                     Rust + 纯 Win32/GDI+ 实现的快捷启动器\n\
                     零第三方依赖，单文件绿色运行",
                );
                let cap = wide("关于");
                MessageBoxW(self.hwnd, t.as_ptr(), cap.as_ptr(), MB_OK | MB_ICONINFORMATION);
            },
            ID_EXIT => self.exit(),
            ID_NEW_GROUP => self.open_input("新建分组", InputAction::NewGroup),
            ID_ADD_FILE => self.add_files(),
            ID_ADD_DIR => self.add_folder(),
            ID_ADD_URL => self.open_input("添加网址", InputAction::AddUrl),
            ID_ICON_S => {
                self.conf.icon_size = 16;
                self.save_conf();
                self.repaint();
            }
            ID_ICON_M => {
                self.conf.icon_size = 32;
                self.save_conf();
                self.repaint();
            }
            ID_ICON_L => {
                self.conf.icon_size = 48;
                self.save_conf();
                self.repaint();
            }
            ID_VIEW_TILE => {
                self.conf.view = 0;
                self.save_conf();
                self.repaint();
            }
            ID_VIEW_LIST => {
                self.conf.view = 1;
                self.save_conf();
                self.repaint();
            }
            ID_REFRESH_ALL => {
                self.icons.clear_mem();
                self.repaint();
            }
            ID_CLEAR_CACHE => {
                self.icons.clear_disk();
                self.repaint();
            }
            _ => {
                if c >= ID_SYS_BASE {
                    if let Some(s) = SYS_ITEMS.get(c - ID_SYS_BASE) {
                        let name = s.name.to_string();
                        let key = s.key.to_string();
                        self.add_link(&name, KIND_SYS, &key);
                    }
                }
            }
        }
    }

    fn on_group_cmd(&mut self, c: usize, gid: u32) {
        match c {
            ID_NEW_GROUP => self.open_input("新建分组", InputAction::NewGroup),
            ID_REN_GROUP => {
                let name = self.db.groups.iter().find(|g| g.id == gid).map(|g| g.name.clone()).unwrap_or_default();
                self.open_input_pre("重命名分组", InputAction::RenameGroup(gid), &name);
            }
            ID_DEL_GROUP => {
                if self.db.groups.len() <= 1 {
                    return;
                }
                if confirm(self.hwnd, "确认删除该分组及其全部链接吗？") {
                    self.db.remove_group(gid);
                    if self.conf.cur_group == gid {
                        self.conf.cur_group = self.db.groups.first().map(|g| g.id).unwrap_or(0);
                        self.conf.cur_tag = self.db.ensure_tag(self.conf.cur_group);
                    }
                    self.db.save(&self.dir);
                    self.save_conf();
                    self.repaint();
                }
            }
            ID_GROUP_UP => {
                self.db.move_group(gid, -1);
                self.db.save(&self.dir);
                self.repaint();
            }
            ID_GROUP_DOWN => {
                self.db.move_group(gid, 1);
                self.db.save(&self.dir);
                self.repaint();
            }
            _ => self.on_main_cmd(c),
        }
    }

    fn on_link_cmd(&mut self, c: usize, targets: Vec<u32>) {
        if c == 0 || targets.is_empty() {
            return;
        }
        // 移动到某个分组
        if c >= ID_MOVE_BASE {
            let gi = c - ID_MOVE_BASE;
            if let Some(g) = self.db.groups.get(gi) {
                let gid = g.id;
                self.move_links(&targets, gid);
            }
            return;
        }
        if c == ID_SEL_CLEAR {
            self.sel.clear();
            self.repaint();
            return;
        }
        if c == ID_DEL_LINK {
            let n = targets.len();
            let msg = if n > 1 { format!("确认删除这 {} 项吗？", n) } else { "确认删除该链接吗？".to_string() };
            if confirm(self.hwnd, &msg) {
                self.delete_links(&targets);
            }
            return;
        }
        let li = match self.db.link_index(targets[0]) {
            Some(x) => x,
            None => return,
        };
        match c {
            ID_RUN => {
                for id in &targets {
                    if let Some(i) = self.db.link_index(*id) {
                        self.launch(i, false);
                    }
                }
            }
            ID_RUN_ADMIN => self.launch(li, true),
            ID_MANAGE => {
                let l = self.db.links[li].clone();
                if let Some(s) = sys_item(&l.path) {
                    if !s.manage.is_empty() {
                        let f = wide(s.manage);
                        unsafe {
                            ShellExecuteW(self.hwnd, std::ptr::null(), f.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL);
                        }
                    }
                }
            }
            ID_EMPTY_BIN => {
                if confirm(self.hwnd, "确定要清空回收站吗？此操作不可撤销。") {
                    unsafe {
                        SHEmptyRecycleBinW(self.hwnd, std::ptr::null(), SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND);
                    }
                }
            }
            ID_OPEN_LOC => {
                let l = self.db.links[li].clone();
                if l.kind == KIND_SYS {
                    return;
                }
                open_location(self.hwnd, &l.path);
            }
            ID_PROPS => {
                let l = self.db.links[li].clone();
                let mut target = l.path.clone();
                let mut ok = !target.is_empty();
                if l.kind == KIND_SYS {
                    match sys_item(&l.path) {
                        Some(s) if !s.props.is_empty() => target = s.props.to_string(),
                        _ => ok = false,
                    }
                }
                if ok {
                    let p = wide(&target);
                    let v = wide("properties");
                    unsafe {
                        ShellExecuteW(self.hwnd, v.as_ptr(), p.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL);
                    }
                }
            }
            ID_EDIT => {
                let name = self.db.links[li].name.clone();
                let id = self.db.links[li].id;
                self.open_input_pre("重命名链接", InputAction::RenameLink(id), &name);
            }
            _ => {}
        }
    }

    // ---------------- 添加 ----------------
    fn add_link(&mut self, name: &str, kind: i32, path: &str) {
        let g = self.conf.cur_group;
        if self.db.group_index(g).is_none() {
            return;
        }
        let tag = self.db.ensure_tag(g);
        self.conf.cur_tag = tag;
        self.db.add_link(tag, name, kind, path, "", "", "", "", false);
        self.db.save(&self.dir);
        self.save_conf();
        self.repaint();
    }

    fn add_files(&mut self) {
        unsafe {
            let mut buf = vec![0u16; 32768];
            let filter: Vec<u16> = "所有文件\0*.*\0\0".encode_utf16().collect();
            let title = wide("添加程序或文件");
            let mut ofn = OPENFILENAMEW {
                lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
                hwndOwner: self.hwnd,
                lpstrFilter: filter.as_ptr(),
                lpstrFile: buf.as_mut_ptr(),
                nMaxFile: buf.len() as u32,
                lpstrTitle: title.as_ptr(),
                Flags: OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_EXPLORER | OFN_ALLOWMULTISELECT | OFN_HIDEREADONLY,
                ..Default::default()
            };
            if GetOpenFileNameW(&mut ofn) == 0 {
                return;
            }
            let mut parts: Vec<String> = Vec::new();
            let mut start = 0usize;
            let mut i = 0usize;
            while i < buf.len() {
                if buf[i] == 0 {
                    if i == start {
                        break;
                    }
                    parts.push(String::from_utf16_lossy(&buf[start..i]));
                    start = i + 1;
                }
                i += 1;
            }
            let mut added = 0;
            if parts.len() == 1 {
                let full = parts[0].clone();
                let name = Path::new(&full)
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| full.clone());
                self.add_link(&name, KIND_PROGRAM, &full);
                added += 1;
            } else if parts.len() > 1 {
                let dir = parts[0].clone();
                for f in &parts[1..] {
                    let full = format!("{}\\{}", dir, f);
                    let name = Path::new(f)
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| f.clone());
                    let tag = self.db.ensure_tag(self.conf.cur_group);
                    self.db.add_link(tag, &name, KIND_PROGRAM, &full, "", "", "", "", false);
                    added += 1;
                }
            }
            if added > 0 {
                self.db.save(&self.dir);
                self.repaint();
            }
        }
    }

    fn add_folder(&mut self) {
        unsafe {
            let mut disp = [0u16; 260];
            let title = wide("请选择文件夹");
            let mut bi = BROWSEINFOW {
                hwndOwner: self.hwnd,
                pszDisplayName: disp.as_mut_ptr(),
                lpszTitle: title.as_ptr(),
                ulFlags: BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE | BIF_EDITBOX,
                ..Default::default()
            };
            let pidl = SHBrowseForFolderW(&mut bi);
            if pidl.is_null() {
                return;
            }
            let mut path = [0u16; 1024];
            let ok = SHGetPathFromIDListW(pidl, path.as_mut_ptr());
            CoTaskMemFree(pidl);
            if ok == 0 {
                return;
            }
            let full = from_wide(&path);
            let name = Path::new(&full)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| full.clone());
            self.add_link(&name, KIND_FOLDER, &full);
        }
    }

    fn drop_files(&mut self, drop: HANDLE) {
        unsafe {
            let n = DragQueryFileW(drop, 0xFFFF_FFFF, std::ptr::null_mut(), 0);
            let mut added = 0;
            for i in 0..n {
                let len = DragQueryFileW(drop, i, std::ptr::null_mut(), 0) as usize;
                let mut buf = vec![0u16; len + 2];
                DragQueryFileW(drop, i, buf.as_mut_ptr(), (len + 2) as u32);
                let full = from_wide(&buf);
                if full.is_empty() {
                    continue;
                }
                if self.add_path(&full) {
                    added += 1;
                }
            }
            DragFinish(drop);
            if added > 0 {
                self.db.save(&self.dir);
                self.repaint();
            }
        }
    }

    /// 把一个文件 / 文件夹路径加进当前分组；返回是否成功
    fn add_path(&mut self, full: &str) -> bool {
        if full.is_empty() {
            return false;
        }
        let name = Path::new(full)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| full.to_string());
        let kind = if Path::new(full).is_dir() { KIND_FOLDER } else { KIND_PROGRAM };
        let tag = self.db.ensure_tag(self.conf.cur_group);
        self.db.add_link(tag, &name, kind, full, "", "", "", "", false);
        true
    }

    // ---------------- 托盘 / 显示 ----------------
    pub fn init_tray(&mut self) {
        let n = unsafe { GetSystemMetrics(SM_CXSMICON) }.max(16);
        let icon = app_icon(n, n);
        self.tray_icon = icon;
        let mut data: NOTIFYICONDATAW = Default::default();
        data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        data.hWnd = self.hwnd;
        data.uID = 1;
        data.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        data.uCallbackMessage = WM_TRAY;
        data.hIcon = icon;
        to_wide_buf("QuickLaunch 快速启动", &mut data.szTip);
        let ok = unsafe { Shell_NotifyIconW(NIM_ADD, &mut data) };
        self.tray_ok = ok != 0;
    }

    fn hide(&mut self) {
        unsafe {
            ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    fn toggle_show(&mut self) {
        unsafe {
            if IsWindowVisible(self.hwnd) != 0 {
                ShowWindow(self.hwnd, SW_HIDE);
            } else {
                ShowWindow(self.hwnd, SW_SHOW);
                SetForegroundWindow(self.hwnd);
                self.repaint();
            }
        }
    }

    pub fn show(&mut self) {
        unsafe {
            ShowWindow(self.hwnd, SW_SHOW);
            SetForegroundWindow(self.hwnd);
        }
        self.repaint();
    }

    fn exit(&mut self) {
        self.db.save(&self.dir);
        self.save_conf();
        unsafe {
            if self.tray_ok {
                let mut data: NOTIFYICONDATAW = Default::default();
                data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                data.hWnd = self.hwnd;
                data.uID = 1;
                Shell_NotifyIconW(NIM_DELETE, &mut data);
            }
            if !self.tray_icon.is_null() {
                DestroyIcon(self.tray_icon);
            }
            DestroyWindow(self.hwnd);
            PostQuitMessage(0);
        }
    }

    pub fn save_conf(&mut self) {
        self.conf.save(&self.dir);
    }
}

fn int_resource(id: usize) -> LPCWSTR {
    id as LPCWSTR
}

fn menu_str(m: HMENU, id: usize, text: &str) {
    let w = wide(text);
    unsafe {
        AppendMenuW(m, MF_STRING, id, w.as_ptr());
    }
}

fn menu_check(m: HMENU, id: usize, checked: bool, text: &str) {
    let w = wide(text);
    unsafe {
        AppendMenuW(m, MF_STRING | if checked { MF_CHECKED } else { MF_UNCHECKED }, id, w.as_ptr());
    }
}

fn menu_sep(m: HMENU) {
    unsafe {
        AppendMenuW(m, MF_SEPARATOR, 0, std::ptr::null());
    }
}

fn menu_move(m: HMENU, groups: &[Group], cur: u32) {
    unsafe {
        let sub = CreatePopupMenu();
        let mut any = false;
        for (i, g) in groups.iter().enumerate() {
            if g.id == cur {
                continue;
            }
            let w = wide(&g.name);
            AppendMenuW(sub, MF_STRING, ID_MOVE_BASE + i, w.as_ptr());
            any = true;
        }
        if !any {
            let w = wide("（没有其他分组）");
            AppendMenuW(sub, MF_STRING | 0x0000_0004, 0, w.as_ptr()); // MF_GRAYED
        }
        let label = wide("移动到");
        AppendMenuW(m, MF_POPUP, sub as usize, label.as_ptr());
    }
}

fn menu_add_sys(m: HMENU) {
    unsafe {
        let sub = CreatePopupMenu();
        for (i, s) in SYS_ITEMS.iter().enumerate() {
            let w = wide(s.name);
            AppendMenuW(sub, MF_STRING, ID_SYS_BASE + i, w.as_ptr());
        }
        let label = wide("添加内置功能");
        AppendMenuW(m, MF_POPUP, sub as usize, label.as_ptr());
    }
}

fn confirm(hwnd: HWND, text: &str) -> bool {
    let t = wide(text);
    let cap = wide("提示");
    unsafe {
        let r = MessageBoxW(hwnd, t.as_ptr(), cap.as_ptr(), MB_YESNO | MB_ICONQUESTION);
        r == IDYES
    }
}

fn open_location(hwnd: HWND, path: &str) {
    unsafe {
        let exe = wide("explorer.exe");
        let arg = wide(&format!("/select,\"{}\"", path));
        ShellExecuteW(hwnd, std::ptr::null(), exe.as_ptr(), arg.as_ptr(), std::ptr::null(), SW_SHOWNORMAL);
    }
}

fn sys_dir(exe: &str) -> String {
    if exe.is_empty() {
        return String::new();
    }
    // shell 命名空间（此电脑 / 回收站）或协议串：原样交给 SHGetFileInfo 解析
    if exe.contains(':') || Path::new(exe).is_absolute() {
        return exe.to_string();
    }
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    format!("{}\\System32\\{}", root, exe)
}

// ---------------- 开机启动 ----------------
const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_NAME: &str = "QuickLaunch";

fn autostart_get() -> bool {
    let sub = wide(RUN_KEY);
    let name = wide(RUN_NAME);
    let mut buf = [0u16; 1024];
    let mut len: u32 = (buf.len() * 2) as u32;
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            sub.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            buf.as_mut_ptr() as *mut std::ffi::c_void,
            &mut len,
        )
    };
    r == 0 && len > 2
}

fn autostart_set(on: bool) {
    let sub = wide(RUN_KEY);
    let name = wide(RUN_NAME);
    unsafe {
        if on {
            let exe = std::env::current_exe().unwrap_or_default();
            let s = format!("\"{}\"", exe.display());
            let mut w: Vec<u16> = s.encode_utf16().collect();
            w.push(0);
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                sub.as_ptr(),
                name.as_ptr(),
                REG_SZ,
                w.as_ptr() as *const std::ffi::c_void,
                (w.len() * 2) as u32,
            );
        } else {
            RegDeleteKeyValueW(HKEY_CURRENT_USER, sub.as_ptr(), name.as_ptr());
        }
    }
}

// ---------------- 全局快捷键 ----------------
const MOD_ALT: u32 = 0x0001;
const MOD_CONTROL: u32 = 0x0002;
const MOD_SHIFT: u32 = 0x0004;
const MOD_WIN: u32 = 0x0008;
const MOD_NOREPEAT: u32 = 0x4000;

fn vk_name(vk: u32) -> String {
    if (0x41..=0x5A).contains(&vk) {
        return ((b'A' + (vk - 0x41) as u8) as char).to_string();
    }
    if (0x30..=0x39).contains(&vk) {
        return ((b'0' + (vk - 0x30) as u8) as char).to_string();
    }
    if (0x60..=0x69).contains(&vk) {
        return format!("Num{}", vk - 0x60);
    }
    if (0x70..=0x87).contains(&vk) {
        return format!("F{}", vk - 0x6F);
    }
    match vk {
        0x20 => "Space".to_string(),
        0x0D => "Enter".to_string(),
        0x09 => "Tab".to_string(),
        0x1B => "Esc".to_string(),
        0x25 => "Left".to_string(),
        0x26 => "Up".to_string(),
        0x27 => "Right".to_string(),
        0x28 => "Down".to_string(),
        0x24 => "Home".to_string(),
        0x23 => "End".to_string(),
        0x21 => "PgUp".to_string(),
        0x22 => "PgDn".to_string(),
        0x2D => "Ins".to_string(),
        0x2E => "Del".to_string(),
        0xBA => ";".to_string(),
        0xBB => "=".to_string(),
        0xBC => ",".to_string(),
        0xBD => "-".to_string(),
        0xBE => ".".to_string(),
        0xBF => "/".to_string(),
        _ => format!("VK{:02X}", vk),
    }
}

fn hotkey_text(mods: u32, vk: u32) -> String {
    if vk == 0 {
        return "未设置".to_string();
    }
    let mut s = String::new();
    if mods & MOD_CONTROL != 0 {
        s.push_str("Ctrl+");
    }
    if mods & MOD_ALT != 0 {
        s.push_str("Alt+");
    }
    if mods & MOD_SHIFT != 0 {
        s.push_str("Shift+");
    }
    if mods & MOD_WIN != 0 {
        s.push_str("Win+");
    }
    s.push_str(&vk_name(vk));
    s
}

/// 功能分区的分割线：在 c_border 基础上再拉开一档对比度
/// （浅色主题压深、深色主题提亮，两边都能看清）
fn sep_color(c: &Conf) -> u32 {
    if luminance(c.c_bg) > 140.0 {
        mix(c.c_border, rgb(0, 0, 0), 0.34)
    } else {
        mix(c.c_border, rgb(255, 255, 255), 0.32)
    }
}

/// 窗口外边框：比内部分割线再深一档
fn border_color(c: &Conf) -> u32 {
    if luminance(c.c_bg) > 140.0 {
        mix(c.c_border, rgb(0, 0, 0), 0.56)
    } else {
        mix(c.c_border, rgb(255, 255, 255), 0.52)
    }
}

fn today_text() -> String {    let mut st: SYSTEMTIME = Default::default();
    unsafe {
        GetLocalTime(&mut st);
    }
    let names = ["日", "一", "二", "三", "四", "五", "六"];
    let wd = (st.wDayOfWeek as usize).min(6);
    format!("{}月{}日 星期{}", st.wMonth, st.wDay, names[wd])
}

/// 程序图标：优先用 exe 里的图标资源（res\app.ico，ID = 1），失败再退回手绘
pub fn app_icon(cx: i32, cy: i32) -> HICON {
    unsafe {
        let inst = GetModuleHandleW(std::ptr::null());
        let h = LoadImageW(inst, 1 as LPCWSTR, IMAGE_ICON, cx, cy, LR_DEFAULTCOLOR);
        if !h.is_null() {
            return h as HICON;
        }
        make_icon(0xFF00_80C0)
    }
}

fn make_icon(accent: u32) -> HICON {
    unsafe {
        let mut c = Canvas::new(32, 32);
        c.fill_round(RECTF { x: 2.0, y: 2.0, w: 28.0, h: 28.0 }, [0.0, 0.0, 0.0, 0.0], accent);
        c.draw_text("Q", RECTF { x: 0.0, y: -1.0, w: 32.0, h: 32.0 }, 20, true, rgb(255, 255, 255), ALIGN_CENTER, false, TRIM_NONE);
        let mut img = std::ptr::null_mut();
        GdipCreateBitmapFromScan0(32, 32, 32 * 4, 0x000E_200B, c.bits as *mut u8, &mut img);
        let mut hicon: HICON = std::ptr::null_mut();
        GdipCreateHICONFromBitmap(img, &mut hicon);
        GdipDisposeImage(img);
        hicon
    }
}
