# Wallpaper Engine 非官方开源实现调研

调研日期：2026-11-09。依据各项目的官方仓库、文档、提交记录及 GitHub Release 元数据。本次没有安装或实测其他应用；功能支持是项目自身声明，针对本机的选型建议属于据此作出的判断。

有其他方案。对 Apple Silicon Mac，最值得先与本项目对比的是 **WallpaperMachine** 和 **deepratna-awale/open-wallpaper-engine-mac**。前者已有正式版本，后者提供较完整的编辑器与 3D/脚本功能，但最新公开包仍是 beta。不能仅凭“支持 Scene”判断 Ender、实时地球、MUSIC STARS 都能正确运行。

官方 Wallpaper Engine 仍未提供原生 macOS/Linux 桌面版，以下均为非官方项目。[官方平台说明](https://help.wallpaperengine.io/en/functionality/linuxmacos.html)

## 可直接评估的 macOS 应用

| 项目 | 架构与功能声明 | 发布和维护情况 | 对本项目的参考价值 |
| --- | --- | --- | --- |
| [WallpaperMachine](https://github.com/MMFoundation/WallpaperMachine) | Apple Silicon、macOS 15+；Swift + Rust + C++，原生 Metal；Scene/Video/Web、多屏独立配置、Workshop 下载。渲染文档列出模型、相机、灯光、粒子、文字与后处理；不能绘制的场景会保留画面并显示原因。 | 正式版 [v1.9.3](https://github.com/MMFoundation/WallpaperMachine/releases/tag/v1.9.3)，2026-11-07 发布；默认分支最近提交 2026-11-09。仓库声明 GPL-3.0-only。 | 优先比较双屏管理、原生 Metal、场景兼容性提示和更新安装流程。最初来自本项目相同上游，不能简单算作毫无关系的独立实现。 |
| [Open Wallpaper Engine macOS](https://github.com/deepratna-awale/open-wallpaper-engine-mac) | macOS 14+；原生 Metal，翻译原始壁纸着色器；声明支持 3D、光照、SceneScript、粒子、音频响应、Scene/Video/Web。网页可选 WebKit/Chromium；附带场景编辑器、FBX/OBJ 导入和多显示器配置。 | 最新公开预发布 [v1.0.0-beta.15](https://github.com/deepratna-awale/open-wallpaper-engine-mac/releases/tag/v1.0.0-beta.15)，2026-11-03；默认分支最近提交 2026-11-07。GPL-3.0。 | 更适合比较复杂脚本、模型、着色器转换和网页兼容性；要注意 README 的默认分支功能可能领先于 beta 安装包。 |

WallpaperMachine 的[场景渲染文档](https://github.com/MMFoundation/WallpaperMachine/blob/main/docs/features/scene-rendering.md)明确说明旧 Vulkan/MoltenVK 后端已经移除。其[代码来源说明](https://github.com/MMFoundation/WallpaperMachine/blob/main/LICENSING.md)记载起点是 `bigsaltyfishes/wallpaper-engine-for-macos` 的 `8c19c002`，并声称后续进行了分离规格与实现的重写。本次核实了该声明的存在，没有审计重写过程。

另一个 macOS 项目的来源链为 Haren Chen / MrWindDog / Unayung 系列，后续由 Deepratna Awale 开发 Metal 渲染与脚本等功能，和当前仓库的直接上游不同。功能和系统要求分别见其 [README](https://github.com/deepratna-awale/open-wallpaper-engine-mac/blob/main/README.md) 与[入门文档](https://github.com/deepratna-awale/open-wallpaper-engine-mac/blob/main/docs/getting-started.md)。

## 其他渲染路线

| 项目 | 路线及范围 | 限制与判断 |
| --- | --- | --- |
| [Almamu/linux-wallpaperengine](https://github.com/Almamu/linux-wallpaperengine) | Linux 为主的 C++ / OpenGL 引擎，X11/Wayland 集成，构建使用 Chromium Embedded Framework。默认分支最近提交 2026-10-30。 | 适合参考 SceneScript、混合、后处理和兼容修复；不是现成的 macOS 桌面应用。主项目 README 要求 OpenGL 3.3，且明确提示兼容性限制。 |
| [waywallen/open-wallpaper-engine](https://github.com/waywallen/open-wallpaper-engine) + [Waywallen](https://github.com/waywallen/waywallen) | C++ / Vulkan 场景渲染器与 Linux 桌面宿主；渲染器现在列出 3D 和 SceneScript。当前正式版本分别为 [v0.3.3](https://github.com/waywallen/open-wallpaper-engine/releases/tag/v0.3.3) 和 [v0.6.0](https://github.com/waywallen/waywallen/releases/tag/v0.6.0)，均于 2026-11-08 发布。 | 这是当前渲染器的同一来源系列，优先检查增量更新价值。旧 [catsout/wallpaper-engine-kde-plugin](https://github.com/catsout/wallpaper-engine-kde-plugin) 已归档，README 指向 Waywallen，不宜继续把旧仓库当活跃独立方案。 |
| [wqLouis/linux-wallpaperengine](https://github.com/wqLouis/linux-wallpaperengine) | Rust / wgpu，Vulkan 与 Metal 路线；macOS 后端标为实验性。支持后处理着色器和 Puppet 网格。最近提交 2026-11-04。 | README 明确不支持用户脚本、文字和粒子，视频纹理只显示静帧。据此判断，目前不适合解决 MUSIC STARS 的需求，但可研究跨平台渲染架构。 |
| [3x-haust/wallpaperengine-mac-renderer](https://github.com/3x-haust/wallpaperengine-mac-renderer) | Almamu 引擎的 macOS / Apple Silicon OpenGL 4.1 移植；离屏输出帧，再由宿主编码成循环视频。最近提交 2026-09-10。 | 不是实时桌面宿主；录制视频本身无法保留实时鼠标、音频或时钟行为。其浮点 FBO、合成尺寸和 UV 修复与我们曾遇到的 Ender 问题有参考关联，但不是即插即用替代。 |
| [OpenWallpaper](https://github.com/mechakotik/openwallpaper) | Wayland 下的 WASM 动态壁纸运行时，使用 `wpe-compile` 将 Wallpaper Engine 场景转换为自己的场景格式，另支持视频/GIF和音频频谱。 | 是转换后运行的另一条路线；官方文档面向 KDE Plasma Wayland、Hyprland 和 wlroots，不能作为现成 Mac 应用使用，也不能由格式转换支持推断完整兼容。 |

## 针对当前壁纸的比较方法

按以下顺序评估两个 macOS 候选，会比直接替换整个引擎更容易判断实际收益：

1. **Ender 全特效**：固定全部作者属性，检查半/四分之一离屏目标、全屏合成、透明加法混合和持续运行后的画面。
2. **Real-Time Earth**：检查模型材质、透视相机、深度和灯光；确认是实际 3D 动画，不是预览图或录制视频。
3. **MUSIC STARS**：从默认配置冷启动，观察加载完成、动态创建图层、文字跨分钟刷新，并单独检查音频响应。
4. **Space Wallpaper ASCII**：检查 WebGL、属性回调、暂停/恢复以及副屏的鼠标坐标。
5. **双屏**：使用同一主屏 3840×2160、副屏 2560×1600，比较独立配置、冷启动、切换壁纸和重载；再测 CPU/GPU、功耗和显存。

基于文档的建议：短期优先做两个原生 Metal 应用的同资源对照；本项目先追踪同源 `waywallen/open-wallpaper-engine` 的新增兼容修复。是否迁移到 Metal，应由上述壁纸的真实结果和资源开销决定。本次调研没有证明其他引擎比当前版本更准确或更省电。

## 查询到的默认分支提交

| 仓库 | 查询到的提交 |
| --- | --- |
| WallpaperMachine | [`56878b5a`](https://github.com/MMFoundation/WallpaperMachine/commit/56878b5a7414924b2c061f86162a58414e6fe71e) |
| deepratna-awale/open-wallpaper-engine-mac | [`993b757e`](https://github.com/deepratna-awale/open-wallpaper-engine-mac/commit/993b757e924b3011bdedf6927b642d8929c8f965) |
| Almamu/linux-wallpaperengine | [`a0ddce58`](https://github.com/Almamu/linux-wallpaperengine/commit/a0ddce586e9ff12e9d31ca6b103f6036cc1b68fe) |
| waywallen/open-wallpaper-engine | [`3774e343`](https://github.com/waywallen/open-wallpaper-engine/commit/3774e34314a00f0c167b0cf35290e69cf2d9743b) |
| Waywallen | [`e2ed0b0a`](https://github.com/waywallen/waywallen/commit/e2ed0b0adc6be6477c7bcc5015c76f4a1bb2b437) |
| wqLouis/linux-wallpaperengine | [`56339066`](https://github.com/wqLouis/linux-wallpaperengine/commit/563390664f5f5f544abb8bc2328b8f9cb5ef4e20) |
| 3x-haust/wallpaperengine-mac-renderer | [`22e81811`](https://github.com/3x-haust/wallpaperengine-mac-renderer/commit/22e81811c707e8a2d6e7b0f17124a25e36ef7f86) |
