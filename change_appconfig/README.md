# 字节跳动小游戏配置编辑器

使用 Rust + egui 框架开发的图形化配置编辑器，用于批量修改字节跳动小游戏项目中的配置字段。

## 功能特性

- 选择字节跳动小游戏项目目录
- 读取并展示当前配置值:
  - appid: 从 project.config.json 读取
  - appId: 从 JS 文件中读取
  - douyinIds: 从 JS 文件中读取
- 批量修改所有配置字段
- 友好的图形界面，适合非技术人员使用

## 使用方法

1. 运行程序: `cargo run --release`
2. 点击"选择目录"按钮，选择包含字节跳动小游戏项目的 build 目录
3. 查看当前配置值
4. 在输入框中输入新的配置值
5. 点击"应用修改"按钮完成修改

## 编译发布

```bash
cargo build --release
```

编译后的可执行文件位于: `target/release/bytegame-config-editor.exe`

## 技术栈

- Rust
- egui (即时模式 GUI 库)
- serde (JSON 解析)
- regex (正则表达式)
- walkdir (文件遍历)
