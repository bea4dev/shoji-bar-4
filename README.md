# shoji-bar-4

[ShojiWM](https://github.com/bea4dev/ShojiWM) 用の Material 3 Expressive なデスクトップシェルです。Rust と [AntiWebUI](https://github.com/bea4dev/AntiWebUI)（`anti-webui-wayland-shell` と `anti-webui-md3e`）で書いています。

## できること

- **上のバー**：浮いた島が並びます。
  - ランチャーのボタン：Material Shapes が変形します（Cookie4 → Cookie9 → Circle）。
  - ワークスペース：前の端が速いばね、後ろの端が遅いばねで動き、伸び縮みしながら移る目印です。ホイールでも切り替えられます。
  - アクティブなウィンドウのアイコンとタイトル。
  - 時計：数字が転がって変わり、ホバーすると Roboto Flex の太さが変わります。
  - 再生中の曲（踊るイコライザー）、トレイ、ネットワーク・Bluetooth・音量・電池。
- **下のドック**：ポインタに近いアイコンほど大きくなります。
  - 起動中のアプリはウィンドウが出るまで跳ねます。
  - 下の点で、ウィンドウの数とフォーカスを示します。
  - 右クリックのメニュー：新しいウィンドウ、デスクトップアクション、ピン留め、閉じる。
- **クイック設定**：
  - タイル：オンで丸くなり、色が変わります。
  - 音量と明るさのスライダー、メディアカード（波打つ進捗）、電源メニュー。
  - スタイルと色：壁紙、種の色、配色のバリアント、ライト／自動／ダーク。
- **カレンダー**：秒まで転がる大きな時計、月のカレンダー、CPU・メモリ・温度の波打つリング。
- **ランチャー**：検索に合わせて、グリッドが FLIP で並び替わります。開くときは順番に現れます。矢印キー、Enter、Esc で操作できます。
- **通知**：通知デーモンになれたときだけ動きます。
  - 右上のカードが、ばねでスライドして入ってきます。
  - クイック設定に履歴が残ります。
  - おやすみモード。
- **OSD**：音量や明るさが変わると、ドックの上にピルが出ます。
- **テーマ**：
  - Material You の HCT で、種の色から配色を作ります。壁紙から色を取ることもできます。
  - バリアントは Tonal / Vibrant / Expressive / Fidelity / Neutral / Mono から選べます。
  - 切り替えると、全体の色がばねで移り変わります。
- **壁紙**：選ぶとクロスフェードしながら、少しズームから戻ります。

何もしていないときは、フレームを要求しません（CPU 0%）。

## ビルドと起動

```sh
cargo build --release
./target/release/shoji-bar-4
```

AntiWebUI は `../../Documents/development/AntiWebUI` をパスで参照しています。

## コマンド

起動中のシェルに、ソケット（`$XDG_RUNTIME_DIR/shoji-bar-4.$WAYLAND_DISPLAY.sock`）経由で命令を送れます。

```sh
shoji-bar-4 msg toggle launcher [OUTPUT]       # OUTPUT を省くと、フォーカスのあるウィンドウの画面
shoji-bar-4 msg toggle quick-settings [OUTPUT]
shoji-bar-4 msg toggle calendar [OUTPUT]
shoji-bar-4 msg close
shoji-bar-4 msg theme dark|light|auto|toggle
shoji-bar-4 msg quit
```

## ShojiWM から使う

`~/.config/shojiwm/src/index.tsx` で、シェルの起動とキーを次のように変えます。

```ts
const SHOJI_BAR_4 = "~/.config/shoji-bar-4/target/release/shoji-bar-4";
COMPOSITOR.process.once("shell", {
  command: `exec ${SHOJI_BAR_4}`,
  runPolicy: "once-per-session",
});

function toggleLauncher() {
  const monitor = HYBRID_WINDOW_MANAGER.getCurrentMonitorName();
  COMPOSITOR.process.spawn({ command: `${SHOJI_BAR_4} msg toggle launcher "${monitor}"` });
}
COMPOSITOR.key.bind("launcher", "Super+A", toggleLauncher);
```

レイヤーの名前空間は `shoji-bar-4-bar`、`-dock`、`-quick-settings`、`-calendar`、`-launcher`、`-notifications`、`-osd`、`-wallpaper` です。ShojiWM のレイヤーエフェクト（ぼかしなど）を当てるときに使えます。

## 設定

設定は `$XDG_STATE_HOME/shoji-bar-4/settings.json`（ふつうは `~/.local/state/…`）にあります。クイック設定で変えると保存されます。手で書き換えてもかまいません。

| キー | 意味 |
|---|---|
| `mode` | `light` / `dark` / `auto`（7〜19 時をライトにする） |
| `seed` | 種の色（`#rrggbb`） |
| `seed_from_wallpaper` | 壁紙から種の色を取る |
| `palette` | `tonal-spot` / `vibrant` / `expressive` / `fidelity` / `neutral` / `monochrome` |
| `wallpaper` | 壁紙のパス（`null` で描かない） |
| `pinned` | ドックにピン留めするアプリのデスクトップ ID |
| `clock_24h` / `clock_seconds` | 時計の表示 |

壁紙の候補は `~/Pictures/wallpapers` から読みます。

## 開発用：オフスクリーンの絵

コンポジタの面を使わずに、各部分を PNG に描けます。ポインタやクリックも模擬できます。

```sh
shoji-bar-4 preview dock out.png --pointer 470,110
shoji-bar-4 preview quick-settings out.png --width 460 --height 900 --scale 1 --light
shoji-bar-4 preview workspaces out.png --ms 0 --frames 40 --every 30   # out-0.png … out-40.png
```

対象は `bar`、`dock`、`workspaces`、`quick-settings`、`calendar`、`launcher`、`notifications` です。`preview` は設定を保存しません。

## AntiWebUI に足したもの

すべて汎用の機能として、ブランチ `shoji-bar-4-support` に入れました。

- `WindowOptions::background`：レイヤーの背景を、既定で透明にしました。
- md3e のダイナミックカラー：
  - `ColorScheme::from_seed` / `from_seed_with`、9 種のバリアント。
  - `seed_colors`（画像から色を取る）。
  - `ColorScheme::lerp`、`set_color_scheme_animated`。
  - material-color-utilities の移植です。
- md3e の形：`MaterialShape`（34 種）、`shape` / `styled_shape`（ばねで変形）、公開の `Ink`（状態レイヤーとリップル）。
- md3e：`DatePickerStyle::no_header()`。
- style：`overflow_x_scroll()`。
