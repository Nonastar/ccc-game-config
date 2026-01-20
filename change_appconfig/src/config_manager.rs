use serde::{Deserialize, Serialize};

/// 应用程序配置结构体
/// 存储从 project.config.json 和 JS 文件中读取的配置信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// 对应 project.config.json 中的 "appid" 字段
    pub appid: String,        
    /// 对应 JS 文件中的 "appId" 字段
    pub app_id: String,       
    /// 对应 JS 文件中的 "douyinIds" 数组，存储为格式化后的字符串（如 "id1","id2"）
    pub douyin_ids: String,   
    /// 对应 project.config.json 中的 "projectname" 字段
    pub appname: String,      
}

impl AppConfig {
    /// 创建一个新的默认 AppConfig 实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 检查配置是否为空
    /// 如果所有字段都为空字符串，则返回 true
    pub fn is_empty(&self) -> bool {
        self.appid.is_empty() && self.app_id.is_empty() && self.douyin_ids.is_empty() && self.appname.is_empty()
    }
}
