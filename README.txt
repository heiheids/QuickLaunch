QuickLaunch 1.0.1  —  迅捷启动
================================================================

一个用 Rust 从零写的 Windows 快捷启动器。
纯 Win32 + GDI+ 自绘，**零第三方依赖**，单文件绿色运行，无需安装。

作者：黑山大叔
主页：https://space.bilibili.com/19428259
GitHub：https://github.com/heiheids
许可：MIT


----------------------------------------------------------------
一、运行
----------------------------------------------------------------

直接双击 QuickLaunch.exe 即可。首次运行会在同目录自动生成：

    conf.ini            配置文件
    db\links.txt        链接数据
    cache\icons\        图标缓存（可随时删除，会自动重建）

整个目录可以随便拷到 U 盘或其它电脑，配置和数据都跟着走。


----------------------------------------------------------------
二、功能
----------------------------------------------------------------

· 分组 + 磁贴/列表两种视图，图标 16~96 可调
· Ctrl + 滚轮 直接缩放图标，滚轮滚动内容
· 全局呼出快捷键，默认 Ctrl + Alt + Q，可在设置里改
· Ctrl + 左键 多选，右键批量「移动到」别的分组 / 批量删除
· 右键菜单按类型分叉：普通程序有「以管理员身份运行 / 打开文件位置」，
  内置的「此电脑」有「管理 / 属性」，「回收站」有「清空回收站」
· 支持从桌面拖入文件、文件夹，也能拖「此电脑」「回收站」这类虚拟项
· 颜色只需设一个基准色，其它部位自动按深浅推导（浅色/深色主题自动切换）
· 直角窗口、可调不透明度、可选阴影、可自定义标题、可选日期显示
· 开机启动开关（写 HKCU 的 Run 键，不需要管理员）
· 内置功能：记事本、计算器、命令行、任务管理器、控制面板、此电脑、
  回收站、资源管理器、系统设置、画图、注册表、服务、设备管理器、
  网络连接、任务计划、关机


----------------------------------------------------------------
三、配置文件
----------------------------------------------------------------

conf.ini 是标准 INI，[main] 段：

    nWidth / nHeight     窗口大小
    nPosX / nPosY        窗口位置
    nIconSize            图标大小 16~96
    nViewType            0 = 磁贴，1 = 列表
    nOpacity             不透明度 60~255
    nFontSize            字号
    bTopMost             窗口置顶
    bShadow              窗口阴影
    bShowTitle           显示标题栏
    bShowDate            显示日期
    strTitle             左上角显示的名字
    clrBase              基准色，其它颜色由它推导
    nHotkeyMod           呼出快捷键的修饰键（1=Alt 2=Ctrl 4=Shift 8=Win）
    nHotkeyVk            呼出快捷键的虚拟键码（0 = 未设置）

db\links.txt 是制表符分隔的纯文本，一行一条，用反斜杠转义：

    G  分组ID  分组名
    T  标签ID  分组ID  标签名
    L  链接ID  标签ID  名称  类型  路径  参数  工作目录  图标  备注  启动次数  管理员

类型：0 = 程序/文件，1 = 网址，2 = 文件夹，4 = 内置功能


----------------------------------------------------------------
四、自己编译
----------------------------------------------------------------

需要 Rust（x86_64-pc-windows-gnu 目标）：

    rustup target add x86_64-pc-windows-gnu
    cargo build --release

产物：target\release\QuickLaunch.exe

说明：
· 没有 [dependencies]，不拉任何 crate，编出来就是单个 exe
· 图标资源需要 windres（MinGW/MSYS2 自带）。找不到时会跳过图标资源，
  程序仍可正常编译运行，只是 exe 文件图标为默认图标。
  也可以用环境变量指定：set WINDRES=C:\path\to\windres.exe
· 想换图标：替换 res\source.png 或 res\source2.webp，然后跑
  res\make_icon.ps1 重新生成 res\app.ico


================================================================
English
================================================================

QuickLaunch 1.0.1

A tiny quick launcher for Windows, written from scratch in Rust.
Pure Win32 + GDI+ custom drawing, **zero third-party dependencies**,
ships as a single portable executable. No installation required.

Author : 黑山大叔 (Heishan Dashu)
Home   : https://space.bilibili.com/19428259
GitHub : https://github.com/heiheids
License: MIT


----------------------------------------------------------------
1. Run
----------------------------------------------------------------

Just double-click QuickLaunch.exe. On first run it creates next to itself:

    conf.ini            settings
    db\links.txt        link data
    cache\icons\        icon cache (safe to delete, rebuilt automatically)

The whole folder is portable - copy it to a USB stick or another PC and
your settings and data come along.


----------------------------------------------------------------
2. Features
----------------------------------------------------------------

· Groups, tile/list views, icon size 16-96
· Ctrl + mouse wheel resizes icons, plain wheel scrolls
· Global show/hide hotkey (default Ctrl + Alt + Q, configurable)
· Ctrl + left click multi-select; right click to move a batch to another
  group or delete them together
· Context menus adapt to the item: regular programs get "Run as
  administrator" / "Open file location"; the built-in "This PC" gets
  "Manage" / "Properties"; "Recycle Bin" gets "Empty Recycle Bin"
· Drag and drop files and folders, including virtual shell items such as
  "This PC" and "Recycle Bin"
· Pick a single base color - every other color is derived from it, and
  light/dark themes switch automatically
· Square window corners, adjustable opacity, optional shadow, custom
  title text, optional date display
· "Start with Windows" toggle (writes the HKCU Run key, no admin needed)
· Built-in items: Notepad, Calculator, Command Prompt, Task Manager,
  Control Panel, This PC, Recycle Bin, Explorer, Settings, Paint,
  Registry Editor, Services, Device Manager, Network Connections,
  Task Scheduler, Shut down


----------------------------------------------------------------
3. Configuration
----------------------------------------------------------------

conf.ini is a plain INI file, section [main]:

    nWidth / nHeight     window size
    nPosX / nPosY        window position
    nIconSize            icon size, 16-96
    nViewType            0 = tiles, 1 = list
    nOpacity             opacity, 60-255
    nFontSize            font size
    bTopMost             always on top
    bShadow              window shadow
    bShowTitle           show title bar
    bShowDate            show date
    strTitle             title text
    clrBase              base color, everything else is derived from it
    nHotkeyMod           hotkey modifiers (1=Alt 2=Ctrl 4=Shift 8=Win)
    nHotkeyVk            hotkey virtual-key code (0 = disabled)

db\links.txt is tab separated, one record per line, backslash escaped:

    G  groupID  groupName
    T  tagID  groupID  tagName
    L  linkID  tagID  name  type  path  args  workdir  icon  remark  runCount  admin

type: 0 = program/file, 1 = URL, 2 = folder, 4 = built-in item


----------------------------------------------------------------
4. Building from source
----------------------------------------------------------------

Requires Rust with the x86_64-pc-windows-gnu target:

    rustup target add x86_64-pc-windows-gnu
    cargo build --release

Output: target\release\QuickLaunch.exe

Notes:
· There is no [dependencies] section at all - nothing is pulled from
  crates.io, the result is a single executable.
· The icon resource needs windres (bundled with MinGW/MSYS2). If it is
  not found the build still succeeds, the exe just keeps the default
  file icon. You can point at it explicitly:
      set WINDRES=C:\path\to\windres.exe
· To change the icon: replace res\source.png or res\source2.webp, then
  run res\make_icon.ps1 to regenerate res\app.ico
