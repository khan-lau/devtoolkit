//! 字体加载与基线对齐
//!
//! 字体策略:
//! - 拉丁字母/数字: 内置 Inter (可变字体, 支持 `wght` 字重轴), 保证各平台外观一致
//! - 中文/日文等 CJK: 加载系统字体作为回退 (macOS 冬青黑体/苹方, Windows 微软雅黑, Linux Noto Sans CJK)
//! - 等宽: egui 内置 Hack, CJK 同样回退到系统字体
//!
//! 基线对齐:
//! egui 用 **主字体** (家族中第一个字体) 的 ascent / 行高布局整行, 回退字体的字形则按
//! "自身 ascent + 居中行高差" 摆放。当主字体与回退字体的纵向度量不同时(CJK 字体的
//! lineGap 往往很大), 中文字形就会整体上浮或下沉, 与相邻的英文/数字错位。
//!
//! 这里直接解析字体文件的 `hhea` / `OS/2` 表拿到度量(与 skrifa 选择规则一致),
//! 按 egui 的布局公式反推出每个回退字体需要的 `y_offset_factor`, 使其基线与 Inter 完全重合。

use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui::{self, FontData, FontDefinitions, FontFamily, FontTweak};

/// 内置拉丁字体: Inter (SIL Open Font License), 见 assets/fonts/LICENSE-Inter.txt
const INTER: &[u8] = include_bytes!("../assets/fonts/InterVariable.ttf");

/// 主字体名(家族首位)
pub const LATIN: &str = "Inter";
/// egui 内置等宽字体名
const HACK: &str = "Hack";

/// 字体纵向度量(以 em 为单位, descent 为负值)
#[derive(Clone, Copy, Debug, PartialEq)]
struct VMetrics {
    ascent: f32,
    descent: f32,
    line_gap: f32,
}

impl VMetrics {
    /// 行高 = ascent - descent + lineGap
    fn height(&self) -> f32 {
        return self.ascent - self.descent + self.line_gap;
    }
}

/// 候选系统字体: (路径, TTC 内的字体索引)
struct Candidate {
    path: PathBuf,
    index: u32,
}

impl Candidate {
    fn new(path: impl Into<PathBuf>, index: u32) -> Self {
        return Self { path: path.into(), index };
    }
}

/// 安装全部字体到 egui 上下文
pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default(); // 1. 拿到默认字体定义
    let latin_metrics = parse_metrics(INTER, 0); // 2. 解析 Inter 这种免费字体度量(字体的指标参数)

    fonts
        .font_data
        .insert(LATIN.to_owned(), Arc::new(FontData::from_static(INTER)));

    let (main_candidates, extb_candidates) = system_candidates();
    let mut fallbacks: Vec<String> = Vec::new();

    for (label, candidates) in [("cjk", main_candidates), ("cjk_extb", extb_candidates)] {
        let Some((data, index)) = load_first(&candidates) else {
            continue;
        };
        // 与主字体基线对齐; 解析失败时退化为不偏移
        let tweak = match (latin_metrics, parse_metrics(&data, index)) {
            (Some(primary), Some(fallback)) => baseline_tweak(primary, fallback),
            _ => FontTweak::default(),
        };
        let font = FontData {
            index,
            ..FontData::from_owned(data).tweak(tweak)
        };
        fonts.font_data.insert(label.to_owned(), Arc::new(font));
        fallbacks.push(label.to_owned());
    }

    // 比例字体: Inter → CJK 回退 → egui 默认(Ubuntu-Light / 表情 / 图标)
    let prop = fonts.families.entry(FontFamily::Proportional).or_default();
    let defaults: Vec<String> = prop.drain(..).collect();
    prop.push(LATIN.to_owned());
    prop.extend(fallbacks.iter().cloned());
    prop.extend(defaults);

    // 等宽字体: Hack → CJK 回退 → 其余默认
    let mono = fonts.families.entry(FontFamily::Monospace).or_default();
    let mut rest: Vec<String> = mono.drain(..).collect();
    rest.retain(|n| n != HACK);
    mono.push(HACK.to_owned());
    mono.extend(fallbacks.iter().cloned());
    mono.extend(rest);

    ctx.set_fonts(fonts);
}

/// 各平台的系统 CJK 字体候选: (主字体候选, CJK 扩展 B 区补充字体候选)
fn system_candidates() -> (Vec<Candidate>, Vec<Candidate>) {
    if cfg!(target_os = "windows") {
        return (
            vec![
                Candidate::new(r"C:\Windows\Fonts\msyh.ttc", 0), // 微软雅黑 (Win7+ 默认)
                Candidate::new(r"C:\Windows\Fonts\msjh.ttc", 0), // 微軟正黑體 (繁体系统)
                Candidate::new(r"C:\Windows\Fonts\Deng.ttf", 0), // 等线 (Win8+)
                Candidate::new(r"C:\Windows\Fonts\simhei.ttf", 0), // 黑体
                Candidate::new(r"C:\Windows\Fonts\simsun.ttc", 0), // 宋体 (精简系统兜底)
                Candidate::new(r"C:\Windows\Fonts\MingLiU.ttc", 0),
                Candidate::new(r"C:\Windows\Fonts\PMingLiU.ttc", 0),
                Candidate::new(r"C:\Windows\Fonts\simkai.ttf", 0),
                Candidate::new(r"C:\Windows\Fonts\simfang.ttf", 0),
            ],
            vec![Candidate::new(r"C:\Windows\Fonts\simsunb.ttf", 0)],
        )
    } else if cfg!(target_os = "macos") {
        let mut main = vec![
            // 冬青黑体 W3: 体积适中(~23MB), macOS 长期内置
            Candidate::new("/System/Library/Fonts/Hiragino Sans GB.ttc", 0),
            // 旧版系统的苹方路径
            Candidate::new("/System/Library/Fonts/PingFang.ttc", 0),
        ];
        // 新版 macOS 的苹方位于 AssetsV2 目录, 索引 3 为 "PingFang SC Regular"
        main.extend(
            find_asset_fonts("PingFang.ttc")
                .into_iter()
                .map(|p| Candidate::new(p, 3)),
        );
        main.push(Candidate::new("/System/Library/Fonts/STHeiti Light.ttc", 1)); // Heiti SC Light

        let mut extb = vec![Candidate::new("/System/Library/Fonts/Songti.ttc", 0)];
        extb.extend(
            find_asset_fonts("Songti.ttc")
                .into_iter()
                .map(|p| Candidate::new(p, 0)),
        );
        return (main, extb);
    } else {
        return (
            vec![
                Candidate::new("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 0),
                Candidate::new("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc", 0),
                Candidate::new("/usr/share/fonts/truetype/wqy/wqy-microhei.ttc", 0),
                Candidate::new("/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf", 0),
            ],
            vec![], // Noto Sans CJK 覆盖已较广, 无专门扩展字体
        )
    }
}

/// 在 macOS 的 `/System/Library/AssetsV2/com_apple_MobileAsset_Font*/` 下查找按需下载的系统字体
fn find_asset_fonts(file_name: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(roots) = std::fs::read_dir("/System/Library/AssetsV2") else {
        return found;
    };
    for root in roots.flatten() {
        if !root
            .file_name()
            .to_string_lossy()
            .starts_with("com_apple_MobileAsset_Font")
        {
            continue;
        }
        let Ok(assets) = std::fs::read_dir(root.path()) else {
            continue;
        };
        for asset in assets.flatten() {
            let candidate = asset.path().join("AssetData").join(file_name);
            if candidate.is_file() {
                found.push(candidate);
            }
        }
    }
    return found;
}

/// 读取第一个存在且可解析的候选字体, 返回 (字体数据, 修正后的索引)
fn load_first(candidates: &[Candidate]) -> Option<(Vec<u8>, u32)> {
    for c in candidates {
        if !c.path.is_file() {
            continue;
        }
        let Ok(data) = std::fs::read(&c.path) else {
            continue;
        };
        // 索引超出 TTC 实际数量时退回 0, 避免 egui 解析失败
        let index = if c.index < face_count(&data) { c.index } else { 0 };
        if parse_metrics(&data, index).is_some() {
            return Some((data, index));
        }
    }
    return None;
}

/// 根据 egui 的排版公式计算回退字体的基线补偿
///
/// egui 对回退字体字形的纵向位置为:
/// `y = ascent_f + 0.5 * (height_p - height_f)` (以 em 计, p = 主字体, f = 回退字体),
/// 而主字体字形基线位于 `ascent_p`; 二者之差即需要向下偏移的量。
/// `y_offset_factor` 会乘以字号, 因此以 em 为单位填入即可适配所有字号。
fn baseline_tweak(primary: VMetrics, fallback: VMetrics) -> FontTweak {
    let offset = primary.ascent - fallback.ascent - 0.5 * (primary.height() - fallback.height());
    return FontTweak {
        y_offset_factor: offset,
        ..FontTweak::default()
    };
}

// ---------------------------------------------------------------------------
// 最小化 sfnt 解析: 仅读取 head / hhea / OS/2 三张表
// ---------------------------------------------------------------------------

/// 从字节切片 `d` 的偏移 `off` 处，安全读取一个 **2 字节大端序无符号整数**。
///
/// - 成功：返回 `Some(u16)`
/// - 失败（越界）：返回 `None`
///
/// 大端序：高位字节在前，如 `0x1234` 存储为 `[0x12, 0x34]`。
fn be_u16(d: &[u8], off: usize) -> Option<u16> {
    return Some(u16::from_be_bytes([*d.get(off)?, *d.get(off + 1)?]));
}

/// 从字节切片 `d` 的偏移 `off` 处，安全读取一个 **2 字节大端序有符号整数**。
///
/// 实现方式：先读 `u16`，再按位重新解释为 `i16`。
/// `as i16` 不做数值转换，只是把同样的 16 位重新解释为有符号数。
///
/// 示例：
/// - `0x0000` → `0`
/// - `0x7FFF` → `32767`
/// - `0xFFFF` → `-1`
fn be_i16(d: &[u8], off: usize) -> Option<i16> {
    return be_u16(d, off).map(|v| v as i16);
}

/// 从字节切片 `d` 的偏移 `off` 处，安全读取一个 **4 字节大端序无符号整数**。
///
/// - 成功：返回 `Some(u32)`
/// - 失败（越界）：返回 `None`
fn be_u32(d: &[u8], off: usize) -> Option<u32> {
    return Some(u32::from_be_bytes([
        *d.get(off)?,
        *d.get(off + 1)?,
        *d.get(off + 2)?,
        *d.get(off + 3)?,
    ]));
}

/// TTC 内的字体数量(单字体文件返回 1)
fn face_count(data: &[u8]) -> u32 {
    if data.get(0..4) == Some(b"ttcf") {
        return be_u32(data, 8).unwrap_or(1).max(1);
    }
    return 1;
}

/// 解析指定索引字体的纵向度量
///
/// 选择规则与 skrifa / FreeType 一致:
/// 1. OS/2 设置了 USE_TYPO_METRICS → 使用 typo 度量
/// 2. 否则使用 hhea
/// 3. hhea 为零时回退到 OS/2 的 typo (非零) 或 win 度量
fn parse_metrics(data: &[u8], index: u32) -> Option<VMetrics> {
    let mut base = 0usize;
    if data.get(0..4)? == b"ttcf" {
        let count = be_u32(data, 8)? as usize;
        let idx = (index as usize).min(count.checked_sub(1)?);
        base = be_u32(data, 12 + 4 * idx)? as usize;
    }

    let num_tables = be_u16(data, base + 4)? as usize;
    let (mut head, mut hhea, mut os2) = (None, None, None);
    for i in 0..num_tables {
        let rec = base + 12 + 16 * i;
        let tag = data.get(rec..rec + 4)?;
        let off = be_u32(data, rec + 8)? as usize;
        let len = be_u32(data, rec + 12)? as usize;
        let table = data.get(off..off.checked_add(len)?)?;
        match tag {
            b"head" => head = Some(table),
            b"hhea" => hhea = Some(table),
            b"OS/2" => os2 = Some(table),
            _ => {}
        }
    }

    let upm = be_u16(head?, 18)? as f32;
    if upm <= 0.0 {
        return None;
    }
    let em = |v: i16| v as f32 / upm;

    // OS/2: fsSelection @62, sTypoAscender @68, sTypoDescender @70, sTypoLineGap @72,
    //       usWinAscent @74, usWinDescent @76
    let typo = os2.and_then(|t| {
        return Some((
            be_u16(t, 62)? & (1 << 7) != 0,
            be_i16(t, 68)?,
            be_i16(t, 70)?,
            be_i16(t, 72)?,
            be_u16(t, 74)?,
            be_u16(t, 76)?,
        ));
    });

    if let Some((true, asc, desc, gap, _, _)) = typo {
        return Some(VMetrics {
            ascent: em(asc),
            descent: em(desc),
            line_gap: em(gap),
        });
    }

    // hhea: ascender @4, descender @6, lineGap @8
    let (mut asc, mut desc, mut gap) = match hhea {
        Some(t) => (be_i16(t, 4)?, be_i16(t, 6)?, be_i16(t, 8)?),
        None => (0, 0, 0),
    };
    if asc == 0 && desc == 0 {
        match typo {
            Some((_, ta, td, tg, _, _)) if ta != 0 || td != 0 => {
                asc = ta;
                desc = td;
                gap = tg;
            }
            Some((_, _, _, _, wa, wd)) => {
                asc = wa as i16;
                desc = -(wd as i16);
                gap = 0;
            }
            None => return None,
        }
    }

    return Some(VMetrics { ascent: em(asc), descent: em(desc), line_gap: em(gap) });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Inter 使用 typo 度量: ascender 1984 / descender -494 / lineGap 0, upm 2048
    #[test]
    fn inter_metrics() {
        let m = parse_metrics(INTER, 0).expect("Inter 可解析");
        assert!((m.ascent - 1984.0 / 2048.0).abs() < 1e-4);
        assert!((m.descent + 494.0 / 2048.0).abs() < 1e-4);
        assert_eq!(m.line_gap, 0.0);
    }

    /// 度量相同的字体不需要补偿
    #[test]
    fn identical_metrics_zero_offset() {
        let m = VMetrics { ascent: 0.9, descent: -0.2, line_gap: 0.1 };
        assert_eq!(baseline_tweak(m, m).y_offset_factor, 0.0);
    }

    /// 冬青黑体 (0.88 / -0.12 / 0.5) 相对 Ubuntu-Light (0.932 / -0.189 / 0.028) 需下移约 0.23em
    #[test]
    fn hiragino_vs_ubuntu_light() {
        let p = VMetrics { ascent: 0.932, descent: -0.189, line_gap: 0.028 };
        let f = VMetrics { ascent: 0.88, descent: -0.12, line_gap: 0.5 };
        let off = baseline_tweak(p, f).y_offset_factor;
        assert!((off - 0.2275).abs() < 1e-3, "offset = {off}");
    }
}
