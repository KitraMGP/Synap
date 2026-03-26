# Synap 桌面端 GUI 交互与视觉设计方案书 (基于 Iced 框架)

本方案书致力于将 Synap 底层硬核的“不可变账本与 DAG 拓扑”转化为极其舒适、符合现代直觉的图形用户界面（GUI）。桌面端将彻底褪去终端的极客粗粝感，依托 Rust 原生的 **Iced** 框架，利用其响应式（Elm 架构）特性，打造一个丝滑、灵动、注重视觉空间记忆的“思维绘图板”。

---

## 一、 设计哲学与 Iced 架构契合度

1. **状态即视图 (State is View)：** Iced 采用 Elm 架构 (`Model` -> `Update` -> `View`)。这与 Synap 底层的“读时过滤 (Read-Time Resolution)”完美契合。底层的不可变区块和指针在读取时被坍缩为一个纯粹的 `Model`（当前有效图谱），Iced 只需负责将这个 `Model` 渲染为舒展的界面，没有任何数据绑定的负担。
2. **空间记忆取代线性目录：** 放弃传统的左侧文件树导航。人类大脑更善于记住“那个想法在画布的左下角”。界面以 2D 无限画布为主体，建立思维的视觉地理学。
3. **温柔的不可变性：** 用户在 GUI 中执行的是符合直觉的“修改”与“删除”操作（点击编辑、点击垃圾桶），但在 Iced 的 `Update` 逻辑层，这些动作会被静默转化为 `EDIT` 和 `TOMBSTONE` 指针的追加。GUI 对用户隐瞒底层的硬核账本，只展现思维的生长。

---

## 二、 核心界面布局：三层视界

整个桌面端采用无边框设计，支持极简明亮（Light）与深邃暗色（Dark）双主题。界面由远及近分为三个 Z 轴层级：

### 1. 底层：无界星图画布 (The Infinite Canvas)

* **功能：** 全局视角的 DAG 拓扑漫游区。
* **视觉呈现：**
* **节点 (Nodes)：** 采用大圆角卡片（柔和的视觉边界），内部显示文本摘要和标签药丸（Pill-shaped tags）。
* **连线 (Edges)：** 使用平滑的贝塞尔曲线（Bézier curves）连接节点，带有随呼吸微闪的渐变色流动效果，暗示思维的流向（`REPLY` 指针）。
* **历史折叠：** 被修改过的节点，其卡片底部会有极其微弱的“堆叠”阴影，暗示其存在历史版本（`EDIT` 链条），但不干扰当前视觉。


* **Iced 技术映射：** 使用 `iced::widget::canvas` 深度定制。维护一个带有缩放（Zoom）和平移（Pan）矩阵的 Camera 状态。

### 2. 中层：沉浸式语境流 (The Context Stream)

* **功能：** 当在画布上点击任意节点时，从屏幕右侧（或左侧）平滑滑出的浮动面板，用于深度阅读和输入。
* **视觉呈现：**
* 它不是一个孤独的编辑器，而是一条“流”。选中的节点高亮居中，上方以褪色样式显示其“前置节点（父母）”，下方留白准备输入“后续派生（孩子）”。
* 排版采用现代排版引擎标准，舒适的行高，优雅的无衬线字体，支持 Markdown 实时渲染（所见即所得）。


* **Iced 技术映射：** 组合使用 `iced::widget::scrollable` 和 `Column`。通过自定义的富文本组件或 Markdown 渲染组件实现。

### 3. 顶层：悬浮捕获与全局滤网 (HUD & Lens)

* **功能：** 零摩擦的灵感捕获与视图控制。
* **视觉呈现：**
* **全局捕获栏：** 屏幕顶部中央悬浮的极简搜索/输入框。按下快捷键（如 `Ctrl+N`）直接呼出。敲击回车直接生成孤立新节点飘落在画布上。
* **滤网控制台 (Lens Bar)：** 位于画布底部的悬浮小药丸菜单。允许用户一键过滤画布：例如“仅显示 #架构 标签”、“隐藏所有孤立节点”、“时间回溯至昨晚”。



---

## 三、 GUI 友好型交互范式

为了兼顾效率与舒适度，操作逻辑将深度融合鼠标手势与基础快捷键：

1. **可视化的思维延伸 (Drag & Drop Linking)：**
* 鼠标悬停在卡片边缘，会出现一个连线锚点。按住锚点拖拽到另一张卡片，松手瞬间，底层自动追加一条 `REPLY` 指针。


2. **就地进化 (Inline Evolve)：**
* 双击画布上的卡片，或在右侧语境流中点击文本，文本块直接变为编辑态（无缝切换）。修改完毕点击空白处或按下 `Ctrl+Enter`，底层自动追加新 Block 和 `EDIT` 指针，UI 瞬间刷新。


3. **视觉化的废弃 (Swipe/Click to Void)：**
* 在节点上右键呼出柔和的环形菜单（Pie Menu），选择“废弃”，或者将节点拖拽出屏幕边缘/拖入特定的垃圾桶图标。底层写入 `TOMBSTONE` 指针，UI 上该卡片化作一阵微尘动画消散（或变为半透明幽灵态，取决于滤网设置）。


4. **历史版本穿梭 (Time Travel Slider)：**
* 针对带有堆叠阴影的节点，点击“时钟”小图标，卡片侧边弹出一个微型滑块。拖动滑块，卡片内容在过往的所有 `EDIT` 版本中实时切换预览。



---

## 四、 Iced 架构实现路径规划

由于 Iced 是纯 Rust 驱动的 GUI，它将与 Synap Core 进行真正的**零开销进程内调用**，无需任何 IPC 或网络中转。

**核心应用状态 (Application State) 定义示例：**

```rust
// Iced 的全局状态
struct SynapApp {
    // 底层服务
    core: SynapService,
    // 当前读取出的过滤视图（从 Core 拉取的 DAG）
    graph_view: Vec<NodeLayout>, 
    // 画布摄像机状态
    camera: CameraState,
    // 右侧抽屉面板状态
    focused_stream: Option<Ulid>,
    // 顶部输入框内容
    quick_input: String,
}

// Iced 的消息循环 (事件分发)
#[derive(Debug, Clone)]
enum Message {
    // 视图交互
    PanCanvas(Vector),
    ZoomCanvas(f32),
    NodeClicked(Ulid),
    
    // 核心数据变动 (触发底层不可变账本操作)
    QuickCaptureSubmitted,
    ContentEdited { id: Ulid, new_content: String },
    NodeLinked { source: Ulid, target: Ulid },
    NodeVoided(Ulid),
    
    // 同步与回收
    TriggerSync,
    TriggerScrub,
}

```

**更新循环 (`update` 逻辑)：**
当用户在 GUI 中完成一次连线操作，Iced 触发 `Message::NodeLinked`。`update` 函数中直接调用 `self.core.reply_thought(source, target)`。由于操作极快（仅仅是 SQLite/KV 的极小量追加），写入完成后立即调用 `self.core.get_graph()` 刷新 `graph_view`，界面在下一帧（16ms 内）丝滑重绘，实现真正的“所见即所得”与底层的“不可变记录”的完美融合。