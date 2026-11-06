# 更新说明 / Changelog

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
