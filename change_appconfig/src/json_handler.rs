use crate::config_manager::AppConfig;
use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// 读取 JSON 配置文件 (通常是 project.config.json)
///
/// # 参数
/// * `path` - JSON 文件的路径
///
/// # 返回值
/// * `Result<AppConfig>` - 成功则返回包含 appid 和 projectname 的 AppConfig，失败返回错误
pub fn read_json_config(path: &Path) -> Result<AppConfig> {
    // 读取文件内容
    let content = fs::read_to_string(path)
        .with_context(|| format!("无法读取文件: {}", path.display()))?;

    // 解析 JSON
    let json: Value = serde_json::from_str(&content)
        .with_context(|| format!("无法解析 JSON 文件: {}", path.display()))?;

    // 提取 appid，如果不存在则默认为空字符串
    let appid = json
        .get("appid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // 提取 projectname，如果不存在则默认为空字符串
    let appname = json
        .get("projectname")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(AppConfig {
        appid,
        app_id: String::new(), // JSON 文件不包含 JS 特有的字段
        douyin_ids: String::new(),
        appname,
    })
}

/// 写入 JSON 配置文件
///
/// # 参数
/// * `path` - JSON 文件的路径
/// * `config` - 包含新值的配置对象
///
/// # 返回值
/// * `Result<()>` - 成功返回 Ok(())，失败返回错误
pub fn write_json_config(path: &Path, config: &AppConfig) -> Result<()> {
    // 读取现有文件内容
    let content = fs::read_to_string(path)
        .with_context(|| format!("无法读取文件: {}", path.display()))?;

    // 解析 JSON
    let mut json: Value = serde_json::from_str(&content)
        .with_context(|| format!("无法解析 JSON 文件: {}", path.display()))?;

    // 更新 appid 字段
    if let Some(appid) = json.get_mut("appid") {
        *appid = Value::String(config.appid.clone());
    }

    // 更新 projectname 字段
    if let Some(projectname) = json.get_mut("projectname") {
        *projectname = Value::String(config.appname.clone());
    }

    // 序列化回字符串，使用 pretty print 保持格式
    let new_content = serde_json::to_string_pretty(&json)
        .with_context(|| "无法序列化 JSON")?;

    // 写入文件
    fs::write(path, new_content)
        .with_context(|| format!("无法写入文件: {}", path.display()))?;

    Ok(())
}

/// 在指定目录中查找 project.config.json 文件
///
/// # 参数
/// * `dir` - 要搜索的目录路径
///
/// # 返回值
/// * `Vec<std::path::PathBuf>` - 找到的文件路径列表（虽然通常只有一个）
pub fn find_json_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut json_files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name() {
                    // 只匹配文件名为 project.config.json 的文件
                    if name == "project.config.json" {
                        json_files.push(path);
                    }
                }
            }
        }
    }

    json_files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_json_config() {
        // 测试用例需要实际的测试文件
    }
}
