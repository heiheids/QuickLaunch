//! 数据模型与持久化：conf.ini（设置）+ db\links.txt（分组/标签/链接）。
use crate::gfx::parse_color;
use std::fs;
use std::path::{Path, PathBuf};

pub const KIND_PROGRAM: i32 = 0;
pub const KIND_FOLDER: i32 = 1;
#[allow(dead_code)]
pub const KIND_FILE: i32 = 2;
pub const KIND_URL: i32 = 3;
pub const KIND_SYS: i32 = 4;
#[allow(dead_code)]
pub const KIND_SHELL: i32 = 5;

#[derive(Clone)]
pub struct Group {
    pub id: u32,
    pub name: String,
}

#[derive(Clone)]
pub struct Tag {
    pub id: u32,
    pub group: u32,
    pub name: String,
}

#[derive(Clone)]
pub struct Link {
    pub id: u32,
    pub tag: u32,
    pub name: String,
    pub kind: i32,
    pub path: String,
    pub param: String,
    pub workdir: String,
    pub icon: String,
    pub remark: String,
    pub run_count: u32,
    pub admin: bool,
}

pub struct Db {
    pub groups: Vec<Group>,
    pub tags: Vec<Tag>,
    pub links: Vec<Link>,
    next_id: u32,
}

impl Db {
    pub fn empty() -> Db {
        let mut db = Db { groups: Vec::new(), tags: Vec::new(), links: Vec::new(), next_id: 1 };
        let g = db.add_group("新的分组");
        db.add_tag(g, "新的标签");
        db
    }

    pub fn default_data() -> Db {
        let mut db = Db::empty();
        let gid = db.groups[0].id;
        let tid = db.tags[0].id;
        let _ = gid;
        for (name, kind, path) in [
            ("记事本", KIND_SYS, "notepad"),
            ("计算器", KIND_SYS, "calc"),
            ("命令行", KIND_SYS, "cmd"),
            ("任务管理器", KIND_SYS, "taskmgr"),
            ("控制面板", KIND_SYS, "control"),
            ("回收站", KIND_SYS, "recyclebin"),
            ("此电脑", KIND_SYS, "thispc"),
        ] {
            db.add_link(tid, name, kind, path, "", "", "", "", false);
        }
        db
    }

    pub fn next_id(&mut self) -> u32 {
        let n = self.next_id;
        self.next_id += 1;
        n
    }

    pub fn add_group(&mut self, name: &str) -> u32 {
        let id = self.next_id();
        self.groups.push(Group { id, name: name.to_string() });
        id
    }

    pub fn add_tag(&mut self, group: u32, name: &str) -> u32 {
        let id = self.next_id();
        self.tags.push(Tag { id, group, name: name.to_string() });
        id
    }

    pub fn add_link(
        &mut self,
        tag: u32,
        name: &str,
        kind: i32,
        path: &str,
        param: &str,
        workdir: &str,
        icon: &str,
        remark: &str,
        admin: bool,
    ) -> u32 {
        let id = self.next_id();
        self.links.push(Link {
            id,
            tag,
            name: name.to_string(),
            kind,
            path: path.to_string(),
            param: param.to_string(),
            workdir: workdir.to_string(),
            icon: icon.to_string(),
            remark: remark.to_string(),
            run_count: 0,
            admin,
        });
        id
    }

    pub fn tags_of(&self, group: u32) -> Vec<usize> {
        (0..self.tags.len()).filter(|&i| self.tags[i].group == group).collect()
    }

    pub fn links_of(&self, tag: u32) -> Vec<usize> {
        let mut v: Vec<usize> = (0..self.links.len()).filter(|&i| self.links[i].tag == tag).collect();
        v.sort_by_key(|&i| self.links[i].id);
        v
    }

    /// 一个分组下所有标签的链接（界面已取消标签栏，按分组聚合展示）
    pub fn links_of_group(&self, group: u32) -> Vec<usize> {
        let tags: Vec<u32> = self.tags.iter().filter(|t| t.group == group).map(|t| t.id).collect();
        let mut v: Vec<usize> = (0..self.links.len()).filter(|&i| tags.contains(&self.links[i].tag)).collect();
        v.sort_by_key(|&i| self.links[i].id);
        v
    }

    /// 取分组的默认标签（没有就建一个），用于向分组内添加链接
    pub fn ensure_tag(&mut self, group: u32) -> u32 {
        if let Some(t) = self.tags.iter().find(|t| t.group == group) {
            return t.id;
        }
        self.add_tag(group, "默认")
    }

    pub fn remove_group(&mut self, id: u32) {
        let tag_ids: Vec<u32> = self.tags.iter().filter(|t| t.group == id).map(|t| t.id).collect();
        for t in tag_ids {
            self.remove_tag(t);
        }
        self.groups.retain(|g| g.id != id);
    }

    pub fn remove_tag(&mut self, id: u32) {
        self.links.retain(|l| l.tag != id);
        self.tags.retain(|t| t.id != id);
    }

    pub fn remove_link(&mut self, id: u32) {
        self.links.retain(|l| l.id != id);
    }

    pub fn group_index(&self, id: u32) -> Option<usize> {
        self.groups.iter().position(|g| g.id == id)
    }
    pub fn tag_index(&self, id: u32) -> Option<usize> {
        self.tags.iter().position(|t| t.id == id)
    }
    pub fn link_index(&self, id: u32) -> Option<usize> {
        self.links.iter().position(|l| l.id == id)
    }

    pub fn move_group(&mut self, id: u32, delta: i32) {
        if let Some(i) = self.group_index(id) {
            let j = i as i32 + delta;
            if j >= 0 && (j as usize) < self.groups.len() {
                self.groups.swap(i, j as usize);
            }
        }
    }

    pub fn move_tag(&mut self, id: u32, delta: i32) {
        if let Some(i) = self.tag_index(id) {
            let j = i as i32 + delta;
            if j >= 0 && (j as usize) < self.tags.len() {
                self.tags.swap(i, j as usize);
            }
        }
    }

    // ---------- 持久化 ----------
    pub fn load(dir: &Path) -> Option<Db> {
        let p = dir.join("db").join("links.txt");
        let txt = fs::read_to_string(&p).ok()?;
        let mut db = Db { groups: Vec::new(), tags: Vec::new(), links: Vec::new(), next_id: 1 };
        let mut max_id = 0;
        for line in txt.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let f: Vec<String> = line.split('\t').map(unesc).collect();
            match f[0].as_str() {
                "G" if f.len() >= 3 => {
                    if let Ok(id) = f[1].parse::<u32>() {
                        max_id = max_id.max(id);
                        db.groups.push(Group { id, name: f[2].clone() });
                    }
                }
                "T" if f.len() >= 4 => {
                    if let Ok(id) = f[1].parse::<u32>() {
                        if let Ok(g) = f[2].parse::<u32>() {
                            max_id = max_id.max(id);
                            db.tags.push(Tag { id, group: g, name: f[3].clone() });
                        }
                    }
                }
                "L" if f.len() >= 13 => {
                    if let Ok(id) = f[1].parse::<u32>() {
                        if let Ok(tag) = f[2].parse::<u32>() {
                            max_id = max_id.max(id);
                            db.links.push(Link {
                                id,
                                tag,
                                name: f[3].clone(),
                                kind: f[4].parse().unwrap_or(0),
                                path: f[5].clone(),
                                param: f[6].clone(),
                                workdir: f[7].clone(),
                                icon: f[8].clone(),
                                remark: f[9].clone(),
                                run_count: f[10].parse().unwrap_or(0),
                                admin: f[11] == "1",
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        if db.groups.is_empty() {
            return None;
        }
        db.next_id = max_id + 1;
        // 清理悬空数据
        let gids: Vec<u32> = db.groups.iter().map(|g| g.id).collect();
        let bad_tags: Vec<u32> = db.tags.iter().filter(|t| !gids.contains(&t.group)).map(|t| t.id).collect();
        for t in bad_tags {
            db.remove_tag(t);
        }
        let tids: Vec<u32> = db.tags.iter().map(|t| t.id).collect();
        db.links.retain(|l| tids.contains(&l.tag));
        Some(db)
    }

    pub fn save(&self, dir: &Path) {
        let d = dir.join("db");
        let _ = fs::create_dir_all(&d);
        let mut out = String::new();
        for g in &self.groups {
            out.push_str(&format!("G\t{}\t{}\n", g.id, esc(&g.name)));
        }
        for t in &self.tags {
            out.push_str(&format!("T\t{}\t{}\t{}\n", t.id, t.group, esc(&t.name)));
        }
        for l in &self.links {
            out.push_str(&format!(
                "L\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t0\n",
                l.id,
                l.tag,
                esc(&l.name),
                l.kind,
                esc(&l.path),
                esc(&l.param),
                esc(&l.workdir),
                esc(&l.icon),
                esc(&l.remark),
                l.run_count,
                if l.admin { 1 } else { 0 }
            ));
        }
        let _ = fs::write(d.join("links.txt"), out);
    }
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\t', "\\t").replace('\n', "\\n").replace('\r', "")
}

fn unesc(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('t') => out.push('\t'),
                Some('n') => out.push('\n'),
                Some('\\') => out.push('\\'),
                Some(o) => out.push(o),
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

// ---------------- 设置 ----------------
#[derive(Clone)]
pub struct Conf {
    pub w: i32,
    pub h: i32,
    pub x: i32,
    pub y: i32,
    pub group_w: i32,
    pub tag_w: i32,
    pub icon_size: i32,
    pub view: i32, // 0 磁贴, 1 列表
    pub show_title: bool,
    pub show_group: bool,
    pub show_tag: bool,
    pub show_search: bool,
    pub topmost: bool,
    pub shadow: bool,
    pub opacity: u8,
    pub cur_group: u32,
    pub cur_tag: u32,
    pub c_base: u32,
    pub font_size: i32,
    pub title_text: String,
    pub show_date: bool,
    pub c_bg: u32,
    pub c_title: u32,
    pub c_title_text: u32,
    pub c_text: u32,
    pub c_group: u32,
    pub c_tag: u32,
    pub c_hover: u32,
    pub c_accent: u32,
    pub c_border: u32,
    /// 全局呼出快捷键：修饰键（MOD_ALT/CONTROL/SHIFT/WIN）与虚拟键码，0 = 未设置
    pub hk_mod: u32,
    pub hk_vk: u32,
}

impl Default for Conf {
    fn default() -> Self {
        Conf {
            w: 431,
            h: 640,
            x: -1,
            y: -1,
            group_w: 84,
            tag_w: 0,
            icon_size: 48,
            view: 0,
            show_title: true,
            show_group: true,
            show_tag: false,
            show_search: true,
            topmost: true,
            shadow: true,
            opacity: 255,
            cur_group: 0,
            cur_tag: 0,
            c_base: parse_color("#8FB3D9").unwrap(),
            font_size: 14,
            title_text: "QuickLaunch".to_string(),
            show_date: true,
            c_bg: parse_color("#F7F9FC").unwrap(),
            c_title: parse_color("#E9F0F7").unwrap(),
            c_title_text: parse_color("#5C7490").unwrap(),
            c_text: parse_color("#2B2E31").unwrap(),
            c_group: parse_color("#EDF3F8").unwrap(),
            c_tag: parse_color("#FAFCFE").unwrap(),
            c_hover: parse_color("#DCE8F3").unwrap(),
            c_accent: parse_color("#5C7490").unwrap(),
            c_border: parse_color("#C9D9E8").unwrap(),
            hk_mod: 0x0002 | 0x0001, // MOD_CONTROL | MOD_ALT
            hk_vk: 0x51,             // Q
        }
    }
}

impl Conf {
    pub fn path(dir: &Path) -> PathBuf {
        dir.join("conf.ini")
    }

    pub fn load(dir: &Path) -> Conf {
        let mut c = Conf::default();
        let txt = match fs::read_to_string(Conf::path(dir)) {
            Ok(t) => t,
            Err(_) => return c,
        };
        let get = |k: &str| -> Option<String> {
            let mut inmain = false;
            for line in txt.lines() {
                let l = line.trim();
                if l.starts_with('[') {
                    inmain = l.eq_ignore_ascii_case("[main]");
                    continue;
                }
                if inmain {
                    if let Some(eq) = l.find('=') {
                        if l[..eq].trim().eq_ignore_ascii_case(k) {
                            return Some(l[eq + 1..].trim().to_string());
                        }
                    }
                }
            }
            None
        };
        let gb = |c: &mut bool, k: &str| {
            if let Some(v) = get(k) {
                *c = v == "1" || v.eq_ignore_ascii_case("true");
            }
        };
        let gi = |v: &mut i32, k: &str| {
            if let Some(s) = get(k) {
                if let Ok(n) = s.parse::<i32>() {
                    *v = n;
                }
            }
        };
        let gu = |v: &mut u32, k: &str| {
            if let Some(s) = get(k) {
                if let Ok(n) = s.parse::<u32>() {
                    *v = n;
                }
            }
        };
        let gc = |v: &mut u32, k: &str| {
            if let Some(s) = get(k) {
                if let Some(n) = parse_color(&s) {
                    *v = n;
                }
            }
        };
        gi(&mut c.w, "nWidth");
        gi(&mut c.h, "nHeight");
        gi(&mut c.x, "nPosX");
        gi(&mut c.y, "nPosY");
        gi(&mut c.group_w, "nGroupWidth");
        gi(&mut c.tag_w, "nTagWidth");
        gi(&mut c.icon_size, "nIconSize");
        gi(&mut c.view, "nViewType");
        gi(&mut c.font_size, "nFontSize");
        gb(&mut c.show_title, "bShowTitle");
        gb(&mut c.show_group, "bShowGroup");
        gb(&mut c.show_tag, "bShowTag");
        gb(&mut c.show_search, "bShowBtnSearch");
        gb(&mut c.topmost, "bTopMost");
        gb(&mut c.shadow, "bShadow");
        gu(&mut c.cur_group, "nCurGroup");
        gu(&mut c.cur_tag, "nCurTag");
        if let Some(s) = get("nOpacity") {
            if let Ok(n) = s.parse::<i32>() {
                c.opacity = n.clamp(60, 255) as u8;
            }
        }
        gc(&mut c.c_bg, "clrBg");
        gc(&mut c.c_title, "clrTitle");
        gc(&mut c.c_title_text, "clrTitleText");
        gc(&mut c.c_text, "clrText");
        gc(&mut c.c_group, "clrGroup");
        gc(&mut c.c_tag, "clrTag");
        gc(&mut c.c_hover, "clrHover");
        gc(&mut c.c_accent, "clrAccent");
        gc(&mut c.c_border, "clrBorder");
        gb(&mut c.show_date, "bShowDate");
        gu(&mut c.hk_mod, "nHotkeyMod");
        gu(&mut c.hk_vk, "nHotkeyVk");
        if let Some(v) = get("strTitle") {
            if !v.trim().is_empty() {
                c.title_text = v;
            }
        }
        let had_base = {
            let mut ok = false;
            gc(&mut c.c_base, "clrBase");
            if get("clrBase").is_some() {
                ok = true;
            }
            ok
        };
        if had_base {
            c.apply_base();
        }
        c.font_size = c.font_size.clamp(11, 20);
        c.icon_size = c.icon_size.clamp(16, 96);
        c.group_w = c.group_w.clamp(60, 200);
        c.w = c.w.clamp(260, 4000);
        c.h = c.h.clamp(200, 4000);
        c.group_w = c.group_w.clamp(40, 200);
        c.tag_w = c.tag_w.clamp(36, 160);
        c
    }

    /// 由单一主色自动派生各部分配色（浅色系走亮色方案，深色系走暗色方案）
    pub fn apply_base(&mut self) {
        let b = self.c_base | 0xFF00_0000;
        let white = 0xFFFF_FFFF;
        let black = 0xFF00_0000;
        let light = crate::gfx::luminance(b) > 140.0;
        if light {
            self.c_bg = crate::gfx::mix(b, white, 0.93);
            self.c_title = crate::gfx::mix(b, white, 0.86);
            self.c_group = crate::gfx::mix(b, white, 0.89);
            self.c_tag = crate::gfx::mix(b, white, 0.95);
            self.c_hover = crate::gfx::mix(b, white, 0.74);
            self.c_border = crate::gfx::mix(b, white, 0.60);
            self.c_accent = crate::gfx::mix(b, black, 0.38);
            self.c_text = crate::gfx::mix(b, black, 0.82);
        } else {
            self.c_bg = crate::gfx::mix(b, black, 0.72);
            self.c_title = crate::gfx::mix(b, black, 0.80);
            self.c_group = crate::gfx::mix(b, black, 0.78);
            self.c_tag = crate::gfx::mix(b, black, 0.70);
            self.c_hover = crate::gfx::mix(b, black, 0.55);
            self.c_border = crate::gfx::mix(b, black, 0.45);
            self.c_accent = crate::gfx::mix(b, white, 0.40);
            self.c_text = crate::gfx::mix(b, white, 0.90);
        }
        self.c_title_text = self.c_accent;
    }

    pub fn save(&self, dir: &Path) {
        let s = format!(
            "[main]\n\
             bShowTitle={}\n\
             bShowGroup={}\n\
             bShowTag={}\n\
             bShowBtnSearch={}\n\
             bTopMost={}\n\
             bShadow={}\n\
             nWidth={}\n\
             nHeight={}\n\
             nPosX={}\n\
             nPosY={}\n\
             nGroupWidth={}\n\
             nTagWidth={}\n\
             nIconSize={}\n\
             nViewType={}\n\
             nOpacity={}\n\
             nCurGroup={}\n\
             nCurTag={}\n\
             nFontSize={}\n\
             bShowDate={}\n\
             nHotkeyMod={}\n\
             nHotkeyVk={}\n\
             strTitle={}\n\
             clrBase={}\n\
             clrBg={}\n\
             clrTitle={}\n\
             clrTitleText={}\n\
             clrText={}\n\
             clrGroup={}\n\
             clrTag={}\n\
             clrHover={}\n\
             clrAccent={}\n\
             clrBorder={}\n",
            if self.show_title { 1 } else { 0 },
            if self.show_group { 1 } else { 0 },
            if self.show_tag { 1 } else { 0 },
            if self.show_search { 1 } else { 0 },
            if self.topmost { 1 } else { 0 },
            if self.shadow { 1 } else { 0 },
            self.w,
            self.h,
            self.x,
            self.y,
            self.group_w,
            self.tag_w,
            self.icon_size,
            self.view,
            self.opacity,
            self.cur_group,
            self.cur_tag,
            self.font_size,
            if self.show_date { 1 } else { 0 },
            self.hk_mod,
            self.hk_vk,
            self.title_text.replace('\n', " "),
            crate::gfx::color_to_hex(self.c_base | 0xFF00_0000),
            crate::gfx::color_to_hex(self.c_bg),
            crate::gfx::color_to_hex(self.c_title),
            crate::gfx::color_to_hex(self.c_title_text),
            crate::gfx::color_to_hex(self.c_text),
            crate::gfx::color_to_hex(self.c_group),
            crate::gfx::color_to_hex(self.c_tag),
            crate::gfx::color_to_hex(self.c_hover),
            crate::gfx::color_to_hex(self.c_accent),
            crate::gfx::color_to_hex(self.c_border),
        );
        let _ = fs::write(Conf::path(dir), s);
    }
}
