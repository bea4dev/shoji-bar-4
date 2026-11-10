# shoji-bar-4

[ShojiWM](https://github.com/bea4dev/ShojiWM) 用の Material 3 Expressive なデスクトップシェルです。Rust と [AntiWebUI](https://github.com/bea4dev/AntiWebUI)（`anti-webui-wayland-shell` と `anti-webui-md3e`）で書いています。

## できること

- **上のバー**：浮いた島が並びます。
  - ランチャーのボタン：Material Shapes が変形します（Cookie4 → Cookie9 → Circle）。
  - ワークスペース：前の端が速いばね、後ろの端が遅いばねで動き、伸び縮みしながら移る目印です。ホイールでも切り替えられます。ワークスペースが増えたり減ったりすると、その場所が開いたり閉じたりして、島の幅も一緒に変わります。
  - アクティブなウィンドウのアイコンとタイトル。
  - 時計：数字が転がって変わり、ホバーすると Roboto Flex の太さが変わります。
  - 再生中の曲（踊るイコライザー）：クリックで、その下にプレーヤーのパネルが開きます。中クリックで一時停止／再生します。
  - トレイ、ネットワーク・Bluetooth・音量・電池。
- **下のドック**：ポインタの下のアイコンが少し浮き上がり、名前が出ます。
  - ふだんは画面の下に隠れています。ポインタを画面の下の端に持っていくと上がってきて、ポインタが離れると少しして下がります。ウィンドウは画面の下まで使えます。
  - ピン留めしたアプリ、区切り線、ほかの開いているアプリの順に並びます。
  - 起動中のアプリはウィンドウが出るまで跳ねます。
  - 下の点で、ウィンドウの数とフォーカスを示します。
  - 右クリックのメニュー：新しいウィンドウ、デスクトップアクション、ピン留め、閉じる。
- **クイック設定**：
  - タイル：オンで丸くなり、色が変わります。
  - 音量と明るさのスライダー、電源メニュー。
    - Lock、Sleep、Log out、Restart、Shut down。ログアウト（ShojiWM を終える `shoji_wm --quit`）、再起動、シャットダウンは、画面の真ん中のダイアログで確かめてから行います。まわりはランチャーと同じく幕でぼかします。アイコンの形は回りながら出てきて、実行ボタンにポインタを乗せると太陽の形に変わります。キャンセル、Escape、外のクリックで何もせずに閉じます。
  - 電池のカード：満ちていく電池の絵と転がる数字、空になるまで／満タンまでの時間とその時刻、状態のピル（充電中・電池・満タン・充電しない）、電力、劣化具合、充電回数。充電中は電池の中を光が流れます。
  - 通知の欄はいつも出ます。ないときは「No notifications」と出ます。
  - 歯車（またはユーザーの絵）で設定のページに移ります。
    - ユーザーの絵：選ぶと `$XDG_STATE_HOME/shoji-bar-4/` に写しを取っておきます。選んでいないときは `~/.face.icon`、`~/.face`、AccountsService の絵を使います。選ぶには zenity が要ります。
    - スタイルと色：壁紙、種の色、配色のバリアント、ライト／自動／ダーク。
    - 時計：24 時間表示、秒の表示。
- **カレンダー**：秒まで転がる大きな時計、月のカレンダー、CPU・メモリ・温度の波打つリング。
- **ランチャー**：画面の真ん中に出ます。検索に合わせて、グリッドが FLIP で並び替わります。開くときは順番に現れます。矢印キー、Enter、Esc で操作できます。
- **プレーヤー**：バーの再生中の島の下に開きます。
  - ほかのパネルと同じテーマの色の台に、カードが載っています。カードにはジャケットを敷き、暗くして文字を読めるようにします。曲が変わると、ジャケットとタイトルが入れ替わります。
  - 波打つシークバー：クリックでそこへ移ります。ドラッグすると、離したところへ移ります。ポインタが乗ると、端に取っ手が出ます。
  - 経過時間と長さ、前へ・再生／一時停止（再生中は丸くなる大きなボタン）・次へ。
- **クリップボードの履歴**：cliphist が覚えているものを、新しい順に画面の真ん中に出します。
  - 文字はアイコン（文・リンク・パス）と最初の 2 行、画像は小さな絵と大きさで表示します。
  - 検索で絞り込めます。画像は `png` や `image` で探せます。
  - ↑↓（PageUp／PageDown）で選ぶと、下のハイライトが伸び縮みしながら移り、リストもそこまでばねでスクロールします。
  - Enter かクリックで、もう一度クリップボードに入れて閉じます。Delete（検索中は Shift+Delete）か × で忘れさせると、その行が閉じて下の行が上がってきます。
  - 開くときは、行が順番に落ちてきます。
  - cliphist に `wl-paste --watch cliphist store` で記録させておく必要があります（`~/.config/shojiwm-md3e` は起動します）。
- **ロック画面**：すべての画面を `ext-session-lock` で覆います。
  - 壁紙にベールをかけ、転がる大きな時計と日付、ユーザーの絵と名前、パスワードの欄を出します。開くときは順番に入ってきます。
  - パスワードは打つたびに Material の形（クッキー、クローバー、花…）が弾んで並びます。Enter で確かめる間は、形がローディングインジケーターに変わります。
  - 違うと欄が揺れて赤くなり、理由が出ます（faillock でアカウントがしばらく止められたときは、PAM の言葉のまま）。合うと全部が大きくなりながら消え、ベールが上がって、同じ壁紙のデスクトップに戻ります。
  - Esc／Ctrl+U で消去、Backspace で 1 文字戻ります。
  - パスワードは PAM（`login` サービス、`pam_authenticate` のみ）で別のスレッドで確かめます。
  - `shoji-bar-4 msg lock`、logind からの Lock（`loginctl lock-session`、クイック設定の電源メニューの Lock）、スリープの前にロックします。外から解除する命令はありません。
  - シェルがロック中に落ちても、ShojiWM はロックしたままにします。そのときは TTY から別のロッカー（`swaylock` など）を同じセッションで起こすと、ロックを引き継いで解除できます。
  - Arch の既定の faillock では、15 分のうちに 3 回まちがえると、10 分間ログインできなくなります（正しいパスワードでも）。
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

AntiWebUI は GitHub の `main` を参照していて、使うコミットは `Cargo.lock` で固定されています。新しい AntiWebUI に上げるときは:

```sh
cargo update -p anti-webui -p anti-webui-md3e
```

手元の AntiWebUI を直しながら試すときは、コミットしない `.cargo/config.toml` で差し替えられます（`.gitignore` 済み）。

```toml
[patch."https://github.com/bea4dev/AntiWebUI.git"]
anti-webui = { path = "/path/to/AntiWebUI/src/anti-webui" }
anti-webui-md3e = { path = "/path/to/AntiWebUI/src/anti-webui-md3e" }
```

## コマンド

起動中のシェルに、ソケット（`$XDG_RUNTIME_DIR/shoji-bar-4.$WAYLAND_DISPLAY.sock`）経由で命令を送れます。

```sh
shoji-bar-4 msg toggle launcher [OUTPUT]       # OUTPUT を省くと、フォーカスのあるウィンドウの画面
shoji-bar-4 msg toggle quick-settings [OUTPUT]
shoji-bar-4 msg toggle calendar [OUTPUT]
shoji-bar-4 msg toggle clipboard [OUTPUT]
shoji-bar-4 msg toggle media [OUTPUT]
shoji-bar-4 msg close
shoji-bar-4 msg lock                             # 解除は、パスワードでだけ
shoji-bar-4 msg theme dark|light|auto|toggle
shoji-bar-4 msg status                         # 開いているパネル、動いている時計の数など
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

[`~/.config/shojiwm-md3e`](../shojiwm-md3e) は、これを済ませた ShojiWM の設定です。この設定では、ウィンドウの枠もシェルと同じ配色になります。

- shoji-bar-4 は、ShojiWM の IPC ソケット（`$XDG_RUNTIME_DIR/shojiwm-$WAYLAND_DISPLAY.sock`）につながったままにします。
- そこに配色を `theme.set` で送ります（`src/wm.rs`）。
- `theme.set` を知らない設定では、何も起きません。

レイヤーの名前空間は `shoji-bar-4-bar`、`-dock`、`-quick-settings`、`-calendar`、`-launcher`、`-clipboard`、`-media`、`-notifications`、`-osd`、`-wallpaper` です。ShojiWM のレイヤーエフェクト（ぼかしなど）を当てるときに使えます。

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

## CPU を使い続けるときの調べ方

```sh
scripts/cpu-report.sh 10               # 今すぐ 10 秒調べて報告を書く
scripts/cpu-report.sh watch 8 30       # 常駐して、8% を 2 回続けて超えたら自動で報告を書く
scripts/cpu-report.sh --sudo watch     # perf の関数ごとの時間も取る（perf だけを sudo で動かす）
```

報告は `~/.local/state/shoji-bar-4/reports/` に書きます。

- スレッドごとの CPU と目覚めの回数：UI スレッドが毎秒 60〜120 回目覚めていれば、フレームを描き続けています。
- `msg status`：止まらない時計（`ticking`）がないか、UI スレッドがすぐ答えるか。
- perf：時間を使っている関数。release ビルドは、このために関数名を残しています。
- D-Bus：シグナルが大量に来ていないか。

最後に、これらからの見立てを書きます。

## 開発用：オフスクリーンの絵

コンポジタの面を使わずに、各部分を PNG に描けます。ポインタやクリックも模擬できます。

```sh
shoji-bar-4 preview dock out.png --pointer 470,110
shoji-bar-4 preview quick-settings out.png --width 460 --height 900 --scale 1 --light
shoji-bar-4 preview workspaces out.png --ms 0 --frames 40 --every 30   # out-0.png … out-40.png
shoji-bar-4 preview quick-settings out.png --click 1548,80 --click-frames 1 --ms 0 --frames 8 --every 40
```

`--drag X1,Y1:X2,Y2` は、クリックのあとに 1 点目で押し、2 点目まで動かして離します。

`--key K` は、クリックのあとにキーを押します（`ArrowDown`、`ArrowUp`、`Enter`、`Delete`、`Escape`、またはその文字を入力）。何度でも書けます。`clipboard` での Enter と Delete は、本物の cliphist とクリップボードに効きます。

`--click-frames` は、クリックのあとに進めるフレーム数です（既定 20）。クリックで始まる動きを、最初から撮るときに小さくします。

対象は `bar`、`dock`、`workspaces`、`quick-settings`、`calendar`、`launcher`、`clipboard`、`media`、`lock`、`notifications` です（`lock` のパスワードは `shoji`。PAM は使いません）。`preview` は設定を保存しません。

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
- 画像のメモリ：大きな写真は 1 枚ずつ、RGB のまま縮小してデコードします。glibc のヒープに残った分は OS に返します。`images::decode_bitmap`（表示せずに画素を取る）も足しました。
- shell：`Battery` の `energy()`、`energy_full()`、`health()`（劣化具合）、`charge_cycles()`（充電回数）。
- md3e：`wavy_progress_eased`（線が新しい値へ動く速さを選べる波打つ進捗バー。シークバー向け）。
