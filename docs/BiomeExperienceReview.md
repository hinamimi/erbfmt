# Biome と比べた erbfmt の使い心地

- 調査日: 2026-11-18（JST）
- 比較対象: Biome CLI **2.6.8**、erbfmt **0.3.4** の開発時点（`6c3e76f`）

## 次に優先すること

**Rails プロジェクトで CLI・VS Code・CI が同じバージョンと設定を使い、
整形できない理由と直すべき差分がすぐ分かるようにする。** これが、今の erbfmt を
Biome に近い使い心地にする一番短い道だと考える。実テンプレートでの安全性検証は
引き続き土台とし、ルール数や言語対応の拡大より、導入から最初の修正までの手順を減らす。

## Biome 2.6 系で参考になること

調査時点の最新安定版は **2.6.8**。11 月 15 日公開の
[GitHub リリース][biome-release]と [npm の `latest`][biome-npm] で確認した。

- **実行結果を説明できる。** 2.6 の `biome inspect config --path` は、対象パスに
  適用された設定とその出所を確認できる。erbfmt でも「CLI は動くのに保存時は動かない」
  とき、実行バージョン・設定ファイル・対象外になった理由を一緒に見られると切り分けやすい。
- **混在構文への対応は段階的。** 2.6 では Tailwind 連携や言語対応を広げている一方、
  HTML・Vue・Svelte・Astro の完全なマークアップ処理は引き続き実験的で、HTML formatter
  も明示的に有効化する。erbfmt でも HTML と Ruby が入り混じる難しい境界は、出力の
  安全性を確かめた条件から順に広げるのが妥当。比較はワークフローを対象とし、Biome の
  HTML formatter を ERB に流用する提案ではない。
- **パッチでも出力の安定性を直す。** 2.6.8 は inline 要素や式に続くテキストの
  余分な改行を修正し、再整形しても同じ結果になるようにした。erbfmt の
  [実テンプレート監査](Audit20261116.md)でも似た領域に不安定な例があった。
  inline の空白はテストケースを蓄積する価値が高い。

出典: [Biome 2.6 の発表][biome-26]、[2.6.8 の変更一覧][biome-release]、
[HTML 設定][biome-html]。

## 利用場面ごとの比較

| 場面 | Biome 2.6.8 | erbfmt 0.3.4 | 次に詰めたい点 |
| --- | --- | --- | --- |
| 初回導入 | npm でバージョンを固定でき、`init` 後の案内からチェックとエディタ設定へ進める。単体バイナリの導入経路もある。 | RubyGems と Bundler で Rails プロジェクトに固定できる。`init` は設定を作る。VS Code 拡張は CLI の別途導入が必要で、Bundler を使う場合はコマンドを設定する。 | Bundler を基準とした導入ガイドと、拡張からの設定・確認手順を一本にする。 |
| 保存・個別修正 | VS Code で保存時整形、診断に対する修正、保存時の安全な修正を設定できる。 | 保存時に使えるフォーマッター、編集中の診断、整形差分ごとの Quick Fix、lint の個別修正、局所 ignore がある。整形と assist は別の操作。 | CLI を解決できない場合や、未対応構文で修正を出せない場合に、原因と次の操作を示す。 |
| 例外・設定の調査 | `inspect config` で設定の出所を見られる。抑制コメントにルール・範囲・理由を記録できる。 | `Show Command` で実行コマンド、作業ディレクトリ、設定パスなどが分かる。lint と整形の ignore を重ねられるが、lint は次の対象行、整形は対応するノード境界に制約がある。 | `Show Command` と CLI から、実効バージョン、設定の出所、対象外・処理不可の理由を確認できるようにする。 |
| PR と CI | `biome ci` で整形と lint などをまとめて検証できる。GitHub 注釈とコミット差分に絞る実行をサポートする。 | `--check` と `--lint` は別実行。lint は NDJSON を使えるが、`--check` は整形が必要なファイル名を表示し、差分や機械可読出力はない。 | 両方を確認する CI の入口と、ファイル・行・次の操作が分かる出力を用意する。 |
| ツール・AI 連携 | CLI に人向け・機械向けの reporter がある。 | stdin と元のファイルパスを渡し、lint の位置や assist の編集範囲を JSON で受け取れる。導入用スキルもある。 | 現在の [NDJSON 契約](JsonInterface.md)を維持し、整形チェックの結果も扱えるようにする。 |

出典: Biome の [Quick start][biome-quickstart]、[VS Code][biome-vscode]、
[CLI][biome-cli]、[CI][biome-ci]、[抑制コメント][biome-suppressions]。
erbfmt の [VS Code](VSCode.md)、[設定](Configuration.md)、[ignore](Ignore.md)、
[CLI](commands.md)、[AI 連携](AI.md)。

### CLI で確認したこと

一時ディレクトリに最小の JS と HTML+ERB を用意し、それぞれの対応言語で操作を試した。
速度の比較や、両者の整形品質を同じ入力で比べる試験ではない。VS Code と配布の比較は
公式資料とローカル実装の確認に基づく。

- **Biome:** `biome check demo.js` は lint の理由、安全な修正候補、整形差分を
  一回の実行で表示し、ファイルは変更しなかった。`biome ci --reporter=github demo.js`
  は位置付き注釈を出した。`biome check --write demo.js` で修正と整形を行うと
  `biome ci demo.js` が通った。`biome inspect config --path demo.js formatter.indentStyle`
  で設定値と出所を確認した。`demo.html.erb` は `biome format` の処理対象にならなかった。
- **erbfmt:** `init` 後、インデント不足と単一引用符の属性を含む ERB を `--check`、
  `--lint`、`--assist` で確認した。チェックはファイル名、lint は位置と理由、
  assist は引用符の修正候補を表示し、確認中はファイルが変わらなかった。
  `--assist --write`、`--write` の順に適用すると、`--check` と `--lint` が両方通った。

## 進め方と完了の目安

| 順序 | 提案 | 完了の目安 |
| --- | --- | --- |
| 継続・最優先 | **ERB の安全性を確かめる。** 既存の監査を繰り返し、inline 空白、属性内の ERB、分岐をまたぐ HTML を中心に、受理・拒否とその理由を記録する。意味に影響し得る変更は、再整形時の一致に加え、代表的な出力を比較する。 | Forem と OpenStreetMap の固定リビジョンで受理・拒否件数を追跡し、再整形で差分が出た事例や、表示に影響した事例を回帰テストへ落とせる。 |
| 次の着手 1 | **Bundler と VS Code の導入をつなぐ。** `bundle exec erbfmt` と `Gemfile.lock` を基準に、設定例、保存時整形、Ruby LSP と併用する場合の設定を短く案内する。既存の `Show Command` を使い、実際に起動した CLI のバージョンと、設定未検出・対象外・実行失敗を切り分ける。 | グローバル erbfmt のない検証用 Rails プロジェクトで、Bundler のバージョンをターミナルと VS Code の両方から確認できる。解決に失敗したとき、その場で次の手順が分かる。 |
| 次の着手 2 | **読み取り専用の CI を一手順にする。** まず、同じ Bundler コマンドで既存の整形チェックと lint を両方実行する CI 例を用意する。その後に共通の CLI 入口を検討し、整形が必要な場所と小さな差分を返す。GitHub 注釈は構造化された結果から生成する。 | 整形違反と lint 違反が同じ PR で報告され、元ファイルを変更しない。対象ゼロ・設定エラー・違反ありを区別でき、既存の CLI と NDJSON を使う連携も継続して動く。 |
| その後 | **既存コードへ段階的に導入し、日々の修正を軽くする。** CI 例ではまず変更されたファイルを既存 CLI に渡す方式を試す。あわせて Quick Fix のラベルに対象と範囲を示し、局所 ignore の理由を記入しやすくする。 | CI 例が削除・改名・空白を含むパス・変更なしを扱える。設定変更時や定期実行では全件を確認できる。エディタでは修正の適用範囲が選択前に伝わり、古い候補は引き続き適用されない。 |

`.gitignore` の自動参照や組み込みの `--changed` は、まず CI 例で導入上の需要を確認する。
Biome の `--changed` は基準 ref に対するコミット済みの変更を対象にし、`--staged` は
別の機能なので、取り入れる場合も二つの意味を分ける。CLI 同梱は導入手順を整えた後の
候補とし、プロジェクトで固定した CLI を優先できることを条件にする。LSP 化は、大きい
ERB の診断・Quick Fix にかかる時間を実測し、起動や再解析がボトルネックの場合に再検討する。
Tailwind 専用ルール、プラグイン基盤、複雑な設定継承は今回の優先項目には入れない。

この調査は [公開ロードマップ](Roadmap.md) の方針を補足する開発資料。実装時期や
搭載バージョンは約束せず、検証で分かった導入のつまずき方に応じて順序を見直す。

[biome-release]: https://github.com/biomejs/biome/releases/tag/@biomejs/biome@2.6.8
[biome-npm]: https://www.npmjs.com/package/@biomejs/biome?activeTab=versions
[biome-26]: https://biomejs.dev/blog/biome-v2-6/
[biome-html]: https://biomejs.dev/reference/configuration/#html
[biome-quickstart]: https://biomejs.dev/installation/quick-start/
[biome-vscode]: https://biomejs.dev/reference/vscode/
[biome-cli]: https://biomejs.dev/reference/cli/
[biome-ci]: https://biomejs.dev/recipes/continuous-integration/
[biome-suppressions]: https://biomejs.dev/analyzer/suppressions/
