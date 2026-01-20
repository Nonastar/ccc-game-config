#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // 隐藏控制台窗口 (仅 Release 模式)

use eframe::egui;

mod config_manager;
mod json_handler;
mod js_handler;
mod ui;

use ui::BytegameConfigEditor;

/// 程序入口函数
/// 负责初始化应用程序窗口，加载图标，并启动 GUI
fn main() -> eframe::Result<()> {
    // 可选加载图标
    // 尝试从 assets/icon.png 读取图标数据
    // 如果读取成功，则转换为 eframe 可用的图标格式
    let icon = std::fs::read("assets/icon.png")
        .ok()
        .and_then(|data| eframe::icon_data::from_png_bytes(&data).ok());

    // 配置原生窗口选项
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0]) // 设置初始窗口大小
            .with_min_inner_size([800.0, 600.0]) // 设置最小窗口大小
            .with_icon(icon.unwrap_or_default()), // 设置窗口图标（如果加载成功）
        ..Default::default()
    };

    // 启动原生 GUI 应用
    eframe::run_native(
        "字节跳动小游戏配置编辑器", // 窗口标题
        options,
        Box::new(|cc| {
            // 初始化应用状态
            // cc (CreationContext) 包含 egui 上下文等信息
            Box::new(BytegameConfigEditor::new(cc))
        }),
    )
}
