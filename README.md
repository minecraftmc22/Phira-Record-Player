# Phira Record Player

> 基于 [TeamFlos/phira-render](https://github.com/TeamFlos/phira-render) 的 Phigros 谱面渲染与 .phirarec 回放播放器。

一款桌面端工具，用于渲染 Phigros 自制谱面为视频，同时支持 **Phira (.phirarec) 回放解析与查看**。基于 Tauri + Vue 3 + Vuetify 构建，Rust 后端驱动 prpr 引擎进行高性能离屏渲染。

## 功能

- **谱面渲染** — 将 Phigros 谱面 (.pez) 渲染为 MP4 视频，支持自定义分辨率、帧率、视频码率
- **.phirarec 回放播放** — 解析 Phira 手游录制回放文件，还原完整游玩过程
- **录制回放查看器** — 浏览回放数据：判定分布（Perfect/Good/Miss）、分数、连击等统计
- **RPE 谱面绑定** — 点击绑定 RPE 编辑器，快速打开谱面源文件
- **多预设管理** — 保存和切换多组渲染配置（分辨率、帧率、码率等）
- **任务队列** — 批量渲染，实时进度显示，支持取消和重试
- **实时日志** — 渲染过程中可查看后端 stderr 日志，便于排查问题
- **Material Design 3 界面** — 基于 Vuetify 3，支持中英文切换

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面框架 | Tauri 1.x (Rust) |
| 前端 | Vue 3 + Vuetify 3 + TypeScript |
| 国际化 | vue-i18n |
| 谱面引擎 | prpr (OpenGL 渲染) |
| 回放解析 | .phirarec 二进制格式 (PHIRAREC header + ZSTD 压缩) |
| 视频编码 | FFmpeg (管道写入帧数据) |

## 截图

<!-- TODO: 添加截图 -->

## 安装

从 [Releases](https://github.com/minecraftmc22/Phira-Record-Player/releases) 下载最新安装包。

**系统要求：**
- Windows 10+ (64 位)
- [FFmpeg](https://ffmpeg.org/download.html) 已安装并添加到 PATH

## 手动构建

**前置条件：**
- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/) 1.60+
- [pnpm](https://pnpm.io/)

```bash
# 克隆仓库
git clone https://github.com/minecraftmc22/Phira-Record-Player.git
cd Phira-Record-Player

# 安装前端依赖
pnpm install

# 开发模式
cargo tauri dev

# 生产构建
cargo tauri build
```

**Windows 特别说明：** 如使用 `gnullvm` 工具链，需确保 `libunwind.dll` 在构建产物中（已通过 `tauri.conf.json` 自动打包）。

## 项目结构

```
Phira-Record-Player/
├── src/                      # Vue 前端
│   ├── App.vue               # 应用入口
│   ├── AboutView.vue         # 关于页面
│   ├── RenderView.vue        # 谱面渲染
│   ├── RecordView.vue        # 回放查看
│   ├── RPEView.vue           # RPE 绑定
│   ├── TasksView.vue         # 任务队列
│   ├── model.ts              # 类型定义
│   └── components/
│       └── ConfigView.vue    # 渲染配置
├── src-tauri/                # Rust 后端
│   ├── src/
│   │   ├── main.rs           # Tauri 入口
│   │   ├── render.rs         # 渲染子进程 (prpr 引擎)
│   │   ├── task.rs           # 任务队列管理
│   │   ├── record.rs         # .phirarec 解析
│   │   ├── preview.rs        # 谱面预览
│   │   ├── ipc.rs            # 进程间通信
│   │   └── common.rs         # 公共工具
│   ├── assets/               # 引擎资源 (音效、字体、皮肤)
│   └── tauri.conf.json       # Tauri 配置
└── package.json
```

## 作者

- **上游项目**：[TeamFlos/phira-render](https://github.com/TeamFlos/phira-render) — Mivik / TeamFlos
- **Fork 维护**：[Minecraftmc22](https://github.com/minecraftmc22/) — .phirarec 回放支持、判定注入、多项 Bug 修复

## 许可证

[GPLv3](LICENSE) — 基于 [TeamFlos/phira-render](https://github.com/TeamFlos/phira-render) (Apache 2.0 → GPLv3)
