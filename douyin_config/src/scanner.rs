use crate::model::{ProjectConfig, ProjectItem, JsConfig};
use std::fs;
use std::path::Path;
use regex::Regex;

use walkdir::WalkDir;

const CONFIG_FILENAME: &str = "project.config.json";

/// 扫描指定目录下的配置文件
/// 
/// 该函数会递归遍历目录，寻找 `project.config.json` 文件。
/// 找到配置文件后，会尝试进一步查找关联的 JS 配置文件（如 `assets/main/index.js`）
/// 以及项目中的预览图片（宽度为 750px 的图片）。
///
/// # Arguments
/// * `root` - 要扫描的根目录路径
///
/// # Returns
/// * `Vec<ProjectItem>` - 扫描到的项目列表
pub fn scan_directory(root: &Path) -> Vec<ProjectItem> {
    let mut results = Vec::new();
    
    // min_depth(1) 避免扫描根目录本身（如果根目录本身就是项目目录，可以改为0，但通常是选父级）
    // max_depth(5) 限制深度，防止遍历太深导致性能问题或不相关的扫描
    for entry in WalkDir::new(root).min_depth(1).max_depth(5).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() == CONFIG_FILENAME {
            let path = entry.path().to_path_buf();
            // 尝试加载 JSON 配置
            if let Ok(config) = load_config(&path) {
                // 尝试查找关联的 JS 文件
                let mut js_path = None;
                if let Some(parent) = path.parent() {
                    // 候选 JS 文件列表，按优先级排序查找
                    let candidates = ["assets/main/index.js", "application.js"];
                    
                    for candidate in candidates {
                        let target = parent.join(candidate);
                        if target.exists() {
                            // 简单的预检查：读取文件内容，检查是否包含 appId 或 douyinIds 关键字
                            // 这样可以避免解析无关的 JS 文件
                            if let Ok(content) = fs::read_to_string(&target) {
                                if content.contains("appId") || content.contains("douyinIds") {
                                    js_path = Some(target);
                                    break;
                                }
                            }
                        }
                    }
                }
                
                // 如果找到了 JS 文件，尝试解析其中的配置
                let mut js_config = None;
                if let Some(ref p) = js_path {
                    if let Ok(cfg) = load_js_config(p) {
                        js_config = Some(cfg);
                    } else {
                        eprintln!("Failed to load JS config from {:?}", p);
                    }
                }

                // 查找预览图片 (匹配任意图片文件)
                // 策略：扫描项目根目录下的所有文件，寻找符合特定条件的图片
                let mut image_paths = Vec::new();
                if let Some(project_root) = path.parent() {
                     for entry in WalkDir::new(project_root).into_iter().filter_map(|e| e.ok()) {
                        let p = entry.path();
                        if p.is_file() {
                             if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                                match ext.to_lowercase().as_str() {
                                    "png" | "jpg" | "jpeg" | "bmp" | "webp" => {
                                        // 检查图片宽度是否为 750
                                        // 这是一个特定的业务规则，用于识别特定的预览图
                                        if let Ok(img) = image::open(p) {
                                            if img.width() == 750 {
                                                image_paths.push(p.to_path_buf());
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                             }
                        }
                     }
                }

                // 构建完整的项目项并添加到结果列表
                results.push(ProjectItem {
                    path,
                    config,
                    js_path,
                    js_config,
                    image_paths,
                    is_modified: false,
                    selected: true, // 默认选中，方便用户直接进行批量操作
                    texture_cache: std::collections::HashMap::new(),
                });
            }
        }
    }
    results
}

/// 加载并解析 project.config.json 文件
fn load_config(path: &Path) -> anyhow::Result<ProjectConfig> {
    let content = fs::read_to_string(path)?;
    let config = serde_json::from_str(&content)?;
    Ok(config)
}

/// 加载并解析 JS 配置文件
/// 使用正则表达式提取配置，因为 JS 文件不是标准的 JSON
fn load_js_config(path: &Path) -> anyhow::Result<JsConfig> {
    let content = fs::read_to_string(path)?;
    
    // 匹配 .appId="xxx" 或 .appId='xxx'
    // 捕获组 1 为 appId 的值
    let re_app_id = Regex::new(r#"\.appId\s*=\s*["']([^"']+)["']"#).unwrap();
    // 匹配 .douyinIds=["xxx", "yyy"]
    // 捕获组 1 为数组内部的字符串
    let re_douyin_ids = Regex::new(r#"\.douyinIds\s*=\s*\[(.*?)\]"#).unwrap();

    let app_id = re_app_id.captures(&content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();

    let mut douyin_ids = Vec::new();
    if let Some(cap) = re_douyin_ids.captures(&content) {
        if let Some(array_str) = cap.get(1) {
            let inner = array_str.as_str();
            // 分割数组内容并清理引号
            for part in inner.split(',') {
                let trimmed = part.trim();
                let trim_matches: &[_] = &['"', '\''];
                let id = trimmed.trim_matches(trim_matches);
                if !id.is_empty() {
                    douyin_ids.push(id.to_string());
                }
            }
        }
    }

    Ok(JsConfig {
        app_id,
        douyin_ids: douyin_ids.clone(),
        douyin_ids_str: douyin_ids.join(","), // 生成用于 UI 编辑的字符串
    })
}

/// 保存 JS 配置文件
/// 使用正则表达式进行替换，以保留原文件的格式和注释
fn save_js_config(path: &Path, config: &JsConfig) -> anyhow::Result<()> {
    let mut content = fs::read_to_string(path)?;
    
    // 替换 appId
    // 查找模式：(.appId\s*=\s*["'])原始内容(["'])
    // 替换为：$1新内容$2
    let re_app_id_replace = Regex::new(r#"(\.appId\s*=\s*["'])[^"']+(["'])"#).unwrap();
    content = re_app_id_replace.replace(&content, |caps: &regex::Captures| {
        format!("{}{}{}", &caps[1], config.app_id, &caps[2])
    }).to_string();

    // 替换 douyinIds
    // 首先从 douyin_ids_str 解析出 ID 列表，以支持用户在 UI 中的修改
    let current_ids: Vec<String> = config.douyin_ids_str.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // 重新构建 JS 数组字符串： "id1","id2"
    let ids_str = current_ids.iter()
        .map(|id| format!(r#""{}""#, id))
        .collect::<Vec<_>>()
        .join(",");
    
    // 替换整个数组内容
    let re_douyin_ids_replace = Regex::new(r#"(\.douyinIds\s*=\s*\[).*?(\])"#).unwrap();
    content = re_douyin_ids_replace.replace(&content, |caps: &regex::Captures| {
        format!("{}{}{}", &caps[1], ids_str, &caps[2])
    }).to_string();

    fs::write(path, content)?;
    Ok(())
}

/// 保存单个项目的所有配置（包括 JSON 和 JS）
pub fn save_project_item(item: &ProjectItem) -> anyhow::Result<()> {
    // 保存 JSON 配置文件
    // 使用 pretty print 格式化输出，方便人类阅读
    let content = serde_json::to_string_pretty(&item.config)?;
    fs::write(&item.path, content)?;
    
    // 如果存在 JS 配置，也一并保存
    if let (Some(js_path), Some(js_config)) = (&item.js_path, &item.js_config) {
        save_js_config(js_path, js_config)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::model::ProjectConfig;
    use regex::Regex;
    use std::fs;

    #[test]
    fn test_config_flatten() {
        let json_data = r#"{
            "appid": "tt123456",
            "projectname": "TestGame",
            "setting": {
                "es6": true
            },
            "compileType": "miniprogram"
        }"#;

        let mut config: ProjectConfig = serde_json::from_str(json_data).expect("解析失败");
        assert_eq!(config.appid, "tt123456");
        assert_eq!(config.projectname, "TestGame");
        config.appid = "tt_new_id".to_string();
        
        let new_json = serde_json::to_string(&config).expect("序列化失败");
        let v: serde_json::Value = serde_json::from_str(&new_json).unwrap();
        
        assert_eq!(v["appid"], "tt_new_id");
        assert_eq!(v["setting"]["es6"], true);
    }

    #[test]
    fn test_js_config_parsing() {
        let js_content = r#"
            // some code
            d.appId="old_app_id",d.douyinIds=["id1","id2"],e._RF.pop()
            // more code
        "#;
        
        let re_app_id = Regex::new(r#"\.appId\s*=\s*["']([^"']+)["']"#).unwrap();
        let app_id = re_app_id.captures(js_content).unwrap().get(1).unwrap().as_str();
        assert_eq!(app_id, "old_app_id");

        let re_douyin_ids = Regex::new(r#"\.douyinIds\s*=\s*\[(.*?)\]"#).unwrap();
        let ids_str = re_douyin_ids.captures(js_content).unwrap().get(1).unwrap().as_str();
        assert_eq!(ids_str, r#""id1","id2""#);

        let new_app_id = "new_app_id";
        let re_app_id_replace = Regex::new(r#"(\.appId\s*=\s*["'])[^"']+(["'])"#).unwrap();
        let new_content = re_app_id_replace.replace(js_content, |caps: &regex::Captures| {
            format!("{}{}{}", &caps[1], new_app_id, &caps[2])
        });
        
        assert!(new_content.contains(r#"d.appId="new_app_id""#));
        
        let new_ids_str = r#""new1","new2""#;
        let re_douyin_ids_replace = Regex::new(r#"(\.douyinIds\s*=\s*\[).*?(\])"#).unwrap();
        let new_content_2 = re_douyin_ids_replace.replace(&new_content, |caps: &regex::Captures| {
            format!("{}{}{}", &caps[1], new_ids_str, &caps[2])
        });
        
        assert!(new_content_2.contains(r#"d.douyinIds=["new1","new2"]"#));
    }

    #[test]
    fn test_full_workflow() {
        use std::path::Path;
        // Setup
        let test_dir = Path::new("test_output");
        if test_dir.exists() { fs::remove_dir_all(test_dir).unwrap(); }
        fs::create_dir_all(test_dir.join("assets/main")).unwrap();
        
        let config_path = test_dir.join("project.config.json");
        let js_path = test_dir.join("assets/main/index.js");
        
        fs::write(&config_path, r#"{"appid": "old_id", "projectname": "old_name"}"#).unwrap();
        fs::write(&js_path, r#"d.appId="old_id",d.douyinIds=["id1"]"#).unwrap();
        
        // 1. Scan
        let mut items = crate::scanner::scan_directory(test_dir);
        assert_eq!(items.len(), 1);
        let item = &mut items[0];
        
        assert_eq!(item.config.appid, "old_id");
        assert_eq!(item.js_config.as_ref().unwrap().app_id, "old_id");
        
        // 2. Modify (Simulate Batch Update)
        let new_id = "new_batch_id";
        item.config.appid = new_id.to_string();
        if let Some(js) = &mut item.js_config {
            js.app_id = new_id.to_string();
            // 注意：douyin_ids_str 的格式取决于用户输入，这里模拟用户输入逗号分隔的字符串（无引号）
            js.douyin_ids_str = "new_d1,new_d2".to_string();
        }
        
        // 3. Save
        crate::scanner::save_project_item(item).unwrap();
        
        // 4. Verify
        let saved_config = fs::read_to_string(&config_path).unwrap();
        assert!(saved_config.contains("new_batch_id"));
        
        let saved_js = fs::read_to_string(&js_path).unwrap();
        assert!(saved_js.contains(r#"d.appId="new_batch_id""#));
        // 我们的逻辑会把逗号分隔的字符串解析并重新组合，所以这里验证组合后的结果
        assert!(saved_js.contains(r#"d.douyinIds=["new_d1","new_d2"]"#));
        
        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();
    }
}
