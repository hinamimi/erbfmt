# Roadmap

更新日: 2026-11-16 / 公開済み: v0.3.3 / 次回リリース: v0.3.4

この文書は、現在できることと、次に設計・実装することを区別するための
ロードマップです。リリース履歴やauditの作業ログは専用ドキュメントに残します。
以下の優先順位は開発方針であり、リリース時期や次のバージョンへの搭載を約束するものではありません。

## v0.3.4向けの優先度1対応

- formatter安全性: Forem / OpenStreetMapの990テンプレートを再走査し、
  5ファイルで見つかった2種類のidempotency不具合を修正。通常・fragment許可の両方で
  受理した全ファイルが安定。[監査結果と再実行手順](Audit20261116.md)を追加した。
- Quick Fix: Rust由来のignore位置、連続ignoreによる抑制の合成、文書バージョンを確認する
  適用処理、編集後の診断自動更新を実装。Extension Host testをCIに追加した。
- JSON: 構文診断の構造化location、lintのignore編集、assistの対応ruleとignore候補を追加。
  [JSON契約](JsonInterface.md)、対応表、Unicode / CRLF / error / streamの契約テストを追加した。

以下の各項目は継続的な改善方針です。受理できない構文をすべて解消した、
または実テンプレートの見た目をすべて検証済みという意味ではありません。

## v0.3.4向けのagent skill

`skills/erbfmt/SKILL.md`を追加し、`npx skills add hinamimi/erbfmt`で導入できる
構成にした。CLI本体のnpm wrapperとは別の、既存CLIを使うためのinstruction package。
ローカルでの検出・Codex / Claude Code向けcopy installをCIで検証する。
GitHubからの導入はpush後、`@v0.3.4`による固定はtag公開後に利用できる。

## 基本方針

erbfmt は `*.html.erb` 向けの formatter / linter です。
Prettier、Biome、dprint に近い編集体験を、ERB template に提供します。

- formatter / linter / assist のengineはRustに置き、wrapperに構文処理を重複実装しない。
- rendered HTMLの意味とRubyの実行内容を保持することを、整形範囲の広さより優先する。
- 安全に判断できない構文は保持または診断し、対応範囲を広げる際はfixtureを先に追加する。
- parser、formatter、linter、editor連携は、小さく分けて変更・検証する。
- 設定・CLI・editorで同じルールと修正候補を使う。

## v0.3.3時点でできること

以下は導入予定ではなく、現在の実装です。実装済みであっても、対応構文や安全判定の拡張は継続します。

| 領域 | 実装済みの内容 |
| --- | --- |
| HTML + ERBの解析 | ERB lexer、HTML tokenizer、mixed template parser。基本のERB制御構文を扱い、HTML optional closing tag / unbalanced fragmentは設定で許可できる。 |
| formatter | indentation、attribute wrapping、ERB call wrapping、inline whitespaceやraw text領域の保持。unit / snapshot / idempotency testと実テンプレートauditのfixtureがある。 |
| Rubyの安全判定 | 内部の部分的なRuby ASTと保守的なwrapping処理。helper名によるRails semantic analysisやRuby全体のpretty-printは行わない。 |
| linter | 12ルール、ルール別severity設定、位置付き診断。HTML属性・nestingとERB構文に関する診断を提供する。 |
| ignore | HTML / ERBコメントの `format`、`lint`、`lint/<rule>`、`all`。formatterは次のノード全体、lintは次の物理行が対象。 |
| assist | 属性の二重引用符化、HTML自己終了タグ修正、冗長なboolean属性値の除去、空のERBタグ削除。候補のJSON出力と `--assist --write` が使える。 |
| CLI / AI連携 | `--check`、`--write`、`--lint-format json`、`--assist-format json`、`--stdin-filepath`。複数ファイルのJSON出力はファイルごとに1行で返す。lint JSONには `rule` がある。 |
| VS Code | 整形、診断、コメント切替、Bundler経由のCLI実行。v0.3.3で該当整形差分の適用、対応lintの個別修正、局所的なignoreのQuick Fixを追加した。 |
| 配布 | v0.3.3をGitHub ReleasesとRubyGems.orgで公開済み。4プラットフォームのbinary / gem、Ruby fallback gem、VSIXを提供する。VSIXにはCLIを同梱しない。 |

実装の配置は次のとおりです。

- `src/syntax/erb/`: ERB lexer
- `src/syntax/html/`: HTML tokenizer / tag utilities
- `src/syntax/template/`: HTML + ERB mixed parser
- `src/syntax/ruby/`: 安全判定用の部分的なRuby AST
- `src/formatter/`: formatting policyと出力生成
- `src/linter/`: ruleとdiagnostic
- `src/assist/`: 修正候補とTextEdit
- `src/ignore/`: 共通のignore directive解析
- `editors/vscode/`: CLIを呼び出すeditor連携

## 優先度1: formatterの安全性（今回対応済み）

- [x] Forem / OpenStreetMapの990テンプレートを通常・fragment許可設定で再走査した。
- [x] 5ファイルの不安定な整形を2種類の回帰テストへ縮小し、修正した。
- [x] inline境界の空白保持とRubyチェーン継続indentの既存保護テストを維持した。
- [x] 受理した全ファイルで2回目のformat差分が出ないことを確認した。
- [x] fragment許可で新たに受理する6ファイルの境界を確認し、cross-ERB-branch HTMLは
  両設定で拒否する契約テストを追加した。parserの受理範囲・既定値は変えていない。
- [x] [監査結果](Audit20261116.md)と読み取り専用の再実行scriptを追加した。

継続方針:

- 別の実テンプレートやupstream更新を確認する際も、具体的なdiffからfixtureを追加する。
- Ruby wrappingや受理構文を広げるときは、保持・拒否するケースも対で追加する。
- idempotencyだけでrendered HTMLの同値性を保証したと扱わない。

## 優先度1: Quick Fixの改善（今回対応済み）

- [x] ignoreの挿入位置をRustのtemplate nodeから生成するようにした。
  複数行属性の途中、Ruby文字列、raw text内部、構文エラーには候補を出さない。
- [x] 連続ignoreは同じ次のtarget行へ適用する契約にした。
  別ルールやformatのignoreを追加しても既存の抑制を解除しない。
- [x] Rustのrule / assist対応情報を使い、editor側のルール対応表を取り除いた。
- [x] 編集で古い診断を破棄し、300msの待機後に再診断する。
  古い処理はcancelし、保存・手動lint・修正適用後は即座に更新する。
- [x] 修正適用時にも文書バージョンを確認し、古いメニューからの編集を拒否する。
- [x] 隣接差分、挿入・削除、末尾改行、CRLF、日本語・絵文字のテストを追加した。
- [x] 実CLIを使うExtension Host testをローカルで実行し、CIのLinux環境に
  `xvfb-run`による実行を追加した。GitHub上の実行結果はpush後に確認する。

継続方針:

- 次の物理行というlint ignoreの契約は維持する。複数行属性の途中をignoreするには
  別の対象範囲の設計が必要なので、現段階では候補を出さない。
- 新しいQuick Fixも、文書更新中と複数診断がある状態で結合テストする。

## 優先度1: JSONと修正候補の契約（今回対応済み）

- [x] [JsonInterface.md](JsonInterface.md)にフィールド、stream、exit status、
  error時のstdout / stderr、rule / assist対応表、互換性方針をまとめた。
- [x] `location`は1始まりのUnicode scalar位置、rangeは0始まり・終端を含まない
  UTF-8 byte offsetとして明記し、VS CodeのUTF-16への変換をテストした。
- [x] 構文診断の位置をJSONの構造化locationで返し、messageからの解析を不要にした。
- [x] lintのignore編集と、assistの対応rule・ignore対象情報をJSONへ追加した。
- [x] `AI.md`の例を現在のruleとassist対象rangeへ同期した。
- [x] 複数file、severity、構文 / 設定 / I/O失敗、対象外file、Unicode / CRLF、
  ignore合成についてCLI契約テストを追加した。

継続方針:

- 追加フィールドを許容し、古いCLIには存在しない任意情報を利用側が扱える仕様にする。
- rule / action ID、位置の単位、出力先、exit statusの変更には移行説明と契約テストを付ける。
- 全失敗がJSONになるとは保証しない。構文診断と設定・実行エラーの出力を区別する。

## 優先度2: lint / assistの精度を改善する

severity設定、duplicate attribute、boolean attribute、unknown attributeの初期実装は完了済みです。
ルール数を増やす前に、現在のルールの誤検知と修正の安全性を改善します。

次にやること:

- HTML content model、unknown / typo attribute、ERBを含む属性の具体例を増やし、
  diagnosticの位置とmessageを改善する。
- 現在の4種類のassistについて、適用前後の意味と、適用しない条件をfixtureで固定する。
- lint ignoreで抑制した箇所とassist候補の関係を決め、CLIとeditorの挙動をそろえる。
- 新規rule / assistは、実テンプレートの問題例、安全な編集範囲、既存ruleとの重複を
  確認してから追加する。Rails helperの意味を推測するruleは対象にしない。

完了条件:

- diagnosticと修正候補の対応が明確で、誤ったルールの修正を提示しない。
- 追加したケースごとに検出・非検出と、修正する・保持するテストがある。
- `samples/lint-next.html.erb` とruleドキュメントが実装に一致する。

## 優先度2: 導入とリリース運用を整える

GitHub Releases / RubyGems.org向けの配布は実装・運用済みです。
次は導入時の迷いと手作業を減らします。

次にやること:

- CLIとVSIXの対応バージョン、古いCLIで利用できないQuick Fix、
  binary未導入・実行権限エラーの案内を整理する。
- Ruby LSPとの併用時に、language ID、formatter選択、診断の役割を説明する。
- Marketplaceの手動VSIX更新手順と、公開後にバージョンを確認する手順を文書化する。
- Marketplace自動公開はTrusted Publishingなどの認証方式を検討し、
  GitHub Release・RubyGems・Marketplaceそれぞれの公開完了を確認できるようにする。
- CLIのbundle / downloadは、配布元、checksum検証、対応OS、更新・失敗時の挙動を
  決めてから導入判断する。Bundlerで固定したCLIを選べる設定は維持する。

完了条件:

- Rustをビルドしない利用者が、対応プラットフォームで導入できる。
- release assetの作成、registryへの公開、インストール検証が区別された手順になっている。
- 初回導入でも、使用中のCLIとextensionのバージョン・実行元を確認できる。

## 必要になった時点で行う整理・計測

独立した大規模リファクタリングを先行させず、変更する機能に合わせて分割します。

- template parser、HTML lint rule、CLI orchestrationの責務が増えた箇所を分割する。
  分割前後で受理構文、診断、format出力が変わらないことを確認する。
- syntax層にformatter / linterのpolicyを持ち込まず、assistも共有の構文情報を使う。
- 大きなテンプレートやディレクトリ実行で処理時間・メモリを計測し、
  性能改善は再現できる計測結果に基づいて行う。

## 当面の対象外

- Ruby全体のpretty-printer / Rails semantic analysis
- LSP、Tree-sitter導入、Biome integration、formatter plugin API
- 利用上の必要性が確認できていないnpm wrapper、crates.io / Open VSX公開
- correctnessを保証できない一括autocorrect

既存のassistやVS Code Quick Fixは、この対象外には含めません。

## 関連ドキュメント

- [Configuration.md](Configuration.md): `erbfmt.json`
- [LintRules.md](LintRules.md): lint rule
- [Ignore.md](Ignore.md): ignore directive
- [AI.md](AI.md): CLI / JSON interface
- [JsonInterface.md](JsonInterface.md): JSON契約とrule / assist対応表
- [VSCode.md](VSCode.md): editor連携
- [Release.md](Release.md): release verification
- [Distribution.md](Distribution.md): 配布方針
- [RubyGem.md](RubyGem.md): Ruby wrapper
- [ForemAudit.md](ForemAudit.md) / [OpenStreetMapWebsiteAudit.md](OpenStreetMapWebsiteAudit.md): 実テンプレートaudit
- [Development.md](Development.md): 開発・構成
