# ChoreoFix 研究開発計画

更新: 2026-11-05。受領した「ChoreoFix 研究開発計画」計画版1.0を反映した。
入力文書のSHA-256は[実験契約](benchmark/experiment-contract.json)に記録する。
研究の中心は、**実資源の唯一の操作権、残存手順、実際の共有効果を結ぶ保証**である。
委譲、投影、並行処理、prefix safety、版の照合をそれぞれ単独の新規性とはしない。

既存A比較の採点対象は **0 / 2,592実行**。追加B–Gは **0 / 1,368実行**で、
最大600実行の予備枠を含む総上限は **4,560実行**である。
追加課題、比較接続、精度設計は未認定であり、この上限を凍結・実行済みmatrixとは扱わない。
予備試験、部品試験、ビルド成功を採点済み実験へ数えない。

## 研究質問と範囲

| 問い | 必要な証拠 |
| --- | --- |
| RQ1 権限と効果 | 一意性、継続保存、受理済み要求と実効果の対応 |
| RQ2 合法な自由度 | 実結果による再考、異なる合法手順、独立作業の実際の重なり |
| RQ3 業務の完了 | 元の業務採点、不正効果、拒否、protocol完了を別々に測る対応比較 |
| RQ4 実装と保守 | 同じ変更に必要な宣言、localside、権限制御、テストの修正負担 |

第一論文は既知の有限role集合、実行時の委譲先選択、複数資源、結果依存分岐、
有限に実行された再考と独立作業の並行性を扱う。安全性は有限prefixにも適用する。
任意の動的トポロジー、永続クラッシュ復旧、外部APIのexactly-once、任意Rustの完全証明、
自然言語の意図の完全理解は完了条件に追加しない。範囲外の課題はnative未対応と記録し、
簡略化して成功にせず、通常のrun失敗にも変換しない。

## 最初の技術判断点: T3/T4と実効果

主ケースは、Aが資源Rで候補v1を作り、独立したCが資源Sで作業し、失敗後にAが
同じEndpointをBへ渡し、Bが候補v2を検証してRをAへ返す。AのR要求とBのS要求が
そろった後に公開する手順とする。
旧A、v1の承認、別の引数、片方しかない資源で実効果を起こせないことを確認する。
競合が通常のbroker、lease、proxyや再計画で同じ業務を完了しても成功と数える。

この[最小機構ケース](src/tasks/handoff)を実装した。AとBは別runtimeで、
失敗した候補からBが再修正・再検証し、RとSの両要求がそろった後だけ版付きの値を公開する。
独立したCはSで計算し、その実結果も公開物に含める。進行はglobalのpar、roll、routeと
直接のEndpoint操作で行い、資源の現在版・検証結果・公開物は物理サービスが所有する。
これはnativeのmemory-backed fixtureであり、実Wasm、実モデル、実成果物Gや採点実験ではない。

候補確定後もBがRを保持していた旧globalでは、受信側が再修正の受信を終えた後に
`Revise`を送信できた。`roll`の後続へ進んでも再訪は許されるため、globalの修正対象とした。
現在はBが検証結果をAへ送り、Rを返してから、Aがその結果を公開側へ送る。
AはR、BはSから公開要求を送り、受信側で両要求と実際の検証結果・現在版を照合する。
資源の保持者表、終了flag、localsideによる操作順の回避は追加していない。

通常の再考、旧承認・別対象・引数変更・負の検証結果・左右の要求不一致6種類、
片方の要求だけでPendingになる経路、承認偽装、実際の否定結果による修正なしの終了に加え、
候補確定後の再修正の拒否を確認した。handoffの6試験に含まれる11ケースと、RPC4試験・
入れ子の選択2試験の計12 native試験、Clippyが成功した。同じ11ケースを含む6試験の
strict-provenance Miriも成功した（255.70秒、最大431,669,248bytes）。拒否された再修正は
transportへframeを渡さず、公開0件の既存の不正要求試験も維持した。通常Git pinを使用し、
source hashと元の反例を`output/review-occurrence-qualification/runtime-array-input/handoff-post-ready`へ保持する。
本体の挙動は変更していない。この個別検証をアプリ全体の最終認定とは扱わない。

[Lean](proofs/README.md)では、任意のtarget・版・値について、両要求と実検証結果・現在版が
一致することと公開判定の同値性を証明した。既存coreのsource実行から操作点の重複がないことも
導いた。`EffectReceipts.lean`では、同じsource実行上で各効果が両資源の実受信を消費する
receiver契約と合成し、両要求・実検証結果・現在版の一致と、どちらの受信も二度目の公開へ
流用できないことを証明した。異なる資源の受信は別の出現になる。各要求の送信roleを対応する
資源へ結び、両資源のcustody履歴が同じsource wordへ射影される条件で、両方の権限上限と
効果の受信対応を合成した。許可する公理はpropext、Quot.soundのみ。
このreceiver契約はadapterの信頼境界であり、任意Rustから自動的に導かれるとはしない。
各受信・引数・実効果を投影と実行記録に結ぶ認定、残る受信側と他の実引数の対応は未完である。
新しいruntime台帳や出現IDは追加していない。

| 目標 | 命題と完了の証拠 | 現状 |
| --- | --- | --- |
| T1 完全実行の対応 | 評価で使うmessage/seq/par/route/resolver/roll/委譲について、globalと理想投影の完全実行を双方向に対応させる | 既存core定理の適用範囲と必要補題を対応付ける。研究全体の証明完了とはしない |
| T2 有限prefixの対応 | 非同期seq、join、分岐知識、任意の有限roll展開、移譲後の継続が合法prefixを保つ | 既存出現モデルを再利用する。深さ0/1/2の試験を一般命題の代わりにしない |
| T3 一意性と継続 | 初期binding、送信future、移送中、受信値、失効を含む全遷移で利用可能権限が高々一つで、継続が保存される | coreのcustody関連定理、1H排他試験、同じsource word上のreceiver契約との最小合成あり。具体的なR/S両binding・実受信・効果入口への適用と最終sourceでの認定は未完 |
| T4 実要求と実効果 | 各共有効果は一つの受理済み動的操作に対応し、対象・版・引数が一致し、必要な全資源と先行結果を満たす。拒否は効果も業務commitも生まない | 両資源の受信を消費する最小receiver契約下で、データ照合と受信の再利用禁止を合成済み。実要求・結果のlocalsideと記録監査あり。具体的な全効果入口との対応、他の引数への一般化と最終sourceでの認定は未完 |
| T5 実結果による分岐 | resolverが同じ動的操作の実結果で分岐し、別対象・旧版・以前のrollの結果を流用できない | 結果駆動の再考試験あり。対象・版・承認の一般的な対応は未認定 |
| T6 進行と条件付き完了 | 到達状態を正常完了、明示的失敗、外部応答待ち、合法遷移可能に分類し、終了する部分のprefixを既決定のまま延長できる | 既存進行定理の前提と適用範囲を照合する。非協力的guestの終了を保証しない |

T3の所在はghost stateとして定義し、runtimeへowner表や新しい権限tokenを追加しない。
利用可能な所有値と、受理後に送信futureが保持する権限ゼロの証拠を区別する。
送信側の局所I/Oエラーを、受信側へ移った権限の失効や再発行の根拠にしない。
初期発行と物理aliasの唯一性、FIFO・非重複配送・保持者からの受信可能性は明示した境界に置く。

`Domain::execute`は通常のhostメソッドである。受信した要求、実引数、必要資源の対応を
確認する箇所を[効果境界とadapter契約](benchmark/effect-boundaries.md)へ列挙する。
現在の一覧はWasm移行済み12課題の6familyを対象とし、残る36課題の認定とは区別する。
WASI import、公開host API、codec、物理bindingからの
迂回・引数差替え・再送を実効果の直前で試す。任意host Rustの効果を型だけで禁止したとは言わない。
新しい許可wrapperでEndpointを置き換えない。

最初の負例は旧所有者の遅延要求、二重移転、引数差替え、旧結果・旧承認、必要資源の欠落である。
故障は貸出、受理、結果受信、返却など同じ論理checkpointへ入れ、拒否直後の実資源を確認する。
現在の記録変異試験は監査の感度の証拠であり、これらの実行時防止試験とは別に数える。
効果成功後に応答が失われたら結果不明とし、権限を再発行せずfail-closedで終了する。

意味論のLean定理、投影データの検査、Rustの差分試験・Miri・必要なKani、Wasm/codec/adapter、
独立採点の信頼範囲を分ける。独自の未証明の正しさaxiomや未完証明を完成として公開しない。
アプリの適合性、公平なpoll、応答、容量、再考の終了は条件付き完了の前提として記録する。
`mem::forget`で権限やfutureを放棄するアプリまで終了させるとは主張しない。

## 固定する比較条件

- 原論文4条件とHibana: 48課題 × 3障害条件 × 3反復 × 5条件 = 2,160実行。
- 最新TraceFix監視条件: 同じ48課題・障害条件・反復で432実行を追加する。
- まず最新版432件と対応するHibana432件を優先し、全2,592件を完了する。
- 原論文比較は `87b01a2c0ae53c5045c9ce3d43cd0235b1831275`、最新版比較と入力は
  `36b4bd108fcd74f7efb17613aebbe3fdf55499a9`。固定前にremoteの変更を確認し、
  既存の記録済み版を黙って置き換えない。比較側のsource・実行エンジンは変更しない。
- 全条件の参加モデルは `gpt-6-luna`、`xhigh`。既存ChatGPT契約のApp Server接続を使う。
  APIキー課金、別モデル、effortの代替を許さない。開発サブエージェントは使わない。
- 課題の目標、参加者、物理資源、安全性、必要なデータ依存と元simulatorの採点を保つ。
  ChoreoFixの手順は改善してよい。変更は採点前に
  [実験契約](benchmark/experiment-contract.json)へ記録する。
- seed、遅延、timeout、tool上限、実行数は[実験設定](experiment.json)と生成matrixで固定する。
  新たな生成比較protocolは、モデル検査だけでなく元のdomain toolとの一致も確認する。
  不適合な派生入力は再生成し、baselineの課題失敗として数えない。

### 比較版

| 対象 | 計画の基準SHA | 認定 |
| --- | --- | --- |
| ChoreoFix | `0954383fe40c5a6e14d330b13c709ff9ff0db9fd` | 更新計画が参照する版。後続の公開進捗やdirty sourceを本比較版と同一視しない |
| TraceFix 原論文 | `87b01a2c0ae53c5045c9ce3d43cd0235b1831275` | 接続・入力認定は未完 |
| TraceFix 最新 | `36b4bd108fcd74f7efb17613aebbe3fdf55499a9` | 接続・入力認定は未完 |
| ZipperGen | `349e0c05b150139a1e1b25243eaf0f9edcb3baad` | 新規。source、native表現範囲、接続、保証の監査が必要 |
| Concordat | `0e3b185167b963df41c68b6dbacd3488c1063e3d` | 新規。source、native表現範囲、接続、保証の監査が必要 |

これらは受領計画の基準版であり、全接続の実験済み認定ではない。変更時は理由と再認定範囲を残す。
TraceFix最新版の順序検査、ZipperGen最新版のparallel/spawn/CPL、Concordatの人間決定・予算分割を
無効化した比較を各方式の代表にしない。文献上の主張は固定sourceと一次資料で確認する。
関連研究は受領計画の[POPL 2008](https://mrg.cs.ox.ac.uk/publications/multiparty-asynchronous-session-types/)、
[ZipperGen論文](https://arxiv.org/html/2604.17612v3)、[CPL論文](https://arxiv.org/html/2605.20923v2)、
[Concordat論文](https://arxiv.org/html/2610.37461v1)を含めて位置付ける。

### 実行予算と課題選定

| 区分 | 設計 | 新規実行数 |
| --- | --- | ---: |
| A1 | 最新版TraceFix432 + ChoreoFix432 | 864 |
| A2 | 原論文4条件を各432 | 1,728 |
| B 共通課題 | 12課題 × 3障害 × 3反復 × ZipperGen/Concordat | 216 |
| C 航空業務 | 50課題 × 通常/pressure × 3反復 × Concordat/ChoreoFix | 600 |
| D ZipperGen例 | 4workflow × 2結果 × 3反復 × 2方式 | 48 |
| E 引継ぎ | 6業務 × 固定/引継ぎ × 3反復 × 4方式 | 144 |
| F1 手順作成/runtime | 8課題 × expert/Luna生成 × 2runtime × 障害0/2 × 3組 | 192 |
| F2 shell/並行性 | 8課題 × 2障害 × 3反復 × structured-tools/直列化の追加2条件 | 96 |
| G 実成果物 | 2ケース × 正常/結果失敗/引継ぎ × 3反復 × 4方式 | 72 |
| 通常分の合計 | 既存A 2,592 + 追加B–G 1,368 | 3,960 |
| 予備枠 | 非採点pilotで用途と精度を事前設計する | 最大600 |
| 総上限 | protocol生成、build、証明、決定的試験、非採点接続確認は別計上 | 最大4,560 |

既存`experiment.json`とrunnerのmatrixはAのみを実行できる。追加B–Gをそこへ未実装のまま
足したり、上限4,560を採点可能件数として表示したりしない。
Bは原48課題から4family×3難易度を結果を見る前のnative表現監査で選ぶ。
AのChoreoFix/最新TraceFix各108件を再利用できない場合、予備枠から最大216件を再実行する。
再利用はsource、binary、Wasm、prompt、interface、model、config、採点がすべて一致する場合に限る。

Cは元の航空業務rewardと確認指標を保ち、対象予約・変更・現在引数への承認を独立監査する。
別の構造化承認ケースでは全方式へ同じ確認情報を与える。simulated customerの条件と利用量も揃える。
Dは反復合意、人間承認、parallel、実行時workflow計画を含める。
Eはreview→merge、build→release、専門家引継ぎ、work-pool、共有解析、再編集後の承認更新を扱う。
Gは実patchを独立testする小さい公開repoと、固定データの数値・図表・本文・出典を外部検査するreportとする。
少数ケースから一般的なcoding能力を主張しない。

F1は48生成campaign、初回+最大4修正、最初の意味適合候補を採用する。
同じ部分仕様・語彙・制御範囲を与え、固定localside・effect adapterへ適合させる。
探索範囲を揃えられなければnative生成比較に限定する。F2は分離した研究artifactとし、
coreへ互換mode、第二の権限経路、通常実装の並行性を削る制限を残さない。
変更負担は同じ8課題へ検証追加、retry規則変更、参加者関係変更を与え、
手編集・生成物・汎用engine修正を分け、熟練度の影響も記録する。

## 測定と分析の事前固定

システム全体のnative比較と、手順・effect interface・prompt情報・予算を揃えた因果比較を分ける。
比較先へ新schedulerや権限制御を足す必要がある課題はnative未対応とし、追加adapter条件を別に記録する。
可視text/function履歴、JSON表現、圧縮・切詰めを保ち、opaque reasoning stateを要求間で持ち越さない。
1run600秒、参加者200tool calls、domain delay1.0、同時run1を維持し、run内部の参加者は並行実行する。

主指標は元の業務の完全完了率。不正な実効果は独立oracleで数え、拒否、合法操作の受理、
protocol完了、domain失敗、timeout、予算切れ、並行性を併記する。
時間とinclusive tokensは失敗を含む全試行を集計し、成功例平均と成功1件当たり総費用を分ける。
cached inputを含め、reasoning tokensをoutputへ二重加算せず、利用量欠測はunknownとする。

task・障害・反復を対応付け、clusterを保つpaired bootstrapの95%区間を用いる。
Aは16family別とleave-one-family-out、Bは採用family、航空業務はtaskをclusterとする。
小さいworkflow/caseは記述的に報告し、同じ集合の主要3対照にはHolm補正を使う。
主要対照の名前と必要精度はM0で本番前に確定する。
完了率の非劣性許容差5percentage pointsは候補であり、実務上の意味と必要精度を非採点pilotで
検討して本番前に凍結する。予備枠を勝てるまで増やさず、精度不足は不確実と報告する。
0違反、有意差なし、有界探索成功を、それぞれ普遍安全性、同等性、形式証明へ読み替えない。

quota、認証/モデル不一致、壊れたadapterはinfrastructure failureとして区別する。
通常の逸脱、予算切れ、timeoutは結果とする。再実行規則は先に固定し、失敗のraw記録を残す。
保証に加え、安全性と完了率を保ち、費用・並行性・変更負担の実質的改善を狙う。
競合と差がなければ、優位性の主張を機構の統合に限定する。

## 実装上の契約

Globalを唯一の連携仕様にし、localsideへEndpointの操作を直接書く。
順序・分岐・再考・並行処理・完了をhelper、wrapper、flag、所有者表、別の状態機械へ移さない。
資源の実体を保存するbufferと、資源の操作権・進行を区別する。

- Global: `seq`、`send`、`route`、resolver、`par`、`roll`。
- Localside: `send`、`recv`、`offer`、`par`を直接記述する。`hibana-runtime`が
  借用したfutureの保存・poll・dropを担当し、通信の進行・所有権はHibana本体が検査する。
- 共有操作に必要な実Endpointを所有値で渡す。そのEndpointで実引数を送り、
  実結果を受けて同じEndpointを返す。空のgrantと別経路の操作権を併存させない。
- 復旧可能な失敗は、実結果をresolverへ渡し、routeとrollで再考へつなぐ。
  元課題が終端失敗を指定する場合は、その結果を保つ。難しい課題を単発試行へ縮めない。
- モデルはコマンド、私有計算、解法を自由に選ぶ。各モデルに独立した実Wasm＋hibana-washを
  一つ割り当てる。各2MiBのcaller-supplied memoryを使う。
- すべてのFS操作は、対応するWASI要求をEndpointで受信した後にChoreoFSへ渡す。
  私有bufferでもraw要求による迂回を作らない。ChoreoFSの表示と操作権を混同しない。
- アプリ資源、ファイル内容、連携状態を`Rc`、`Arc`、`RefCell`、`Mutex`で共有しない。
  データは参加者が所有する。resolver入力は実結果であり、別のworkflow cursorにしない。
- 外部RPCの相関と記録はI/O担当の責務であり、taskの許可を与えない。
  既存の共有や旧経路は整理対象である。別名への置換で完了としない。

## 現在のsourceと検証範囲

現在選択する依存の固定値は[アプリのmanifest](Cargo.toml)と[lock](Cargo.lock)、
[runnerのmanifest](tools/runner/Cargo.toml)と[lock](tools/runner/Cargo.lock)、
[効果境界の検証crate](proofs/endpoint-handoff/Cargo.toml)に記録する。
この計画書へ「現在のpin」を重複して転記しない。各認定は、その実行のsource manifest・
dependency tree・binary hashへ対応付け、下記では検査した版を個別に示す。
通常のGit依存を使い、診断用のpath patchを認定へ流用しない。

11月3日の先行依存更新では、本体`e22c0b9`へ外部executor依存と旧試験経路の除去をまとめ、
実際に選択した本体からmemory guardを使うよう統一した。並行分離条件に追随していなかった
Lean証明テンプレートを修正し、実Endpoint6試験と既存135命題を通した。後続`82efe26`では
retained-rollの展開文脈と分離証拠を生成側へ補い、モデルの条件・命題は弱めずに
全生成Lean gateを通した。source適合検査は12,310試行・4,936 cutで不一致0件だった。

11月3日の依存更新は本体`d892df3`を選んだ段階であり、
全Kani・追加Miri等を含む[当時の認定](https://github.com/hibanaworks/hibana/actions/runs/41629197677)は取消済みである。
配置修正`741b4b0`を含め、当時の認定を成功とは扱わない。
EPF`d98822a`は14 CI job、WASI`60b51a9`は通常CIが成功し、それぞれmainへ取り込んだ。
本体を更新したEPF`67933c6`はmanifestとlockだけの変更で、全検証成功とは扱わない。
WASI`b4f8d7e`は同じ依存更新に加え、重複したSMT権限モデルをLeanへ集約した。
権限マスクの一般命題を10から14へ補い、実ChoreoFSの160照合とVMの520照合を維持した。
Z3入力・呼出し・CI依存を削除し、自然数のbitwise恒等式が使う標準公理も正確に監査する。
この版のCIとローカル全検証は未完である。
アプリのglobal・localsideと各libraryのproduction Rustは変更していない。
この依存更新を、組合せの全gate成功やChoreoFixの実Wasm・動的監査の認定として扱わない。

11月5日は、本体の通常43割当を完走した`b6a9607`と公開版`724f3e3`について、
製品コード537ファイル、公開試験301ファイル、Lean仕様185ファイル、利用例7ファイルと
実際のstable compilerの一致を確認した。元の各割当の実行記録とhashを保持し、
変更のない製品の通常検証へ再利用する。検査器の正規化・記録形式の変更は別の認定とし、
新driverの32分割中12分割の成功を全割当成功とは報告しない。拡張認定も未完である。
通常入力と完全に同じ拡張8割当を削除した。4,096実行列、41,984スコープ検査入力、
16不合法sourceを含む一致を確認しており、通常43割当の入力域は変えていない。
現在の追加割当は深い固定8件と深さ4〜8の抽出5件であり、全構文の網羅や一般証明ではない。
本体の再検査を初めから繰り返さず、公開した同じ本体へconsumerを揃えて検証する。
EPF`0266f00`はこの本体を実際に解決し、native113件とMiri102件が成功した。
doctest2件、29欠陥の感度検査、6 Leanモジュールの定理・公理検査、Clippyと
no_std/thumbv6mビルドは、同じ実装・仕様・native入力で成功した`93c65fe`の記録へ結ぶ。
グラフ・履歴部分集合・出現割当等の全数の意味照合はnativeへ集約し、Miriで同じ全列挙を
繰り返す6箇所を除いた。nativeの入力・操作列・assertionは残し、MiriはVM storage、
resolverの寿命、実Endpoint・取消・観測、source/workspaceの境界を検査する。
このツール別の範囲を全入力のMiri成功へ読み替えない。停止した旧Miri実行は未完として残す。
EPFのこの成功を、本体の拡張認定、ChoreoFixの統合認定や実モデル結果へ数えない。
その後、二つの実runtimeから得たhandoffの155記録では、自動監査が45秒で打ち切られた。
同じ記録の実観測順には既存sourceモデルで合法な解釈があることを確認し、モデルやglobalを
変更せず、EPFの探索を「全順序と全対応を作ってから検査」から「各記録の対応を選び、
そのsource prefixの前提を確認してから次へ進む」形へ置き換えた。公開EPF`21078b0`では、
同じ155記録・34送信受理を独立検査器まで通し、2回のrelease実行は0.47秒と0.71秒だった。
これは固定入力の診断結果であり、一般的なthroughput評価や任意入力の終了保証ではない。
native114件、doctest2件、6 Leanモジュールの定理・公理監査と29欠陥の感度検査が成功した。
変更した探索を呼ぶ4 targetのMiri34件を再実行し、その他のMiri69件は同じ実装・試験・依存の
`0266f00`の成功記録を保持する。最終検査を消す変異は、確定済み分岐と後のresolver判断の
矛盾を見逃すことを確認した。Leanには、同じ不成立prefixを後続操作では修復できない命題を
追加した。通常gateの最初の呼出しは、正しく失敗した変異の旧試験名照合で終了code 1に
なったため、その呼出し全体を成功とは報告しない。照合名だけを修正し、成功済みの試験と
証明を保持して、未実行だった感度試験を完了した。source/hashと各結果の対応は
`output/review-occurrence-qualification/epf-automatic-source/epf-source-prefix-qualification-20261105-231532`
へ記録する。consumerの依存を公開版へ揃え、無効だったtest用のEPF最適化profileを除去した。
この組合せでのhandoffの実受信・公開効果の対応は検証中であり、完走前に認定へ数えない。
WASI`82253a5`も同じ公開本体で全ローカルgateが成功した（140.00秒、
全子孫の最大1,186,414,592bytes）。実ChoreoFS例、Lean権限照合、Pico 2のリンク・容量、
実Wasmシェル、strict-provenance Miriと配布archiveの検査を含む。
アプリ・runner・handoffのmanifest、lock、Cargo metadataを照合し、本体は一つだけを
解決する。EPF・WASIのGit依存からも同じ本体が選ばれ、Tokio/mio/socket2は存在しない。
この組合せのChoreoFix統合検証は別途実行し、library単体の成功で代用しない。

本体の`8792e63`ではKani321件が成功し、全子孫の最大memoryは2,248,671,568bytesだった。
この版はlane mismatchの観測値の説明を訂正し、tap記録の標準変換を追加した。
EPFはフィールド解釈を複製せずに完全な記録を読める。
この全gateは生成した不合法sourceへ二つの異なる拒否理由を要求する検査側の誤りで停止した。
生成器を修正した本体`77c54e6`では、修正前に回帰試験が失敗し、修正後の88試験と
停止した分割が成功した。174形・616,608操作試行でruntime-only/model-onlyは0、
120件の実コンパイル拒否を確認した。最終sourceでの全gate完了とは扱わない。
本体の`0356494`は、確定した操作の16-bit source位置、実行actor、logical roleとlaneを
記録し、Evidenceのword 2へ観測連番を保持する。旧版で4試験の失敗を確認し、修正版では
本体823試験、公開経路36試験、strict-provenance Miri5試験、Lean9定理とZ3、Clippy、
no_std/thumbv6m buildが成功した。不要になった受信previewの観測計画・補助型とfieldを削除した。
このSHAでKani321件が成功し、全子孫の最大memoryは2,588,558,128bytesだった。
protocol-shapesの32分割中0〜3は成功したが、分割4のshape 781でmodel-onlyが8件発生し、
全gateは停止した。runtime-onlyは0件。prefix `[2,4,5,6]`等の後のreceive 0/3をRust側が
拒否する不一致だった。原因は本体の受信選択にある。別の合法なroute armのframeを
先に見た際、独立laneの要求を調べる前に`PhaseInvariant`を返していた。
通常3ロールの小さいglobalでも再現し、修正候補ではそのframeを元のFIFOへ保持したまま
独立受信を進める。受信の確定前にpayloadやroute/rollを進めない。未知のheaderは拒否する。
公開Endpointの回帰試験、同じ構造の通常／委譲付き生成ケース、source上の許可を示すLean定理を
追加した。修正版`c1d9212`をfeature/endpoint-delegationへ公開した。
この版は公開試験28件、受信のstrict-provenance Miri5件、生成器88試験、Clippy、
11workload×8metric×2反復の処理回数ゲートを通した。全headが別contractの場合も
Pendingで隠さず拒否する。Kani321件・protocol fuzz・32分割・全Miri等を含む全gateは、
このSHAを固定してローカルで実行中であり、まだ完了として扱わない。
同じSHAで[通常CI](https://github.com/hibanaworks/hibana/actions/runs/41270764123)、
[protocol fuzz](https://github.com/hibanaworks/hibana/actions/runs/41270765157)、
[並行再入Miri](https://github.com/hibanaworks/hibana/actions/runs/41270766502)も起動した。
CIの受付・実行中を成功と扱わない。失敗入力とログは保持する。
同じ本体`c1d9212`のローカルKani321件は成功し、全子孫最大memoryは
2,638,577,664bytesだった。別seed `20261101`のCI fuzz深さ5〜8と並行再入Miriは成功した。
深さ4は、不合法な一つのsourceに複数の所有権違反があるとき、生成検査が二つの異なる
先頭診断を要求する問題で停止した。実際のコンパイルはsource上で裏づけられる違反を拒否した。
生成検査の修正と停止入力の再検証は未完であり、全gate成功とは扱わない。
WASIの`378e7d7`は本体`c1d9212`とruntime`4d73bcf`を選択し、二つの利用例と
八つのEndpoint試験をhibana-runtimeへ書き換えた。Tokioはmanifestとlockから除いた。
閉じたメモリ内fixtureのdriverを使い、OSのreadinessやI/O性能の検証とは区別する。
CIのguard追加後の`e9bcf88`を公開し、[WASIのCI](https://github.com/hibanaworks/hibana-wasip1-runtime/actions/runs/41270905426)を起動した。
同CIは成功した。native、Clippy、Pico 2、Lean/Z3と、Miriのlib81試験・実Endpoint利用例8試験を
通過した。本体のローカル全gate成功後に同じsourceでWASIのローカル全gateを開始する。
consumerも本体・WASI・runtimeの公開pinを揃えてCIへ渡す。最新handoffと全移行済みfamilyの
再認定は未完であり、旧pinでの認定結果を流用しない。
作業版では、隣接する結果bindingの暗黙の並行化へ依存する箇所を、必要な複数の結果を
一つに束ねる宣言へ書き換えた。結果不要の枝は一呼出しも含めたasyncの子とし、
六familyの参加者起動とEndpointの並行send/返却から枝ごとの属性を除いた。
globalの操作順・依存は変更していない。公開版`c1f4885`のCIでは、native、Clippyと
実receiverのMiriを通過したが、実Wasmのビルドで停止した。guestのビルドscriptだけが
Rust 1.95.0を指定し、CIが用意したstableのWasm targetを使っていなかったためである。
hibana-washの`7e69adf`で開発・ビルドを最新stableへ統一した。MSRVの宣言は変えていない。
新しいguestとruntimeを選択した統合認定は未完である。
既存pinの機構試験を、新runtimeでの統合認定へ読み替えない。
全familyの機構検証はRust runnerの`--qualify-wasi`へ集約している。source・binary・
receiptを保存し、テスト0件や途中のsource変更では成功としない。
最新pinでの全family再認定結果を得るまで成功件数へ加えない。

runtimeの`61ae78b`は、配列localside宣言の入力式内のasync scopeも変換する。
修正前に成功・子のエラー・借用値の3試験がすべて失敗することを確認し、同じ試験を修正後に
通した。native89試験、compile入力13件、strict-provenance Miri60試験、Lean/Z3、
no_std/Pico linkを含む全gateが同じsourceで成功した。全子孫最大memoryは699,056,128bytes。
下記の75条件は従来のruntime `dc0a13a`での認定であり、修正版のconsumer再認定とは区別する。
修正版を通常のGit pinで選択したconsumerでは、アプリ40試験・runner27試験とClippy、
6Eの実Wasm34条件を再実行した。protocol完了・監査違反0件、928件の記録変異を拒否した。
全子孫最大memoryは1,705,820,160bytes。binaryとsource manifestを
`output/review-occurrence-qualification/runtime-array-input/consumer`へ対応付けて保持する。
モデル呼出し・採点算入は0件。残るfamilyの旧認定を、この修正版の認定へ読み替えない。



runtimeの`6c0782b`では、結果bindingの隣に宣言した子まで終了を待つ問題を修正した。
旧macroで停止する再現を確認し、結果bindingは明示したFutureだけを待つ形へ揃えた。
native98試験、compile入力15件（成功2／拒否13）、strict-provenance Miri70試験、
Lean/Z3、Clippy、no_std/Pico linkを含む全gateが同じsourceで成功した。
全子孫最大memoryは701,661,184bytes。`4d73bcf`はこの実装のままAGENTS.mdへ規則と例を追加した版。
`8fba570`も実行コードは同じで、Actions再開とmacOS/ARM LinuxのCIを追加した版である。
最初のCIは検査前に共有scriptの実行権限で停止した。`078ca32`でCIの起動手順を修正し、
[再実行](https://github.com/hibanaworks/hibana-runtime/actions/runs/41271100947)を起動した。
同CIではLinuxのroot Future配置によるstack overflowと、ANSI色によるcompile-fail診断の
誤集計が見つかった。`2c3a75c`でroot Futureをentryの最終領域にpinし、診断を無着色で
取得するようにした。試験のstack予算・入力数・コンパイル拒否条件は変えていない。
Macの同一source全gateは成功し、Linuxも同じstack試験を通過した。旧Z3での定数配列の
logic指定を修正した`3ded31b`は[Mac・Linux両CIの全gate](https://github.com/hibanaworks/hibana-runtime/actions/runs/41272640920)が成功した。
この時点では、この修正版をアプリ・runner・handoff検証crateで選択した。現在のpinはmanifest/lockを正とする。
これらのruntime検証を、現在のcore・WASI・consumerの組合せの認定へ読み替えない。
関数属性一つとRustのasync/awaitを使い、並行枝の一呼出しasyncも許容する。
新しい並行構文は増やさず、pin・task handle・joinの組立て・子の完了管理をruntimeへ置く。

2026-11-01のinterface更新では、runtimeの関数属性を`scope`へ揃え、枝や結果の宣言ごとの
属性を削除した。並行処理は`async { ... };`、値の合流はその子を並べたasync本体の
`.await?`で書く。成功値は宣言順に返し、固定配列は実際のNの配列を返す。
明示した親の返値、早期return、error、借用の解放を保ち、結果receiverや追加wakerは置かない。
`baf9f41`の[Mac・ARM Linux両CI](https://github.com/hibanaworks/hibana-runtime/actions/runs/41284637777)は成功した。

consumerで露呈した常駐I/Oの成功型の推論不足もruntime側で修正した。`6896654`では、
成功値を利用しないerror-only loopへunitの文脈を与え、値を返すloopとlabel付きbreak値を保つ。
両方向の回帰入力は修正前に失敗した。同一sourceの全gateが成功し、native134試験
（再帰起動するhelperの重複を除く）、compile入力15件、strict-provenance Miri105試験、
Clippy、doctests、実Endpointの別process例、Lean/Z3、no_std/Pico linkを確認した。
全子孫の最大memoryは698,941,440bytes。`6896654`の[Mac・ARM Linux両CI](https://github.com/hibanaworks/hibana-runtime/actions/runs/41285432031)も成功した。既存のLean/Z3はschedulerの性質の検査であり、
macro変換全体のrefinementを証明したとは扱わない。
4KiB入力二つは8,264bytes、三つの配列入力は12,376bytesで、既存の保存領域予算内だった。

アプリ・runner・handoff検証crateのruntime pinをこの公開版へ更新した。各localsideの
結果用の属性と不要な中間bindingを除き、Endpoint操作を直接書くasync枝から値を返す。
globalとEndpoint操作順は変更していない。通常Git依存のhandoff検証は12試験が成功し、
全子孫最大memoryは377,716,736bytesだった。これはnativeの実Endpoint試験で、
モデル呼出し0件。さらにアプリ通常試験41件、runner通常試験31件とrunnerのClippyが成功した。
アプリ通常試験の全子孫最大memoryは1,721,173,960bytes、runnerは681,492,480bytesだった。
実Wasmの全条件と実モデルの再認定は未完であり、この通常試験に算入しない。
記録とsource hashは`output/review-occurrence-qualification/runtime-scope-values/`へ保持する。

WASIの`18da231`もruntime `6896654`へ揃え、[同版の全CIゲート](https://github.com/hibanaworks/hibana-wasip1-runtime/actions/runs/41287865228)が成功した。
実Wasmの標準ライブラリ製シェル、正常・不正なChoreoFS操作、Endpointの貸出・返却、
native、Clippy、Miri、Lean/Z3、Pico 2の確認を含む。WASIのproduction sourceは
`e9bcf88`と同じで、変更は開発用runtimeのmanifest/lockだけである。
アプリとhandoff検証crateもこのWASI pinへ更新する。このCI成功を、ChoreoFix全familyの
実Wasm再認定や本体の最終source全gate成功へ読み替えない。
同じ`18da231`でローカル全gateも成功した。native lib83件・利用例8件、doctest6件、
strict-provenance Miriのlib81件・利用例8件、Lean/Z3、実Wasmシェル、Pico 2、packageを確認した。
全子孫最大memoryは1,189,691,392bytes。開始前後の全tracked sourceのhashは一致した。

本体のoffer修正は`f542167`へ公開し、検証用差分の書式だけを直した`8ecc7b1`を固定した。
dropしたpreviewの保持データと、実行中Futureの借用を分け、別のtyped操作を通常の進行検査へ渡す。
入れ子のrollを含む生成ケースは、600完走記録・64,008の通常操作試行とparked-preview試行で
sourceとの差が0件だった。旧productionへ戻すと同じ生成入力のshape 24がsendの
`PhaseInvariant`で失敗した。初期の感度wrapperは後段のmodel mismatchだけを要求していたため、
実際に先行するこの実行時失敗を検査する形へ訂正した。最終版でのwrapper再実行も全gateへ含める。
`8ecc7b1`のCIでは、保持中のpreview frameと異なる枝のtyped receiveが同じRxを
再pollしてpanicする問題が生成試験で見つかった。2ロールの短い例でも再現し、`4fb3868`で
保持中のFIFO headを新たなpoll対象から除外した。独立laneの受信とPendingからの再開は保つ。
実Endpoint17試験、strict-provenance Miri2試験、Lean9定理とZ3の5組を確認した。
生成6形状の通常3,510試行・parked-preview384試行でsourceとの差は0件であり、
この修正だけを戻すと同じ生成入力のshape 18が元のRx再pollのpanicを検出した。
phase表の証明は保持frameのRx借用を扱っていなかったため、lane除外のモデルを補った。
現在は`4fb3868`に対して、protocol fuzz・32分割の生成検査・Kani・Miri・形式検証等の
ローカル全gateを実行している。旧版で中断したKaniを全件成功とは扱わず、consumerの
core pinも全gate完了前には更新していない。

runtimeの配列child属性の欠落は`2a6a24c`で修正済み。子のcfgを保持し、無効な子の
入力評価・patternのdropは保つ。同一sourceのローカル全gateとMac・ARM Linux両CIが
成功した。native144試験、compile入力16件、strict-provenance Miri115試験と既存の
Lean/Z3、no_std/Pico link等を含み、全子孫最大memoryは1,369,489,408bytesだった。
mainをこの版へ統一し、検証用branchは削除した。アプリ・runner・handoff検証crateと
WASIの開発用runtimeもこのpinへ揃える。WASIの新pinは`6314831`で、production sourceは
`18da231`と同じである。この新しいconsumerの組合せの全認定はまだ完了していない。

ChoreoFixの`eff88e2`のCIは、native・Clippy・receiverのMiriを通過し、実Wasmの
codebase群5試験を871.36秒で完了した後、writing群の途中でjob全体の45分制限に達した。
全75条件の成功とは扱わない。各試験のdeadlineとmemory guardを変えず、job全体の
時間枠を90分にして再実行する。比較実験やモデル利用の結果には算入しない。

runtimeの中核はno_std・割当なし・caller-owned storageである。参加者は借用した値や
所有値で直接宣言し、runtimeがFutureの配置・poll・キャンセル・子の終了待ちを担当する。
12課題とrunnerから起動joinツリー、task登録・終了回収、guest/I/Oの二重の終了値管理を
除去した。各localsideにはEndpointの貸出・要求・結果・返却をそのまま記述する。
I/O担当はファイル内容、descriptor、部分JSON行とRPC IDを所有し、連携手順の権限を持たない。
アプリ・runner・試験・検証crateの全依存からTokioを除去した。
全メッセージ型の使用箇所で推論できるlifetimeを省略し、型定義の借用関係を維持した。

選択中のruntimeは、Futureを最終のinline slotに置いてから参照の平衡木を一度だけ構築する。
既存Futureの繰り返しコピーと入力を二重に保持するawait wrapperを除去し、利用側へ
pin操作やstack増量を要求しない。同じ入力とstack予算で旧実装のoverflowを検出する
感度検査がある。runtimeの`dc0a13a`はnative85試験、compile入力13件、strict-provenance
Miri57試験、Lean16定理/Z3、no_std/Pico linkを含む全gateが同じsourceで成功した。
外部通知はOS descriptorを所有し、scoped Futureへのpointerを保持しない。
通知の合流、full socket、最後のproducer後のEOF、closed reader、Rust割当0件をnativeで検査した。
hostのOS呼出しはnativeで検査する。Lean/Z3はschedulerとstorageの性質を扱い、
macro変換全体や任意のRustの詳細化証明ではない。

アプリ・runner・12課題の行は前回の全体認定であり、今回のhandoff変更を含む全targetの
再認定は別途行う。handoffの2行は今回の変更後の個別検証である。

| 構成 | sourceを対応付けて確認した範囲 |
| --- | --- |
| アプリ | 全targetのRust40試験、Clippy、全依存treeのTokio/mio/socket2不在 |
| runner | Rust27試験、Clippy、全依存treeのTokio/mio/socket2不在 |
| 移行済み12課題 | 実Wasm75条件。74条件がprotocol完了・監査違反0件。CI/CDの早期終了1条件は共有効果0件 |
| handoff検証crate | native12試験。handoffの6試験に含まれる11ケース、RPCの4試験、入れ子の選択2試験 |
| handoffのMiri | 同じ11ケースを含む6試験。候補確定後の再修正拒否と、Pendingからの再開を含む |

アプリとrunnerは同じruntime pinを使う。実Wasmの各binary・source manifest・build receiptを
保存し、buildと試験の前後でsource hashが一致することを確認した。
全子孫の最大memoryは1,658,519,552bytesで、2,750,000,000bytesの停止上限内だった。
記録は`output/review-occurrence-qualification/projected-wire-contract/final-source`に保持する。
監査対象74記録への変異1,409件をすべて拒否した。これは記録監査の検出力であり、
実行時攻撃1,409件の阻止ではない。モデル呼出し・採点済み実験への算入は0件である。
各条件の業務結果は元scoreへ別に記録する。

6Eの監査は、実行binaryの`--projection`から保存した不変の通信情報と、実際にattachした
programからの観測を照合する。全frameの16-bit wire label、lane、logical peer、委譲の
resource/event envelopeを検査し、レビューのintent・要求・実結果の返信もlogical labelで
区別する。34条件で928記録変異を拒否した。全観測を同じ不正label/laneへ変える変異と、
同じpeer間の別の合法な通信へ要求を差し替える変異を含む。静的な通信対応の検査であり、
動的なroute/rollの実行可能性を再生する検査とは区別する。アプリの進行管理は増やしていない。

16Eも同じcompiled projection監査を使う。実際にattachする13個のprogramから通信情報を
書き出し、実行前に保存したbinary・projectionのhashと照合する。actorの対応は投影に
含まれる情報から求める。元課題の深さ0/1/2の実Wasm3条件が完走し、各条件の16記録変異、
計48件を拒否した。同一frameの全観測のwire labelまたはlaneを一斉に変更する変異を含む。
早期終了の別条件では共有効果は0件だった。native40試験・runner29試験と両Clippyも通過した。
記録は`runtime-array-input/cicd-wire`に保持する。これらはモデル呼出し0件の機構試験であり、
採点比較や動的な分岐・反復の再生には数えない。

動的なprotocol監査はhibana-epfへ集約する。アプリの連携仕様をEPF用に書き直さず、
実行binaryに含まれるsealed RoleProgramの不変情報と、同じ実行の出現記録を照合する。
受理・送信commit・受信commit、route選択とrollのvisit、委譲前後のactorを区別し、
欠落した記録や一意に対応づかない出現から適合を認定しない。監査の履歴は操作権を与えない。
EPFの`f27f8de46292d10c6b12fa194ba5fcaf33a781fe`（`feature/dynamic-audit`）は、
本体`0356494`の標準16-byte記録を使い、入力のEOFを待たずに1件ずつ観測を出力する。
VMからも観測連番を読み、連番による絞り込みを実CLIで検査した。actor、logical role、lane、
16-bit source位置の解釈をEvidence schema 3へ固定し、古いschemaは有効なimageを置き換えずに拒否する。
旧本体では連番が失われる回帰試験が失敗し、修正版ではnative32試験、strict-provenance Miri27試験、
Clippy、docs、no_std host/thumbv6m buildが同じsourceで成功した。最大memoryは1,373,831,168bytes。
source/rawと修正前の結果は`runtime-array-input/epf/commit-fields`に保持する。
設定の失敗がpolicy枠を消費する3経路の修正前後の検査は`epf/atomic-configuration`、
EOF待ちを戻すライブ観測の感度検査は`epf/final`へ対応付ける。観測CLIへpolicyを指定した場合も
黙って既定の観測へ切り替えず拒否する。動的適合の認定やChoreoFixの効果監査との接続は含まない。
上記は旧版での境界である。現在のEPFには、compiled sourceから有限展開、各visitのroute、
producer間の順序と操作の対応を構成し、動的なsource実行として照合する入口を追加した。
EPF `2a865a0`は本体`40298f0`を選択し、同一sourceでローカル全gateとCIが成功した。
通常101試験、strict-provenance Miri 96試験、compile-fail doctest 1件、Lean 50項目、
17種類の故障対照、Clippyとno_std host/thumbビルドを含み、最大memoryは693,190,656bytesだった。
探索の容量不足・不成立は認定を返さず、globalの違法判定としない。
6Eでは全tap記録と独立したproducer checkpointを保存し、runnerからEPFへ渡す接続を実装した。
受理した各wire callを既存のframe journalへ一対一で対応付け、レビューの効果監査へつなぐ。
依存版はmanifest/lockを正とする。本体のTokio撤去は実装済み。
WASI `6930f7a`のCIも成功した。この組合せでの本体全gateと実Wasmによる動的監査の
統合認定は、成功記録がそろうまで未完とする。
この接続を実Wasmで動かした際、終了前にqueueへ置いたcheckpointが出力されず、監査が
不完全な記録として拒否した。外部I/O所有者が書込み済みprefixと未送信bytesを保持し、
終了通知も同じqueueから出力するよう修正した。通常I/O 6試験とClippyは成功したが、
修正後の実Wasm動的監査は再検証が必要である。他familyへの展開と最終sourceの全gateも未完。
runtime `09a803a`の同一sourceのローカル全gateは成功した。consumerはこの通常Git版を選択する。
本体`40298f0`のCIはKani全321件・生成形状32分割が成功した。final-form内の検査用テストは
Lean専用へ移行した入口にもZ3起動を要求して失敗したため、`dc043ee`で修正した。
修正後の当該7試験は成功したが、最終sourceの全gate成功へは読み替えない。
I/O修正後の`bfe8709`ではcheckpointが届いたが、実Wasmの2経路で記録数が
3,043/5,063件、5,512/8,568件に不足し、EPFが拒否した。小さい診断リングを1ms間隔で
回収しても全履歴は保存できないため、本体`194eac8`に借用した記録先へ各eventを同期的に
渡す入口を追加した。待機のない実Endpoint257操作を含む7試験のMiri、記録先の寿命を
守るcompile-fail、Clippy、thumbビルドは成功した。EPF `05d4f00`はこの本体を選択し、
診断窓を超える実Endpointの履歴を自動展開・動的照合する試験を含む通常102試験とClippyが成功した。
6Eのtimerによる回収とそのraceは削除し、同じEndpoint操作を直接並行に駆動する。
この組合せのcompile検査は成功したが、EPFと本体の最終全gate、consumer実Wasmの再認定は未完。
接続版`1768896`の実Wasmでは6Eが完走し、独立したcheckpointの5,063件すべてを保存した。
EPFはその記録の603番目で、resolverの決定記録を伴わない分岐選択を検出した。
本体`d11c788`は、outgoing `offer()`の後にbranch送信する経路でも、実際のresolver呼出し時に
決定を一度記録する。commit時の遅延記録と専用の補助field・型を削除した。
実Endpointの観測28試験、tapの6試験、roll再訪を含むMiri試験が成功し、生成器の両resolver経路で
記録欠落を戻すと実行試験が失敗した。固定カタログは15種類へ増やし、既存の範囲も維持する。
EPF `7cb148a`はこの通常Git版でnative104試験・Clippyと追加Miri試験が成功した。
実Endpointの左右分岐・roll再訪からsource展開と動的監査を自動構築し、resolver記録だけを
削除した完全なprefixは拒否した。manifest/lockをこの組合せへ揃え、consumerの実Wasmと
最終sourceの全gateを再実行する。これらの個別成功を全体認定へ流用しない。
接続版`2fe2526`の実Wasmは完走し、checkpointの5,064件すべてを保存した。
その動的監査は2,082番目で、分岐を受信で知った参加者の返信が、その参加者自身のresolver選択として
記録されていたため不成立だった。本体`9b8a3b3`では、controllerの新しい選択に記録を限定し、
保持済みの選択を使う返信の重複記録を除去した。操作の受理や進行の規則は変えていない。
修正前に実Endpointの回帰試験が失敗し、修正後はtapの11試験・Clippy・同試験のMiriが成功した。
生成検査にも、選択記録のactor/session/scopeと実際のresolver決定との対応と故障対照を追加した。
EPFの挿入境界2補題は、命題を変えずに証明の各方向を分け、禁止axiomへの依存を除いた。
EPF `e5e82bf`は同一sourceでnative104試験、strict-provenance Miri99試験、Lean53命題、
20種類の欠陥対照、Clippy・doc・no_std・thumbを含むローカル全gateが成功した。
その後、接続版`670a9d0`の6E実Wasmは完走し、独立checkpointの4,972件をすべて保存したが、
自動監査の候補生成と再走査が大きく、検証を停止した。この停止を成功には数えない。
EPF `343d675`は検証済みscope索引を保持し、route両腕の反復数と、後のofferより先に
実行したtyped操作を候補へ反映する。完成した候補は従来の実行検査へ通し、検査条件は維持する。
最終の小さい候補生成回帰を加える前の同実装で、保存済み実記録4,972件・送信受理902件を
通常のEPF CLIが77秒で照合した。これは全consumer認定や採点実験ではない。
runnerは不変source索引と一時検査領域を別々に所有する。EPF `edddb15`の同一sourceでは、
通常109試験、strict-provenance Miri104試験、doc2件、Lean53命題、25種類の欠陥対照、
Clippy・no_std・thumbを含むローカル全gateが成功し、全52追跡ファイルのhashも一致した。
最大memoryは690,667,520bytesだった。`daa1457`はCIと検査stageの分割だけを変更し、
Rust・Lean・依存は同じである。同版のCIも通常・単体Miri・統合Miriの3stageすべてが成功した。
EPF `85b90b3`は、commitされなかったresolver判断と、その後の反対側の送信試行を
一つの確定済み分岐へ押し込めていた検査側の不足を修正した。実Endpointによる両順序の
取消試行を受理し、commit後の反対側への変更、完了済み操作の再利用、判断とpollの不一致は
拒否する。各試行を、確定済みprefixを保つ検査済みsource選択へ対応付ける。
同一sourceの通常113試験、strict-provenance Miri108試験、doc2件、Lean57命題、
29種類の欠陥対照、Clippy・no_std・thumbを含むローカル全gateが成功し、全sourceのhashも
一致した。全子孫の最大memoryは698,318,848bytesだった。この版を公開してrunnerのGit pinへ
選択した。consumerのビルド・実Wasm・動的監査の再認定はまだ成功として扱わない。
接続版`db82293`は通常Git pinで保存済み4,972件・送信受理902件を68.15秒で再監査し、
動的監査と既存のframe・効果監査が成功、違反0だった。native・ClippyとhandoffのMiri6試験も
成功した。この保存記録の再監査を、新しい実Wasm全条件の完走や採点実験には数えない。
前の2記録の不成立も実Wasm動的監査の成功とは数えない。
本体`d3f2e22`は`9b8a3b3`から観測試験2ファイルだけを修正し、本番Rust・Lean・依存は同じ。
commit前にも実際のresolver判断を記録する契約と、通信・分岐のcommitがないことを分けて検査する。
関連15試験のMiriと通常試験は成功したが、最終sourceのprotocol fuzz・Kani等を含む全gateは未完。

取消された`Bus::emit`の内部receiptへ完了通知を返すと、残りのcapture・checkpoint・終了通知の
出力が止まる不具合を、修正前の実I/O試験で再現した。出力所有者は取消済みの内部待ち手への
通知だけを省き、受理済みbytesと後続queueを最後まで送る。外部RPCの不明・取消済み・重複応答は
引き続き拒否する。キュー内と部分書込み後の取消、後続のlive receipt、出力故障、RPC応答を含む
I/Oの通常7試験とstrict Clippyが成功した。モデル呼出しは0で、global・localsideの操作順は変えない。
EPF `85b90b3`のローカル全gateは上記のとおり完了した。修正後consumerの実Wasm全条件は、
成功記録がそろうまで未完とする。
WASI `6930f7a`のCI成功をChoreoFix全体の認定へ広げない。
過去のChoreoFix接続準備版のCIでは、GitHubの支払・利用上限によりjob開始前に拒否された。
sourceの検査失敗としては数えず、注記を保存してローカル検証を行う。

検証用JSONLの保存では、細かなJSON断片ごとのファイル書込みが負荷になっていた。
既存のファイル所有者で標準I/Oバッファを使い、記録への応答と監査の前に明示的にflushする。
同じ保存済み1M記録を使ったローカル1組の比較で、12記録変異の保存・監査は19.88秒から
2.92秒となり、12ファイルと拒否結果はすべてbyte単位で一致した。実タスクやcoreの
通信性能の測定には数えない。33通常試験とClippy、成功応答前の可視性・書込み失敗の
伝播の既存試験は通過し、最終sourceでの実Wasm全条件は再実行する。

runnerは、独立要求、同じ参加者の並列要求、turn/start応答前の要求、guest終了後の
接続切断・timeout、部分JSON行の受信キャンセルを実I/Oで検査した。
終了・失敗後も、受理済みの実要求への応答を確認してから元scoreを取得する。
MiriはmacOS kqueueに対応しないため、handoffでは検証用のpolling clockを使い、
同じEndpoint・受信・実効果の試験本体を実行する。OSのwake配送はMiriの検査対象へ含めない。
Miriを含む検証crateの全子孫最大memoryは1,378,172,928bytesだった。
記録は`output/review-occurrence-qualification/projected-wire-contract/handoff`に保持する。

1Hの通常遅延でcommitの重なりが2件になった記録は保持する。parは同時実行を許すが、
実装の完了時刻が異なる通常実行で必ず3件が重なるとはしない。独立したA/C/Eの実要求を
Endpointで受理した後、試験用I/O境界で3件がそろうまで保留する条件を別に追加した。
返答なしで3件が受理され、解放後の実commitも3件重なった。globalとlocalsideは変更していない。
この開始条件を制御する機構試験を、通常の性能測定や実モデルの並行性へ混ぜない。

本体の最終sourceでのKani・protocol fuzzを含む全gate、他familyの投影に基づくwire監査、
具体的な全効果入口への定理適用、残る36課題の委譲移行は未完である。
比較接続の開発は保留し、利用可能な条件でChoreoFix自身の実験を進める。
元の比較実装は変更しない。

| 対象 | 現状 | 残る確認・変更 |
| --- | --- | --- |
| 1E/1M/1H | 実資源Endpointの要求・結果・返却。1Hは7人が実結果で再考。実Endpoint・実Wasm試験と実モデルの予備記録あり | source固定後の全条件の認定と比較 |
| 2E/2M/2H | 両sectionの実Endpoint要求・結果・返却。実Wasm15条件と3/5/7人のLuna予備試験が通過 | source固定後の全条件の認定と比較 |
| 6E | 実Repo Endpointによるglobal。各拒否後に修正・再レビューを行い、承認後の並行な後続へ進む | 通常Git pinで実Wasm34条件、投影に基づくwire監査と928記録変異の拒否。旧sourceの3人の実モデル予備試験あり。現版の実モデル・全条件の認定は未完 |
| 8E | DB・test環境の実Endpoint委譲、単独所有の結果ファイル、実Wasm3条件、Luna成功・終端失敗の予備試験 | source固定後の認定と比較 |
| 9E/9M/9H | 実fork Endpointを両方受け取り、実要求・結果・返却。3/5/7人の実Wasm・実モデル予備試験あり | source固定後の認定と比較 |
| 16E | build/deploy環境の実Endpoint委譲。実Wasm3条件、早期終了拒否、3人のLuna予備試験あり | source固定後の認定と比較 |
| 3E | nativeの研究report protocolと意味試験あり | 実Endpoint委譲・実Wasmへの移行 |
| その他の課題 | 元の48入力・825 checklist項目を照合するmanifestとmatrixあり | 各課題のglobal/localside、実Wasm、比較入力の認定 |

同じ本体pinを使っていても、別sourceの予備記録を最終source全体の認定へ読み替えない。
固定script、実モデル、元simulatorの完了、protocol完了を別々に報告する。

移行済み6familyの旧native coordinator、専用のunit貸出broker、要求待ち行列と
完了集計を除去した。置き換えられた経路だけを試すテストと古いPython試験も除去した。
委譲版のglobalとlocalsideは前版からbyte単位で変更していない。実験runnerでは記録fileを
一つのI/O threadが所有し、書き込み・flushの結果を返す。元simulatorの参照も一つのI/O
threadが所有し、非同期の元toolを並行に開始する。共有FileのMutex、simulatorのArc、
callbackのMutexは除去した。これらのI/O窓口は業務の進行や許可を判定しない。

空targetから全実験binaryをbuildし、4分54秒、最大963,675,552bytesで成功した。
compile時間の改善は確認していない。36 native試験、アプリとrunnerのClippy、runnerの
14試験が通過した。元の48課題×3障害条件の初期化、独立操作と競合の感度、tool例外の返却も
実simulatorで確認した。移行済みの課題数が48になったという意味ではない。

同じ通常Git pinとbinaryで、1E/1Mの9条件、1Hの5条件、レビュー34条件が実Wasmで成功した。
1Hは元の障害0/1/2、追加の2回失敗、通常遅延を含む。通常遅延では独立commitが3件重なり、
違反0だった。coding各記録の10変異、計140件と、レビューの826記録変異を拒否した。
これらはモデル呼出し0の機構試験で、比較の採点数は増やさない。

ChoreoFix `252b821cffcec3a5f172aa64e018db8e709cc8da`のbinaryと通常Git依存で、
1Hの7人と6Eの3人のLuna xhighを、ChatGPT subscriptionから実Wasmで実行した。
1Hは79.57秒、inclusive 1,820,465tokens。各人が実際のテストに2回失敗した後で成功し、
資源要求・返却各42件、独立commitの同時実行2件、元score完了・監査違反0だった。
6Eは30.29秒、inclusive 284,702tokens。4回のレビュー、資源要求・返却各6件で完走し、
実装/レビューの重なり3組、merge/レビューの重なり1組を観測した。
raw、各参加者のusage、binary/source receiptは
`output/canonical-model-qualification/subscription-1H-1793407820497950000`と
`subscription-6E-1793408026651117000`に保持する。
これらは2件の非採点pilotで、全課題・全条件の認定や比較上の優位性ではない。

レビュー監査は要求・close・結果の出現を一対一で照合する。ただし同じframeの全観測で
当時の6E監査には、wire labelまたはlaneを一斉に変えても受理する不足があり、上記記録で
再現した。現在は実行binaryの投影と照合し、34条件で同じ種類の変異を拒否している。
動的なsource実行全体を再構築したという意味ではない。
未移行36課題と、T3/T4の最小合成を具体的な全効果入口へ適用する認定も残る。

### 以前の版の予備試験

以下は個別の移行時点の記録であり、現在の全source・全依存版の認定とは区別する。

直近の16E変更では全target build、native56件、Clippy、fmtとsource前後hash一致が成功した。
全子孫の最大memoryは1,418,739,712bytes。実Wasmの元障害条件0/1/2はすべて元score成功・
違反0件、実資源要求・返却各3件だった。各14変異を拒否し、作業前のguest終了も効果0件で拒否した。
シェルの私有ファイル、パイプ、ループ、構文エラー後の修正、51,200Bの出力と実memory growthも検査した。
これらの試験はRust runnerへ移し、対応する自作Python試験は削除した。

3人のLuna xhighによる16E予備試験も13.14秒で完走し、inclusive 131,503tokens、
全員のChatGPT認証・モデル・effort一致、元score成功・違反0件を確認した。
組み込みと別processの同じauditorが一致し、実モデル記録の14変異も拒否した。
これは監査アルゴリズムが二つ独立にあるという意味ではない。

2E/2M/2Hも空の資源通知を除去し、実Endpointの並行受け取り・実要求・実結果・並行返却へ移した。
全target build、native56件、Clippyが成功し、build前後のsource hashは一致した。
元の障害3条件、終端レビュー失敗、通常遅延を各人数で実Wasm実行し、15条件が通過した。
各14変異、合計210件を拒否し、終端失敗を元scoreの失敗として保った。通常遅延では
3/5/7人でそれぞれ1/2/3件の独立したwriteが重なった。旧writing監査CLIとPython監査は削除した。
raw記録、実行binary、Wasm、sourceとhashを保持し、再生成可能な変異入力のコピーはhashと結果を残して削除した。
3/5/7人のLuna xhigh予備試験も完走した。全員のChatGPT認証・モデル・effort・usageを確認し、
元score成功、違反0件、実資源要求・返却は各6/10/14件だった。実モデルで観測した同時writeは
それぞれ1/1/3件である。inclusive tokensは328,617 / 526,721 / 743,788、所要時間は
21.58 / 20.18 / 21.14秒。比較性能の結果へ読み替えない。model試験全子孫の最大memoryは2,178,547,712bytes。
18記録すべてについて別processで同じauditorを実行し、組み込み監査と一致した。
3つのmodel記録も各14変異を拒否した。両監査は実行processが別であり、アルゴリズムは同じである。

6Eはglobalとlocalsideを全面的に書き直した。ファイル操作の反復はWASIのprivate I/O
choreographyに置き、業務のglobalには実要求、唯一のRepo Endpointの貸出、実操作、
実結果、返却を記述する。開発者の仲介roleとレビュー用補助viewは削除した。
最初のレビューと次にレビューする対象は実際の要求から選ぶ。再考中に他方のレビューへ
切り替えられ、独立した実装・レビューとmerge・レビューは実際に並行駆動する。
mergeと正常終了は承認された枝へ置く。選択と実レビュー結果は、分岐を知る必要がある
参加者へglobalの通信で伝える。進行はEndpointが管理し、アプリにphaseや完了flagを置かない。

両featureが拒否された後にも、他方の修正と再レビューを並行実行するようglobalとlocalsideを修正した。
現行globalのbuildは294秒、全子孫の最大memoryは1,092,634,112bytesで、
build前後のアプリ・core source hashは一致した。
元障害条件0/1/2、最初の対象A/B、一方を続ける・交互・交互の後に同じ対象を続ける3順序を
実Wasmで実行した。短縮遅延18条件と通常遅延8条件の計26条件すべてで元score完了・資源違反0件。
各16変異、合計416件を監査が拒否した。監査は実frameとWASI要求、業務引数、元simulator結果とscore、
貸出・返却、最新実装への承認後のmerge、guest終了を照合する。
通常遅延8条件すべてで実装・レビュー、merge・レビューの実時間が重なった。
両拒否後の交互順序では、修正・再レビューを含む3組以上の実装/レビューの重なりを確認した。
試験は173.38秒、最大memory479,723,520bytes。実行時の拒否試験と記録変異の検出力は別の証拠である。
アプリとrunner全targetのClippy、runner通常12試験とfmtが通過した。
26記録は別processで同じ監査を実行し、組み込み監査と一致した。二つの独立アルゴリズムとは数えない。

その後、同じ引数の再レビュー要求を削除しても旧監査が過去の要求を再利用して受理することを
実記録の変異で再現した。監査を要求・引数ファイルclose・結果の出現ごとの一対一対応に修正し、
直前版の26記録が引き続き通り、22種または25種、合計626記録変異をすべて拒否した。
これらは記録監査の検出力であり、626件の実行時攻撃試験ではない。

元の採点matrixは変えず、両feature拒否後の3回目・4回目の拒否を追加する8機構条件を作った。
4回目の拒否で、内側の反復を終えたglobalが外側へ再入し、レビュー待ちの開発者に再実装を
要求する不整合を実Wasmで再現した。globalの反復境界を修正し、localsideへの例外分岐や
操作順の回避は加えない。初回承認を明示的なrouteで受け、拒否の場合だけ修正・再レビューの
反復へ入り、承認を受けてから後続へ進むglobalへ修正した。追加8条件は実Wasmで完走し、
実効果と監査が通過した。モデル呼出しは0で、採点対象の比較実験には数えない。

source生成・lane割当の上限は、論理方向の索引、因果計算の一括化、作業領域の再利用により
解消した。初回承認の分岐を含む現在の1586event globalも、sourceと投影の意味照合を
維持したままforbidで通常buildが通過した。
正の順序義務は、activeなsource順序のcover（間に別のactive操作がない前後関係）を
すべて含む候補へ絞った。余分な直接順序の検査は全件維持し、従来の全操作対の検査との
一致を含む投影135試験が通過した。小さいsourceは全choice/holder条件からcoverを独立に
求め、候補に欠落がないことも照合する。有限な後方・推移的source順序について、これらの
検査が全祖先のcoverageとsource-closedな進行のreadinessを保つLean補題が通過した。
1586eventの通常buildは1分7秒、最大840,564,736bytesで通過した。これは増分buildで、
clean build時間ではない。修正後の空targetからのbuildは4分55秒、最大1,120,649,216bytesで
通過し、実Wasm34条件に使ったbinaryとbyte単位で一致した。forbidは維持している。
const評価の上限は解消したが、cold build時間の改善は残る。rustcの標本ではconst評価が
実行されており、Cargo timingsとprocess sampleを保持した。
その先の実Wasmで見つかった送信のOutOfOrderとofferの停止は、
小さい実Endpoint試験へ切り出し、本体で修正した。resolverの確認が受理済みの送信候補を
同labelの別の出現へ戻さないこと、別armへ移る場合も送信方向・logical role・payloadを
照合することを守る。offerは、全祖先の確定した選択から送れる候補を、追加frameを待つ
候補より先に提示する。実受信frameの選択と、後段のadmission/commit検査は維持する。
localsideの操作順・通信・並行性に回避処理は加えていない。
修正coreを使う追加8条件は、3回目・4回目の拒否を含めて成功し、監査違反0、各記録の
25変異、計200変異をすべて拒否した。同じbinaryで通常26条件も完走し、違反0、626記録変異を
すべて拒否した。通常遅延8条件では、実装/レビューとmerge/レビューの実効果の重なりも確認した。
新しい送信/offerの3回帰試験はstrict-provenance Miriを通過し、候補順位の本番関数はKaniで
検査した。送信候補とsourceの対応、無frameで送れる候補の優先規則に関するLeanとZ3も通過した。
送信の修正だけを外す変異は実Endpointの2試験、offerの修正だけを外す変異も2試験で失敗し、
元のsourceへ戻すと3試験すべてが通過した。検査前後のsource hashも照合した。
その後、coreを`e139912e`、WASI runtimeを`6b62998b`へ固定し、ローカルpatchなしで
同じ1586eventを再buildした。4分53秒、最大1,104,922,112bytesで通過し、依存するcoreは
一つだけであることを確認した。このbinaryで通常26条件と追加8条件を再実行し、全34条件が
成功、監査違反0、826記録変異をすべて拒否した。build前後のsource hashと実行binaryのhashを
記録した。228.39秒、最大75,726,848bytesで、モデル呼出しは0、採点実験には数えない。
WASIのローカルgateも通常Git pinで通過した。実Wasmのシェル例、LeanとZ3、Pico 2の
resource link、Miri、doc、packageを含む。Wasm専用のguest sourceをhost向けのCargo exampleと
して自動検出しないようにし、通常のcargo test、Clippy、packageも確認した。
現版のモデル認定は残る。

修正coreはlib817試験、roll継続再利用13試験、Clippy、no_std/ARM gateを通過した。
cover変更後のLean全127moduleと全1233定理の公理監査、Z3の14query、依存関係の変異を
拒否するMiri1試験、word/filter/intersectionのKani3件も通過した。Kani3件はword演算の検査で、
cover候補の再帰導出全体を証明したものではない。分岐履歴のMiri2件と方向索引のMiri1件、
全u8域の方向索引・parallel候補のKani2件、条件付き依存の上下界に関するLeanとZ3も通過した。
これらをcore全ゲートやT3/T4の合成の完了としては扱わない。

前版の3人のLuna xhighは通常遅延・障害条件2で39.13秒で完走した。両featureが一度ずつ拒否された後に
成功し、元score完了、違反0件、実Endpoint要求・返却は各6件だった。全員のChatGPT認証、モデル、
effort、usageを確認し、inclusive tokensは360,770だった。実モデルの記録も16変異を拒否した。
16個のscript記録と1個のモデル記録は別processでも同じauditorを実行し、組み込み監査と一致した。
監査アルゴリズム自体は同じである。使用binary、Wasm、runtime source、モデル記録を保持した。
これらはローカルcore修正を選ぶ予備試験である。前版のモデル記録を変更後sourceのモデル認定へ
流用しない。当時は通常Git pinと変更後sourceのモデル接続確認が未完だった。
現在の1H・6Eの確認範囲は上段の記録に従い、これらの旧試験を採点済みとは数えない。

効果入口の契約と12課題・6familyの受信処理を[一覧](benchmark/effect-boundaries.md)に整理した。
6Eの実受信処理を別runtime間で直接動かし、引数のすり替え・対象違いの拒否と、
正しい要求の実結果の返却を確認した。16Eでは、承認した引数と異なる要求を受け取っても
実効果を呼んでいた受信側の不足を再現し、3つのCI/CD処理を修正した。文章作成も、
2つの資源要求の一致だけでなく、承認済み引数との一致を受信側で確認する。
比較用に残すのは実入力の値であり、進行flagや別の権限表は追加していない。

修正後のnative19件とClippyが通過した。実Wasmは16Eの元障害3条件・作業前の終了拒否と、
2E/2M/2Hの各5条件、計19条件が通過した。資源違反0件、監査の252変異を拒否し、
文章作成の終端レビュー失敗3件は元scoreの失敗として保った。通常遅延の独立writeは
3/5/7人で1/2/3件重なった。完走した18記録を別processの同じauditorでも照合し、
組み込み監査と一致した。監査アルゴリズムは同じである。
build前後のアプリ・core・WASI・washのsource hashは一致した。
buildは306秒、全子孫の最大memoryは1,088,407,040bytesだった。
これらはローカルcore修正での機構試験で、model呼出し・採点実験は0件である。
この19条件を現在の通常Git pinで認定し直す作業と、T3/T4の一般的な合成は残る。

1Hの委譲版には、通常のmodule localsideを動かさず、現在のglobalを公開`project`で
直接駆動する排他試験を追加した。A/Bそれぞれのdesign・implementationを完了し、
AがDB用Endpointを保持する間はBの受信がPendingとなり、Aの実要求・結果・返却後に
同じEndpointをBが取得できる。保持中の別attachも両者について拒否される。
この1つの操作列はローカルcore修正を選ぶmodule単位の試験で通過した。
全スケジュールの証明や通常Git pinでの全crate認定としては数えない。

旧native RPC、専用CLIと小型shellは削除済み。実験用transportは一つの値がqueueとbindingを
所有し、runtimeが借用する。現行の外部JSON接続は`ExternalIo`がstdin/stdoutと応答対応を所有する。
API結果ファイルはファイル担当が内容・offsetを所有し、metadataもglobalの要求・応答で処理する。
旧native codebaseの要求queueとrunnerの共有記録・callbackは`252b821`で除去した。
後続でAPI／codebaseの旧Python接続も除去した。未移行課題は現行の実Wasm経路へ移す。

通常の実験コマンドとsubscription試験は、同じrunnerから移行済み12課題を実行する。
選択したruntimeのbuild receiptと全sourceを照合し、実行前にbinary、wash、入力を保存する。
この入口から6E・障害条件2をLuna xhigh 3人で実行し、43.02秒で元課題を完了した。
実モデル3人の重なり、review 4回、実装／reviewの重なり3組、merge／reviewの重なり1組、
違反0件、inclusive 426,541 tokensを独立監査で確認した。採点比較への算入は0件である。
監査器のthread応答IDの決め打ちを、記録されたrequestとの対応へ修正した。
同じraw記録で監査と9種類の記録変異の拒否を確認し、モデルは再実行していない。
runnerの通常テスト24件、Clippy、buildが通過した。wire label／laneの投影照合は引き続き未完である。

食事課題も共通のRust runnerから実Wasm・元simulator・記録監査を実行する形へ統一した。
3／5／7人の3条件を通常遅延で完走し、独立した食事の最大重複は1／2／3件、違反は0件だった。
早すぎる要求、重複、型違い、分割書込み、表示名引数の保存、両資源の返却を検査し、
9種類×3記録の変異を拒否した。モデル呼出しは0件で、試験全体は54.25秒だった。
globalの構成とEndpoint操作順は変更していない。roleとsessionの定数を監査と共用し、
専用のPython subprocess、別監査binary、未使用の旧接続・監査scriptを削除した。
この時点で残っていたAPI／codebaseの旧CLIは、下記のRust経路への統一で除去した。

同じ通常Git pinのbinaryを使い、9HのLuna xhigh 7人による非採点pilotを一度実行した。
19.784秒で元課題を完了し、実モデル7人と独立した食事3件の重なり、14本のEndpoint返却、
違反0件、inclusive 571,238 tokensを独立監査で確認した。人工的なbarrierは使っていない。
実記録の10種類の変異を拒否し、raw model／shell／transport／元採点、sourceとbinary hashを保存した。
runtimeのbuild前後のsource一致、アプリ21試験、runner24試験、両者のClippyを確認した。
このpilotと機構試験は採点比較へ数えず、wire label／laneの投影との意味照合は未完のままである。

APIとcodebaseの実Wasm機構試験も、通常runnerと同じRust simulator接続・監査・
source保存へ統一し、旧CLIと最後の自作Python接続を削除した。元simulatorは変更していない。
APIは3条件・22.45秒、codebaseは14条件・415.52秒で成功し、記録変異42件・140件を拒否した。
APIではDB_ADMIN／BACKEND_DEVの結果file読取りを両方Pendingにした後にTESTERを開始し、
結果の単独所有と独立した私有I/Oを確認した。これはモデルなしの明示的な試験用scheduleである。
codebaseは1Hの再考、1E／1Mの正常な業務失敗、早すぎるcommit、引数・型・重複の拒否、
独立commitの並行性を保つ。全17条件で実行前にruntime、wash、runner、source／receiptを保存した。
アプリ21試験、runner24試験、両者のClippy・build、実行前後のsource一致が成功した。
全子孫の最大memoryは704,987,136bytes。モデル呼出しと採点比較への算入は0件である。
記録は`output/review-occurrence-qualification/rust-boundary-unification`に保持する。

通常runnerの終了判定で、内側のparticipant結果に保存したtimeout／接続errorを
集約しない欠陥を修正した。guestの完了後でもモデル参加者が失敗すれば、記録のerrorと
CLIの失敗終了へ反映する。runtimeのerror・異常終了・backend失敗も同じ集約で確認する。
正常に完了したprotocolとモデルが返す業務失敗は、引き続き元scoreの失敗として区別する。
実際のbyte-stream driverへ成功したguest終了応答を返した後、接続切断／timeoutを起こす
2回帰試験が成功した。旧集約だけへ戻すと両試験が失敗し、復元後はrunner全28試験・
Clippy・buildが成功した。最大memoryは705,511,424bytes、モデル呼出し0件だった。
記録は`output/review-occurrence-qualification/participant-result-qualification`に保持する。
全員の完了報告が揃った後のgroup errorも失わず、通常runnerへ返すよう修正した。
未完了RPCを残して全員の完了通知を返すbyte-stream試験で、この経路の拒否を確認した。
この修正だけを戻すと回帰試験が失敗し、復元後の全29試験・Clippy・buildが成功した。
この追加検査の最大memoryは701,235,200bytes、モデル呼出し0件である。

## 本体・統合の保証

本体の現在の検証対象は`feature/checker-resumption-preparation`で公開する。
ユーザーの指示に従い、固定したsourceのCI成功後にmainへ取り込み、不要な開発branchを削除する。
リリースと論文artifactの認定は、実際に完走した検証・consumerの組合せへ別に対応付ける。
coreのRust 2024、no_std、heap割当なし、caller storage、lockなしを維持する。
採用Pico/Pico 2のstack/SRAM/flash予算、命令数・コンパイル容量gateを保つ。
`long_running_const_eval`のallow/expectは禁止し、元のglobalとforbidで検査する。

現在の本体は生成に`OperationIndex`を再利用し、actor行の検査は生成iteratorから独立している。
検査済みactor bindingを信頼する範囲と、source所有権との別の照合を区別する。
関連Rust変異試験、Lean/Z3、strict Miri、Kaniと、100role/4,000eventのARM投影build、
1,542eventの実Endpoint試験が成功している。後続`b38178a`は試験bufferのリーク修正であり、
consumerの実行コードは変わらない。

10月末の検証はActions予算の制約でローカル実行した。11月1日からはActionsも再開し、
公開したSHAのCIと、sourceを固定したローカル検証を区別して記録する。
core `967a89bbbbb392387e70a74b579ecf39ec21b5c5`ではKani全319件が成功し、
全子孫の最大memoryは2,647,310,336bytesだった。元の記号入力とunwinding検査を維持した。
独立していた所有権の4証明を同じ検証入口へ統合した。本体の実行挙動は変更していない。
その全gateは、入れ子5選択の生成ケース`spines 5/22409`のRust compileがmemory上限を超えて停止した。
`d5b32c2a44c1c0dc30573425007dfe2be1ff1b59`で投影作成を同期関数へ分け、型付きglobalを
試験のasync frameへ保持しないよう生成器を修正した。本体の実行コードは変更していない。
同じ3契約の構文・操作text・probe一覧が不変であることを照合し、36完走・1,092呼出し・
2,043,136操作試行でモデルと実行時の不一致0、2,240局所拒否、全感度controlが成功した。
最大memoryは2,731,363,112bytesで、元と同じ2.75GB guard、forbid、最適化とassertionを維持した。
この版のKani全319件は成功し、全子孫の最大memoryは2,587,378,000bytesだった。
続く生成検査では、同じ公開message contractのaliasからresolver入力を作り直すと、
受理時と異なる分岐になる試験器の不具合が露呈した。`510fe108`で受理時の入力を保存し、
再生とLean照合で同じ入力を使うよう修正した。元の停止shardは188形・681,484操作試行で
runtime-only/model-onlyの不一致0だった。元の試験器の処理だけを戻すと回帰試験が失敗する。
同版の全gateはrolled predecessorのKaniで2.75GB上限を超え停止した。
`bb33e40c1e7e7f5d7b338e41b0b7448ef10f3eb3`では元の`u8 % 3`の値域を0/1/2へ分割し、
他の記号入力とassertionを維持した。個別の3証明は成功したが、最終sourceのKani全件と
protocol fuzz、Miri等を含む全gateは再実行が必要であり、全体成功とは扱わない。
アプリの通常Git pinは`e139912e`のままで、固定する本体版への更新と統合認定は残っている。
旧pinのMiri生成木群は180秒で時間切れになっている。停止を成功へ変えず、必要な検査を通す。
アプリ/executor/transportの前提とモデル定理、Rust実装の試験を区別し、任意Rust全体の完全証明を
追加の無期限な条件へしない。具体的な不一致が見つかったら、その修正を試験拡大より優先する。

生成検査は、深さ1全数、同型を除いた小さい合法形、長さ4〜5の入れ子相互作用、深いseed付き
抽出と縮小を区別する。両側が複合的なpar、選択前の受信、委譲先のchoice/roll/par/再委譲を含める。
合法形は実Endpointを並行駆動し、初手だけでなく継続も確認する。非合法形はコンパイル拒否を確認する。
決めた対象表の完了を網羅済みと呼び、全構文・全scheduleの証明と混同しない。

TLS/QUICは修正coreを実際に選択した全体試験・CI・44件interopで統合を確認する。
ACK、再送、TLS、輻輳制御、timer、byte offset等の必要データは保ち、引継ぎflagや二重完了表を除去する。
本体be87736を使うQUIC `7fd3fad`のCIと44件interopは成功済みだが、現在pinの認定へ流用しない。
CI待ちの間も独立した実装作業は続ける。

## 残る工程

| 工程 | 次の成果物と通過条件 | 状態 |
| --- | --- | --- |
| M0 比較の固定 | 基準SHA、構文範囲、native表現監査、課題と接続、指標・精度・再実行規則を固定する | 更新計画を記録。新競合と精度設計は未認定 |
| M1 意味論 | T1–T6の定義・前提・反例・必要補題と、既存core定理の対応表。評価に使うpar/roll/委譲を含む | T3/T4の最小receiver契約下の合成済み。残る命題の対応と、具体的な二資源・実効果への適用を進める |
| M2 実装と証明 | 効果入口一覧、実行時の負例、Leanと実装照合、必要なcore/WASI gate。通常Git pinでレビューと1Hを認定する | 通常Git pinの実Wasmはレビュー34条件・1H5条件・食事3条件、実モデルは1H7人・6E3人・9H7人のpilotが成功。wire監査、全条件・全効果入口とT3/T4の一般合成は未完 |
| M3 課題と接続 | 全48課題と採用追加課題のglobal/localside・実Wasm・元採点・接続を認定する | 12課題を移行済み。残る36課題を移行する。比較接続の開発は保留 |
| M4 本比較 | A1の864件を先に実行し、A2とB–Eを認定後に実行する。全identityにrawと終了理由を残す | 採点0件 |
| M5 原因と分析 | F/G、8課題の変更負担、cluster対応集計、負例と不確実性を整理する | 未実行 |
| M6 論文化 | 最新owned実装へ草稿を同期し、各主張と定理・測定・TCB・再現artifactを対応付ける | 草稿。結果表と結論は未確定 |

直近の作業順は、(1)本体の最終sourceで全gateを完了、(2)T3/T4と全効果入口・wire監査の対応、
(3)通常Git pinの更新と移行済み課題の実験・残る課題の移行、
(4)T1/T2へpar/rollを接続しT5/T6の前提を固定、とする。比較接続の再現作業は進めない。
M1と効果境界を最初の技術判断点にし、
資源の合成保証が成立しないまま2,592件を先に消化しない。独立したM3の作業は進めてよい。

M3では旧nativeの進行管理・不要な入口・自作Pythonを除去し、残る課題を委譲と実Wasmへ
移行する。比較接続の新規開発と応答境界の再現作業は行わない。ChoreoFixの実験は
利用可能な接続条件で進め、source、model、設定と実行条件を保存する。
比較対象がその条件で動かない場合は未実行と記録し、ChoreoFixの実験を止めない。
元の比較sourceを変更せず、条件の違いがある結果を同一条件の比較として報告しない。

各認定にはsource/lock/proof/compiler/binary/Wasm/model/configとbuild receiptを保存する。
古いsnapshotの`.cargo/config.toml`前提やauditの呼出し数も最終経路で検査する。
正常・拒否・故障・早期終了、実資源の唯一性と結果、実際の並行性、oracleの感度を確認する。
旧成功を変更後sourceへ流用せず、具体的な不一致の修正を試験拡大より優先する。

受領計画の技術予算8–12人週は納期の約束ではなく、M1で再見積もりする。
全runが600秒を使う逐次実行の上限はAが432時間、最大4,560件で760時間である。
実際の所要時間は非採点pilotの分布・接続制限・再試行から見積もり、上限計算と区別する。
予備試験の成功を成功率・速度・品質の優位性へ外挿しない。

## ローカル作業と完了判定

重い検証は同時に一つ、Cargo/Kani jobsは1。全子孫合計3,000,000,000bytesで停止するguardを使う。
現在は余裕を取り2,750,000,000bytesのguardで実行する。上限停止は未検証として残し、
予算の引上げ、globalの縮小、検査の無効化で成功へ変えない。検証前後のsource hashを照合する。
最新stableで開発し、Miri/Kaniだけは各ツールが必要な検証compilerを使う。

既存のdirty sourceを保ち、同名packageの別checkoutでtarget directoryを共有しない。
本体等の検証済み修正は開発branchへpushする。研究repoの現在sourceはGitHubの
`development/source-capture-preparation`へ公開し、旧検証ログ・sourceコピー・build生成物は追加しない。
GitHub Actionsも利用し、公開したSHAに対する必須検査を起動する。以前のActions予算を
理由に`[skip ci]`を常用しない。ローカル検証は引き続き実施し、CIの受付・実行中と成功を区別する。
古いbuild・不要cache・未使用の実験成果物は削除し、profilerと論文に必要なraw証拠は保持する。
ビルド成功後に毎回この整理を行い、置き換えたbinaryや未使用target、一時ログを溜めない。
現在使う成果物・再利用するcacheと、未解決の反例・検証停止の証拠は残す。圧縮保管へ逃がさない。

完了にはT1–T6と実装の対応、受理した全課題の適合性、実際に使う接続条件の記録、
事前に凍結したA–Gの実験・負例・独立採点・集計、論文と再現artifact、必要な統合gateが必要である。
未対応と除外母集団を明記する。最大600の予備枠を使い切ること自体は完了条件にしない。
未実行・間接的な証拠は完了に数えず、全Rust・全分散障害の完全証明を新たな無期限の条件にしない。
