//! 时间戳与时间格式化工具
//!
//! 支持:
//! - 时间戳(秒/毫秒) 转换为时间字符串
//! - 时间字符串 转换为时间戳(秒/毫秒)

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone};
use eframe::egui;

use crate::i18n::Texts;
use crate::theme;

/// 常用时间字符串格式(无时区, 按本地时间解析)
const DATETIME_FORMATS: &[&str] = &[
    "%Y-%m-%d %H:%M:%S%.f",
    "%Y-%m-%d %H:%M:%S",
    "%Y-%m-%d %H:%M",
    "%Y/%m/%d %H:%M:%S%.f",
    "%Y/%m/%d %H:%M:%S",
    "%Y/%m/%d %H:%M",
    "%Y-%m-%dT%H:%M:%S%.f",
    "%Y-%m-%dT%H:%M:%S",
    "%Y年%m月%d日 %H:%M:%S",
    "%Y年%m月%d日 %H:%M",
];

/// 仅日期格式(时分秒补零)
const DATE_FORMATS: &[&str] = &["%Y-%m-%d", "%Y/%m/%d", "%Y年%m月%d日"];

/// 时间戳转换错误
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TsErr {
    /// 时间戳不是整数
    NotInteger,
    /// 时间戳超出可表示范围
    OutOfRange,
    /// 时间字符串为空
    Empty,
    /// 无法解析时间字符串
    ParseFailed,
    /// 本地时间转换存在歧义
    Ambiguous,
}

impl TsErr {
    /// 转换为当前语言的错误文案
    pub fn msg(self, t: &Texts) -> String {
        match self {
            TsErr::NotInteger => t.ts_err_not_int.to_string(),
            TsErr::OutOfRange => t.ts_err_range.to_string(),
            TsErr::Empty => t.ts_err_empty.to_string(),
            TsErr::ParseFailed => t.ts_err_parse.to_string(),
            TsErr::Ambiguous => t.ts_err_ambiguous.to_string(),
        }
    }
}

/// 时间戳单位
#[derive(Clone, Copy, PartialEq, Eq)]
enum TsUnit {
    /// 根据位数自动识别: 10 位以内视为秒, 否则视为毫秒
    Auto,
    Seconds,
    Millis,
}

/// 时间戳工具界面状态
pub struct TimestampTool {
    t: Texts,

    // 时间戳 -> 时间字符串
    ts_input: String,
    ts_unit: TsUnit,
    ts_out_local: String,
    ts_out_utc: String,
    ts_error: Option<TsErr>,
    ts_last_key: (TsUnit, String),

    // 时间字符串 -> 时间戳
    dt_input: String,
    dt_out_ms: String,
    dt_out_s: String,
    dt_error: Option<TsErr>,
    dt_last_key: String,

    // 当前时间
    now_ms: i64,
    now_s: i64,
    /// 当前时间字符串(格式 "YYYY-MM-DD HH:MM:SS.mmm", 每秒定时刷新)
    now_time_str: String,
}

impl TimestampTool {
    pub fn new(t: Texts) -> Self {
        let now = Local::now();
        Self {
            t,
            ts_input: String::new(),
            ts_unit: TsUnit::Auto,
            ts_out_local: String::new(),
            ts_out_utc: String::new(),
            ts_error: None,
            ts_last_key: (TsUnit::Auto, String::new()),
            dt_input: String::new(),
            dt_out_ms: String::new(),
            dt_out_s: String::new(),
            dt_error: None,
            dt_last_key: String::new(),
            now_ms: now.timestamp_millis(),
            now_s: now.timestamp(),
            now_time_str: now.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
        }
    }

    /// 语言切换时更新文案
    pub fn set_lang(&mut self, t: Texts) {
        self.t = t;
    }

    /// 渲染工具界面
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let t = self.t.clone();
        self.update_now();
        self.update_ts_result();
        self.update_dt_result();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(4.0);
            ui.label(egui::RichText::new(&t.ts_title).size(20.0).strong());
            ui.add_space(8.0);
            theme::card(ui, |ui| self.section_now(ui));
            ui.add_space(12.0);
            theme::card(ui, |ui| self.section_ts_to_time(ui));
            ui.add_space(12.0);
            theme::card(ui, |ui| self.section_time_to_ts(ui));
            ui.add_space(8.0);
        });
    }

    /// 刷新当前时间显示(秒级去重)
    ///
    /// 只有秒变化时才更新, 避免鼠标移动等事件触发重绘时时间疯狂跳动;
    /// 界面每秒由定时器重绘一次, 因此当前时间每秒更新一次。
    fn update_now(&mut self) {
        let now = Local::now();
        let s = now.timestamp();
        if s == self.now_s {
            return;
        }
        self.now_s = s;
        self.now_ms = now.timestamp_millis();
        self.now_time_str = now.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    }

    /// 强制刷新当前时间(刷新按钮点击, 不受秒级去重限制)
    fn force_update_now(&mut self) {
        let now = Local::now();
        self.now_s = now.timestamp();
        self.now_ms = now.timestamp_millis();
        self.now_time_str = now.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    }

    /// 当前时间区
    fn section_now(&mut self, ui: &mut egui::Ui) {
        let t = self.t.clone();
        theme::section_title(ui, &t.ts_section_now);

        egui::Grid::new("now_grid")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label(&t.ts_local_time);
                ui.horizontal(|ui| {
                    ui.monospace(&self.now_time_str);
                    if ui.small_button(&t.ts_refresh).clicked() {
                        self.force_update_now();
                    }
                });
                ui.end_row();

                ui.label(&t.ts_millis);
                ui.horizontal(|ui| {
                    ui.monospace(self.now_ms.to_string());
                    theme::copy_button(ui, &t.ts_copy, &self.now_ms.to_string());
                });
                ui.end_row();

                ui.label(&t.ts_seconds);
                ui.horizontal(|ui| {
                    ui.monospace(self.now_s.to_string());
                    theme::copy_button(ui, &t.ts_copy, &self.now_s.to_string());
                });
                ui.end_row();
            });
    }

    /// 时间戳 -> 时间字符串
    fn section_ts_to_time(&mut self, ui: &mut egui::Ui) {
        let t = self.t.clone();
        theme::section_title(ui, &t.ts_ts_to_time);

        ui.horizontal(|ui| {
            // 精确分配 70x20(与 add_sized 一致), 文字在区域内水平居左、垂直居中
            let (rect, _) = ui.allocate_exact_size(egui::vec2(70.0, 20.0), egui::Sense::hover());
            ui.painter().text(
                rect.right_center(),
                egui::Align2::RIGHT_CENTER,
                &t.ts_timestamp,
                egui::FontId::proportional(14.0),
                theme::TEXT,
            );

            ui.add(
                egui::TextEdit::singleline(&mut self.ts_input)
                    .hint_text(t.ts_hint_ts.clone())
                    .margin(egui::Margin::symmetric(8, 14))
                    .desired_width(360.0)
                    .vertical_align(egui::Align::Center),
            );
            if ui.button(&t.ts_fill_ms).clicked() {
                self.ts_input = self.now_ms.to_string();
            }
            if ui.button(&t.ts_fill_s).clicked() {
                self.ts_input = self.now_s.to_string();
            }
        });

        ui.horizontal(|ui| {
            ui.label(&t.ts_unit);
            if theme::selectable_label(ui, self.ts_unit == TsUnit::Auto, &t.ts_unit_auto) {
                self.ts_unit = TsUnit::Auto;
            }
            if theme::selectable_label(ui, self.ts_unit == TsUnit::Seconds, &t.ts_unit_s) {
                self.ts_unit = TsUnit::Seconds;
            }
            if theme::selectable_label(ui, self.ts_unit == TsUnit::Millis, &t.ts_unit_ms) {
                self.ts_unit = TsUnit::Millis;
            }
        });

        match self.ts_error {
            Some(err) => {
                ui.colored_label(theme::ERROR, err.msg(&t));
            }
            None if !self.ts_out_local.is_empty() => {
                ui.label(&t.ts_local_time);
                ui.horizontal(|ui| {
                    ui.monospace(&self.ts_out_local);
                    theme::copy_button(ui, &t.ts_copy, &self.ts_out_local);
                });
                ui.label(&t.ts_utc_time);
                ui.horizontal(|ui| {
                    ui.monospace(&self.ts_out_utc);
                    theme::copy_button(ui, &t.ts_copy, &self.ts_out_utc);
                });
            }
            None => {}
        }
    }

    /// 时间字符串 -> 时间戳
    fn section_time_to_ts(&mut self, ui: &mut egui::Ui) {
        let t = self.t.clone();
        theme::section_title(ui, &t.ts_time_to_ts);

        ui.horizontal(|ui| {
            // 精确分配 70x20(与 add_sized 一致), 文字在区域内水平居左、垂直居中
            let (rect, _) = ui.allocate_exact_size(egui::vec2(70.0, 20.0), egui::Sense::hover());
            ui.painter().text(
                rect.right_center(),
                egui::Align2::RIGHT_CENTER,
                &t.ts_time_str,
                egui::FontId::proportional(14.0),
                theme::TEXT,
            );

            ui.add(
                egui::TextEdit::singleline(&mut self.dt_input)
                    .hint_text(t.ts_hint_dt.clone())
                    .margin(egui::Margin::symmetric(8, 14))
                    .desired_width(360.0)
                    .vertical_align(egui::Align::Center),
            );
            if ui.button(&t.ts_fill_now_time).clicked() {
                self.dt_input = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
            }
        });

        match self.dt_error {
            Some(err) => {
                ui.colored_label(theme::ERROR, err.msg(&t));
            }
            None if !self.dt_out_ms.is_empty() => {
                ui.label(&t.ts_millis);
                ui.horizontal(|ui| {
                    ui.monospace(&self.dt_out_ms);
                    theme::copy_button(ui, &t.ts_copy, &self.dt_out_ms);
                });
                ui.label(&t.ts_seconds);
                ui.horizontal(|ui| {
                    ui.monospace(&self.dt_out_s);
                    theme::copy_button(ui, &t.ts_copy, &self.dt_out_s);
                });
            }
            None => {}
        }
    }

    /// 输入变化时重新计算 时间戳 -> 时间字符串
    fn update_ts_result(&mut self) {
        let key = (self.ts_unit, self.ts_input.clone());
        if self.ts_last_key == key {
            return;
        }
        self.ts_last_key = key;

        match timestamp_to_strings(&self.ts_input, self.ts_unit) {
            Ok((local, utc)) => {
                self.ts_error = None;
                self.ts_out_local = local;
                self.ts_out_utc = utc;
            }
            Err(e) => {
                self.ts_error = Some(e);
                self.ts_out_local.clear();
                self.ts_out_utc.clear();
            }
        }
    }

    /// 输入变化时重新计算 时间字符串 -> 时间戳
    fn update_dt_result(&mut self) {
        if self.dt_last_key == self.dt_input {
            return;
        }
        self.dt_last_key = self.dt_input.clone();

        match datetime_to_timestamps(&self.dt_input) {
            Ok((ms, s)) => {
                self.dt_error = None;
                self.dt_out_ms = ms.to_string();
                self.dt_out_s = s.to_string();
            }
            Err(e) => {
                self.dt_error = Some(e);
                self.dt_out_ms.clear();
                self.dt_out_s.clear();
            }
        }
    }
}

/// 时间戳转换为本地时间与 UTC 时间字符串
fn timestamp_to_strings(input: &str, unit: TsUnit) -> Result<(String, String), TsErr> {
    let raw: i64 = input
        .trim()
        .parse()
        .map_err(|_| TsErr::NotInteger)?;

    let dt: DateTime<Local> = match effective_unit(unit, raw) {
        TsUnit::Seconds => Local
            .timestamp_opt(raw, 0)
            .single()
            .ok_or(TsErr::OutOfRange)?,
        _ => Local
            .timestamp_millis_opt(raw)
            .single()
            .ok_or(TsErr::OutOfRange)?,
    };

    let fmt = "%Y-%m-%d %H:%M:%S%.3f";
    Ok((
        dt.format(fmt).to_string(),
        dt.with_timezone(&chrono::Utc).format(fmt).to_string(),
    ))
}

/// 时间字符串转换为 (毫秒时间戳, 秒时间戳)
fn datetime_to_timestamps(input: &str) -> Result<(i64, i64), TsErr> {
    let s = input.trim();
    if s.is_empty() {
        return Err(TsErr::Empty);
    }

    // 1. 尝试带时区的 ISO8601 / RFC3339
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok((dt.timestamp_millis(), dt.timestamp()));
    }

    // 2. 尝试本地时间格式
    for fmt in DATETIME_FORMATS {
        if let Ok(ndt) = NaiveDateTime::parse_from_str(s, fmt) {
            let local = local_from_naive(ndt)?;
            return Ok((local.timestamp_millis(), local.timestamp()));
        }
    }

    // 3. 尝试仅日期格式(默认 00:00:00)
    for fmt in DATE_FORMATS {
        if let Ok(nd) = NaiveDate::parse_from_str(s, fmt) {
            let ndt = nd.and_hms_opt(0, 0, 0).ok_or(TsErr::ParseFailed)?;
            let local = local_from_naive(ndt)?;
            return Ok((local.timestamp_millis(), local.timestamp()));
        }
    }

    Err(TsErr::ParseFailed)
}

/// 本地时区无歧义地将 NaiveDateTime 转为带时区的本地时间
fn local_from_naive(ndt: NaiveDateTime) -> Result<DateTime<Local>, TsErr> {
    Local
        .from_local_datetime(&ndt)
        .single()
        .ok_or(TsErr::Ambiguous)
}

/// 确定实际使用的时间戳单位
fn effective_unit(unit: TsUnit, raw: i64) -> TsUnit {
    match unit {
        TsUnit::Auto => {
            if raw.unsigned_abs().to_string().len() <= 10 {
                TsUnit::Seconds
            } else {
                TsUnit::Millis
            }
        }
        other => other,
    }
}
