Add-Type -AssemblyName System.Drawing

$src = 'D:\@工作区\迅捷启动\res\source2.png'
$outPng = 'D:\@工作区\迅捷启动\res\app.png'
$outIco = 'D:\@工作区\迅捷启动\res\app.ico'

$bmp = [System.Drawing.Bitmap]::FromFile($src)
$w = $bmp.Width; $h = $bmp.Height
Write-Host "source: ${w}x${h}"

$rect = New-Object System.Drawing.Rectangle 0, 0, $w, $h
$data = $bmp.LockBits($rect, [System.Drawing.Imaging.ImageLockMode]::ReadWrite, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$stride = $data.Stride
$bytes = New-Object byte[] ($stride * $h)
[System.Runtime.InteropServices.Marshal]::Copy($data.Scan0, $bytes, 0, $bytes.Length)

# 白底 -> 透明；JPEG 边缘的灰白做线性过渡
$minX = $w; $minY = $h; $maxX = -1; $maxY = -1
for ($y = 0; $y -lt $h; $y++) {
  $row = $y * $stride
  for ($x = 0; $x -lt $w; $x++) {
    $i = $row + $x * 4
    $b = $bytes[$i]; $g = $bytes[$i+1]; $r = $bytes[$i+2]
    $m = $b; if ($g -lt $m) { $m = $g }; if ($r -lt $m) { $m = $r }
    if ($m -ge 250) { $a = 0 }
    elseif ($m -le 238) { $a = 255 }
    else { $a = [int](($m - 238) * 255 / 12); $a = 255 - $a }
    $a = [int]($a * $bytes[$i+3] / 255)   # 源图已有透明通道时按比例保留
    $bytes[$i+3] = [byte]$a
    if ($a -gt 16) {
      if ($x -lt $minX) { $minX = $x }
      if ($x -gt $maxX) { $maxX = $x }
      if ($y -lt $minY) { $minY = $y }
      if ($y -gt $maxY) { $maxY = $y }
    }
  }
}
[System.Runtime.InteropServices.Marshal]::Copy($bytes, 0, $data.Scan0, $bytes.Length)
$bmp.UnlockBits($data)

$cw = $maxX - $minX + 1
$ch = $maxY - $minY + 1
Write-Host "content bbox: ${cw}x${ch} at ($minX,$minY)"
$crop = $bmp.Clone((New-Object System.Drawing.Rectangle $minX, $minY, $cw, $ch), [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)

# 补成正方形（居中），只留 1% 边距
$side = [int]([Math]::Max($cw, $ch) * 1.02)
$square = New-Object System.Drawing.Bitmap $side, $side, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$gs = [System.Drawing.Graphics]::FromImage($square)
$gs.Clear([System.Drawing.Color]::Transparent)
$gs.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$gs.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
$ox = [int](($side - $cw) / 2); $oy = [int](($side - $ch) / 2)
$gs.DrawImage($crop, $ox, $oy, $cw, $ch)
$gs.Dispose()
Write-Host "square: ${side}x${side}"

function Resize([System.Drawing.Bitmap]$b, [int]$n) {
  $d = New-Object System.Drawing.Bitmap $n, $n, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
  $g = [System.Drawing.Graphics]::FromImage($d)
  $g.Clear([System.Drawing.Color]::Transparent)
  $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
  $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
  $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
  $g.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
  $g.DrawImage($b, (New-Object System.Drawing.Rectangle 0, 0, $n, $n), (New-Object System.Drawing.Rectangle 0, 0, $b.Width, $b.Height), [System.Drawing.GraphicsUnit]::Pixel)
  $g.Dispose()
  return $d
}

$square.Save($outPng, [System.Drawing.Imaging.ImageFormat]::Png)
Write-Host "png written"

# ---- 组装 .ico ----
$sizes = @(16, 24, 32, 48, 64, 128)
$blobs = @()
foreach ($n in $sizes) {
  $b = Resize $square $n
  if ($n -ge 64) {
    # 大尺寸用 PNG 压缩（Vista+ 支持）
    $ms = New-Object System.IO.MemoryStream
    $b.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $blobs += ,@{ size = $n; png = $true; data = $ms.ToArray() }
    $ms.Dispose()
  } else {
    # 小尺寸用 32bpp BMP + AND 掩码，兼容性最好
    $rowBytes = $n * 4
    $maskRow = [int]([Math]::Ceiling($n / 32.0) * 4)
    $biSize = 40
    $imgSize = $biSize + $rowBytes * $n + $maskRow * $n
    $buf = New-Object byte[] $imgSize
    # BITMAPINFOHEADER
    $bw = [BitConverter]::GetBytes([int]$biSize); [Array]::Copy($bw, 0, $buf, 0, 4)
    $bw = [BitConverter]::GetBytes([int]$n);      [Array]::Copy($bw, 0, $buf, 4, 4)   # width
    $bw = [BitConverter]::GetBytes([int]($n * 2));[Array]::Copy($bw, 0, $buf, 8, 4)   # height = 2n
    $bw = [BitConverter]::GetBytes([int16]1);     [Array]::Copy($bw, 0, $buf, 12, 2)  # planes
    $bw = [BitConverter]::GetBytes([int16]32);    [Array]::Copy($bw, 0, $buf, 14, 2)  # bpp
    $bw = [BitConverter]::GetBytes([int]0);       [Array]::Copy($bw, 0, $buf, 16, 4)  # BI_RGB
    $bw = [BitConverter]::GetBytes([int]($rowBytes * $n + $maskRow * $n)); [Array]::Copy($bw, 0, $buf, 20, 4)

    $rect2 = New-Object System.Drawing.Rectangle 0, 0, $n, $n
    $d2 = $b.LockBits($rect2, [System.Drawing.Imaging.ImageLockMode]::ReadOnly, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $px = New-Object byte[] ($d2.Stride * $n)
    [System.Runtime.InteropServices.Marshal]::Copy($d2.Scan0, $px, 0, $px.Length)
    $b.UnlockBits($d2)

    # XOR 位图：自下而上
    for ($y = 0; $y -lt $n; $y++) {
      $srcRow = ($n - 1 - $y) * $d2.Stride
      $dstRow = $biSize + $y * $rowBytes
      [Array]::Copy($px, $srcRow, $buf, $dstRow, $rowBytes)
    }
    # AND 掩码：alpha == 0 置 1
    $maskBase = $biSize + $rowBytes * $n
    for ($y = 0; $y -lt $n; $y++) {
      $srcRow = ($n - 1 - $y) * $d2.Stride
      for ($x = 0; $x -lt $n; $x++) {
        if ($px[$srcRow + $x * 4 + 3] -eq 0) {
          $buf[$maskBase + $y * $maskRow + [int][Math]::Floor($x / 8)] = $buf[$maskBase + $y * $maskRow + [int][Math]::Floor($x / 8)] -bor (0x80 -shr ($x % 8))
        }
      }
    }
    $blobs += ,@{ size = $n; png = $false; data = $buf }
  }
  $b.Dispose()
}

$count = $blobs.Count
$headerSize = 6 + 16 * $count
$total = $headerSize
foreach ($e in $blobs) { $total += $e.data.Length }

$ico = New-Object byte[] $total
$ico[0] = 0; $ico[1] = 0; $ico[2] = 1; $ico[3] = 0
$ico[4] = [byte]($count -band 0xFF); $ico[5] = [byte](($count -shr 8) -band 0xFF)
$offset = $headerSize
$idx = 6
foreach ($e in $blobs) {
  $n = $e.size
  $ico[$idx] = if ($n -ge 256) { 0 } else { [byte]$n }
  $ico[$idx+1] = if ($n -ge 256) { 0 } else { [byte]$n }
  $ico[$idx+2] = 0; $ico[$idx+3] = 0
  $bw = [BitConverter]::GetBytes([int16]1);  [Array]::Copy($bw, 0, $ico, $idx+4, 2)
  $bw = [BitConverter]::GetBytes([int16]32); [Array]::Copy($bw, 0, $ico, $idx+6, 2)
  $bw = [BitConverter]::GetBytes([int]$e.data.Length); [Array]::Copy($bw, 0, $ico, $idx+8, 4)
  $bw = [BitConverter]::GetBytes([int]$offset);        [Array]::Copy($bw, 0, $ico, $idx+12, 4)
  [Array]::Copy($e.data, 0, $ico, $offset, $e.data.Length)
  $offset += $e.data.Length
  $idx += 16
}
[System.IO.File]::WriteAllBytes($outIco, $ico)
Write-Host ("ico written: {0} bytes, {1} entries" -f $total, $count)

$crop.Dispose(); $square.Dispose(); $bmp.Dispose()
