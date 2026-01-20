# 字节小游戏配置助手 (Douyin Config Editor)

这是一个用于批量管理和修改字节跳动小游戏项目配置的桌面应用程序。基于 Rust 和 egui 开发，旨在提高多项目维护的效率。

## ✨ 主要功能

*   **自动扫描**: 递归扫描指定目录下的所有小游戏项目 (`project.config.json`)。
*   **智能识别**: 自动关联项目下的 JS 配置文件 (`assets/main/index.js`) 和预览图片。
*   **可视化预览**: 并排显示项目中的预览图片（宽度为 750px 的图片），方便快速确认项目内容。
*   **批量修改**:
    *   统一修改所有选中项目的 AppID。
    *   统一修改所有选中项目的项目名称。
    *   统一修改 JS 配置中的 DouyinIDs。
*   **双重配置同步**: 修改 AppID 时，会自动同步更新 `project.config.json` 和关联的 JS 文件。
*   **无损读写**: 采用 JSON 无损读写策略，保留配置文件中所有未显式定义的字段。
*   **友好交互**: 支持中文界面，针对 Windows 优化了字体显示（微软雅黑）。

## 🚀 快速开始

### 运行程序

1.  下载最新的 Release 版本（如果有）。
2.  双击 `douyin_config_editor.exe` 运行。
3.  点击顶部的 "📂 选择根目录" 按钮，选择包含小游戏项目的父文件夹。
4.  程序会自动列出所有扫描到的项目。

### 批量操作

1.  在顶部的 "批量修改" 区域输入需要统一的 AppID 或项目名称。
2.  点击对应的 "应用" 按钮。
3.  确认列表中的修改（修改项会标记为红色）。
4.  点击 "💾 保存所有更改" 按钮生效。

## 🛠️ 开发构建

确保你已经安装了 [Rust](https://www.rust-lang.org/) 环境。

### 依赖安装

```bash
# 或者是自动安装
cargo fetch
```

### 运行开发版本

```bash
cargo run
```

### 打包发布

```bash
# 构建 Release 版本（会自动隐藏控制台窗口并添加图标）
cargo build --release
```

构建产物位于 `target/release/douyin_config_editor.exe`。

## 📁 项目结构

*   `src/main.rs`: 程序入口，窗口配置。
*   `src/app.rs`: UI 布局和交互逻辑。
*   `src/model.rs`: 数据模型定义 (ProjectConfig, JsConfig 等)。
*   `src/scanner.rs`: 文件扫描、解析和保存逻辑。
*   `build.rs`: Windows 资源编译脚本（用于添加图标）。

## 📝 注意事项

*   **JS 解析**: JS 配置文件的解析基于正则表达式，目前仅支持标准的 `.appId = "..."` 和 `.douyinIds = [...]` 格式。
*   **字体**: 程序默认尝试加载 Windows 系统的微软雅黑字体，在非 Windows 平台可能会回退到默认字体。
