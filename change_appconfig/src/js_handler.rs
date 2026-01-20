use crate::config_manager::AppConfig;
use anyhow::{Context, Result};
use regex::Regex;
use std::fs;

/// 读取 JS 配置文件
/// 从 JS 文件内容中提取 appId 和 douyinIds
///
/// # 参数
/// * `path` - JS 文件的路径
///
/// # 返回值
/// * `Result<AppConfig>` - 包含提取出的配置信息
pub fn read_js_config(path: &std::path::Path) -> Result<AppConfig> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("无法读取文件: {}", path.display()))?;

    // 提取配置字段
    let app_id = extract_app_id(&content).unwrap_or_default();
    let douyin_ids = extract_douyin_ids(&content).unwrap_or_default();

    Ok(AppConfig {
        appid: String::new(), // JS 文件不包含 appid (JSON 特有)
        app_id,
        douyin_ids,
        appname: String::new(), // JS 文件不包含 appname
    })
}

/// 将配置写入 JS 文件
/// 使用正则表达式替换文件中的 appId 和 douyinIds 字段
///
/// # 参数
/// * `path` - JS 文件的路径
/// * `config` - 包含新值的配置对象
///
/// # 返回值
/// * `Result<bool>` - 如果文件内容被修改返回 true，否则返回 false
pub fn write_js_config(path: &std::path::Path, config: &AppConfig) -> Result<bool> {
    let mut content = fs::read_to_string(path)
        .with_context(|| format!("无法读取文件: {}", path.display()))?;

    let original_content = content.clone();

    // 替换 appId
    // 匹配模式: appId="xxxx" 或 appId='xxxx'
    if let Ok(re) = Regex::new(r#"appId\s*=\s*["']([^"']*)["']"#) {
        let new_val = format!("appId=\"{}\"", config.app_id);
        if re.is_match(&content) {
            println!("Replacing appId in {}", path.display());
            content = re.replace_all(&content, new_val).to_string();
        } else {
            // println!("appId pattern not found in {}", path.display());
        }
    }

    // 替换 douyinIds
    // 匹配模式: douyinIds=[xxxx]
    if let Ok(re) = Regex::new(r#"douyinIds\s*=\s*\[([^\]]*)\]"#) {
        let new_val = format!("douyinIds=[{}]", config.douyin_ids);
        if re.is_match(&content) {
             println!("Replacing douyinIds in {}", path.display());
             content = re
                .replace_all(&content, new_val)
                .to_string();
        } else {
            // println!("douyinIds pattern not found in {}", path.display());
        }
    }

    // 只有当内容实际发生变化时才写入文件
    if content != original_content {
        fs::write(path, &content)
            .with_context(|| format!("无法写入文件: {}", path.display()))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 从内容中提取 appId
/// 查找 appId="value" 或 appId='value' 的模式
fn extract_app_id(content: &str) -> Option<String> {
    let re = Regex::new(r#"appId\s*=\s*["']([^"']*)["']"#).ok()?;
    re.captures(content)?.get(1).map(|m| m.as_str().to_string())
}

/// 从内容中提取 douyinIds
/// 查找 douyinIds=[value] 的模式
fn extract_douyin_ids(content: &str) -> Option<String> {
    let re = Regex::new(r#"douyinIds\s*=\s*\[([^\]]*)\]"#).ok()?;
    re.captures(content)?.get(1).map(|m| m.as_str().to_string())
}

/// 递归查找指定目录下的所有 .js 文件
///
/// # 参数
/// * `dir` - 根目录路径
///
/// # 返回值
/// * `Vec<std::path::PathBuf>` - 所有找到的 .js 文件路径列表
pub fn find_js_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    use walkdir::WalkDir;

    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "js")
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_app_id() {
        let content = r#"appId="test123""#;
        assert_eq!(extract_app_id(content), Some("test123".to_string()));
    }

    #[test]
    fn test_extract_douyin_ids() {
        let content = r#"douyinIds=["id1","id2"]"#;
        assert_eq!(extract_douyin_ids(content), Some("\"id1\",\"id2\"".to_string()));
    }

    #[test]
    fn test_extract_real_content() {
        // 测试真实场景下的代码片段
        let content = r#"d.rewardVideoAd=void 0,d.nowid=0,d.appId="appId",d.douyinIds=["id1","id2"],e._RF.pop()"#;
        assert_eq!(extract_app_id(content), Some("appId".to_string()));
        assert_eq!(extract_douyin_ids(content), Some("\"id1\",\"id2\"".to_string()));
        
        let mut new_content = content.to_string();
        let config = AppConfig {
            appid: "".to_string(),
            app_id: "new_app_id".to_string(),
            douyin_ids: "\"new_id1\",\"new_id2\"".to_string(),
            appname: "".to_string(),
        };
        
        // 手动应用写入逻辑进行测试
         if let Ok(re) = Regex::new(r#"appId\s*=\s*["']([^"']*)["']"#) {
            new_content = re.replace_all(&new_content, format!("appId=\"{}\"", config.app_id)).to_string();
        }
        if let Ok(re) = Regex::new(r#"douyinIds\s*=\s*\[([^\]]*)\]"#) {
            new_content = re.replace_all(&new_content, format!("douyinIds=[{}]", config.douyin_ids)).to_string();
        }
        
        assert!(new_content.contains(r#"d.appId="new_app_id""#));
        assert!(new_content.contains(r#"d.douyinIds=["new_id1","new_id2"]"#));
    }
}
