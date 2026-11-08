# Real-Time Earth 3D 场景支持

用户已明确选择继续补齐 3D 支持。目标是原始 workshop 3557068717 正常实时渲染；保留现有 2D 场景与双屏功能。

## 已确认缺口

- WPModelObject、WPCameraObject 只解析 schema，生成渲染节点的 visitor 为空。
- 透视场景仍选择默认二维正交相机；没有使用场景相机的位姿/FOV。
- MDL 已解析 normals/tangents，但生成 GPU 顶点时没有上传。
- Vulkan 自定义 pass 只有颜色附件，材质的深度/剔除状态没有应用。
- 初始 visible=false 的脚本控制图层被禁止运行，MAIN 计算结果没有写入 shared；运行日志反复出现 undefined.toFixed/toExponential。
- engine.screenResolution 尚未绑定，视角脚本会读取它。

## 方案

扩展现有模型解码、SceneNode、材质和 Vulkan pass，复用纹理、shader 转译、图层层级与每屏独立实例。独立 3D 后端需要复制资源与脚本集成；专用地球替代渲染不能忠实运行壁纸资产，因此不采用。

1. 接通模型节点，上传位置、法线、切线、UV/索引，保留材质槽与原始材质。
2. 对透视场景选择实际相机，采用相机对象的位姿、近远裁面和 FOV；保留 2D 相机路径。
3. 为需要深度的场景目标配置独立深度附件，正确管理首写清理、后续加载、布局/同步；应用材质深度与剔除规则，并让批次兼容性包含深度附件。
4. 运行隐藏的纯控制图层脚本；补齐实际需要的引擎坐标绑定；保持每个场景独立的脚本状态。
5. 补充模型/相机/深度/脚本回归测试，重新构建带独立构建号的 app，验证副屏 Earth 与主屏现有壁纸并行、重新启用、动态画面，并回归 Ender 全特效。

不修改 Steam 壁纸源资产，不使用预览图或替代视频冒充渲染成功。对解析、着色器或 GPU 失败保留可诊断错误，不将首帧通知等同于视觉验收。

参考：Khronos Vulkan Depth Guide（https://docs.vulkan.org/guide/latest/depth.html）；Wallpaper Engine SceneScript update（https://docs.wallpaperengine.io/en/scene/scenescript/reference/event/update.html）。
