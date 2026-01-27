use crate::model::ProjectItem;
use crate::scanner;
use eframe::egui;
use rfd::FileDialog;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Write};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// 应用程序的主状态结构体
/// 维护了整个应用程序的生命周期、数据和 UI 状态
pub struct MyApp {
    /// 当前扫描的根目录路径，None 表示尚未选择
    root_path: Option<PathBuf>,
    
    /// 扫描到的所有项目列表
    projects: Vec<ProjectItem>,
    
    // --- 批量修改输入缓存 ---
    // 这些字段绑定到 UI 的输入框，用于收集用户想要批量应用的值
    
    /// 批量修改的目标 AppID
    batch_appid: String,
    /// 批量修改的目标项目名称
    batch_projectname: String,
    /// 批量修改的目标 DouyinIDs (逗号分隔字符串)
    batch_douyin_ids: String,
    
    /// 底部状态栏显示的提示消息
    status_msg: String,
}

impl MyApp {
    /// 清空当前所有数据和缓存
    fn clear_data(&mut self) {
        self.projects.clear();
        self.batch_appid.clear();
        self.batch_projectname.clear();
        self.batch_douyin_ids.clear();
        self.status_msg.clear();
    }

    /// 应用程序初始化
    /// 在此配置 egui 上下文、字体和安装必要的扩展（如图片加载器）
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 安装图片加载器，支持 png, jpeg 等格式的显示
        // 如果不安装，egui::Image 将无法加载本地文件
        egui_extras::install_image_loaders(&cc.egui_ctx);
        
        // 配置自定义字体（主要为了支持中文字符）
        Self::configure_fonts(&cc.egui_ctx);
        
        // 返回默认状态
        Self::default()
    }

    /// 配置字体
    /// 尝试加载系统中的 "微软雅黑" 字体，以确保中文能正常显示
    fn configure_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        // 尝试加载系统字体 (Windows: 微软雅黑)
        // 注意：这里硬编码了路径，仅适用于 Windows。跨平台需要更复杂的逻辑。
        // TODO: 在非 Windows 平台上添加 fallback 逻辑
        let font_path = "C:\\Windows\\Fonts\\msyh.ttc";
        
        if let Ok(font_data) = fs::read(font_path) {
            // 将字体数据加载到 egui 的字体系统中
            fonts.font_data.insert(
                "Microsoft YaHei".to_owned(),
                egui::FontData::from_owned(font_data),
            );

            // 设置为 Proportional (非等宽) 和 Monospace (等宽) 的首选字体
            if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                vec.insert(0, "Microsoft YaHei".to_owned());
            }
            if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                vec.insert(0, "Microsoft YaHei".to_owned());
            }

            // 应用新的字体配置
            ctx.set_fonts(fonts);
        } else {
            eprintln!("Warning: Failed to load Microsoft YaHei font from {}", font_path);
        }
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            root_path: None,
            projects: Vec::new(),
            batch_appid: String::new(),
            batch_projectname: String::new(),
            batch_douyin_ids: String::new(),
            status_msg: "准备就绪。请选择包含小游戏项目的文件夹。".to_owned(),
        }
    }
}

impl MyApp {
    /// 执行扫描操作
    /// 调用 scanner 模块扫描 root_path 下的所有项目
    fn scan(&mut self) {
        if let Some(path) = &self.root_path {
            self.status_msg = "正在扫描...".to_string();
            self.projects = scanner::scan_directory(path);
            self.status_msg = format!("扫描完成，共找到 {} 个配置文件", self.projects.len());
        }
    }

    /// 保存所有已修改的项目
    /// 遍历项目列表，只保存标记为 `is_modified` 的项目
    fn save_all(&mut self) {
        let mut success = 0;
        let mut fail = 0;
        
        for item in &mut self.projects {
            if item.is_modified {
                match scanner::save_project_item(item) {
                    Ok(_) => {
                        item.is_modified = false;
                        success += 1;
                    }
                    Err(e) => {
                        eprintln!("保存失败 {:?}: {}", item.path, e);
                        fail += 1;
                    }
                }
            }
        }
        self.status_msg = format!("保存结束：成功 {} 个，失败 {} 个", success, fail);
    }
    
    /// 批量应用 AppID
    /// 将 batch_appid 的值应用到所有选中的项目
    fn apply_batch_appid(&mut self) {
        if self.batch_appid.trim().is_empty() { return; }
        for item in &mut self.projects {
            if item.selected {
                // 更新 JSON 配置中的 appid
                item.config.appid = self.batch_appid.clone();
                // 同时更新 JS 中的 AppID
                if let Some(js) = &mut item.js_config {
                    js.app_id = self.batch_appid.clone();
                }
                item.is_modified = true;
            }
        }
        self.status_msg = "已批量应用 AppID (含JS)，请点击保存生效。".to_string();
    }

    /// 批量应用项目名称
    fn apply_batch_name(&mut self) {
        if self.batch_projectname.trim().is_empty() { return; }
        for item in &mut self.projects {
            if item.selected {
                item.config.projectname = self.batch_projectname.clone();
                item.is_modified = true;
            }
        }
        self.status_msg = "已批量应用项目名称，请点击保存生效。".to_string();
    }

    /// 批量应用 DouyinIDs
    /// 仅针对存在 JS 配置的项目
    fn apply_batch_douyin_ids(&mut self) {
        if self.batch_douyin_ids.trim().is_empty() { return; }
        
        // 移除所有空格和换行
        let cleaned_ids = self.batch_douyin_ids.replace(|c: char| c.is_whitespace(), "");
        self.batch_douyin_ids = cleaned_ids.clone();

        for item in &mut self.projects {
            if item.selected {
                if let Some(js) = &mut item.js_config {
                    js.douyin_ids_str = cleaned_ids.clone();
                    item.is_modified = true;
                }
            }
        }
        self.status_msg = "已批量应用 DouyinIDs (仅JS)，请点击保存生效。".to_string();
    }

    /// 将项目目录打包为 ZIP 压缩包
    fn build_zip(&mut self, index: usize) {
        let item = &self.projects[index];
        // 获取 project.config.json 所在的目录
        let config_dir = match item.path.parent() {
            Some(p) => p,
            None => {
                self.status_msg = "错误：无法获取配置文件所在目录".to_string();
                return;
            }
        };

        // 打包父目录：获取 config_dir 的父目录
        let project_root = match config_dir.parent() {
            Some(p) => p,
            None => {
                // 如果没有父目录（即 config_dir 已经是根目录），则回退到 config_dir
                config_dir
            }
        };

        let project_name = if item.config.projectname.is_empty() {
            project_root.file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "project".to_string())
        } else {
            item.config.projectname.clone()
        };

        let zip_filename = format!("{}.zip", project_name);
        // 压缩包放在 project_root 的同级目录下
        let zip_path = match project_root.parent() {
            Some(p) => p.join(&zip_filename),
            None => project_root.join(&zip_filename),
        };

        self.status_msg = format!("正在打包父目录: {} ...", zip_filename);

        match self.create_zip(project_root, &zip_path) {
            Ok(_) => {
                self.status_msg = format!("打包成功: {}", zip_path.display());
                // 自动打开所在的文件夹
                if let Some(parent) = zip_path.parent() {
                    let _ = open::that(parent);
                }
            }
            Err(e) => {
                self.status_msg = format!("打包失败: {}", e);
            }
        }
    }

    /// 创建 ZIP 文件的辅助函数
    fn create_zip(&self, src_dir: &Path, dst_file: &Path) -> anyhow::Result<()> {
        let file = File::create(dst_file)?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        let mut buffer = Vec::new();
        let mut it = WalkDir::new(src_dir).into_iter();

        while let Some(entry) = it.next() {
            let entry = entry?;
            let path = entry.path();
            let name = path.strip_prefix(src_dir)?;

            if name.as_os_str().is_empty() {
                continue;
            }

            // 跳过一些不必要的文件夹和文件
            if path.is_dir() {
                let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if dir_name == "node_modules" || dir_name == ".git" || dir_name == ".svn" {
                    it.skip_current_dir();
                    continue;
                }
                
                zip.add_directory(name.to_string_lossy(), options)?;
            } else {
                // 跳过当前的 zip 文件（如果它碰巧在源目录中）
                if path == dst_file {
                    continue;
                }

                zip.start_file(name.to_string_lossy(), options)?;
                let mut f = File::open(path)?;
                f.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
                buffer.clear();
            }
        }

        zip.finish()?;
        Ok(())
    }
}

impl eframe::App for MyApp {
    /// 每一帧的 UI 更新函数
    /// 这里定义了整个应用程序的 UI 布局
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut zip_index = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            // --- 顶部工具栏 ---
            ui.horizontal(|ui| {
                ui.heading("🛠️ 字节小游戏配置助手");
                // 右对齐按钮
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("📂 选择根目录").clicked() {
                        // 打开文件夹选择对话框
                        if let Some(path) = FileDialog::new().pick_folder() {
                            self.clear_data();
                            self.root_path = Some(path);
                            self.scan();
                        }
                    }
                    // 仅当已选择路径时显示刷新按钮
                    if self.root_path.is_some() && ui.button("🔄 刷新列表").clicked() {
                        self.scan();
                    }
                });
            });
            
            // 显示当前路径
            if let Some(path) = &self.root_path {
                ui.horizontal(|ui| {
                    ui.small(format!("当前路径: {}", path.display()));
                    if ui.button("📁 打开").clicked() {
                        // 使用系统默认文件管理器打开目录
                        let _ = open::that(path);
                    }
                });
            }
            
            ui.separator();

            // --- 批量操作区 ---
            // 仅在有项目时显示
            if !self.projects.is_empty() {
                ui.group(|ui| {
                    ui.label(egui::RichText::new("批量修改 (仅针对选中项目)").strong());
                    
                    let label_width = 90.0; // 固定标签宽度以对齐输入框
                    
                    // Row 1: AppID
                    ui.horizontal(|ui| {
                        ui.add_sized([label_width, 20.0], egui::Label::new("统一 AppID:"));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("应用").clicked() { self.apply_batch_appid(); }
                            ui.add(egui::TextEdit::singleline(&mut self.batch_appid).desired_width(f32::INFINITY));
                        });
                    });
                    
                    // Row 2: Project Name
                    ui.horizontal(|ui| {
                        ui.add_sized([label_width, 20.0], egui::Label::new("统一项目名:"));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("应用").clicked() { self.apply_batch_name(); }
                            ui.add(egui::TextEdit::singleline(&mut self.batch_projectname).desired_width(f32::INFINITY));
                        });
                    });

                    // Row 3: DouyinIDs
                    ui.horizontal(|ui| {
                        ui.add_sized([label_width, 20.0], egui::Label::new("统一 DouyinIDs:"));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("应用").clicked() { self.apply_batch_douyin_ids(); }
                            ui.add(egui::TextEdit::singleline(&mut self.batch_douyin_ids).desired_width(f32::INFINITY));
                        });
                    });
                    
                    ui.add_space(5.0);
                    
                    // 保存按钮，使用醒目的颜色和大小
                    if ui.add_sized(
                        [ui.available_width(), 30.0],
                        egui::Button::new(egui::RichText::new("💾 保存所有更改").heading().color(egui::Color32::WHITE))
                        .fill(egui::Color32::from_rgb(0, 100, 200))
                    ).clicked() 
                    {
                        self.save_all();
                    }
                });
            }

            ui.add_space(10.0);

            // --- 列表显示区 ---
            // 使用 ScrollArea 支持滚动
            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.projects.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.label("暂无项目，请选择正确的根目录。");
                    });
                } else {
                    for (idx, item) in self.projects.iter_mut().enumerate() {
                        // 使用 push_id 确保每个组件 ID 唯一
                        ui.push_id(idx, |ui| {
                            ui.group(|ui| {
                                // 项目标题行
                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut item.selected, "");
                                    
                                    // 显示相对路径或文件夹名作为标题
                                    let display_name = item.path.parent()
                                        .and_then(|p| p.file_name())
                                        .map(|s| s.to_string_lossy())
                                        .unwrap_or_default();
                                        
                                    ui.heading(display_name);
                                    
                                    if item.is_modified {
                                        ui.label(egui::RichText::new("● 待保存").color(egui::Color32::RED));
                                    }
                                    
                                    ui.add_space(5.0);
                                    if ui.button("📦 打包").clicked() {
                                        zip_index = Some(idx);
                                    }
                                });
                                
                                // 基础信息编辑
                                ui.horizontal(|ui| {
                                    ui.label("AppID:");
                                    if ui.text_edit_singleline(&mut item.config.appid).changed() {
                                        item.is_modified = true;
                                    }
                                    
                                    ui.add_space(20.0);
                                    
                                    ui.label("Name:");
                                    if ui.text_edit_singleline(&mut item.config.projectname).changed() {
                                        item.is_modified = true;
                                    }
                                });

                                // JS 配置编辑（如果存在）
                                if let Some(js_config) = &mut item.js_config {
                                    ui.separator();
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("JS Config:").small().strong());
                                        ui.label(egui::RichText::new("AppID").small());
                                        if ui.text_edit_singleline(&mut js_config.app_id).changed() {
                                            item.is_modified = true;
                                        }
                                        ui.label(egui::RichText::new("Douyin IDs").small());
                                        if ui.text_edit_singleline(&mut js_config.douyin_ids_str).changed() {
                                            // 自动移除空格和换行
                                            js_config.douyin_ids_str = js_config.douyin_ids_str.replace(|c: char| c.is_whitespace(), "");
                                            item.is_modified = true;
                                        }
                                    });
                                }
                                
                                // 图片预览区
                                if !item.image_paths.is_empty() {
                                    ui.separator();
                                    ui.label(egui::RichText::new(format!("预览图 (共{}张):", item.image_paths.len())).small().strong());
                                    
                                    // 显示图片路径列表（方便调试）
                                    ui.collapsing("查看图片路径", |ui| {
                                        for img_path in &item.image_paths {
                                            ui.label(egui::RichText::new(img_path.to_string_lossy()).monospace().small());
                                        }
                                    });

                                    // 使用 columns 布局并排显示所有图片
                                    ui.columns(item.image_paths.len(), |columns| {
                                        for (img_idx, ui) in columns.iter_mut().enumerate() {
                                            let img_path = &item.image_paths[img_idx];
                                            
                                            ui.group(|ui| {
                                                ui.vertical_centered(|ui| {
                                                    ui.label(egui::RichText::new(format!("Image #{}:", img_idx + 1)).small().strong());
                                                    
                                                    // 检查缓存，如果未加载则尝试加载
                                                    if !item.texture_cache.contains_key(img_path) {
                                                        // 尝试加载图片文件
                                                        let texture = if let Ok(img) = image::open(img_path) {
                                                            let size = [img.width() as _, img.height() as _];
                                                            let image_buffer = img.to_rgba8();
                                                            let pixels = image_buffer.as_flat_samples();
                                                            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                                                size,
                                                                pixels.as_slice(),
                                                            );
                                                            // 加载到 GPU 纹理
                                                            // 使用特定的名称 (idx, img_idx) 确保唯一性
                                                            Some(ui.ctx().load_texture(
                                                                format!("p{}_img{}", idx, img_idx),
                                                                color_image,
                                                                egui::TextureOptions::default()
                                                            ))
                                                        } else {
                                                            None
                                                        };
                                                        item.texture_cache.insert(img_path.clone(), texture);
                                                    }

                                                    // 显示图片或错误信息
                                                    if let Some(Some(texture)) = item.texture_cache.get(img_path) {
                                                        // max_width 限制图片宽度适应列宽
                                                        ui.add(egui::Image::new(texture).max_width(ui.available_width()));
                                                    } else {
                                                        ui.colored_label(egui::Color32::RED, "❌ 加载失败");
                                                        ui.label(egui::RichText::new(img_path.to_string_lossy()).small());
                                                    }
                                                });
                                            });
                                        }
                                    });
                                }
                                
                                // 显示配置文件路径（弱化显示）
                                ui.label(egui::RichText::new(item.path.to_string_lossy()).weak().small());
                            });
                        });
                        ui.add_space(4.0);
                    }
                }
            });

            // --- 底部状态栏 ---
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.separator();
                ui.label(&self.status_msg);
            });
        });

        if let Some(idx) = zip_index {
            self.build_zip(idx);
        }
    }
}
