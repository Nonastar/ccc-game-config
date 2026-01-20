#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // 在 Release 模式下隐藏 Windows 控制台窗口，避免弹出黑色命令行窗口

// 声明项目中的模块
mod app;      // 应用程序主逻辑和 UI 定义
mod model;    // 数据模型定义
mod scanner;  // 文件扫描和处理逻辑

use app::MyApp;
use eframe::egui;

// 程序入口点
// 返回 eframe::Result<()> 以处理可能的启动错误
fn main() -> eframe::Result<()> {
    // 设置原生窗口选项
    let options = eframe::NativeOptions {
        // 配置视口（窗口）属性
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([750.0, 800.0]) // 设置初始窗口大小
            .with_title("Douyin Config Editor"), // 设置窗口标题
        ..Default::default()
    };
    
    // 启动 eframe 应用程序
    eframe::run_native(
        "Douyin Config Editor", // 应用程序名称（用于持久化存储等）
        options,
        // 创建应用程序实例的闭包
        // cc (CreationContext) 包含了 egui 的上下文，用于初始化字体、样式等
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}
