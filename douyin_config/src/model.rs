use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::fmt;

/// 对应 project.config.json 文件的结构体
/// 使用 serde 进行序列化和反序列化
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectConfig {
    /// 小游戏 AppID，默认为空字符串
    #[serde(default)]
    pub appid: String,
    
    /// 项目名称，默认为空字符串
    #[serde(default)]
    pub projectname: String,

    /// 使用 flatten 属性收集所有未显式定义的字段（如 setting, miniprogramRoot 等）
    /// 这确保了在读取和保存过程中，我们不关心的字段数据也能被原样保留，不会丢失
    #[serde(flatten)]
    pub extra: Value,
}

/// 对应 JS 配置文件（如 assets/main/index.js）中提取的配置信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JsConfig {
    /// 从 JS 代码中提取的 appId
    pub app_id: String,
    /// 从 JS 代码中提取的 douyinIds 列表
    pub douyin_ids: Vec<String>,
    /// 用于 UI 编辑的 douyinIds 字符串形式（逗号分隔）
    /// 使用 #[serde(skip)] 避免将其序列化到 JSON 中，这只是一个 UI 辅助字段
    #[serde(skip)]
    pub douyin_ids_str: String, 
}

/// UI 列表中单个项目的完整状态模型
#[derive(Clone)]
pub struct ProjectItem {
    /// 配置文件 (project.config.json) 的绝对路径
    pub path: PathBuf,
    /// 解析后的 project.config.json 配置内容
    pub config: ProjectConfig,
    /// 关联的 JS 配置文件路径（如果存在）
    pub js_path: Option<PathBuf>,
    /// 解析后的 JS 配置内容（如果存在）
    pub js_config: Option<JsConfig>,
    /// 项目目录下找到的符合条件的预览图路径列表
    pub image_paths: Vec<PathBuf>,
    /// 标记当前项目是否有未保存的修改
    pub is_modified: bool,
    /// 标记当前项目是否在 UI列表中被选中（用于批量操作）
    pub selected: bool,
    
    /// 图片纹理缓存
    /// key: 图片路径
    /// value: egui 纹理句柄（加载成功为 Some，失败为 None）
    /// 用于避免重复加载同一张图片，提高性能
    pub texture_cache: std::collections::HashMap<PathBuf, Option<egui::TextureHandle>>,
}

/// 手动实现 Debug trait 以优化输出格式，避免打印过长的 texture_cache 内容
impl fmt::Debug for ProjectItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProjectItem")
            .field("path", &self.path)
            .field("config", &self.config)
            .field("js_path", &self.js_path)
            .field("js_config", &self.js_config)
            .field("image_paths", &self.image_paths)
            .field("is_modified", &self.is_modified)
            .field("selected", &self.selected)
            // 仅打印缓存大小，而不是具体内容
            .field("texture_cache", &format!("HashMap(len={})", self.texture_cache.len()))
            .finish()
    }
}
