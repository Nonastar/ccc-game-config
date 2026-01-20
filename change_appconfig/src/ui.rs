use crate::config_manager::AppConfig;
use crate::json_handler::{find_json_files, read_json_config, write_json_config};
use crate::js_handler::{find_js_files, read_js_config, write_js_config};
use eframe::egui;
use std::path::PathBuf;

/// 设置自定义字体以支持中文显示
/// 尝试加载 Windows 系统自带的中文字体（微软雅黑、黑体、宋体）
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 尝试加载系统字体
    let font_paths = [
        "c:\\Windows\\Fonts\\msyh.ttc", // 微软雅黑
        "c:\\Windows\\Fonts\\simhei.ttf", // 黑体
        "c:\\Windows\\Fonts\\simsun.ttc", // 宋体
    ];

    let mut font_data = None;
    for path in font_paths {
        if let Ok(data) = std::fs::read(path) {
            font_data = Some(data);
            break;
        }
    }

    if let Some(data) = font_data {
        // 注册字体数据
        fonts.font_data.insert(
            "my_font".to_owned(),
            egui::FontData::from_owned(data),
        );

        // 将自定义字体设置为比例字体（Proportional）的首选，用于常规文本显示
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "my_font".to_owned());

        // 将自定义字体作为等宽字体（Monospace）的备选
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("my_font".to_owned());

        ctx.set_fonts(fonts);
    }
}

/// 应用程序主状态结构体
pub struct BytegameConfigEditor {
    /// 当前选择的项目目录
    project_dir: PathBuf,
    /// 当前加载的配置
    config: AppConfig,
    /// 用户正在编辑的新配置
    new_config: AppConfig,
    /// 状态栏显示的消息
    status_message: String,
    /// 是否正在进行修改操作（用于显示加载动画）
    is_modifying: bool,
    /// 是否显示成功提示
    show_success: bool,
    /// 记录本次操作修改了哪些文件
    modified_files: Vec<String>,
    /// 预览图片列表，存储图片的 URI 和二进制数据
    preview_images: Vec<(String, Vec<u8>)>,
}

impl BytegameConfigEditor {
    /// 创建新的应用程序实例
    ///
    /// # 参数
    /// * `cc` - eframe 创建上下文
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 初始化字体和图片加载器
        setup_custom_fonts(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        
        Self {
            project_dir: PathBuf::new(),
            config: AppConfig::new(),
            new_config: AppConfig::new(),
            status_message: String::from("请选择字节跳动小游戏项目目录"),
            is_modifying: false,
            show_success: false,
            modified_files: Vec::new(),
            preview_images: Vec::new(),
        }
    }

    /// 加载项目配置
    /// 扫描目录下的 JSON 和 JS 文件，提取配置信息，并查找预览图片
    fn load_config(&mut self) {
        if self.project_dir.as_os_str().is_empty() {
            return;
        }

        self.config = AppConfig::new();

        // 尝试查找预览图片 (PNG)，且宽度必须为 750px
        self.preview_images.clear();
        let walker = walkdir::WalkDir::new(&self.project_dir).into_iter();
        for entry in walker.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "png") {
                 // 找到一个 PNG 文件，读取并检查宽度
                 if let Ok(data) = std::fs::read(path) {
                     // 尝试加载图片获取尺寸
                     if let Ok(img) = image::load_from_memory(&data) {
                         if img.width() == 750 {
                             // 将路径转换为 file URI 格式，并确保使用正斜杠
                             let uri = format!("file:///{}", path.display().to_string().replace("\\", "/"));
                             self.preview_images.push((uri, data));
                         }
                     }
                 }
            }
        }

        // 读取 JSON 配置 (project.config.json)
        let json_files = find_json_files(&self.project_dir);
        for file in json_files {
            if let Ok(cfg) = read_json_config(&file) {
                self.config.appid = cfg.appid;
                self.config.appname = cfg.appname;
                self.status_message = format!("成功加载配置: {}", file.display());
            }
        }

        // 读取 JS 配置 (查找包含 appId 和 douyinIds 的 JS 文件)
        let js_files = find_js_files(&self.project_dir);
        for file in js_files {
            if let Ok(cfg) = read_js_config(&file) {
                let mut found = false;
                if !cfg.app_id.is_empty() {
                    self.config.app_id = cfg.app_id.clone();
                    found = true;
                }
                if !cfg.douyin_ids.is_empty() {
                    self.config.douyin_ids = cfg.douyin_ids.clone();
                    found = true;
                }
                if found {
                    break;
                }
            }
        }

        // 初始化新配置为当前值，以便用户编辑
        self.new_config = self.config.clone();
    }

    /// 应用用户修改的配置
    /// 将新配置写入到 JSON 和 JS 文件中
    fn apply_modifications(&mut self) {
        if self.project_dir.as_os_str().is_empty() {
            self.status_message = String::from("请先选择项目目录");
            return;
        }

        // 格式化 douyinIds: 按逗号分割，确保每个ID都有双引号
        if !self.new_config.douyin_ids.is_empty() {
            let formatted_ids = self.new_config.douyin_ids
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| {
                    if s.starts_with('"') && s.ends_with('"') {
                        s.to_string()
                    } else {
                        format!("\"{}\"", s)
                    }
                })
                .collect::<Vec<String>>()
                .join(",");
            self.new_config.douyin_ids = formatted_ids;
        }

        // 同步 appid 到 app_id (确保 JSON 和 JS 使用相同的值)
        // 用户只需输入一次 AppId，程序会自动同步到两个字段
        self.new_config.app_id = self.new_config.appid.clone();

        self.is_modifying = true;
        self.modified_files.clear();

        // 修改 JSON 文件
        let json_files = find_json_files(&self.project_dir);
        for file in &json_files {
            match write_json_config(file, &self.new_config) {
                Ok(_) => {
                    self.modified_files.push(format!("JSON: {}", file.display()));
                }
                Err(e) => {
                    self.status_message = format!("修改 JSON 失败: {}", e);
                    self.is_modifying = false;
                    return;
                }
            }
        }

        // 修改 JS 文件
        // 遍历所有 JS 文件并尝试替换，只有真正修改了内容的文件才会被记录
        let js_files = find_js_files(&self.project_dir);
        for file in js_files {
            match write_js_config(&file, &self.new_config) {
                Ok(modified) => {
                    if modified {
                        self.modified_files.push(format!("JS: {}", file.display()));
                    }
                }
                Err(e) => {
                    self.status_message = format!("修改 JS 失败: {}", e);
                    // 继续尝试修改其他文件，不立即停止
                }
            }
        }

        self.is_modifying = false;
        self.show_success = true;
        self.status_message = format!(
            "成功修改 {} 个文件",
            self.modified_files.len()
        );
        // 更新当前配置为新配置
        self.config = self.new_config.clone();
    }

    /// 重置配置
    /// 将编辑框中的值恢复为最初加载的配置值
    fn reset(&mut self) {
        self.new_config = self.config.clone();
        self.status_message = String::from("已重置为原始值");
    }
}

impl eframe::App for BytegameConfigEditor {
    /// GUI 更新循环
    /// 每一帧绘制 UI 界面
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // 顶部标题
                ui.vertical_centered(|ui| {
                    ui.heading("字节跳动小游戏配置编辑器");
                    ui.add_space(10.0);
                });

                ui.separator();
                ui.add_space(10.0);

                // 目录选择区
                ui.horizontal(|ui| {
                    ui.label("项目目录:");
                    if ui.button("选择目录").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("选择字节跳动小游戏项目目录")
                            .pick_folder()
                        {
                            self.project_dir = path;
                            self.load_config();
                        }
                    }

                    if !self.project_dir.as_os_str().is_empty() {
                        ui.label(format!("{}", self.project_dir.display()));
                    }
                });

                ui.add_space(10.0);

                // 状态信息
                ui.label(&self.status_message);
                ui.add_space(10.0);

                // 如果未加载配置，显示提示信息
                if self.config.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.label("请选择包含 project.config.json 和 JS 文件的项目目录");
                    });
                    return;
                }

                // 配置展示与修改区
                egui::Grid::new("config_grid")
                    .num_columns(3)
                    .spacing([10.0, 10.0])
                    .striped(true)
                    .show(ui, |ui| {
                        // appid 输入框
                        ui.label("AppId:");
                        ui.label(&self.config.appid);
                        ui.text_edit_singleline(&mut self.new_config.appid);
                        ui.end_row();

                        // douyinIds 输入框
                        ui.label("douyinIds (JS):");
                        ui.label(&self.config.douyin_ids);
                        ui.text_edit_singleline(&mut self.new_config.douyin_ids);
                        ui.end_row();

                         // appname 输入框
                        ui.label("AppName:");
                        ui.label(&self.config.appname);
                        ui.text_edit_singleline(&mut self.new_config.appname);
                        ui.end_row();
                    });

                ui.add_space(20.0);

                // 操作按钮区
                ui.horizontal(|ui| {
                    if ui.button("应用修改").clicked() {
                        self.apply_modifications();
                    }

                    if ui.button("重置").clicked() {
                        self.reset();
                    }
                });

                ui.add_space(10.0);

                // 修改进度提示
                if self.is_modifying {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("正在修改配置...");
                    });
                }

                // 成功提示和修改文件列表
                if self.show_success {
                    ui.add_space(10.0);
                    ui.colored_label(egui::Color32::DARK_GREEN, "✓ 修改成功!");
                    ui.separator();
                    ui.label("已修改的文件:");
                    for file in &self.modified_files {
                        ui.label(format!("  - {}", file));
                    }
                }

                ui.add_space(20.0);
                
                // 预览图片区域
                // 显示所有宽度为 750px 的 PNG 图片
                if !self.preview_images.is_empty() {
                    ui.separator();
                    ui.heading(format!("图片预览 (共 {} 张, 宽度 750px)", self.preview_images.len()));
                    ui.add_space(10.0);
                    
                    // 使用水平滚动区域展示图片
                    egui::ScrollArea::horizontal().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for (uri, data) in &self.preview_images {
                                ui.vertical(|ui| {
                                    // 限制单张图片的显示区域宽度
                                    ui.set_max_width(400.0);
                                    ui.label(format!("路径: {}", uri));
                                    ui.add(
                                        egui::Image::from_bytes(uri.clone(), data.clone())
                                            .max_width(750.0) // 限制显示宽度
                                            .fit_to_original_size(0.5) // 缩放显示
                                    );
                                });
                                ui.add_space(20.0);
                            }
                        });
                    });
                }

                ui.add_space(20.0);
                ui.separator();
                // 底部状态栏
                ui.horizontal(|ui| {
                    ui.label("版本: 0.1.2");
                    ui.label(" | ");
                    ui.label("支持修改 AppId 和 douyinIds 配置，显示 750px 宽图片");
                });
            });
        });
    }
}
