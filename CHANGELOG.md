# 更新说明 / Changelog

## v1.0.9 — 2026-11-15

### 中文

**改进**

- **窗口阴影收窄一半、颜色变淡**：阴影带宽 12px → 6px，起始不透明度 18 → 6。
  实测（面板左边缘往外的亮度梯度，255 为纯白）：
  改前 `253 231 216 203 191 185 181 179`（向外铺 8px，边缘最暗 179）；
  改后 `251 240 234 232`（只铺 4px，边缘 232）。
  阴影带更薄、边缘也不再发闷。
- 窗口总尺寸不变，因此可视面板四周各多出 6px 可用空间。

**实现**

- 新增常量 `SHADOW_W`（=6），替换原先散落在 `build_layout`、`drag_split`、
  阴影绘制三处的硬编码 12，以后调宽度只需改这一个值。
- 阴影层间距与带宽同步缩放（`(9-i)*0.8` → `(9-i)*0.4`）；
  否则层会画到画布外被裁掉，看起来像硬边。
- 因为带宽减半后各层叠得更密，起始 alpha 必须一起降（18 → 6），
  不然边缘反而比原来更黑。

### English

**Improved**

- **Window shadow is half as wide and lighter**: band 12 px → 6 px, start alpha 18 → 6.
  Measured luminance gradient outward from the left panel edge (255 = white):
  before `253 231 216 203 191 185 181 179` (spread over 8 px, darkest 179);
  after `251 240 234 232` (spread over 4 px, darkest 232).
- The window size is unchanged, so the visible panel gains 6 px on each side.

**Implementation**

- New `SHADOW_W` constant (=6) replaces the hard-coded 12 that was repeated in
  `build_layout`, `drag_split` and the shadow painting code.
- Layer spacing scales with the band width (`(9-i)*0.8` → `(9-i)*0.4`); otherwise the
  layers fall outside the canvas and get clipped into a hard edge.
- Because the narrower band stacks the layers more densely, the start alpha had to drop
  (18 → 6) — leaving it would have made the edge darker than before.

## v1.0.8 — 2026-11-15

### 中文

**新增**

- **分组栏宽度可拖动**：分组栏与快捷方式区之间那条竖线现在是可拖动的分隔条。
  鼠标移上去光标变成左右箭头，左右拖动即时改变分组栏宽度，松手后写入
  `conf.ini` 的 `nGroupWidth`，下次启动保持。
- **分组栏最窄可缩到一个字宽**：以前有 `font_size*4+36` 的硬性下限（14 号字时约 92px），
  现在下限改成按字号计算（`font_size + 10`，最少 18px），14 号字时最窄 24px，
  正好放得下一个字。内容区始终至少保留 120px，不会被挤没。
- 分割线有 ±3px 的容错带，不用精确点在那一像素上。

**说明**

- 只改了分隔条的交互与分组栏宽度下限，其余布局、配色、圆角、间距均未改动。

### English

**Added**

- **Draggable group pane width**: the vertical line between the group pane and the shortcut
  area is now a splitter. The cursor becomes a left-right arrow over it; dragging resizes the
  pane live and the width is written to `nGroupWidth` in `conf.ini` on release, so it persists.
- **The group pane can shrink to a single character wide**: the old hard floor of
  `font_size*4+36` (about 92 px at font size 14) is replaced by a font-derived minimum
  (`font_size + 10`, at least 18 px), which is 24 px at font size 14 — just enough for one
  character. The content area always keeps at least 120 px.
- The splitter has a ±3 px tolerance band, so you don't have to hit a single pixel.

**Notes**

- Only the splitter interaction and the width floor changed; the rest of the layout, colors,
  corner radii and spacing are untouched.

## v1.0.7 — 2026-11-15

### 中文

**新增**

- **网址自动抓取网站图标**：添加网址后自动去站点取图标，不再是一个空白方块。
  取图顺序：先试 `<站点>/favicon.ico`；失败再解析首页 HTML 里的
  `<link rel="icon">`（rust-lang.org 这类没有根 favicon.ico 的站点靠这条），
  `data:` 内联图标与 svg 会跳过，`apple-touch-icon` 作为兜底候选。
- 抓取在**后台线程**完成，通过消息通知界面回填，添加网址时不会卡窗口；
  启动时也会给已有网址条目补齐图标。
- 图标按真实内容落盘：PNG 存 `.png`、ICO 存 `.ico`。
  这点很关键——不少站点的 `favicon.ico` 其实是 PNG，若一律按 `.ico` 保存，
  会走 `PrivateExtractIcons` 提取失败（实测 rust-lang、baidu 就是这种情况）。

**改进**

- **网址名称去掉 `www.` 前缀**：`https://www.bilibili.com/` 显示为 `bilibili.com`。
  只改显示名，实际访问的 URL 不变（有些站点必须带 www 才能打开）。

**实现**

- 新增 `src/favicon.rs`：WinHTTP（`winhttp.dll`，工具链自带导入库）直连，
  零第三方依赖；3 秒超时、512 KB 上限；结果缓存在 `cache\icons\fav_*.{ico,png}`。
- 新增 `examples/probe_favicon.rs`：走产品代码同一路径做真实联网测试。

### English

**Added**

- **Automatic website icons**: adding a URL now fetches the site's icon instead of showing
  a blank tile. Order: `<site>/favicon.ico` first, then `<link rel="icon">` parsed from the
  home page HTML (this is what makes sites like rust-lang.org work — they have no root
  `favicon.ico`). `data:` inline icons and SVGs are skipped; `apple-touch-icon` is a
  last-resort candidate.
- Fetching happens on a **background thread** and reports back via a window message, so
  adding a URL never freezes the UI. Existing URL entries are backfilled on startup.
- Icons are stored with their real extension: PNG as `.png`, ICO as `.ico`. This matters —
  many sites serve a PNG from `favicon.ico`, and saving it as `.ico` makes
  `PrivateExtractIcons` fail (verified with rust-lang and baidu).

**Improved**

- **`www.` prefix is dropped from the display name**: `https://www.bilibili.com/` shows as
  `bilibili.com`. Only the label changes; the URL actually opened is untouched, since some
  sites require `www.`.

**Implementation**

- New `src/favicon.rs`: direct WinHTTP (`winhttp.dll`, import library ships with the
  toolchain), still zero third-party dependencies; 3 s timeout, 512 KB cap, results cached
  in `cache\icons\fav_*.{ico,png}`.
- New `examples/probe_favicon.rs` exercises the real production code path over the network.

## v1.0.6 — 2026-11-15

### 中文

**修复**

- **程序图标恢复**：v1.0.4 / v1.0.5 打出的 exe 图标与 1.0.3 不一致（Explorer 里显示错误）。
  原因是 `build.rs` 里手写的 `.rsrc` 段生成器有两个错误：
  1. 资源目录项指向子目录时，偏移的最高位必须置 `1`（`0x80000000`）；
     漏置后根目录被当成数据项，整棵图标表都解析不出来。
  2. 数据项的 `OffsetToData` 是 **RVA**，必须配一条 `IMAGE_REL_AMD64_ADDR32NB`
     重定位，让链接器把“段内偏移”换算成“段 RVA + 偏移”。
- 现已补齐子目录标志，并输出带段符号与重定位的 COFF 目标文件。

**验证**

- 新增 `examples/verify_icon.rs`：直接解析 PE 资源表，把 exe 内嵌的 6 张图标与
  `res/app.ico` **逐字节**比对，结果 6/6 一致；与旧版 1.0.3 的 exe 图标逐像素比对，
  差异 0/1024。
- 该校验已并入 `build.ps1`，以后构建失败会直接报出来，不会再悄悄产出坏图标。

### English

**Fixed**

- **Application icon restored**: builds from v1.0.4 / v1.0.5 produced an exe whose icon
  differed from 1.0.3 (Explorer showed the wrong icon). Two bugs in the hand-written
  `.rsrc` section generator in `build.rs`:
  1. Directory entries pointing at a subdirectory must set the high bit of the offset
     (`0x80000000`). Without it the root directory is read as a data entry and the whole
     icon table is unparsable.
  2. A data entry's `OffsetToData` is an **RVA**, so it needs an
     `IMAGE_REL_AMD64_ADDR32NB` relocation for the linker to turn the section-relative
     offset into "section RVA + offset".
- The subdirectory flag is now set, and the emitted COFF object carries the section symbol
  and relocations.

**Verification**

- New `examples/verify_icon.rs` parses the PE resource table and compares all six embedded
  images with `res/app.ico` byte for byte — 6/6 identical. A pixel comparison against the
  original 1.0.3 exe icon shows 0/1024 differing pixels.
- This check is part of `build.ps1`, so a broken icon can no longer slip through silently.

## v1.0.5 — 2026-11-15

### 中文

**修复**

- **右键“属性”点了没反应**：以前用的是 `ShellExecuteW(hwnd, "properties", 路径)`，
  而这个调用对 exe 和文件夹会返回 `31`（`SE_ERR_NOASSOC`）并且静默失败，
  只有 `.lnk` 这类自身注册了 properties 动词的对象才碰巧能用。
  现在改用 `ShellExecuteExW` + `SEE_MASK_INVOKEIDLIST`，exe、文件夹、快捷方式、网址都可靠。
- **属性窗口指向哪一层**：条目现在连的是目标程序，属性窗口如果直接打开 exe，
  看到的就是程序属性而不是快捷方式属性。因此新增 `src` 字段记录原始快捷方式路径，
  右键“属性”优先打开那个 `.lnk` / `.url` 的属性窗口；
  快捷方式已被删除时自动回退到目标程序属性。
- 万一 shell 仍然拒绝，会退一步在资源管理器里选中该文件，不再出现“完全没反应”。

**数据**

- `links.txt` 的 L 行新增第 14 列 `src`（原始快捷方式路径）。
  旧数据只有 13 列，加载时该字段为空，**完全兼容，无需迁移**；
  由旧版迁移过来的 .lnk 条目在迁移时顺带把原路径写进 `src`。

**其它**

- “在文件夹中显示”与“属性”统一收进新模块 `src/shell.rs`，避免两处各自实现 shell 调用。

### English

**Fixed**

- **Right-click → Properties did nothing**: the old code called
  `ShellExecuteW(hwnd, "properties", path)`, which returns `31` (`SE_ERR_NOASSOC`) for
  executables and folders and fails silently. Only `.lnk` objects — which register their own
  properties verb — happened to work. It now uses `ShellExecuteExW` with
  `SEE_MASK_INVOKEIDLIST`, which is reliable for exes, folders, shortcuts and URLs.
- **Which object gets the property sheet**: entries now point at the target program, so opening
  the exe's properties would no longer be the shortcut's properties. A new `src` field stores
  the original shortcut path; Properties opens that `.lnk` / `.url` sheet when it still exists
  and falls back to the target program otherwise.
- If the shell still refuses, the containing folder is opened with the file selected, so the
  action is never a no-op.

**Data**

- New 14th column `src` (original shortcut path) in `L` rows of `links.txt`. Files with only
  13 columns load with an empty `src` — **fully backward compatible**, no migration needed.
  Entries migrated from older versions record their `.lnk` path in `src` at migration time.

**Other**

- "Show in folder" and "Properties" now live in the new `src/shell.rs` module.

## v1.0.4 — 2026-11-15

### 中文

**修复**

- **快捷方式失效**：把桌面上的快捷方式（`.lnk`）拖进来后，以前存的是快捷方式自身的路径，
  用户在桌面上把那个图标删掉，条目就再也启动不了。现在入库前会先把 `.lnk` 解析成它指向的
  **源程序真实路径**，同时带出启动参数、工作目录和图标，删掉桌面图标不影响使用。
- **网址快捷方式**：`.url` 文件同样会解析成真实网址并以网址类型保存，不再指向那个 `.url` 文件。
- **历史数据自动修复**：启动时对仍然存在的 `.lnk` / `.url` 条目做一次静默迁移，
  改写成真实目标后再保存；快捷方式已被删掉的条目保持原样，不做破坏性改动。
- 解析支持多级快捷方式（快捷方式指向另一个快捷方式）、相对路径和环境变量（如 `%SystemRoot%`）。
  指向应用商店应用（UWP）的快捷方式解析不出文件路径时，按原样保留。

**实现**

- 新增独立模块 `src/shortcut.rs`（约 320 行），通过系统自带 COM（`IShellLinkW` + `IPersistFile`）
  读取快捷方式，**不引入任何第三方依赖**。
- 拖放、`WM_DROPFILES`、以及“添加文件”对话框统一走 `add_path()`，行为一致。

**构建**

- 本机无 windres 时，`build.rs` 会用纯 Rust 直接生成 `.rsrc` 段目标文件再交给链接器，
  程序图标照常嵌入 exe（零第三方工具）。

### English

**Fixed**

- **Broken shortcuts**: dragging a desktop `.lnk` in used to store the shortcut's own path, so
  deleting the desktop icon permanently broke the entry. The shortcut is now resolved to the
  **target program's real path** at insert time, together with its arguments, working directory
  and icon, so removing the desktop icon no longer matters.
- **Internet shortcuts**: `.url` files are resolved to the actual URL and stored as a URL entry.
- **Automatic repair of existing data**: on startup, entries whose `.lnk` / `.url` still exists are
  silently migrated to the real target and saved. Entries whose shortcut is already gone are left
  untouched — nothing destructive.
- Resolution follows chained shortcuts, relative paths and environment variables
  (e.g. `%SystemRoot%`). UWP / Store-app shortcuts without a file path are kept as-is.

**Implementation**

- New self-contained module `src/shortcut.rs` (~320 lines) using the built-in COM interfaces
  `IShellLinkW` + `IPersistFile` — still **zero third-party dependencies**.
- Drag & drop, `WM_DROPFILES` and the "Add file" dialog all go through `add_path()`.

**Build**

- When `windres` is unavailable, `build.rs` now emits the `.rsrc` COFF object by itself in pure
  Rust, so the exe keeps its icon without any external tool.

## v1.0.3 — 2026-11-06

### 中文

**新增**

- **拖动图标改顺序**：按住图标移动超过 5px 进入拖动，被拖的项实时跟着挪位、其余项自动让位，
  松手写回顺序并保存。拖动中有 `SetCapture`，拖到窗口外松手也不会乱。
  搜索筛选状态下不启用（可见项不全，重排会打乱没显示的那些）。

**改进**

- **光标形态**：鼠标划过图标保持普通箭头，按下拖动时才变成手形；分组栏、标题按钮、
  设置项等可点击区域仍是手形。

**数据**

- `links.txt` 的 L 行最后一列原本是恒为 `0` 的保留字段，现在用作组内排序值（`order`）。
  排序键为 `(order, id)`，所有 order 都是 0 时行为与旧版完全一致，**旧数据无需迁移**。
  新加的链接和「移动到」其它分组的链接都按 `组内最大值 + 1` 追加到末尾。

### English

**Added**

- **Drag icons to reorder**: hold an icon and move more than 5 px to start dragging. The dragged
  item follows the cursor live while the others shift out of the way, and the new order is saved
  on release. `SetCapture` is used so releasing outside the window is safe. Disabled while a
  search filter is active.

**Improved**

- **Cursor shape**: hovering an icon keeps the normal arrow; it only turns into a hand while
  actually dragging. Group bar, title buttons and settings rows keep the hand cursor.

**Data**

- The last column of an `L` row in `links.txt` used to be a reserved `0`; it now stores the
  in-group sort order. Sorting is by `(order, id)`, so when every order is 0 the behaviour is
  identical to older versions and **no migration is needed**. New links and links moved to
  another group are appended at the end (`max + 1`).

## v1.0.2 — 2026-11-06

### 中文

**修复**

- **系统程序图标模糊**：此前 `.lnk` 快捷方式、此电脑、回收站等走 `SHGetFileInfo(SHGFI_LARGEICON)`，
  拿到的只有 32×32，放大到 40px 显示就糊了。现在改为
  `SHGetFileInfo(SHGFI_SYSICONINDEX)` + `SHGetImageList(SHIL_JUMBO)` 取 256×256 再降采样；
  普通 exe 也改成按当前图标大小向系统要图，不再固定 48×48。
  图标缓存版本升到 `ICON4`，旧缓存自动失效重建。
- **打开设置后阴影变大**：覆盖层的压暗层原本画满整个窗口，把四周 12px 的阴影留白也涂黑了，
  看起来像阴影放大了一圈。现在只压暗内容区，阴影留白保持原样。

**新增**

- **窗口外边框**：1px 描边，配色和功能分区线同源、再深一档，画在静态层里不增加每帧开销。
- **图片缩略图**：把图片文件拖进窗口，磁贴显示的是图片本身的缩略图（等比缩到 128px 内，
  保持长宽比），不再显示通用图片图标。

**改进**

- **功能分区分割线**（标题栏下边线、分组栏右边线）加深并改为整像素实线。
  原来是抗锯齿铺开的两行淡色，几乎看不见。

### English

**Fixed**

- **Blurry icons for shell items**: `.lnk` shortcuts, This PC and Recycle Bin used
  `SHGetFileInfo(SHGFI_LARGEICON)`, which only returns 32×32 — upscaled to 40 px it looked soft.
  Now we take the icon index and pull a 256×256 icon from `SHGetImageList(SHIL_JUMBO)`.
  Plain executables are also extracted at the current icon size instead of a fixed 48×48.
  Icon cache format bumped to `ICON4` so stale low-res entries are rebuilt.
- **Shadow growing when the settings panel opens**: the dim layer covered the whole window,
  including the 12 px shadow margin, which made the shadow look much bigger. It now dims
  only the content area.

**Added**

- **Window border**: a 1 px outline derived from the same color family as the separators,
  one step darker. It lives in the cached static layer, so it costs nothing per frame.
- **Image thumbnails**: dropping an image file now shows a thumbnail of the image itself
  (scaled to fit 128 px, aspect preserved) instead of a generic file-type icon.

**Improved**

- **Separator lines** (below the title bar, right of the group bar) are darker and drawn
  as crisp single-pixel lines instead of two washed-out anti-aliased rows.

---

## v1.0.1 — 2026-11-06

首个公开版本 / First public release.

- 零第三方依赖，纯 Win32 + GDI+ 自绘，单文件绿色运行
- 分组、磁贴/列表视图、16–96 图标缩放（Ctrl+滚轮）
- Ctrl + 左键多选，右键批量移动 / 删除
- 全局呼出快捷键（默认 Ctrl + Alt + Q，可在设置里录制）
- 开机启动开关（写 HKCU Run 键，无需管理员）
- 单基准色推导整套配色，浅色/深色主题自动切换
- 支持拖放文件、文件夹，以及此电脑 / 回收站等 shell 虚拟项
- 内置 16 个系统功能项，此电脑有「管理 / 属性」，回收站有「清空回收站」
- 中英文说明文档
