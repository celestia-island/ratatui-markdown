# スクロールシステム

> フォーカス可能アイテムのナビゲーションを備えたスマートなハイブリッドスクロール。

## 概要

`scroll` モジュールは、2つのモードをサポートするハイブリッドスクロールシステムを提供します：

1. **自由スクロール** — フォーカス可能アイテムがビュー内にない場合、コンテンツは自由にスクロールします
2. **エンゲージ** — フォーカス可能アイテムがビューポート中央に入ると、カーソルが最初のアイテムに固定され、キーボードナビゲーションが可能になります

`scroll` 機能フラグで制御されます（デフォルトで有効）。

## HybridScrollView

スクロール、フォーカス領域、レンダリングを管理するコアウィジェットです：

```rust
pub struct HybridScrollView { /* fields */ }

impl HybridScrollView {
    pub fn new() -> Self;
    pub fn with_left_padding(self, padding: bool) -> Self;
    pub fn with_cursor_indicator(self, show: bool) -> Self;

    // コンテンツ管理
    pub fn set_content(&mut self, lines: Vec<Line<'static>>, regions: Vec<FocusableRegion>);
    pub fn set_lines(&mut self, lines: Vec<Line<'static>>);
    pub fn clear(&mut self);
    pub fn is_empty(&self) -> bool;

    // スクロール状態
    pub fn total_lines(&self) -> usize;
    pub fn get_scroll_offset(&self) -> usize;
    pub fn get_viewport_height(&self) -> usize;
    pub fn set_scroll_offset(&mut self, offset: usize);

    // ナビゲーション
    pub fn scroll_up(&mut self);
    pub fn scroll_down(&mut self);
    pub fn scroll_to_top(&mut self);
    pub fn scroll_to_bottom(&mut self);
    pub fn page_up(&mut self, lines: usize);
    pub fn page_down(&mut self, lines: usize);

    // エンゲージメント
    pub fn is_engaged(&self) -> bool;
    pub fn engaged_cursor(&self) -> Option<(usize, usize)>;
    pub fn selected_item_id(&self) -> Option<&str>;
    pub fn engage_first(&mut self);
    pub fn engage_by_id(&mut self, id: &str) -> bool;

    // レンダリング
    pub fn render(&mut self, f: &mut Frame, inner_area: Rect, outer_area: Rect, theme: &impl RichTextTheme);
}
```

### 設定

- **with_left_padding**: 表示されるすべての行に左パディングを1列追加します
- **with_cursor_indicator**: エンゲージされたカーソル行に `> `（2列）を表示します（`left_padding` より優先されます）

`effective_padding()` メソッドは実際に使用されるパディングを返します：
- カーソルインジケーターが有効な場合は `2`
- 左パディングのみの場合は `1`
- それ以外の場合は `0`

## フォーカス可能領域とアイテム

```rust
pub struct FocusableItemRange {
    pub start_line: usize,    // 開始（含む）
    pub end_line: usize,      // 終了（含まない）
    pub id: String,           // 一意の識別子
}

pub struct FocusableRegion {
    pub items: Vec<FocusableItemRange>,
}
```

領域はインタラクティブになる行の範囲を定義します。ビューポートの中央が領域を通過すると、スクロールビューは自動的にエンゲージし、カーソルは最初のアイテムに移動します。

### エンゲージメントの動作

- 領域内に**下**にスクロールすると、**最初**のアイテムにエンゲージします
- 領域内に**上**にスクロールすると、**最後**のアイテムにエンゲージします
- 領域内の最後のアイテムを過ぎてナビゲートすると、**ディスエンゲージ**して自由スクロールに戻ります
- `scroll_to_top()` と `scroll_to_bottom()` は常にディスエンゲージします
- 領域内では、`scroll_up`/`scroll_down` でカーソルがアイテム間を移動します

## その他のスクロールウィジェット

### ScrollableList<T>

マウス/キーボードナビゲーションとオプションの境界線付きレンダリングを備えたジェネリックなスクロール可能リスト：

```rust
pub trait ListItemRenderer {
    fn render_item(&self, index: usize, is_selected: bool, width: usize) -> Line<'static>;
}
```

### ArrowScrollbar

Unicode 矢印記号で描画されるカスタムスクロールバー：

```rust
pub fn render_arrow_scrollbar(
    area: Rect,
    buf: &mut Buffer,
    top: usize,
    bottom: usize,
    theme: &impl RichTextTheme,
);
```

### FollowScrollState

コンテンツの自動追従用（例：ストリーミング出力）：

```rust
pub struct FollowScrollState {
    // ビューポートが最下部にあるかどうかを追跡
}
```

### ScrollableRenderResult

シンプルなスクロール可能パネルラッパー：

```rust
pub fn render_scrollable(
    lines: &[Line],
    scroll_offset: usize,
    area: Rect,
    buf: &mut Buffer,
) -> ScrollableRenderResult;
```

## 例

```rust
use ratatui_markdown::scroll::{HybridScrollView, FocusableItemRange, FocusableRegion};

let mut scroll = HybridScrollView::new()
    .with_cursor_indicator(true);

// コンテンツ行
let lines: Vec<Line> = (0..100)
    .map(|i| Line::raw(format!("Line {}", i)))
    .collect();

// 行 30〜32 をフォーカス可能にする
let region = FocusableRegion {
    items: vec![
        FocusableItemRange { start_line: 30, end_line: 31, id: "item-a".into() },
        FocusableItemRange { start_line: 31, end_line: 32, id: "item-b".into() },
        FocusableItemRange { start_line: 32, end_line: 33, id: "item-c".into() },
    ],
};

scroll.set_content(lines, vec![region]);

// 自由スクロールで下へ；行30がビューポート中央に入ると自動エンゲージ
scroll.scroll_down();

// アイテム間をナビゲート
while scroll.scroll_down() == InputResult::Continue {
    if let Some(id) = scroll.selected_item_id() {
        println!("選択中: {}", id);
    }
}
```
