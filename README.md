# Wallpaper Engine for macOS

当前正式版：**0.2.1（构建 2026110906）**。

[下载 0.2.1](https://github.com/pzc163/wallpaper-engine-for-macos/releases/tag/v0.2.1) · [查看完整发布说明](docs/releases/0.2.1.md)

## 0.2.1 新增与修复

| 内容 | 更新说明 |
| --- | --- |
| 场景 3D 支持 | 增加模型、透视相机、光源、深度缓冲和 32 位索引支持，补齐 Real-Time Earth 实时地球所需的渲染路径。 |
| 网页壁纸支持 | 增加基于系统 WebKit / WebGL 的本地网页壁纸后端，支持作者属性、暂停、帧率及鼠标输入桥接，覆盖 Space Wallpaper ASCII 的使用场景。 |
| Ender 全特效 | 修复降采样纹理复制、离屏同步及图层合成问题，消除全特效开启后的 1/2、1/4 阶梯重影和过曝白块。 |
| MUSIC STARS | 修复黑色加载遮罩不消失、文字裁切、动态脚本及模型兼容问题，并修复时钟纹理不刷新的情况。 |
| 设置与稳定性 | 补齐属性下拉选项；修复大范围数值滑块导致的界面卡死，以及壁纸重载时残留绘制任务造成的崩溃。 |
| 本仓库更新入口 | 构建 2026110906 将应用内“检查更新”、发布页面和项目地址切换到 `pzc163/wallpaper-engine-for-macos`。 |
| 发布包 | 提供 Apple Silicon 的 DMG、ZIP、包含递归子模块的完整源码包和 SHA-256 校验清单。运行依赖已内置，无需为应用额外安装 Homebrew 动态库。 |

构建 2026110905 已通过 MUSIC STARS 默认配置的双屏冷启动、时钟跨分钟更新、恒星与文明状态更新及重载检查；6 个 C++ 回归测试目标全部通过。2026110906 在此基础上更新应用内仓库地址并重新编译，渲染引擎保持同一提交。Ender、实时地球和网页壁纸在本次开发过程的前序构建中完成了对应实机验收，未在每个构建上重复所有项目。

3D 和网页兼容性仍在完善中，并非所有 Wallpaper Engine 项目都能完整运行。MUSIC STARS 的实际音频输入联动、拖尾画质与性能尚未纳入本次验收。详细验证范围见[发布说明](docs/releases/0.2.1.md)。

## 关于

这是一个 Wallpaper Engine 在 macOS 上的开源实现，使用 C++、Objective C++、Rust、Swift 混合实现。渲染引擎使用魔改后的 open-wallpaper-engine，额外支持了很多功能。

本仓库基于 [bigsaltyfishes/wallpaper-engine-for-macos](https://github.com/bigsaltyfishes/wallpaper-engine-for-macos) 继续开发，渲染修复维护在 [pzc163/open-wallpaper-engine](https://github.com/pzc163/open-wallpaper-engine)。这是对原始 Wallpaper Engine 的非官方开源实现，欢迎提交兼容性问题和 PR。

![单屏音频响应](assets/截屏2026-05-19%2001.07.30.webp)
![多屏音频响应](assets/截屏2026-05-19%2001.10.55.webp)
![GUI](assets/截屏2026-05-19%2001.08.49.webp)

## 兼容性

本程序只支持 **Apple Silicon（arm64）**，不支持 Intel Mac。当前 2026110906 预编译包要求 **macOS 27.2 或更新版本**，这是内置动态库的最低系统要求。实机验证环境为 macOS 27.2、Apple M3 Ultra，主屏 3840×2160、副屏 2560×1600；源码的较低部署目标不代表此发布包能在旧系统运行。

## 功能特性

渲染引擎相较原版（提交哈希 4145cfc90ed348d219a843d8549668b1ac9d3b4f），有以下功能差异：

- [x] 使用 Apple vDSP 加速的音频 FFT 变换加速
    - [x] 音频响应
- [x] 使用 FFmpeg 和 Apple VideoToolBox 实现的音频和视频管线
    - [x] 音频解码硬件加速
    - [x] 视频纹理
- [x] 参考 linux-wallpaperengine 实现的脚本引擎
    - [x] 部分支持 SceneScript
- [x] 使用 Rust 完全重新实现的着色器管线，具有更好的兼容性和更快的代码生成速度
- [x] 支持用户自定义属性、缩放系数
- [x] 支持多个显示器独立运行壁纸，以及镜像模式
- [x] 支持基础 3D 场景渲染，包括模型、相机、光源与深度
- [x] 支持本地 WebKit / WebGL 网页壁纸及属性桥接
- [x] 支持场景动态图层外观及异步文字更新

具体兼容范围和已知限制见上方 0.2.1 说明。

## 使用教程

首先你需要有一份正版的 Wallpaper Engine 副本，本程序需要使用原版 Wallpaper Engine 的资源，程序不认除了 Steam 以外的安装路径。

### 1. 安装原版 Wallpaper Engine

1.1 安装 Steam 并登录

这一步是为了保证你能通过 Steam 订阅创意工坊内容，并自动下载和更新。直接去 Steam 官网下载安装 macOS 版本的 Steam 即可

1.2 安装 steamcmd

这一步是为了绕过 Steam 系统检测安装 Wallpaper Engine，你有两种方式安装

- 使用 [Homebrew](https://brew.sh/zh-cn/) 安装

```shell
brew install steamcmd
```

- 从 [官网](https://developer.valvesoftware.com/wiki/SteamCMD) 下载

```shell
mkdir ~/Steam && cd ~/Steam
curl -sqL "https://steamcdn-a.akamaihd.net/client/installer/steamcmd_osx.tar.gz" | tar zxvf -

# 添加到环境变量
echo "export PATH=$PATH:$HOME/Steam" >> ~/.zshrc
source ~/.zshrc
```

1.3 安装 Wallpaper Engine

1.3.1 打开 SteamCMD 并登录

```shell
> steamcmd

Steam Console Client (c) Valve Corporation - version 1778284286
-- type 'quit' to exit --
Loading Steam API...OK

Steam> login 用户名 密码 <可选的Steam令牌验证码>
Logging in using username/password.
Logging in user 'UserName' to Steam Public...This account is protected by a Steam Guard mobile authenticator.
Please confirm the login in the Steam Mobile app on your phone.
```

如果没有给验证码，你就需要去你的 Steam APP 上批准登录

1.3.2 安装 Wallpaper Engine

```shell
# 设置平台类型为 Windows
Steam> @sSteamCmdForcePlatformType windows

# 安装 Wallpaper Engine
Steam> app_update 431960 validate
```

安装过程可能会很长，因为会连带创意工坊内容一起下载下来，安装完成后后续的更新、创意工坊内容的下载可以使用 Steam 客户端进行

### 2. 安装 Wallpaper Engine for macOS

从[本仓库发布页面](https://github.com/pzc163/wallpaper-engine-for-macos/releases)下载程序：打开 DMG 后将 `Wallpaper Engine.app` 拖入 `Applications`，或解压 ZIP 后复制到 `应用程序` 文件夹。升级前请先退出正在运行的旧版本，再替换应用。

0.2.1 已内置运行所需的动态库，并提供 `SHA256SUMS.txt` 校验文件。发布包采用 ad-hoc 签名，尚未经过 Apple Developer ID 公证；若首次启动被系统拦截，可在系统设置的“隐私与安全性”中批准运行。

**更新入口：** 从构建 2026110906 起，应用内“检查更新”和项目链接均使用本仓库。此前的 2026110905 及更早构建仍指向原上游，需要先从上面的发布页面手动下载并替换一次。

### 3. 使用

#### 3.1 设置壁纸

如果你只是在主显示器上用，那就是开箱即用的，直接去壁纸页面选择壁纸、在显示器页面中展开`Primary`显示器的设置，点击启用，然后应用即可。副屏需要你在设置页面中启用显示器才能设置壁纸。考虑到 MacBook Air 没有散热，不建议 MacBook Air 多屏启用场景类型壁纸。

#### 3.2 音频响应

在壁纸页面选择壁纸，然后展开通用设置，启用音频响应即可，程序会请求系统录音权限，允许即可。

#### 3.3 16:10 屏幕支持

程序支持自定义缩放系数，在壁纸页面选择壁纸、展开显示器设置、展开对应显示器的设置，选择一个合适的缩放模式，再按需调整缩放系数。

缩放模式算法：

1. 无：壁纸里面设置了多大分辨率就按多大渲染，超出屏幕范围会直接裁剪
2. 拉伸：会把壁纸暴力拉倒屏幕分辨率，可能会变形
3. 匹配：根据壁纸分辨率X、Y中离屏幕分辨率最近的一侧进行缩放，该模式为等比缩放
4. 填充：根据壁纸分辨率X、Y中离屏幕分辨率最远的一侧进行缩放，该模式为等比缩放

#### 3.4 镜像壁纸

在设置中调整对应显示器模式为镜像即可

需要注意主显示器壁纸是设置给主显示器的，主显示器不按型号区别，只认系统汇报的那个显示器，因此镜像到主显示器时，壁纸不会随主显示器变化而变化。

#### 4. 日志记录

程序默认只记录 INFO 及更低级别的日志，你可以通过环境变量`WALLPAPER_ENGINE_LOG_LEVEL`调整这一级别，支持`trace`, `debug`, `info`, `error`, `warning`, `off`多个级别(级别逐级递减)。

## 编译程序

本程序使用 Nix 作为构建系统，大多数库都由 Nix 直接提供。但由于使用了 Swift 的一些专有库，编译前你需要在系统中安装 Xcode，版本要求 Xcode 26 以上。

```shell
# 克隆本仓库
git clone --recurse-submodules https://github.com/pzc163/wallpaper-engine-for-macos.git

# 编译
cd wallpaper-engine-for-macos
nix build
```

渲染引擎作为 Git 子模块固定到对应修复提交，克隆时请保留 `--recurse-submodules`。若通过压缩包获取源码，请选择 Release 中的 `Wallpaper-Engine-0.2.1-source.tar.gz`，其中包含递归子模块；GitHub 自动生成的源码压缩包不包含子模块内容。

已有编译产物时，可使用 `scripts/package-macos.py` 封装运行依赖，操作方法见[0.2.1 打包说明](docs/releases/0.2.1.md#打包已编译应用)。

## 开源协议

本程序在 GPL-2.0 协议下开源

## 参考项目

其他非官方实现的功能、维护状态及与本项目的关系，见 [2026-11-09 开源方案调研](docs/research/2026-11-09-wallpaper-engine-alternatives.md)。

- 渲染引擎 [waywallen/open-wallpaper-engine](https://github.com/waywallen/open-wallpaper-engine)
- 渲染管线/着色器处理器/视频纹理/脚本引擎 [Almamu/linux-wallpaperengine](https://github.com/Almamu/linux-wallpaperengine)
- 程序图标 [Unayung/wallpaper-engine-mac](https://github.com/Unayung/wallpaper-engine-mac)
