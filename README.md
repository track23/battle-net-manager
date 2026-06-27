# BattleNetManager — 战网账号切换工具

![软件截图](docs/images/01.png)

> ⚠️目前仅支持战网国服。本软件由第三方独立开发，与暴雪娱乐（Blizzard Entertainment）及战网官方和网易无关，亦未获得其任何形式的授权或认可。

站在巨人的肩膀上 — 此项目参考了 [KManager](https://github.com/kingkideng/KManager)

## 简介

BattleNetManager 是一款轻量级的 Windows 桌面应用，帮助拥有多个**暴雪战网（国服）**账号的玩家快速切换账号。无需反复输入账号密码，一键即可在不同账号之间无缝切换。

### 主要功能

- **账号管理** — 添加、编辑、删除多个战网账号
- **分组管理** — 将账号按用途分组，一目了然
- **一键切换** — 选择账号后自动完成配置切换，无需手动修改文件
- **战网进程管理** — 切换前自动检测并处理正在运行的战网客户端
- **旧版数据迁移** — 支持从旧版本（C# 版）导入账号数据，无缝升级

## 软件优势

- **🪶 体积小巧** — 基于 Tauri 构建，安装包仅数 MB，不臃肿
- **🔒 安全开源** — 代码完全开源，无后门、无遥测、无广告，欢迎审计
- **💾 数据本地存储** — 所有账号信息和配置文件均保存在本地，不会上传到任何服务器

## 技术栈

| 层级     | 技术                      |
| -------- | ------------------------- |
| 前端     | SolidJS + Tailwind CSS v4 |
| 桌面框架 | Tauri 2.0                 |
| 后端     | Rust                      |
| 打包     | NSIS (Windows Installer)  |

## 开发

### 环境要求

- Node.js 20+
- Rust 1.70+ (Windows MSVC 工具链)
- Windows x64

### 前端开发（浏览器调试）

```bash
cd webui
npm install
npm run dev
```

浏览器访问 `http://localhost:1420`，使用 MockBridge 模拟数据进行 UI 开发。

### 完整开发（Tauri 桌面端）

```bash
cd src-tauri
cargo tauri dev
```

### 构建发布版本

```bash
cd src-tauri
cargo tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`。

## 许可证

[MIT](LICENSE)
