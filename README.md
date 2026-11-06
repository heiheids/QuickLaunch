# QuickLaunch（迅捷启动 · Rust 版）

纯 Rust + Win32/GDI+ 实现的磁贴式快捷启动器，功能对标 **Lucy 快速启动 1.7.8**。
**零第三方依赖**（`Cargo.toml` 里 `[dependencies]` 为空），单文件绿色运行，体积约 **385 KB**。

![主界面](分析/screens/v3_final_crop.png)

## 一、构建

```powershell
# 需要 rustup（x86_64-pc-windows-gnu 工具链）
cargo build --release
# 产物：target\release\QuickLaunch.exe
```

工具链说明：GNU 版 Rust 自带 MinGW，但缺少 `gdiplus` 的导入库。`build.rs` 利用 GNU ld
支持「直接链接 DLL」的特性，把 `%SystemRoot%\System32\gdiplus.dll` 作为链接入参传入，
因此**不需要安装 Visual Studio，也不需要任何 crate**。

## 二、目录结构

```
QuickLaunch.exe      主程序（单文件）
conf.ini             设置与配色（首次运行自动生成）
db\links.txt         分组 / 标签 / 链接数据（UTF-8，制表符分隔）
cache\icons\*.icn    图标缓存（预乘 ARGB 原始像素，可随时删除）
```

全部为绿色便携：拷走整个目录即可，路径可写相对路径（如 `..\工具\x.exe`）。

## 三、界面与操作

- **整体为直角风格**（窗口、面板、按钮、选中块均无圆角）。
- **左侧分组栏**：单击切换分组；右键可新建 / 重命名 / 删除 / 上下移动分组。
  分组项高度随字号自适应，只比文字略高，列宽保证 4 个汉字完整显示。
- **右侧内容区**：单击启动；右键可运行、以管理员身份运行、打开文件位置、属性、重命名、删除、
  **移动到 ▸ 分组**。内置功能项（此电脑 / 回收站）的右键菜单会换成 **管理**（计算机管理）、**属性**，
  回收站额外有 **清空回收站**。
- **多选**：按住 **Ctrl + 左键** 逐个加选 / 取消；选中项显示强调色描边和淡色底。
  在多选状态下右键，菜单变成 **打开 N 项 / 移动到 ▸ / 删除 N 项 / 取消选择**，
  移动和删除都作用于整个选区。单击空白处、切换分组或执行完操作后选区自动清空。
- **标题栏**：程序名称（可改）+ 日期 + 搜索 / 菜单 / 关闭。
- **关闭按钮 = 隐藏到托盘**；托盘图标单击显隐，右键有菜单。
- **拖动标题栏或空白处移动窗口**，拖四边/四角缩放。
- **Ctrl + 滚轮** 直接缩放图标；**滚轮** 滚动内容。
- **拖文件进窗口**即可添加链接；**从桌面拖"此电脑""回收站"** 这类虚拟项也能添加
  （走 OLE IDropTarget + Shell IDList Array，见下）。
- 没有二级标签栏：链接按**分组**聚合展示；标签只存在于数据层（每条链接仍挂在标签上），
  添加链接时自动使用分组下的默认标签。

![设置面板](分析/screens/v3_settings_crop.png)

## 四、设置项（标题栏 ☰ → 设置…）

| 项目 | 说明 |
|---|---|
| 程序名称 | 标题栏左上角显示的名字 |
| 主题色 | **只选一个颜色**，其余部位（标题栏、分组栏、正文、悬停、边框、强调色、文字色）自动按明暗推导 |
| 不透明度 | 窗口整体透明度 60–255 |
| 字体大小 | 11–20，分组、标题、链接名同步缩放 |
| 图标大小 | 16–96（也可 Ctrl+滚轮） |
| 视图方式 | 磁贴 / 列表 |
| 显示日期 | 标题栏日期开关 |
| 显示标题栏 / 窗口置顶 / 窗口阴影 | 独立开关 |
| 开机启动 | 写入 / 删除 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\QuickLaunch`，无需管理员 |
| 呼出快捷键 | 点一下进入录制状态，按下组合键即可（必须含 Ctrl / Alt / Shift / Win）；Esc 取消，Delete 清除 |

主题色规则：亮色系（亮度 > 140）自动走浅色方案，深色系自动走深色方案，各部位按层级
取不同深浅，保证对比度。

## 四点五、全局呼出快捷键

`RegisterHotKey` 注册在窗口上，`WM_HOTKEY` 里直接 `toggle_show()`，默认 `Ctrl+Alt+Q`。

- 存在 `conf.ini` 的 `nHotkeyMod` / `nHotkeyVk`（Win32 的 MOD_* 与虚拟键码，0 = 未设置）。
- 设置里点「呼出快捷键」进入录制：只按修饰键不算，必须带至少一个修饰键
  （否则会抢走全系统的裸键输入），注册失败会弹窗提示被占用。
- 用 `MOD_NOREPEAT`，按住不放不会连续触发。

## 四点六、程序图标

`res/app.ico` 由 `res/make_icon.ps1` 从 `res/source.jpg` 生成：白底抠成透明
（按 `min(R,G,B)` 做阈值 + 羽化，吃掉 JPEG 边缘的灰白），再按内容包围盒裁切居中，
输出 16/24/32/48/64/128 六档（大尺寸用 PNG 压缩）。

`build.rs` 用 `windres` 把 `app.rc` 编成 COFF 目标文件再交给链接器，图标就进了 PE 资源段；
托盘和窗口类都用 `LoadImageW(hInstance, 1, IMAGE_ICON, …)` 从资源里取，取不到才退回手绘。

> 本机 windres 是 ANSI 版，遇到中文路径会失败，所以 build.rs 先把 rc/ico 复制到
> ASCII 的临时目录再编译。找不到 windres 时只打一条 cargo warning，不影响其余功能。

## 五、数据格式

`db\links.txt` 每行一条，制表符分隔，`\` 转义：

```
G	分组ID	分组名
T	标签ID	所属分组ID	标签名
L	链接ID	标签ID	名称	类型	路径	参数	工作目录	图标	备注	启动次数	管理员
```

类型：`0` 程序 / `1` 文件夹 / `3` 网址 / `4` 内置功能（记事本、计算器等关键字）/ `5` Shell 命令。

## 六、绘制性能

界面全部由 GDI+ 软件光栅化，因此**每一帧的代价必须按"像素 × 混合质量"来算**。
实测（466×798 窗口，Windows 11）：

| 场景 | 优化前 | 优化后 |
| --- | --- | --- |
| 悬停一次（关阴影） | 41 ms | 0.94 ms |
| 悬停一次（开阴影） | 145 ms | 0.97 ms |
| 命中测试 | 0.035 µs | 0.035 µs |
| 静态层重建（开阴影） | 每次悬停都做 | 8.6 ms，仅改设置/改尺寸时做 |

关键手段：

1. **静态层缓存**：阴影 + 面板 + 标题栏 + 分组栏底 + 内容区底只在尺寸/配色/标题/日期变化时
   重绘到独立的 `bg` 画布，之后每帧只做一次 1:1 贴图，再叠加悬停相关的动态部分。
2. **CompositingQuality 用 Default 而不是 HighQuality**：后者走 gamma 校正混合，
   在 32bpp 逐像素透明表面上慢约 14 倍（单次 442×774 填充 12 ms → 0.9 ms）。这一项是最大头。
3. **直角走矩形快速路径**：`fill_round` / `stroke_round` 在圆角为 0 时直接调用
   `GdipFillRectangle`，省掉路径构建与 AA 描边。
4. **布局缓存**：`Layout` 按几何相关参数的哈希缓存，鼠标移动不再每次重算并排序链接列表。
5. **GDI+ 对象复用**：画布持有唯一的 `GpBrush` / `GpPen` 和 `StringFormat` 缓存，
   不再每次绘制都 Create/Delete。
6. **图标键不再 `format!`**：`IconStore::get` 直接对两段字符串做 FNV，去掉每链接每帧的分配。

设置环境变量 `QL_BENCH=1` 启动可复测，结果写到同目录 `bench.txt` 后自动退出。

## 六点五、图标提取

**不能只用 `GdipCreateBitmapFromHICON`**：对「有 1bpp 掩码、但颜色位图 alpha 全 255」的图标，
GDI+ 会返回一整块不透明的黑色背景（快捷方式、部分 exe 图标都会中招）。

现在的做法是直接读 HICON 的原始位图：

1. `GetIconInfo` 拿到 `hbmColor` / `hbmMask`；
2. `GetDIBits` 把 32bpp 颜色位图取成自上而下 BGRA，**保留真实 alpha**；
3. 再 `GetDIBits` 取 1bpp 掩码（1 = 透明）；
4. 颜色位图 alpha 全 0 或全 255 时（说明没有 alpha 通道）改用掩码决定透明度；
5. 自己做 alpha 预乘后写入 `.icn`。

`.icn` 头是 `ICON3`；旧的缓存会被自动忽略并重新提取。

## 六点六、桌面虚拟项的拖放

`WM_DROPFILES` 只承载 `CF_HDROP`，桌面上的"此电脑""回收站"是 shell 命名空间项，
永远不会走它。所以注册了一个 OLE `IDropTarget`（`src\drop.rs`，纯 FFI 手写 vtable）：

1. `Drop` 里先试 `CF_HDROP`（普通文件 / 文件夹），失败再试 `"Shell IDList Array"`；
2. 解析 `CIDA`：`cidl` + `aoffset[]`，取父 PIDL 与项目 PIDL 拼成完整 PIDL；
   （两种偏移约定都试一遍，靠 `SHGetNameFromIDList` 是否成功来判定）
3. `SHGetNameFromIDList(..., SIGDN_DESKTOPABSOLUTEPARSING)` 拿到 `::{CLSID}`，
   映射到内置功能 key；
4. 结果装箱后 `PostMessageW(WM_APP+10)` 回窗口线程，由 `App` 落库。

`src\drop.rs` 里有 `cargo test` 可跑的单元测试，用真实 PIDL 验证 CIDA 解析。

内置功能的图标不再借用 `explorer.exe`，而是把 `::{CLSID}` 交给
`SHParseDisplayName` + `SHGetFileInfo(SHGFI_PIDL)` 取，所以此电脑是显示器图标、
回收站是垃圾桶图标。


## 七、已实现 / 未实现

**已实现**：无边框直角透明窗口、逐像素透明与投影、分组栏（无二级标签栏）、磁贴/列表视图、
16–96 图标、图标提取与缓存、六类链接启动（含管理员）、四套右键菜单、托盘、拖放添加
（含桌面虚拟项）、分组内筛选搜索、单主色自动配色、完整设置面板、开机启动、Ctrl+滚轮缩放、
程序名自定义、**Ctrl 多选 + 批量移动/删除**、亚毫秒级悬停重绘。

## 八、关于

作者：黑山大叔

- 主页：https://space.bilibili.com/19428259
- GitHub：https://github.com/heiheids

**未实现（P2 计划）**：全局热键、拖拽排序、贴边自动隐藏、Everything / UWP 集成、
导入 Lucy 的 `link.db`。
