# WSL USB Identity Manager

*[English](README.md) | [日本語](README.ja.md)*

[usbipd-win](https://github.com/dorssel/usbipd-win) で USB デバイスを WSL に転送する環境で、
**どれがどの物理デバイスなのか**を確実に把握するためのツール。

> **状態: 仕様策定中。** 実装はまだありません。
> 要件は [docs/requirements.ja.md](docs/requirements.ja.md) を参照してください。

## 解決する問題

WSL への USB 転送は動きます。同じアダプタが 2 個以上になるまでは。

- **CH340 はシリアル番号を持ちません。** 3 個挿さっていても、USB 記述子には
  区別できる情報が存在しません。Windows が答えられるのは
  「どのポートに刺さっているか」までです。
- **BUSID は安定した名前ではありません。** `usbipd` の BUSID は
  `Hub_#NNNN` と `Port_#MMMM` の組であり、ハブ番号は Windows がハブを
  認識した順に振られます。ドックを後から接続すれば、その先のデバイスの
  BUSID は全部変わります。**同じ BUSID が別のデバイスを指すようになります。**
- **COM 番号も `/dev/ttyUSB*` も**同じ理由で動きます。
- **アダプタやプローブが名乗るのは自分自身で、その先にあるものではありません。**
  CH340 のシリアル番号は（あったとしても）アダプタのものであって、
  配線されたボードのものではありません。WCH-LinkE のシリアル番号はプローブのもので、
  デバッグピンのボードを載せ替えてもその番号は変わりません。

つまり「CH340 を Ubuntu に attach して」という指示が、現状のツールでは成立しません。

## 何を識別するのか

「このデバイス」と言うとき、普通は手前のアダプタではなく**ボード**を指しています。
1 本のケーブルの先には、最大で 3 つの別々のものがあります。

| | 例 | 識別手段 |
| --- | --- | --- |
| **Port** | BUSID、`COM8`、`/dev/ttyUSB2` | 物理トポロジパス — 永続化しない |
| **Transport** | CH340、CH343、WCH-LinkE | USB 記述子（シリアル番号がある場合） |
| **Target** | ESP32-S3、CH32X035 | **ボード自身に問い合わせる** |

**アダプタやプローブの先にある Target まで識別することが、本ツールの目的です。**
ネイティブ USB のボードは Transport と Target が同一なので、
シリアル番号だけで足ります。それ以外はボードに聞くしかありません。

### 現在対応している Target

| 系統 | 識別に使うもの |
| --- | --- |
| **ESP32 系**（CH340 / CP210x 等の裏） | eFuse MAC とチップ種別 |
| **CH32 RISC-V 系**（WCH-Link / WCH-LinkE の先） | 部品 UUID とチップ署名 |

それ以外（Arduino、RP2040、STM32、または不明なボードが繋がった生のアダプタ）は
**Transport までの識別に留まり**、そのことを隠さずに表示します。
系統の追加は、識別処理を足すだけで済む構造にします。

## 方針

**「どこに接続されているか」**と**「それが何という物理デバイスなのか」**を分離します。

| | 例 | 永続化 |
| --- | --- | --- |
| Runtime Connection | BUSID、COM 番号、`/dev/ttyUSB0`、Attach 状態 | しない |
| Identity | USB シリアル、基板から読んだ ID、ユーザーが付けた名前 | する |

USB 記述子から特定できないものは、**接続先の基板そのものに問い合わせて**識別します。
これは [board-identify](https://github.com/tanakamasayuki/board-identify) が Linux 側で
到達したのと同じ結論であり、本ツールは**同じ識別子フォーマットを使う**ので、
両者が同じデバイスに同じ名前を付けます。

ただし問い合わせは対象を乱します。ESP32 の eFuse MAC を読めばファームウェアが再起動し、
WCH-Link に attach すればターゲットのコアが止まります。
そのため本ツールは**定期的な問い合わせを行いません**。
ボタンを押したとき、あるいは（オプションを有効にした場合は）
まだ誰も使っていない接続直後の短い時間内にのみ実行します。
それ以外はキャッシュで答え、**すべてのデバイスに確信度を表示する**ので、
推測が事実として表示されることはありません。

## スコープ

本ツールが受け持つのは Windows 側です。
デバイスの列挙、識別、`usbipd` の状態と操作、Attach 先 Distribution の管理、
ユーザー定義の名称とメモ。

**WSL 内部は管理しません。** udev ルールもデバイスノードの権限も
シンボリックリンクも、既存の Linux 側の仕組みに委ねます。
情報が取得できる場合に表示するだけで、変更は行いません。

## 動作要件

- Windows 10 / 11（x64）
- [usbipd-win](https://github.com/dorssel/usbipd-win) 5.x
- WSL 2
- WebView2 Runtime — Windows 11 には同梱。Windows 10 でも大多数の端末に導入済み

## インストール

初回リリース後に対応予定です。

```console
winget install <package-id>
```

インストールを避けたい場合向けに、リリースページでポータブル ZIP も配布します。

## ドキュメント

- [要件定義](docs/requirements.ja.md)
- [事前調査結果](docs/research-findings.ja.md) — 本設計の根拠となる実測データ
- [識別ポリシー](docs/identification-policy.ja.md)
- [言語・フレームワーク・配布方式の評価](docs/platform-evaluation.ja.md)

## 関連プロジェクト

- [board-identify](https://github.com/tanakamasayuki/board-identify) — WSL 側の対応ツール。
  WSL 内に安定したシンボリックリンクを発行する
- [ch32rv](https://github.com/ch32-riscv-ug/ch32rv) — WCH-Link プローブの実装
- [usbipd-win](https://github.com/dorssel/usbipd-win) — 本ツールが操作する USB/IP ホスト

## ライセンス

MIT。[LICENSE](LICENSE) を参照。
