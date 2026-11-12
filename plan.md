# ChoreoFix 研究開発計画

更新: 2026-11-11。受領した「ChoreoFix 研究開発計画」計画版1.0を反映した。
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
最小ケースの8ロール・2runtimeの実行では、両資源の実受信と公開効果を、同じcompiled sourceと
完全なproducer captureから構成したEPFの動的出現へ結び付けた。実transportの両要求の引数が
実際の肯定的検証・現在版と一致し、公開の観測より前に別々の受信があること、片方の出現を
他方へ差し替えると独立検査器が拒否することを確認した。これは具体的な一実行と既存の負例の
対応であり、任意Rustのrefinementや全課題の認定ではない。残る受信側と他の実引数の対応は未完である。
新しいruntime台帳や出現IDは追加していない。

| 目標 | 命題と完了の証拠 | 現状 |
| --- | --- | --- |
| T1 完全実行の対応 | 評価で使うmessage/seq/par/route/resolver/roll/委譲について、globalと理想投影の完全実行を双方向に対応させる | 既存core定理の適用範囲と必要補題を対応付ける。研究全体の証明完了とはしない |
| T2 有限prefixの対応 | 非同期seq、join、分岐知識、任意の有限roll展開、移譲後の継続が合法prefixを保つ | 既存出現モデルを再利用する。深さ0/1/2の試験を一般命題の代わりにしない |
| T3 一意性と継続 | 初期binding、送信future、移送中、受信値、失効を含む全遷移で利用可能権限が高々一つで、継続が保存される | coreのcustody定理と最小receiver合成あり。8ロール・2runtimeの実R/S受信・公開を同じEPF出現へ照合済み。全効果入口への適用と最終artifact認定は未完 |
| T4 実要求と実効果 | 各共有効果は一つの受理済み動的操作に対応し、対象・版・引数が一致し、必要な全資源と先行結果を満たす。拒否は効果も業務commitも生まない | 最小receiver契約でデータ照合と受信の再利用禁止を合成済み。具体的handoffで両受信・引数・実検証・現在版・公開の対応と出現差替え拒否を確認。全効果入口、他の実引数、最終artifact認定は未完 |
| T5 実結果による分岐 | resolverが同じ動的操作の実結果で分岐し、別対象・旧版・以前のrollの結果を流用できない | 結果駆動の再考試験あり。対象・版・承認の一般的な対応は未認定 |
| T6 進行と条件付き完了 | 到達状態を正常完了、明示的失敗、外部応答待ち、合法遷移可能に分類し、終了する部分のprefixを既決定のまま延長できる | 既存進行定理の前提と適用範囲を照合する。非協力的guestの終了を保証しない |

T3の所在はghost stateとして定義し、runtimeへowner表や新しい権限tokenを追加しない。
利用可能な所有値と、受理後に送信futureが保持する権限ゼロの証拠を区別する。
送信側の局所I/Oエラーを、受信側へ移った権限の失効や再発行の根拠にしない。
初期発行と物理aliasの唯一性、FIFO・非重複配送・保持者からの受信可能性は明示した境界に置く。

`Domain::execute`は通常のhostメソッドである。受信した要求、実引数、必要資源の対応を
確認する箇所を[効果境界とadapter契約](benchmark/effect-boundaries.md)へ列挙する。
受信側の一覧と各課題の移行記録を対応付ける。現在の移行済みは29課題・23family、残り19課題であり、最終sourceでの全family認定とは区別する。
共通採点の因果不足を修正した`common-environment-v2`で、4M・14Eの実Wasm計8条件と
実AI5人・4人のpilotが共通採点・EPF・実効果監査まで成功した。既存のversion-1予備記録を
その合格結果へ読み替えず、他の課題の共通因果採点の認定は別に進める。
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

現在選択する依存は[アプリのmanifest](Cargo.toml)と[lock](Cargo.lock)、
[runnerのmanifest](tools/runner/Cargo.toml)と[lock](tools/runner/Cargo.lock)、
[効果境界の検証crate](proofs/endpoint-handoff/Cargo.toml)に記録する。
この計画書へ「現在のpin」を重複して転記しない。各認定は、その実行のsource manifest・
dependency tree・binary hashへ対応付け、下記では検査した版を個別に示す。
開発中は、現在検査中の本体を含む正本の隣接worktreeを直接参照する。
アプリ・runner・効果境界検証crateとWASI・EPFは、実際のCargo metadataでも同じ本体へ揃える。
古いGit固定値へのfallbackや別checkoutは作らない。本体の検査中はそのsourceを変更せず、
実行記録にはGitの基点・dirty patch・source hashを保存する。
検査中の版を使った開発・予備実行と、そのsourceの最終認定を区別する。
ビルド入力の保存は、アプリの`source/`と隣接する本体・WASI・EPFを一つのworkspaceとして扱う。
同じソース一覧でビルド前後を照合し、Gitの基点・差分と、相対依存を保った実ファイルを記録する。
この保存処理の変更は、ローカル依存の変更・追加・削除、bundle移動後のCargo依存解決、
build入力の欠落・改変・逸脱を拒否するrunner試験まで通過した。現行sourceを対象にした
全Wasm認定での保存・再監査はまだ完了していない。
11月7日、アプリ・runner・handoff・EPF・WASIの5つのlocked Cargo metadataで、
検査中の本体正本だけへ解決されることを確認した。証拠は
`output/verification/current-core-source-20261107/refresh.u5RyZxfh/`へ保存した。
CIも同じ隣接配置へ変更し、EPFの全分割は開始時に選んだ一つの本体commitを使う。
WASIのpackage検査は古いGit依存専用の処理を除き、同じ本体でarchiveをビルドし、
そのGit基点・差分・入力hashの不変性を記録する形へ変更した。
依存解決、format、shell構文、workflowのYAML読込みは成功した。
保存処理・package検査の実行と、この組合せの全アプリ認定はまだ未実施である。

11月6日の全アプリnative検査では、公開本体`724f3e3`が、以前に通っていた同じ
1,586イベントのreview globalをconst評価上限で拒否した。これは未対応の新しいglobalではなく、
11月5日の本体`9bb361112`で、各wordごとに参加者集合を列挙し直すようにした性能退行だった。
当該変更だけを戻すと同じglobalが50.12秒で通り、部分木の枝刈りを保持して参加者の列挙を
wordループの外へ出した作業sourceでも49.36秒で通った。元のglobalとlocalsideは変更していない。
その後、同じ作業本体でreview・codebase・CI/CDの実ホスト3本を4分39秒でビルドした。
これは本体の検査入力として固定した公開consumer sourceでの結果であり、通常consumerの
未認定本体への依存差替えや全体認定の成功とは扱わない。
修正版の公開本体`a4d48cdc`では、通常Rust gate、38件の必須strict-provenance Miri、
54のLean実装モデル、11 workloadの処理量予算と、統一した生成適合試験の通常43割当が
同じ変更のないsourceで完了した。32分割と共有契約15種の6,530ケース、
39,538完走script、30,644,346次操作probe、7,376,442 parked-preview probeで
runtime-only/model-onlyは0だった。全割当と保存した結果のhashを照合し、
6種類のproduction欠陥を戻す感度検査と縮小controlも通過した。
2 compilerを使った一つの分割は2,750,000,000bytesのguardで停止したため未完として保存し、
同じ生成入力・lockを1 compilerで実行して合格した。上限や入力域は変更していない。
最終の継続実行は最大1,439,449,088bytesだった。通常CI契約の有限入力域の結果であり、
全protocolの証明、新しい全Kani成功や拡張認定の完了とは扱わない。
source・command・結果は本体の`target/consumer-capacity-qualification/final-audit`へ保持する。
100 role・4,000 eventのmixed/hierarchicalのcompile負荷検査も通過した。
通常CIの公開実行を待って同じローカル検査を繰り返すのではなく、この合格した公開本体へ
consumerの通常Git依存を揃え、consumer自身の統合検証へ進む。
同じSHAの[公開通常CI](https://github.com/hibanaworks/hibana/actions/runs/42130742532)も
全割当で成功し、mainへfast-forwardした。READMEの画像参照だけを修正した後続版は
実装を変えていないため、consumerは合格済み実装SHAを維持する。

この退行を本体の通常CIでも検出するため、固定した実global・localsideを使う必須回帰を追加した。
6Eを含む3ホストのビルドに加え、実Wasmで1Hの3件同時commit受理と16Eの障害条件0・1・2、
計4条件がローカルで完走した。1Hはcompiled source399イベント・35 program、
16Eは160イベント・13 programである。独立した3件のcommit要求を受理するまで応答を
保留する元の試験を使い、単なる時間の重なりへ依存しない。モデル呼出しと採点実行は0件。
記録は本体の`target/choreofix-regressions/run.kgEA61jY/evidence-core-contract`へ保持する。
この回帰は本体のビルド容量と実Endpoint進行を守り、consumer全体の認定を重ねない。

WASI`b5d1f98`はこの公開本体をmanifest・lock・実際のCargo metadataで一つだけ選択し、
134.78秒で全ローカルgateを通過した。実ChoreoFS、実Wasmシェル、Leanの権限照合、
Pico 2、strict-provenance Miri、配布archiveを含み、最大memoryは1,185,546,240bytesだった。
検査中のsourceは変更せず、成功後に公開した。アプリのglobal・localsideを変更した結果や、
ChoreoFix全体の統合認定としては扱わない。

EPF`38b76e0`も同じ公開本体を選択し、全ローカルgateを716.67秒で完了して公開した。
native125件、doctest2件、strict-provenance Miri109件、6つのLean証明モジュールと
35種類の欠陥検出が成功し、最大memoryは1,371,471,872bytesだった。
アプリ・runner・効果境界検証crateの依存更新は、この二つの公開libraryを使う。
以下のEPFの探索改善と実6E保存記録の結果は、元の検査版へ対応付けたまま保持する。
このlibrary単体の合格を、更新後ChoreoFixの全実Wasm・実モデルの合格へ読み替えない。

この組合せのChoreoFix `1adcff7`で、アプリ44件、runner33件、handoff native15件・
実receiverのMiri6件とClippyが成功した。1,586イベントのglobalを変更していない。
全familyの実Wasm認定は、最初の6E guestが終了して4,972件の完全captureを保存した後、
重いオフライン監査を改善するため明示的に停止した。guestの不正進行や試験合格とは扱わず、
停止理由・元のsource・binary・captureを`output/qualified-stack-a4-20261106/wasi`へ保持する。

公開EPF `e0690aa`では、不変のsend-call解析を一度だけ行い、各prefixの受信対応は
毎回作り直す。送信候補は(session, source, 証拠中の順序)で索引化し、受信ごとの
全prefix再走査を除去した。source出現数の必要条件も一巡の集計にした。
同じ4,972記録・902受理の公開監査結果は旧`38b76e0`とbyte単位で一致し、
同じ最適化・assertion・算術検査設定のCLIで94.90秒から41.72秒になった。
これは一つの保存記録のオフライン監査であり、アプリやQUICの性能の結果ではない。
最終sourceでnative128件、doctest2件、strict-provenance Miri112件、Lean6モジュール、
38種の欠陥検出とno_std/embedded/Clippyが成功し、最大memoryは877,051,904bytesだった。
本体SHAは変更せず、公開後に三つのmanifest・lock・実Cargo metadataをこのEPFへ揃えた。
認定driverは同じ記録の動的監査を一回だけ行い、その結果を記録変異試験と共有する。
監査器だけを通常debug/testでも最適化し、debug assertionとoverflow検査は有効のままにする。
更新後の全実Wasm・実モデル認定は、完走した証拠がそろうまで未完である。
この更新後sourceでrunner33件、handoff native15件・Miri6件とClippyが成功した。
通常debug runnerによる保存済み実Wasm記録の監査と25種の記録変異は48.71秒で完了し、
元の公開監査JSON全体が旧pinの結果と一致した。最大memoryは744,980,480bytesだった。
変更していないアプリ・wash・入力検査のnative結果は`1adcff7`の記録を保持し、
driver・文書の整理だけを理由に同じ義務を最初から実行し直さない。
根拠は`output/epf-indexed-stack-20261106`へ保存する。

同じ固定版の全実Wasm認定では、レビュー16条件の成功後、17条件目の監査中に
2,867,527,872bytesで2,750,000,000bytesのguardが停止した。元の入力・source・binary・
16件の成功結果は保持する。同じ17条件目の7,086記録を同じ監査binaryで単独再実行すると、
162.08秒・最大628,441,088bytesで成功した。試験プロセス内でのケース間の確保の蓄積と
切り分け、監査を一ケースごとの子プロセスで実行するよう試験側を変更した。
本体・EPF・アプリのglobal/localside・34条件の入力定義・上限は変更していない。
Rustの個別テストで26通常条件・8追加条件を選択できるようにし、全入力が一度ずつ
割り当てられることをnativeで確認する。CIの通常呼出しは全34件を実行する。
変更後runnerのnative34件とClippyは成功した。残りの実Wasm条件の再開と集約は未完で、
停止した全体呼出しを成功とは扱わない。

EPFの公開`712b49bb`は、同じ不変sourceに対して、必要な先行操作と既確定の分岐から
成立しない選択を先に除く。委譲で保持者が変わる場合も含む260,512問い合わせを、
固定sourceの全選択と2種類の操作順の全有効部分列から得た比較実装と照合した。
これは全interleavingや任意のRust実装の一般証明ではない。native125件、doctest2件、
6 Leanモジュール、欠陥検出35件、strict-provenance Miri109件が同じ製品sourceで通過した。
故障注入driverの対象修正では、成功済み19件を保持して残る16件を実行し、全35件の一致を確認した。

その版で、停止していた実6Eの保存記録を、入力・容量・global・localsideを変更せずに
最後まで監査できた。完全capture8,127記録と1,475送信受理を20,254展開eventへ対応付け、
最終の独立した実行検査を通過した。proof容量は元の16,255のままで、CLI実時間480.41秒、
全子孫の最大memoryは632,979,456bytesだった。binary・入力・結果とsourceのhash、
producer checkpointを`hibana-epf/target/query-domain-qualification/real-capture-final`へ保持する。
この記録は固定consumerを用いた本体回帰fixtureの実WASI出力であり、新しい通常Git pinでの
consumer再実行、全familyの認定、実効果の一般保証、実モデル・採点実験の完了ではない。
以前に停止した試行は停止のまま残す。通常の本体CIへ監査探索を重ねず、consumerの
依存更新と最終の実Wasm認定は、本体の必要な検証が完了した公開版で実施する。

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
この組合せで、handoff検証crateのnative15件とstrict-provenance Miriの実receiver6件、
Clippyが成功した。Miriの外側の壁時計は113.23秒で、Miri内の試験時計の255.80秒と区別する。
全子孫の最大memoryは432,275,456bytesだった。完全captureとsource/hash、三つの実依存解決、
実行ログは同じ親ディレクトリの`handoff-epf21078b0/`へ保持する。API課題の採点・実モデル・
移行済み全familyの認定には数えない。次は通常runnerと実Wasmの統合認定を行う。
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
具体的な全効果入口への定理適用、残る29課題の委譲移行は未完である。
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
| 3E | 現版で通常18＋追加8＋不正入力1条件、実AI3人の予備試験が元採点・EPF／実効果監査まで成功。28記録の6,797記録変異を拒否 | 最終core／consumer artifactの固定と本実験。13課題目として算入済み。比較上の優位は未評価 |
| 4E | 実REPO／TEST_ENV Endpoint、API再合意と両実装、編集権返却後のテスト。native22条件＋不正引数3条件、実Wasm5条件と実AI3人の予備試験が元採点・EPF／実効果監査まで成功 | 14/48へ算入。採点比較と最終sourceでの全family認定は未完 |
| 4M | 64条件のnative／EPF、実受信側の負例2件、実Wasm5条件、実AI5人のpilotが共通採点・EPF・実効果監査まで成功。共通課題の改訂は手順の採点ではなく公開内容と検証結果を対象とする | 15課題目として算入済み。最終sourceの全family認定・本実験は未完。手順と版の範囲は`benchmark/ports/4M.md` |
| 14E | 実装置Endpoint3本、実受信側の負例2件、実Wasm3条件と実AI4人のpilotが共通採点・EPF・実効果監査まで成功。simulatorは必要な入力の因果関係を検査し、独立した順序を許す | 16課題目として算入済み。最終sourceの全family認定・本実験は未完。範囲は`benchmark/ports/14E.md` |
| 13E | 実装置Endpoint2本、実受信側の負例3件、実Wasm3条件と実AI3人のpilotが共通因果採点・EPF・実効果監査まで成功。試料・試験・品質結果を実データで結ぶ | 17課題目として算入済み。最終sourceの全family認定・本実験は未完。範囲は`benchmark/ports/13E.md` |
| 13M | 全体global（589イベント、16 actor）と7つの直接localsideを実装。修正した最新本体で、4-Wasmの10条件と実AI4人のpilotが共通因果採点・動的EPF／実効果監査まで成功。実AIは86.04秒、710,689 inclusive tokens、実効果17件、違反0 | 18課題目として算入済み。最終sourceの全family認定・本実験は未完。範囲は`benchmark/ports/13M.md` |
| 5E | CTとpanelの実Endpoint委譲、分割・逆順・反復panel、独立した紹介と薬剤レビュー。実Wasm9条件、3関数のstrict-provenance Miri、実AI3人のpilotが共通採点・動的EPF／実効果監査まで成功。pilotは31.45秒、257,121 inclusive tokens、6実効果、違反0 | 19課題目へ算入。残る29課題と最終sourceの全family認定・本実験は未完。元toolにない処方・投薬効果は認定しない。範囲は`benchmark/ports/5E.md` |
| 5M | 実Wasm8条件とLuna5人のpilotが共通採点・動的EPF／効果監査まで成功。pilotは48.37秒、571,513 inclusive tokens、12実効果、違反0 | 20課題目。元の失敗記録と修正後readerの再監査を区別。範囲は`benchmark/ports/5M.md` |
| 5H | native92完走条件・要求差替え12条件、5関数のMiri、実Wasm8条件とLuna7人pilotが成功。pilotは48.16秒、679,177 inclusive tokens、16実効果、違反0、実効果の重なり7組 | 21課題目。共通採点・動的EPF／効果監査まで成功。最終source全family認定・本実験は未完。範囲は`benchmark/ports/5H.md` |
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
未移行33課題と、T3/T4の最小合成を具体的な全効果入口へ適用する認定も残る。

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
| M3 課題と接続 | 全48課題と採用追加課題のglobal/localside・実Wasm・共通採点・接続を認定する | 29課題・23familyを移行済み、残り19課題。14Hの12実Wasm条件・7人実モデル・独立監査と28条件のreceiver Miriを追加。11Mは工具と再作業の部品までで全体未完。最終sourceの全family認定・本実験は未完。比較接続の開発は保留 |
| M4 本比較 | A1の864件を先に実行し、A2とB–Eを認定後に実行する。全identityにrawと終了理由を残す | 採点0件 |
| M5 原因と分析 | F/G、8課題の変更負担、cluster対応集計、負例と不確実性を整理する | 未実行 |
| M6 論文化 | 最新owned実装へ草稿を同期し、各主張と定理・測定・TCB・再現artifactを対応付ける | 草稿。結果表と結論は未確定 |

直近の作業順は、(1)本体の最終sourceで全gateを完了、(2)T3/T4と全効果入口・wire監査の対応、
(3)検査中の最新本体へ参照を揃えた移行済み課題の実験・残る課題の移行、
(4)T1/T2へpar/rollを接続しT5/T6の前提を固定、とする。比較接続の再現作業は進めない。
M1と効果境界を最初の技術判断点にし、
資源の合成保証が成立しないまま2,592件を先に消化しない。独立したM3の作業は進めてよい。

M3では旧nativeの進行管理・不要な入口・自作Pythonを除去し、残る課題を委譲と実Wasmへ
移行する。比較接続の新規開発と応答境界の再現作業は行わない。ChoreoFixの実験は
利用可能な接続条件で進め、source、model、設定と実行条件を保存する。
11月7日、3Eの旧unit grant・opaque domain call・native専用実行を除去し、
研究・DOCUMENTの貸出／実要求／実結果／返却・DATABASEの同じ交換・実レビュー結果に
基づく再修正を一つのglobalと直接のlocalsideへ書き直した。初回レビューの順序を選べ、
修正と他方のレビューが独立する箇所はparを維持する。最終combineは反復の外へ置き、
実際に受け取った両方の肯定的レビューをDOCUMENTの所有者へ転送してから要求する。
新しい`report-wasi`、実ChoreoFSのclose入力のcodec、実receiverの差替え拒否と
応答返却の試験、実Endpointで最終combineまで駆動するnative試験を追加した。
正負の実レビュー結果とメッセージの一致検査は、6Eと3Eで同じ`ReviewReply`に集約した。
6Eの既存wire schemaとcodecの規則は維持する。旧native試験が対象にした、複数回の拒否後の
再考、両資源の排他、研究同士・DOCUMENT／DATABASE・レビュー／他方の修正の重なり、
早すぎるcombineの実効果0という義務は、実Wasmの新経路で再検査する必要がある。
3回目・4回目の拒否、初回A/B、承認・combine後の再修正も含める。
これは実装中であり、移行済み12課題の件数には加えない。
現在検査中の本体正本を使ったlib testのビルドは5分12秒で成功し、
引数と実レビュー結果の2試験は通過した。一方、実Endpointの全体試験はofferの
PhaseInvariant、receiverの正負prefix試験2件は時間切れとなった。試験構成・global・
本体のどこに原因があるかはまだ確定しておらず、localsideの並べ替えで回避しない。
実行証拠は本体の`target/offer-reset-regression/consumer-native.49qGrxiV/`へ保存した。
全子孫の最大メモリは1,611,546,624bytesだった。通常runnerの3E prompt・snapshot・監査入口も
接続したが、実Wasm・元採点・EPF／効果監査はまだ未実行である。
続く診断実行で、PhaseInvariantはDATABASE所有者が分岐外の最初の受信をofferしていた
箇所と特定した。DOCUMENTのprefixも送信済みの既知要求をofferでは受信していなかった。
本体の公開契約に合わせ、資源所有者の初期受信と反復外のcombineをtyped recvへ書き直し、
guest側も研究・初回執筆・文献更新の既知送信にofferを使う箇所を除去した。
続いて、editorの私有I/Oを初期提出待ちで止めず、参加者へ他人のレビュー通知を残さないよう、
3Eのglobalを整理した。両方の初期提出の実結果は、実レビューを行う二つのserviceが直接受信する。
再修正の実結果は、その担当review serviceへ渡してから再レビューする。レビューintentと
実結果の通知先は、DOCUMENT所有者・担当service／研究者・editorのうち必要な相手だけにした。
各review serviceは本人の肯定的な実結果を送ったら終了し、guestの終了は既存のWASI Exitだけで
表す。重複するtask ExitA/Bとその待機・中継は削除した。最終combineは引き続きすべてのrollの
外で一度だけ実行する。本体は変更していない。
監査は、compiled projectionが定めるintentの全受信者・一つの実要求・必要な実結果を一意に
消費し、初期提出の4受信と各再修正の実結果を対応させる形へ更新した。
最初の再ビルドでは、他方のレビューと並行して修正する分岐で、修正者にもその分岐を知らせる
必要があることを投影検査が検出した。4つの並行分岐は、その必要なintent受信をglobalへ残し、
担当外の完了済み参加者へ通知しない構造にした。監査の期待する受信者を別の表へ複製せず、
同じcompiled projectionから導く。
他方が未承認の間は、その当事者にも実レビュー結果が分岐知識として必要なため、共有部分では
5受信者への通知を保持し、他方の実承認を受けた後の最終レビューだけを3受信者にする。
review serviceは実際の他方の承認メッセージを受けて最終部分へ進み、進行flagは持たない。
これらの変更後、現在検査中の本体正本を用いたアプリのlib試験34件が全件成功した。
3Eの引数・実レビュー結果の検査、実receiverの差替え拒否／実結果返却、実Endpointでの
最終combineまでの5試験も含む。ビルドは5分5秒、全子孫最大memoryは
1,617,379,328bytesだった。証拠は本体の
`target/offer-reset-regression/consumer-known-steps-v5/`へ、sourceの前後hash、Git基点・差分、
未追跡入力とともに保存した。同じ実行でrunnerのClippyも成功した。
アプリ側のClippyが指摘したrange patternと一度しか実行しないloopは修正し、fmtを確認したが、
修正後のClippyとrunnerの通常36試験が成功した。同じソースの全ホストbuildは5分3秒で成功した。
続く実Wasmでは、障害なし6条件と初回A拒否後にAを再レビューする1条件の計7条件が、
元simulator完了・EPF動的監査・実効果の照合まで成功した。通常遅延では研究同士と
文献更新／他方の執筆の重なりを確認した。初回A拒否後にBをレビューする条件と、
両側拒否後の条件ではOutOfOrderで失敗した。
記録により、globalに記述済みの4番目のintent送信をeditorのlocalsideが書き落とし、
次のreview要求を先に送っていたと特定した。対称な4分岐にその送信を直接追加した。
本体とglobal、試験の条件・期待値は変更していない。修正後のv2では通常18条件と追加8条件、
計26条件が実Wasm・元simulatorの完了・EPF動的監査・実効果照合まで成功した。
4種類の並行intentを実際に通り、それぞれ投影が定める4受信者へ送ったことも記録で確認した。
集計は`output/experiments/wasi-certification/current-core-report-20261108-v2/report-summary.json`、
各条件のsource／binary識別子付きデータは同所の`report-data.csv`、
受信者の被覆は`report-intent-coverage.json`へ保存した。モデル呼出し0の機構試験であり、
不正操作の追加試験・記録変異・実モデルは未完なので、移行済み12課題の件数には加えない。
続いて、承認後に再編集を要求する実Wasm試験を加えた。旧sourceは余分な編集を実行しなかったが、
終了済みの業務fragmentでofferし続けて要求の応答が停止した。実際の肯定的レビューを受信したら
完了した業務Endpointのfragmentを手放し、私有WASI I/Oを続けるようにした。
別の進行flagや所有者表、workflowメッセージは追加せず、compiled projectionの内容も同一である。
この現版で関連native5件、アプリClippy、承認後の再編集を拒否した同じ実Wasmの完走・元採点・
EPF動的監査・実効果照合が成功した。証拠は
`output/experiments/wasi-certification/current-core-report-post-approval-v3/`に保存した。

同じ検査中の最新本体と現版のbinaryで、3人の`gpt-6-luna`・`xhigh`を既存のChatGPT接続から
実際に動かした。元の障害条件2・通常遅延で57.12359775秒、全員正常終了、元課題完了、違反0。
実レビュー4回のうち2回が拒否され、AIが実際の結果を受けて2回修正してから統合した。
資源Endpointの実要求と返却は各7件、研究同士の実効果の重なり1件、レビュー／再修正の重なり2件。
3人のモデルturnも同時に動作している。inclusive使用量は500,202tokens（入力496,655＋出力3,547）で、
cacheを含み、出力内のreasoningを再加算していない。5,742件のproducer記録のEPF照合と、
実効果・使用モデル・設定・usage・保存source／binaryを結ぶ独立run監査も成功した。
rawとsourceは`output/experiments/model-qualification/current-core-report-3E-depth2-v1/`、監査とguardは本体の
`target/offer-reset-regression/consumer-report-model-v1/`へ保存した。
これは非採点pilot1件であり、比較上の優位や現版の全条件認定ではない。
現版の通常条件の再確認と不正操作・記録変異の拡充が残るため、移行済み12課題の件数は維持する。
その後、現在のruntimeで通常18＋追加8条件をすべて再確認し、元課題・protocolの完了、違反0、
EPFの動的照合と実効果監査まで成功した。通常遅延の独立研究、文献更新／他方の執筆、
レビュー／他方の修正の重なりも維持した。追加の実Wasm拒否試験は、早すぎるcombine、
不完全・担当外の引数、承認後の研究／執筆／文献更新／修正、実際のcombine完了後の再修正を拒否し、
その後の正しい作業も元simulatorで完走した。拒否の効果は追加されず、元の実効果は13件だった。
この試験だけは、combineの実応答後の入力を検査するため3人のguestを生存させる。
その制御を通常遅延の並行性や実モデルの速度へ数えない。

記録変異器は、各実business frameの欠落・同時に両観測を書き換える偽造・重複、Endpointの
包絡、WASI close、wire label／lane、実効果と受信の順序、元結果と完了を検査する。
初回・再レビューの同じ引数も実際のframe出現ごとに試し、13種類の性質を有限な記録で確認した。
主要な27 scripted条件＋実モデル1件の28記録に対し、6,797変異をすべて拒否した。
前段の狭い承認後試験は主要件数へ重複計上せず、保存済みの補助記録として区別する。
実行時攻撃6,797件の阻止やEPFそのものの変異認定という意味ではない。
既存の動的監査が通った同一rawを、判定規則を変更していないbusiness／wire監査へ再利用した。
新しい検査器についてrunner native36件、Clippy、実行binaryのbuildも成功した。
通常CIのWasm入口にも、この記録変異の呼出しを追加した。

続いて、通常18条件・追加8条件の実入力が重複なく揃い、空入力を実行前に拒否する試験を追加した。
通常CIでは不正入力群も含む3群の実行を要求する。runner native37件、Clippy、buildが成功した。
この実行前検査の追加では本体や課題runtimeを変更していないため、実Wasmの既存認定は再実行しない。
アプリ・runner・handoff・EPF・WASIの5つのCargo依存解決も再確認し、すべて検査中の正本
`../hibana`一つを使い、古いGit版やTokioを解決していない。基点は`9a826247`で、検査中の
dirty sourceを含む1,253ファイルが先の記録と一致した。記録は
`output/verification/current-core-source-20261108-report-domain/checked.json`に保存した。
これは現在作業版の一致確認であり、本体の最終gate完了という報告ではない。

主要集計は`output/experiments/task-qualification-3E-20261108/summary.json`と`data.csv`であり、
各raw、元の動的認定、変異、binaryとsourceのhashを結ぶ。
本体・EPF・WASI・アプリのRust／manifest等693入力は通常26条件の認定からbyte単位で同一で、
新しいruntime sourceが追加されていないことも照合した。変更した検査器・入力の追加認定と、
不変なruntime／EPFの既存認定を分け、無関係な本体検査を繰り返していない。
3Eの委譲実装とこの有限な機構認定は完了とし、対応数を13/48へ更新する。
全課題、最終sourceの基盤gate、T3/T4の全実効果への一般化と本実験は引き続き未完である。
元のsource・binary・rawと失敗記録は
`output/experiments/wasi-certification/current-core-report-20261108-v1/`、guardの記録は本体の
`target/offer-reset-regression/consumer-report-wasi-v1/`へ保持する。全子孫最大memoryは
1,276,936,192bytesで、容量による停止ではなかった。
診断と同じsourceのrunnerビルド、journalの2試験、source保存とsnapshotの各1試験、
6Eの共通review codecの1試験は成功した。新しい3E監査を実記録へ適用した成功ではない。
証拠は本体の`target/offer-reset-regression/consumer-diagnostics-v1/`で、入力hashとGit差分に加え、
未追跡入力もその実行時のhashと照合して保存した。
同じv2の既存6Eは通常26条件と追加4条件が成功し、4回拒否される追加4条件が失敗した。
実際の肯定的レビューと5件の承認送信の後、サービスが終了待ちのため再度offerし、
まだ合法な古いrollの拒否送信を選んでエラーになった。本体の文書化済みのelastic rollと
矛盾する挙動ではなく、承認済みサービスへ後続通知・終了待ちを残したアプリ側の終端設計を
整理する。v2の全family集約は失敗であり、後続familyは未実行。raw・source・全ホストbinaryを
同じv2のディレクトリへ保持し、guard最大memoryは1,284,341,760bytesだった。
6Eのglobalを、他方が未承認の間の通知と、他方が承認済みの最終レビューの通知へ分けた。
intentは実処理と分岐知識に必要な相手だけに送り、並行修正者を含む4分岐は4受信者を保つ。
レビューサービスは自分の承認を送った時点でそのEndpointを所有したまま終了する。
不要になったtask ExitA/B・ReviewExitA/Bとその送受信を除去し、実WASIの終了と
hibana-runtimeによる参加者の完了待機を使う。RepoExitは実処理所有者の終了として残す。
監査も同じcompiled projectionのintent受信者と、実レビュー結果による必要な通知を照合する。
フラグ・別の進行管理やoffer候補を無視する処理は追加していない。
本体は変更しておらず、この6E整理後のbuild・実行確認は未完である。
整理後の最初のbuildで、初回レビュー選択によって最初の実装の位置が変わるため、
その選択をもう一人の開発者にも通知する必要があると投影検査が指摘した。
FirstA／FirstBは両開発者を含む4受信者へ送り、必要な分岐知識をglobalへ残した。
この診断はv2の`current-core-review-endpoint-end-v2/build.log`へ保存した。
単独条件の診断も、通常と同じ`--qualify-wasi`のbuild・source保存・実Wasm・監査経路を使う。
任意のtest filterを明示した記録は常に`all_families_passed:false`とし、通常CIはfilterなしの
全familyを実行する。該当なし・空の試験結果は失敗として扱う。
単独の実Wasm診断v3では再拒否を含む両方のmergeまで進んだが、merge後の重複実装要求が
返らず62.75秒で停止した。実WASI入出力と業務操作を同じlogical roleのfragmentへ残すと、
業務終了後にもelastic rollの合法な再訪が残る。検査中の本体は変更していない。
この問題を、独立した業務用役割の返却だけで閉じようとしたglobalは、遠隔の制御役が
返却後にもrollを再開できるため、本体のReentryResourceKnowledge検査に拒否された。
本体の検査は緩めず、制御の継続そのものをREVIEWの実Endpointにした。Repository所有者から
WASIのレビュー参加者へ渡し、各code_reviewの前に実レビューserviceへ貸す。
serviceは実結果と必要な全通知を送ってから、同じEndpointを返す。最後の肯定結果では
ReviewsCompleteという型のowned messageで返し、レビュー参加者→A→B→Repositoryへ
その実値を渡して終了する。各開発者は自分のmerge・Repo Endpoint返却・この継続の転送を
awaitした後に業務fragmentを手放し、独立したWASI I/Oは続けられる。
完了flag・保持者表・例外的なoffer候補の無視は追加していない。
11月8日、const射影を実評価するnative buildが、`long_running_const_eval=forbid`のまま
5分09秒・全子孫最大1,607,221,248bytesで成功した。本体の再入検査は、source順序に沿う
依存の逆向き伝播と受信許可の前向き伝播へ整理し、分岐の所属・選択履歴を索引で参照する。
本体のnative9試験が成功し、操作依存460,800通り、受信許可276,480通り、分岐所属6,912通りを
別のsource走査・順序計算と照合した。有限な照合であり、Rust全体のrefinement証明ではない。
compiler側の変更で、resident imageの書出し、実行時の進行判定、Leanのsource規則は変更していない。

ChoreoFixのnative35試験も成功した。試験の途中で未完の制御Endpointや業務fragmentを捨てると
sessionが停止するため、prefix試験では実値を全枝の確認が終わるまで保持する。
待機中の開発者が再実装しないこと、同じbindingの二重発行を拒否すること、独立したWASI I/Oを
進められることを同じ通信列で確認した。35件のbuildは開発用の増分cacheを有効にした条件で
5分23秒、全子孫最大1,652,637,696bytesだった。実モデル呼出し・採点実験には算入しない。
本体の関連gate、runner、現在のglobalの実Wasm・動的監査・効果監査・記録変異は、
同じ最新本体sourceを直接参照して確認する。古いGit固定版への切替えは行わない。

続いて本体native843件・Clippy・thumbv6m/thumbv8mのno_std、アプリClippy、runner36件と
Clippyが成功した。実Wasmの4回拒否ケースは両mergeと全guest終了まで進んだが、Repositoryが
制御Endpointの返却と終了通知を受理せず停止した。12イベントへ縮め、選択外の終了列を
`offer()`で読む構成だったと確認した。`offer()`は選択内のpreviewであり、同じglobalの
typed受信は反復1〜3回で成功した。連携列を選択の外へ置いたこちらのAPI対応の誤りで、
この停止に対する本体の進行判定の変更は行っていない。
終了列を最初の選択の各腕の末尾へ移し、すべてのrollの外で権限を閉じるglobalとした。
各腕の受け渡し順を保ち、localsideのEndpoint操作列をそのまま再確認する。

修正後の4回拒否ケース（`review_boundary::repeated_01`）は、現在検査中の本体で
実Wasmのprotocol完了、guest3人の正常終了、元simulatorの完了・違反0を確認した。
producer自身のcheckpointと9,168記録をEPFが照合し、効果監査も成功、記録変異31種類を
すべて拒否した。runtimeは2.84秒で完了し、監査時間とは区別する。
このsourceで本体native844件、Clippy、thumbv6m/thumbv8mのno_std、アプリClippy、
runner36件・Clippy・buildが成功した。アプリnative35件も同じglobalで成功している。
残るレビュー条件と本体の生成検査は未完であり、1ケースを全family認定へ読み替えない。
今回の実行はモデル呼出し0・採点実験0である。
残る対称な3経路も同じbinaryで合格し、4回拒否・初回A/B・2種類の進め方の4経路すべてが
実Wasm・元採点・EPF／実効果監査まで通過した。記録変異は4×31＝124種類すべてを拒否した。
各条件のsource／binary／入力hash付きの集計は
`output/experiments/wasi-certification/current-core-review-scoped-finish-summary/summary.json`と`data.csv`に保存した。
通常26＋追加8の全34条件を現版で完走したという意味ではない。

通常生成検査の39/50割当は、このcompiler最適化前のsourceの結果として保持する。
実行時の本体・画像生成・モデルが同じである範囲の証拠と、変更したcompiler検査の認定を区別し、
最終sourceの全50割当が完了したとは報告しない。
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

## 4E：API変更・実Wasm・実AIの接続（11月8日）

`src/tasks/api_coordination/`のglobalが、API定義・合意・両実装・API変更後の再合意を
記述する。REPOの実Endpointを所有値で渡し、各周回でバックエンド先行とフロントエンド先行を
選べる。テスト前に編集Endpointを返し、テスターがREPOとTEST_ENVの両Endpointから要求する。
受信側は実際のAPI・両実装の結果と候補を照合し、両要求を受信してから`run_tests`を呼ぶ。
localsideへEndpoint操作を直接書き、owner表・進行flag・別の状態機械は追加していない。
各人が独立したWasm、WASI guest、ChoreoFS I/Oを持つ。元の4Eにない失敗注入やretryは足していない。

現版のアプリnative37件には、API変更0／1／3回と全実装順2＋4＋16＝22条件の実Endpoint／EPF
照合、不正なAPI名・型・余分な引数の3条件を含む。全targetのClippy、runner native37件と
Clippyも成功した。以前のnative専用sourceの5,174観測・3,802 commit対応・244 fixture効果は
`output/experiments/api-coordination-native-v6/`に保存した過去の結果であり、新しいWASI sourceの件数へ
読み替えない。

実Wasmは、両初回実装順、API変更1回／3回、元の通常遅延の計5条件が成功した。32実効果、
14,083観測を、元のsimulator採点、compiled wire、EPFの有限source execution、実要求と
実効果の対応まで照合した。全5条件のsourceは実行前後で同一である。

同じruntime binaryで、Luna xhighの実AI3人も通常コマンドから完走した。
38.378149秒、inclusive tokens 287,517、元タスク完了、違反0、実効果4件。
3,446観測に対するEPFと実効果監査、モデルの返却設定・認証・usageの独立監査も成功した。
cached inputとreasoning outputはそれぞれ既存のinput/outputに含まれ、二重加算していない。
モデル接続には既存のChatGPT認証を使い、APIキーを要求しない。

主証拠は`output/experiments/wasi-certification/current-core-coordination-v4/`と
`output/experiments/model-qualification/current-core-coordination-4E-v1/`、集計は
`output/experiments/task-qualification-4E-20261108/{summary.json,data.csv}`。
各記録に実際のsource、dirty patch、hash、binary、compiled projection、rawと監査を残す。
モデル入口・prompt・監査dispatchの3ファイルだけを追加した後は、runtimeの入力とbinary、
既存のscriptと監査本体が不変であることを確認し、5条件を再実行せず元の証拠を保持した。

WASIの汎用I/O中継は`hibana-wasip1-runtime`の`next_file`が担当する。アプリの進行や
資源権限を持たず、受信したWASI要求をEndpointで中継し、実際のclose呼出しとguestの借用を
返す。保留中の要求の取消・再開・fd違いを実Endpointとstrict-provenance Miriで検査した。
WASI native84件、doc7件、関連Miri3件、Clippy、no_std thumbv8mが成功した。
2つのignored export testはLean照合用であり、このnative件数には算入していない。

4Eを14課題目として移行済みへ算入し、残る34課題を進める。この6記録は非採点の機構・予備試験で、
全入力、一般的なRust refinement、比較優位、最終core／全consumerの完了を証明しない。
EPFの実行照合と実効果の照合も別の義務として保持する。既存3Eの27条件と実AIの結果は
保存したsource／binaryに対する認定であり、汎用WASI I/O中継へ移行した現版の全再認定にはしない。

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


## 共通simulatorの採点漏れ修正（11月8日）

4Mの正式な元ToolRegistryでも、mergeを最初、設計を最後、共有ライブラリ変更後の
再試験なしとした12操作を「完了・違反0」と認定する記録を保存した。元の判定は
公開された成果物の内容と、その内容に対する検証結果を確かめていなかった。
逆順であること自体を不正の根拠とせず、比較実装の意図についても推定しない。
操作順を直接採点する中間実装は撤回・削除し、実験には使用しない。
当初の`benchmark/`の`common-environment-v1`では、4Mに実際のfixture
ソースコード、公開内容、設計文書、内容hashに結び付いたreview／test結果を持つ。
呼出順を強制せず、同じ有効な公開結果には同じ完了判定を返す。設計を最後にする方法、
merge後のreview／test、やり直し、未公開の追加実験も許していた。このうち公開後の検証で
先の公開を正当化できる判定は不十分であり、後述のversion-2で修正した。
物理資源の同時使用は別に記録する。以下のversion-1記録を現行sourceの認定に読み替えない。
全方式に同じ環境と結果基準を適用し、Hibanaのglobalや実装手順を採点へ持ち込まない。
この4Mは元の手順指定から明示的に改訂した共通課題であり、元課題の比較結果とは混ぜない。
元simulatorの過去の実験は旧source/hashの証拠として保持し、新版の結果へ読み替えない。
4Mのnative／EPFは64条件・896 fixture効果・20,992 capture記録が成功した。これは
制御付きnative並行性の検査であり、実Wasm・実AI・採点実験ではない。移行済み件数は14のまま。
その後、5人のWASI私有I/O choreographyを同じglobalへ合成した版でも、64条件・
896 fixture効果・22,336 capture記録のEPF照合と、実受信側の不正入力2条件が成功した。
証拠は`output/experiments/shared-refactor-native-v7/`にsource／binary／hashと保存した。
同じ実装の実Wasm5条件が、70実効果・18,850 capture記録、共通採点の完了・違反0、
EPF・実効果監査まで成功した。証拠は`output/experiments/refactor-wasi-qualification-v3/`。
通常遅延で2回目のreviewがtest中に先行完了したため、merge／reviewの実測重なりは0だった。
毎回の重なりを要求する条件を修正し、並行進行の能力はnativeの制御付き64条件で
厳密に検査する。実測の重なりを捏造せず、同じ監査器の記録変異168件の拒否も保存した。
記録変異は監査器の検査であり、実行時攻撃の阻止数へ加算しない。
5人のLuna xhighを独立したWasmに接続したpilotは61.072589917秒、660,689 inclusive tokens
（input 656,820＋output 3,869）で全員完了し、14実効果・7,191 capture記録について
共通採点の完了・違反0、EPF・実効果監査が成功した。cache／reasoningは既に内数である。
証拠は`output/experiments/model-qualification/current-core-refactor-4M-v1/`。
4Mを15課題目に算入し、残る33課題を進める。これらは非採点の機構試験・pilotであり、
旧課題の結果、手法間の比較、最終core／全family認定とは区別する。

## 11Eの共通環境の物理資源と報告（11月8日）

別の工作機械の仕事まで両機械の競合として数えていたことを、実ToolRegistryの並行呼出しで
再現した。仕事の実引数から必要な機械を決める一つのmetadataへ修正し、同じ機械の重複使用は
actorが同じでも容量へ数える。別機械の同時使用を逐次化せず、実際の競合だけを記録する。
操作の例外終了でも、そのactor／toolの実使用を解放する。

11Eの完了は、実際に完了した4仕事の結果を含むsupervisorの報告から導く。独立した進行flagや
指定順序は置かず、全24順序、早い報告と後の再提出、実行中の仕事の除外を検査した。
11Eの10検査と既存4Mの8検査、計18検査が通過。共通環境48課題×3条件の144初期化、
Rust経由の独立／競合の実呼出し、例外返却、入力toolの16検査も通過した。
証拠は`output/experiments/workshop-world-v1/`。これは共通simulatorの検査であり、
11Eのglobal／localside・実Wasm・実モデルの認定ではない。移行済みは15課題のままとする。

## 必須の因果関係を共通採点する修正版（11月8日）

「逆順も通るから自由な採点器として適切」という判断は撤回した。独立した仕事の順序は
自由にし、試料の準備から利用、該当する成果物のレビュー／テスト完了から公開という
業務上必要な因果関係を、全方式共通の採点対象とする。採点器はChoreoFixのglobalやEPFを
読み込まない。前述のversion-1の記録は履歴であり、現在の正本は`common-environment-v2`である。

4Mの公開後の検証による正当化と、14Eの未準備の細胞での試験を実ToolRegistryで再現して
修正した。公開時点で同じ内容のレビューとテストが完了していること、試料・準備・分析・
判断の実際の完了済み入力の出現を照合する。後の証拠で早い効果を正当化せず、後の再試行は
新しい効果として扱う。観測済みの不正公開や物理資源の競合は、回復後にも消さない。
共通試験には14Eの独立順序・両判断の2,304条件、入力欠落・実行中の入力・古い版、4Mの
公開時点の照合と共有依存の変更を含む。これは全てのtopological orderの全数証明ではない。

この修正版の4M・14Eは、実Wasm計8条件・109効果・32,052 capture記録が共通採点と
EPF／実効果監査まで成功し、記録変異297件を拒否した。実AIの14Eは4人・66.562709917秒・
481,388 inclusive tokens、4Mは5人・57.091362083秒・627,103 inclusive tokensで同じ監査を
通過した。各source／binary／採点器hashと証拠は`output/experiments/causal-qualification-summary.json`に
結び付ける。旧version-1の予備結果や、他課題の因果採点の認定へ読み替えない。

## 13Eの試料と実装置Endpointを使う移行（11月8日）

CHEMIST、BIOLOGIST、ANALYSTの3人とSPECTROMETER／CENTRIFUGEの実Endpointで、
8操作を一つのglobalに記述した。独立した初期処理はparで実行し、試料、実際のbioassay、
両品質結果を同じ連携で渡す。bioassay自体には装置を要求しない。最終検証の受信側は、
両品質結果が実際に受信した試料・試験・測定値と一致することを実効果の前に確かめる。
localsideのEndpoint操作をhelperへ隠さず、保持者表、進行flag、第二の状態機械を追加していない。
本体とLeanは変更していない。

共通simulatorも試料と完了済み入力の出現を持つ形に修正し、13Eでは合法な順序と任意の
culture入力の24条件、入力を各一つ除く条件、古い試料／別の試験の結果、処理中の品質結果、
独立装置の同時使用と実競合を検査した。共通試験は計40件成功した。遅延は全方式共通の
合成値[1,5]秒と記録した倍率であり、実研究室の性能測定ではない。13E追加後のmanifestは
`7e24be01f4e9ffe856e9981e5577584a4599421a548b37b9e1732f72b097ddf7`である。

本番受信側を直接使う負例3件、アプリnative44件、runner37件、両Clippyが成功した。
実Wasm3条件は、早い最終要求・重複要求・省略したcell_type・長い任意文字列の分割入力・
通常遅延を含み、24効果・6,904 capture記録が共通採点、EPF、実効果監査まで成功した。
記録変異87件の拒否は監査器の検査であり、実行時攻撃数ではない。通常遅延では独立した
装置処理の重なり2件を、追加の開始barrierなしで観測した。

同じ版で独立した3つのWasmをLuna xhigh／既存ChatGPT契約に接続したpilotは
42.346456625秒で完了した。8実効果・2,968 capture記録・違反0、独立装置の重なり1件を
観測し、3人のmodel turnが重なっていた。inclusiveは356,591 tokens（input354,673＋
output1,918）で、cacheとreasoningは内数である。共通採点・EPF・実効果監査が成功した。
証拠は`output/experiments/lab-qualification-summary.json`、固定sourceのWasm証拠と
`output/experiments/model-qualification/current-core-lab-13E-v1/`に保持する。
13Eを17課題目に算入し、残り31課題と最終全family認定を進める。採点比較は引き続き0件であり、
これらの有限試験と一つのpilotから手法間の優位や任意のRustの正しさを主張しない。

## 13MのQC・試料・実Endpoint継続（11月8日、アプリ移行中）

元sourceで、試料が存在する前のQC承認、拒否後に再測定しない再審査、新たな合成後の
旧承認の流用が通ることを再現した。最初の12試験のうち10件が失敗したsourceと観測を
`output/experiments/lab-quality-development-v1/before/`と`before.log`へ保持した。
修正前の実装を復元する追加ビルドや感度gateは作っていない。

共通環境では承認を実際の合成・精製・測定・生物試験の完了済み出現へ結び付けた。
CHEMISTの拒否後の再測定は同じ精製試料を測るので、架空の新しいbatchへ置き換えない。
BIOLOGISTの拒否後は新しい試験を要し、その試験に対応したANALYSTの品質結果も必要になる。
QCや品質測定の順序、審査対象の切替は自由にし、装置を使わない生物試験とQCの重なりを許す。
QCの実操作はFUME_HOODとSPECTROMETERの両方を占有する。

任意の合成名・culture文字列、未準備・処理中の入力、新batch、古い成功による後の失敗の
隠蔽、両品質順、4回の拒否を含む対象切替、障害条件0/1/2を含め、共通試験53件が成功した。
Rust側の共通環境接続も両審査順×障害0/1/2の6条件・84 tool試行で完了・違反0となり、
入力metadata、実応答、固定sourceとharness hashを保存した。runner通常37件、入力16件と
Clippyも成功した。aggregateは`output/experiments/lab-quality-world-summary.json`である。
13M更新後のmanifestは`f8f9f205a58f4d607d91a52b7a804c756c7c824747bc7a196654768ab98f0e54`。
既存13E等のsource-bound結果をこのmanifestへ読み替えない。

上記6条件は共通世界とデータ接続の認定であり、Endpoint、実Wasm、実AIの結果ではない。
その後、589イベント・1,191 source行の全体globalを書き、16 actorすべてのsealed imageを
EPFの静的source/image検査へ通した。実結果の所有者FUME_SERVICEがQCの結果分岐を決め、
審査中の科学操作Endpointを保持する。承認済みEndpointはQCへ返し、拒否された対象だけを
科学者へ戻す。CENT_SERVICEへ承認済みBIOLOGYを退避して古いretry権を復活させない。

本番のinitial_biologyをそのまま使い、別runtime間で実BIOLOGY Endpointを各試行ごとに
往復させ、1/2/4回の失敗後に成功する3条件（計10試行）を確認した。QCでの実recv commit後に
旧retryを再実行すると、Pendingやframe送出ではなくEndpointエラーになる。これを含むnative
3試験と、同じ3条件のstrict-provenance Miriが成功した。全体の動的EPFや実効果の認定ではない。

globalには初回の生物試験とCHEMIST審査、およびBIOLOGIST拒否後の再試験とCHEMIST審査の
parを記述した。13Mの実効果の重なりはまだ測定していない。ChoreoFixの品質測定は両承認後へ
置く手順を採り、共通世界が許すQC/品質の両順序より狭いことを実験前に明記する。

投影時のconst上限に対して、所有権検査の不変な転送位置を索引化し、同一のsource分岐条件で
確定したownerとguardの事実だけを再利用した。parの受信知識を共有する変更ではない。
coreのsource39試験（直接走査との236,196 query照合を含む）、focused Miri3件、Clippy、
no_std/thumbv6mが成功した。本体全gateは未完であり、この部分検証から完了とは報告しない。
証拠は`output/experiments/lab-quality-global-summary.json`へ分けて記録する。

分析担当の直接WASI localside、分光器と遠心機・生物試験の受信側を追加した。
本番の遠心機受信側に対して、対象外要求・貸出後の引数変更・正しい要求の3条件を検査し、
前2条件は効果0、正しい要求は受信値どおりの1効果となった。さらに実fixtureの準備・精製後に
化合物名・精製結果・細胞準備結果・校正結果を各一つ差し替える4条件で、生物試験の効果に
入る前の拒否を確認した。これら7条件はnativeとstrict-provenance Miriで成功した。
native計6試験と両Clippyが成功した。Miriは受信側2関数が73.80秒／60.74秒で成功した。
分析担当と分光器はまだ通し実行ではなくコンパイル／Clippyの段階と区別する。
証拠とsource hashは`output/experiments/lab-quality-receiver-summary.json`へ保存する。

さらに合成と二資源QCの受信側を実装した。合成3条件では、無効な化合物・貸出後の
差替えを効果0で拒否し、任意の長い化合物名をそのまま実効果へ渡した。QCの4条件では、
3つの本番受信側を一緒に実行し、偽の元測定・SPECTROMETERのみの差替え・FUME_HOODのみの
差替えをQC効果0で拒否した。正しい両要求では1回だけ検証し、両資源へ同じ実結果を返した。
最初のCHEMIST審査の有限prefixであり、QCのすべての再拒否経路の認定とは区別する。
native計8試験、fixture Clippy、アプリ全対象Clippyが成功した。本体とglobalの意味は変更していない。
追加MiriもQC4条件（863.40秒、最大578,404,352 bytes）、合成3条件（76.27秒、最大
497,975,296 bytes）が成功した。最初のQC Miriはnativeと同じ30秒の壁時計制限で停止したため、
条件・assertionとnative制限は変えず、Miriの解析時間用制限だけを300秒へ分けた。
全589イベントのQC Miriは拡張認定とし、毎pushには積み上げない。通常CIの全native条件と
既存の所有権・取消Miri境界は維持する。結果とsourceは
`output/experiments/lab-quality-qc-summary.json`へ保存する。

科学者・QC担当を含む7つの直接localsideと4つの独立したWasmの起動を実装し、
アプリ全対象の厳格Clippyと本番ChoreoFSを読む15試験（13Mの8試験＋共有I/Oの7試験）が成功した。
精製／再分析と成功した生物試験の応答は、実処理結果と実QC結果を同じ応答に含む。
生物試験自体が失敗した場合は即時に実結果を返し、次の試験を許す。
次は実結果によるQC反復を含む全体の実Wasm／動的EPF／実効果監査と4人のpilotを検査する。
成功の後続を拒否の反復へ混ぜず、承認の代わりに私有I/O終了や進行flagを用いない。
移行済みは17/48のままであり、13Mを完了数へ加えない。

共通採点の因果関係は全手法に適用する。14Eは準備済み入力、4Mは公開する内容と一致する
完了済みreview/test、11Eは報告時点で完了した仕事を検査する。4Mのreview/test/公開を
各モジュール単位で混ぜた720順序では、必要な4依存を満たす80順序すべてが成功し、残る
640順序は未検証の公開として検出された。共通53試験は1.684秒で成功した。全48課題の
因果要件を認定したという意味ではなく、EPF適合性とは別の共通世界の採点検査である。

13Mの4つの実Wasmを初めて通し実行した。最初のCHEMIST審査・障害0の1条件は、
実効果11件、共通課題完了・違反0となり、3,297観測の動的EPFと要求／効果の監査も成功した。
生物試験とQCの重なり1件を観測し、記録変異198件を拒否した。実行sourceとrawは
`output/experiments/lab-quality-wasi-development-v5/`、監査結果は
`output/experiments/lab-quality-development-v1/first-completed-wasi-audit-v2.json`へ保持する。
次のBIOLOGIST先行条件は、分光器がglobalの生物試験結果を受け取る前に貸出を試みて失敗した。
その受信と並行fragmentの使い方を修正し、同じglobalのnative15試験とアプリClippyが成功した。
実Wasmの全条件と最新sourceの認定、実AIは未完である。最初の成功を全条件へ広げて数えない。

その後のv7では、CHEMIST先行・BIOLOGIST先行の障害0の2条件が共通採点・動的EPF・実効果監査まで
成功した。障害1では、BIOLOGYをQCへ渡した後に本体の`offer()`が古い失敗通知を公開した。
typed sendは同じ操作を拒否しており、offer公開時の再入資源の保持確認が欠けていた。
共通admissionへ既存の確認を接続し、別runtimeの最小再現（失敗1/2/4回、後続の両分岐）と
本番BIOLOGY fragmentの3条件が成功した。globalやlocalsideによる回避は追加していない。
生成catalogの18番に同じ構造を加え、preview自体の合法性も既存source判定で確認する。
公開操作のLean判定にもraw再入準備とentry header検査の接続漏れがあり、これを補った。
保存済みRust記録の再照合は3variant、完走24件、操作質問5,616件、成功preview1,668件で差分0。
本体の通常全gate、最終sourceの実Wasm10条件と実AIは未完で、13Mは17/48へ算入しない。

初期修正は正当な返却待ちのpreviewも止め、通常gateの既存3件が失敗した。期待値を維持したまま、
未消費の返却を待つ非commitのpreviewと実send/recvの完了を分け、3件とも成功した。
返却が残る構造を生成catalog 19番へ追加した。現在の18/19番は、受理6variant・完走48件、
操作質問9,228件・成功preview3,402件と全scoped質問・不正記録controlが一致し、拒否1variantも確認した。
本体の通常Rust gate（Clippy・全native target・doctest・文書・no_std・必須Miri39件）と、
本番BIOLOGY fragmentの再確認も成功した。Miriの当該gateはnightly-2026-05-28であり、
先に行ったnightly-2026-10-12の個別Miriとは別に記録する。
これは他の生成割当、拡張認定、実Wasm10条件の完走を意味しない。
集約は`output/experiments/lab-quality-development-v1/core-offer-focus-summary.json`へ保存する。

v8の実Wasmも障害0の両順序は成功したが、障害1で同じ古い通知26を公開して停止した。
将来の返却を含む全体globalでは、まだ受信していない返却を待つ判定が、送信previewまで許していた。
最小再現に後続の分岐と実返却を加え、最初のpollを相手の受信前へ移すと同じ通知4で失敗した。
送信previewには現在のentry資源を要求し、受信・ローカルpreviewの返却待ちは維持する。
候補列挙も同じadmissionで判定する修正後、最小再現は失敗1/2/4回・後続の両分岐を通過し、
実際に資源を受け戻した後の内側反復の再開も成功した。本番BIOLOGY fragmentにも将来の返却を加え、
3条件が成功した。生成18番を同じ構造へ更新し、19番のローカルpreviewとの対照を維持する。
以前のRust・Lean・生成の成功は旧sourceの記録として保持する。強めた修正後の全gateと実Wasmは
再確認中であり、13Mを移行済みへ算入しない。

v9では障害0の両順序が動的・実効果監査まで成功し、障害1の実Wasmも本体の停止点を越えて
共通採点の完了・違反0・実効果14件まで進んだ。その4,239観測のEPF監査がCPUを占有したため、
試験を11分31秒時点で明示停止し、実行記録とsourceを保持した。障害1は監査合格に数えない。
EPFの候補構築で、確定済みactorや欠けた前提を後続分岐より先に検査し、既に検査したprefixの
再検査を除いた。小さいsourceの全分岐参照との300,544質問・2,186履歴の一致、Rust132件、
doctest2件、Clippy、no_std/thumbv6m、Lean6モジュール、strict-provenance Miri24件が成功した。
実記録の監査完了や性能改善率はまだ確立していない。13M試験は実Wasm10条件の記録を先に保存し、
続いて全件の動的・実効果監査を要求する。監査待ちを成功扱いせず、実行済みデータも取り直さない。
根拠は`output/experiments/lab-quality-development-v1/epf-case2-query-diagnosis/summary.json`。

v10は同じアプリbinaryを再利用し、障害0の両順序と障害1のCHEMIST先行の実行記録を保存した。
次のBIOLOGIST先行・障害1は、BIOLOGISTのQC拒否後の承認までは進んだが、残るCHEMISTの
QC選択をFUME_SERVICEが受信せず300秒で停止した。全10条件・動的監査・実AIは未完である。
2資源の実操作、並行返却、反復後の選択、後続の判定を含めた小さい実Endpoint試験は成功し、
本番の待機を再現していない。本体バグや生成検査の見逃しとまだ断定せず、localsideとの対応も調べる。
最初の縮小例はroute外の通知をofferで待っており不適切だったため、本体バグの根拠にしない。
順位付き生成カタログは1委譲資源の入力域であり、共有driverの固定複数資源試験とは区別する。
根拠は`output/experiments/lab-quality-development-v1/wasi-qualification-v10-state.json`。

### 13Mで露呈したofferの本体不具合と生成検査の補充（11月9日）

v10の上記調査を進め、正当なglobalで、peerの送信前に開始した同じofferをPendingの
まま保持すると継続を受け取れないことを実Endpointで再現した。resolver候補だけが
cursorの可否を使い、rollの入口資源の現在のcustody検査を通っていなかった。本体の
候補選択を共通の検査へ揃えた。ChoreoFixのlocalsideの操作順による回避は行っていない。

生成文法では表現できたが、従来の入力とscheduleは、この動的roll入口・資源返却・
後続の入力を待つofferの組合せを実行していなかった。Pendingのofferをdropしていた
probeとは区別し、同じFutureを保ったまま再開するlocalsideを生成する。最小の1資源の
契約カタログを追加し、別途2資源の実Endpoint回帰試験で、拒否1・2・4回と入力到着前後を
明示的に検査した。複数資源だけが欠けていたという説明にはしない。

この追加生成入力は、同じ公開メッセージ契約に一致する古いparked previewを、別の
有効なsource occurrenceへのtyped sendで再利用する本体不具合も検出した。idleな
typed sendは古い出現が有効な場合だけ再利用し、それ以外は保持済み分岐知識で通常の
候補を探す。activeなRouteBranchは引き続き正確な出現に固定する。Leanの検査側も
この二つを区別するよう修正した。任意Rust実行のrefinement証明とは主張しない。

最終の性能整理後の本体で、厳格Clippy・native全対象・doctest・文書・no_std／embedded・
必須Miri39件が成功した。処理回数は11ケース・8指標すべて既存予算内で、上限は変更して
いない。追加カタログは4契約variant×8script×2poll順序の64実行・1,792公開操作が
完走した。最終本体の記録は、修正済みLeanで照合した前の記録と全入力・全観測がbyte単位で
一致することを確認し、同じ入力の証明を再実行せず引き継いだ。whole問合せ46,144、成功済み
parked-preview問合せ14,240、scope問合せ102,528でruntime-only／model-onlyは0。
Leanのmodel gateも132 moduleと54 standalone moduleで成功している。

これは追加ケースと通常Rust部分の認定であり、通常生成検査53割当すべての完了ではない。
続く`lab-quality-wasi-development-v12`では、同じ最終本体のbinary
`416d251d84d5ff2662df6565a5e71b38a56ee3343bfa96a337ab504a1002c1c2`で、localsideを
変更せず4-Wasmの全10条件が完走した。元のBIOLOGIST先行・障害1の停止条件も通過した。
全10条件の共通因果採点と動的EPF／実効果監査が成功し、違反0、記録変異198件を拒否した。
通常遅延の両者再拒否条件では、biologyとQCの実効果が3件重なった。試験と全監査は270.08秒、
全子孫の最大memoryは1,478,393,856bytesで、保存したsourceは前後で一致する。
v11は必須の`PYTHONHASHSEED=0`の起動設定漏れで実効果前に停止したため不合格として残し、
設定を直したv12は同じsourceと再利用可能なbuild cacheで実行した。本体検査は取り直していない。
続く実AI4人のpilotも、同じ課題の共通採点・動的EPF・実効果監査まで成功した。
gpt-6-luna／xhighの4参加者は86.042776292秒、inclusive total tokensは710,689で、
実効果17件、違反0、biology／QCの重なり1件、11,241観測である。モデルと実Wasmの
実行後、最初の監査は負荷調査のため明示的に中断した（元の試験exit130）。モデルを
再実行せず、保存した全入力・実行binary・sourceのhashを確かめ、新しい監査器で独立に
再監査して成功した。元の中断試験を合格へ書き換えない。再監査は2,179.55秒、最大
716,455,936bytesであり、このCPU負荷を解消済みとはしない。

EPFだけの最適化は、後続イベントが先行条件にならないこと、再入の先行条件は厳密に前の
イベントであること、送信公開は既存の受理のみを待つことを利用する。core・アプリと実AIの
記録は変更していない。全展開を走査する独立参照との有限比較、native132件、doctest2件、
Lean6 module（一般命題3件追加）、strict-provenance unit Miri24件、Clippy・no_std・
thumbv6mが成功した。全integration Miriの再実行や任意履歴の効率性を主張しない。

13Mを18課題目へ算入し、残り30課題へ進む。本体の通常生成検査53割当の全完了と、
最終sourceの全family認定、採点比較は引き続き未完である。このpilotは採点比較へ算入しない。
source/hashと結果は`output/experiments/lab-quality-development-v1/current-offer-focus-v2.json`
および同ディレクトリの最終Rust・生成実行・モデル照合記録に保存する。

### 5E／5M／5Hの共通世界と5E移行（11月9日、未認定）

元のMedicalSimでは、公開引数`test_type`を読まず、空の検査結果でもpanel完了flagが
立っていた。患者名の曖昧一致と永続done表も削除し、実際に完了した患者結果と、現在の
入力の出現から完了を導く。診断は同じ患者の実検査結果を必要とし、設定された画像解釈は
実画像の完了を必要とする。別患者やPendingの結果、差替え前の結果で完了にはならない。

専門panelを一般panelの完了まで待たせる条件は、課題説明と公開引数から導けないため
入れない。両panelとCTを実際に並行実行でき、一般panelの再検査は独立した心臓検査を
無効化しない。薬剤レビューは任意のcaller文字列を保持し、存在しない「診断→処方」の
順序を採点へ足さない。5Eの有効72順序、5Mの実画像・conference、5Hの患者分離と
手術準備を含むmedical17試験、共通世界全70試験が1.518秒で成功した。

共通契約は`common-environment-v3`へ進めた。新定数を含むRust input toolの16試験が
成功し、新buildが生成したmanifestと選択中の内容の完全一致も確認した。既存pilotの
元sourceやv2のmanifestはその版の証拠として保持し、v3へ読み替えない。根拠は
`output/experiments/medical-consultation-development-v1/qualification.json`。

5Eのglobalと4つの直接受信側はcompile・native実行を通過した。全体4条件はpanelの
一括・分割・逆順・反復を実Endpointで完走し、CT受信側3prefixは不正modalityと
委譲後の患者差替えを実効果前に拒否する。200回の実receipt更新と不正更新のatomicityも
成功した。3試験関数の記録は`endpoint-v8/`で、Miri・実Wasm・実AIの成功には含めない。
panelごとのEndpointを実要求時に貸出・返却し、実結果で完了したらretry所有者から
移す。保持するのはtest名ごとの最新実receiptであり、反復履歴や進行flagではない。
3人の直接guest localsideと起動を実装し、実Wasm9条件と動的EPF／効果監査がすべて
成功した。ERのCTは一般panelの前・途中・後に要求でき、紹介は並行に進む。
実効果数は8/8/8/8/8/8/6/6/12件、全条件の違反は0。通常遅延の先頭条件には
panel／CTの重なり2組があり、監査記録の欠落・改変・重複81件も拒否した。
試験と監査は168.99秒、buildを含む全子孫最大memoryは1,984,446,464 bytes。
本番全対象の厳格Clippyと3関数のstrict-provenance Miriも成功した。Miriは全体4条件・
CT受信prefix3条件・実receipt200更新を含み、467.89秒、最大1,454,555,136 bytesである。

同じbinaryで、Luna xhigh3人が独立した実Wasm shellを使うpilotも成功した。元課題完了、
全guest exit 0、実効果6件、違反0、動的EPF／実効果監査成功。実行は31.452171791秒、
257,121 inclusive tokens、capture 3,818件で、モデルturnの同時実行は3人。
実効果の重なりは1組であり、scripted条件のpanel／CT重なりを実AIの結果へ読み替えない。
採点比較には算入しない。scripted sourceは`medical-consultation-wasi-v1/`、実AIは
`model-qualification/current-core-medical-5E-v1/subscription-5E-1794175974163030000/`へ保持した。
共通binary SHA-256は`8f062fb1e9b91e236de11a247cb7d5e9247683543d1e61b2a45c03d26db78fc1`。
元のdomain toolsには処方・投薬・紹介・所見配送の操作がなく、その効果や通信要件を
共通完了bitだけで認定しない。詳細は`benchmark/ports/5E.md`へ記録した。
5Eを19課題目へ算入し、残る29課題へ進む。本体の通常生成検査は固定sourceで
constructors-0/1/10/11が合格し、残る49割当は未完。このアプリ開発のために本体を変更していない。

### 5M移行と実AI5人の認定（11月9日）

5Mを20課題目へ算入し、残る28課題へ進む。各localsideへEndpointの貸出・実要求・
結果・返却を直接記述し、CT/MRI各2操作の6順序と、panelの一括・分割・逆順・反復を
許す。診断は実際の所見に結び付け、conferenceは受理した引数と実room要求が一致して
から実効果を呼ぶ。本体sourceは変更していない。

native4関数では全体6条件と実receiverの差替え7条件を検査し、不正時はDomainの呼出し
自体が0件であることを確認した。strict-provenance Miriは実receiver3関数／7条件で成功。
appとrunnerの厳格Clippyも成功した。実Wasm8条件は共通因果採点・動的EPF・実効果監査が
すべて成功し、実効果は15/15/15/15/15/15/12/18件、違反0。通常遅延の先頭条件では
独立した実効果の重なり16組を観測し、その記録の変異234件を監査器が拒否した。

同じbinaryのLuna xhigh5人pilotは48.365349708秒、571,513 inclusive tokens、
実効果12件、違反0、13,072観測、独立実効果の重なり5組で元課題を完了した。
初回の試験は実行完了後、監査器のtask-ID allowlistに5Mがない不備でexit101となった。
全担当toolの物理resource metadataから検査対象を導く形へ直し、欠落・余分・重複を
拒否する39 native試験とClippyに合格した。元raw・binary・sourceのhashを確認し、
モデルを再実行せず、同じ記録の動的EPF／効果監査を成功させた。初回試験の失敗は残す。
再監査は839.39秒、全子孫最大323,436,544bytesであり、model/runtime時間と区別する。

source-bound記録と別の監査器provenanceは
`output/experiments/model-qualification/current-core-multi-specialty-5M-v1/subscription-5M-1794183661020374000/`
へ保存した。scripted条件は`output/experiments/multi-specialty-wasi-v2/`、集計と停止証拠は
`output/experiments/multi-specialty-development-v1/qualification.json`へ保持する。
採点比較へは算入しない。詳細は`benchmark/ports/5M.md`。

次の5Hはglobalの実装へ着手した。患者ごとの診断と手術／ICU経路、共有CTの6順序、
麻酔から手術への同じOR Endpointの移動を記述した。library checkは9.95秒、最大
1,334,411,264bytesで成功。実Endpointのnativeは画像処理90順序と、A手術までBを
開始しない条件・B ICUまでAを開始しない条件の計92条件を完走し、各16実効果を確認した。
nativeは1.20秒、compileを含む最大534,560,768bytes。最初の試験側のpar幅と未実行の
private I/O終了を正しい投影へ合わせた。本番globalと本体を回避のために変えていない。

5Hの説明／checklistのICUベッドは2床だが、共通metadataはcapacity省略により1床と
なっていた。2件目の実利用を誤って競合とする反例を保存し、metadataの容量を2にした。
共通effect／scorerのコードは不変。世界の90画像順序、2床と3件目の競合、実ICU呼出し
3件の同時実行を含む73試験が1.546秒、input tool16試験が成功した。換気装置は1台のまま
2件目と3件目を競合とし、世界は呼出し自体を直列化しない。共通契約をv4に進め、manifestの
差分が5H metadataだけであることを確認した。過去の5E／5M等のrawやv3認定は書き換えない。

5Hは7つの直接Wasm localside、起動・runner・実効果監査まで実装した。受信側のnativeは
6関数で92完走条件と要求差替え12条件が成功し、差替え時はDomain呼出し自体が0件。
その5関数／12条件のstrict-provenance Miriは79.31秒、最大606,453,760bytesで成功した。
全92条件をMiriへ算入しない。app／runnerの厳格Clippyとrunner通常40試験も成功。
実Wasm8条件の試験を通常のfamily認定入口へ接続し、CT6順序・MRIの配置・分割と一括panel・
不正入力後の正常要求を検査する。動的EPF／実効果監査と実モデルは未認定なので20/48を維持。
本体通常生成はcontracts-2まで計7/53が成功し、残る46割当と最終全family認定も未完。

### 5Hの実Wasm・7人の実モデル認定（11月9日）

5Hを21課題目へ算入し、残る27課題へ進む。固定sourceで実Wasm8条件が共通採点・
動的EPF／実効果監査まで成功し、実効果は19/19/19/19/19/19/16/21件、全条件違反0。
通常遅延の先頭条件では独立実効果の重なり18組、記録変異219件の拒否を確認した。
後者は監査器の記録変異であり、実行時攻撃数ではない。試験と全監査は626.24秒、
compile込みの全子孫最大は2,459,451,392bytes。sourceは前後で一致する。

同じsource／binaryのLuna xhigh7人pilotは48.162432792秒、679,177 inclusive tokens、
実効果16件、違反0、独立実効果の重なり7組、同時model turn最大7で元課題を完了した。
10,073観測が動的EPF／実効果監査に成功した。実行と監査を含む試験は276.00秒、
最大1,343,782,912bytes。モデル時間と監査CPU時間を混同しない。採点比較へ算入しない。

本体・EPF・WASIの変更やlocalsideの順序による回避は追加していない。実受信側の
差替え拒否を5E／5M／5HのMiri CI入口へ接続した。scripted記録は
`output/experiments/critical-care-wasi-v1/`、実モデルは
`output/experiments/model-qualification/current-core-critical-care-5H-v1/subscription-5H-1794191523709141000/`、
集約は`output/experiments/critical-care-development-v1/qualification.json`へ保持する。
このsource-bound認定を最終版全family・全入力の保証へ読み替えない。

### 次の6Mに向けた共通世界の因果検査（11月9日）

6Mの元採点器は、レビュー／test／CIを先に呼び、最後に実装しただけでmergeなしでも
完了した。merge後の承認も完了としていた。2つの実反例を通常の世界toolで再現し、
元v4のsource・入力・失敗logを保存した。本体やChoreoFixのglobalの欠陥ではない。

共通v5は6E／6M／6Hの完了flagを削除し、実際のsource文字列、test定義、公開物と
配置物、完了済みの内容一致receiptから結果を判定する。未承認mergeや未検証deploymentを
実行禁止するのではなく、実際の効果と違反を保存する。後の検証では過去の効果を正当化しない。
reviewとtest準備は独立順序を保ち、CIは実際の公開snapshotとtest定義を入力にする。
6Hの依存は認証→決済→通知という実データのbindingへ明記し、rollback後の旧配置receiptを
新しい配置へ使わない。この抽象tool世界を実compilerや任意のcodeの正しさとは主張しない。

直接世界9試験には720のtest／review／merge順序（有効80）、遅延・Pendingの検証、
source差替え、CIと独立実装の重なり、6Eの両順序と6Hのrollback／再配置を含む。
共通全82試験は1.748秒、最大33,210,368bytes。Rust入力16試験も成功した。
元の48課題・825 checklistの件数は不変。optional sourceは共通tool入力へ追加し、
実payloadがその値を保持する1試験が成功した（同じfixtureの既存I/O7試験も成功）。
app／runnerのClippyとrunner40試験も成功。旧sourceへ欠陥を復元する検査は追加していない。

最初の入力gateは生成したmanifestを標準出力へ出すだけでファイルを更新しなかったため
旧v4との不一致で3試験が失敗した。Rust生成器の出力を正本manifestへ保存し、16試験を
成功させた。失敗記録は保持する。6Mの全体global／WASI localsideとv5での実Wasm・実モデルは未完。
移行済み21/48と、各旧pilotの元source／world版は維持する。

本体の通常生成は同じ固定sourceでcontracts-3～12も成功し、現在17/53、残り36割当。
それぞれの実入力・操作記録・source照合と拒否controlを保存した。既存の成功割当は再実行せず、
本体sourceを編集しないまま次の未実行割当へ進む。これを通常全gate完了とは報告しない。

## 6Mの実装・テスト・レビューの実データ経路（11月9日、前半のみ）

`src/tasks/feature_delivery/`に候補作成・テスト定義・レビューのglobalと、実際のrepository／
test-data／review-data受信側を実装した。REPOは各開発者へ実Endpointを貸し、受理済みの
完全な引数と、そのEndpointからの要求の一致を効果直前に検査する。同じ候補結果を
レビューとテストの所有者へ送り、テスト担当にも実候補を渡す。TEST_WRITERはTEST_ENVを
一度受け取り、同じ継続を両featureへ使って最後に一度返す。操作ごとの貸出役、admission packet、
別の所有者表や進行flagは置かない。これはまだ全体の6M choreographyではない。

通常Endpointの前半native試験はレビュー順×テスト順の全4通りで完走し、各6件のfixture効果を
検査した。最初のレビューが他方の実装完了を待たずに始まることも全4通りで要求する。
REPOのsource／対象の差替え2件、TEST_ENVの対象差替え1件は対応する効果前に拒否された。
同じ2つの否定的受信試験（計3ケース）はstrict-provenance Miriでも成功し、40.69秒、
全子孫最大432,373,760bytesだった。共通世界と同じ`tool_name/success/data`の結果形式を使うが、
nativeのデータfixtureであり、共通simulatorの実行やWasm・実AIの完了とは数えない。

最初のglobalはレビュー順の選択を開発者へ伝えておらず、本体の投影検査で拒否された。
選択した実レビュー要求をglobalへ明示して修正した。否定試験の最初のcallerは要求後すぐに
資源Endpointをdropし、受信側の照合より先にsessionを中断していた。callerを実応答待ちまで
保持し、単なるsession faultでなく期待した引数照合で拒否されることを確認した。
本体の修正やlocalsideによる不整合の回避、旧欠陥の復元は行っていない。

payloadの1～3番の既存契約を保ってrepository codecを共有のshellデータ層へ移し、
4番のtest targetと5番のCI suiteを追加した。旧aliasや二つ目のcodecは残さない。
要求型の取り違えと余分なfieldの拒否を実試験で検査する。
通常CIへ3つの前半native試験と、2つの否定的Miri試験を接続した。
最終sourceでfmt、アプリ全対象の厳格Clippy、通常CIと同じ3つのnative試験と
2つのcodec試験が成功した。全子孫最大1,731,018,752bytes。Miriと通常Rustのsource hashも
最終状態と一致した。解決済みの開発ログ・型診断と不要なapp buildを削除した。
次は承認／再修正・merge・CI失敗後の修正／再テストをglobalへ接続し、5-Wasmと実AIへ進む。
移行済みは21/48、残り27のまま。sourceと検証記録は
`output/experiments/feature-delivery-development-v1/`へ保存する。

## 6MのCI実受信側と共通世界v6（11月9日、全体globalの接続は未完）

`feature_delivery`へBUILD_SERVERの実Endpointを使うCI要求／結果のsource断片と受信側を追加した。
WASI側の受理済みsuiteと資源Endpointからの要求を照合してからbackendへ進み、成功結果は
公開済み候補集合・repository hash・実テストの出現IDと定義全体・公開より後のCI出現へ結ぶ。
通常のCI失敗は失敗値のまま返信する。入力を確認したというだけの返信と未使用のsnapshot copyは削除した。
テスト結果はTEST_ENVから実際に受信したTEST_WRITERが同じ値をCIへ転送し、別の転送経路を置かない。

nativeは両feature×11データ条件の22件を含む4試験が成功した。CI成功2・通常失敗2、
backend前の拒否6・読取りのCI backend呼出し後の不整合結果の拒否12である。
同一テストIDの内容差替え、同じ版に対する古いCIの再利用も含む。元の前半4順序・要求差替え3件も通過した。
Miriは両feature×4つの所有権経路（成功・通常失敗・効果前／後の拒否）の8件を全数確認し、
199.43秒、全子孫最大454,836,224bytesだった。JSON fieldの変異条件はnativeの22件へ残す。
未変更の元受信側2試験・3ケースのMiriは、入力とsource対応を記録して41.40秒の成功を引き継ぐ。
最終Rustのfmt・Clippy・native4試験も成功し、guard最大1,708,457,984bytes。

共通世界v6は全手法へ同じ公開候補集合とCIが実際に使った不変のtest inputを返す。
採点器には、同一内容の再mergeを以前のCIで完了と認める反例が実際にあった。
その失敗sourceとログを保持し、CI開始が対象publicationの完了より後であることを照合するよう修正した。
元の修復試験2件も再merge後に新たなCIが必要だと検査し、既存の拒否期待は維持した。
操作の許可表やworkflow gateは追加せず、不正な効果の記録と違反は引き続き共通世界へ残す。
共通84試験（720順序・有効80は不変）、入力16試験、runner通常40試験、app／runner Clippyが成功した。
runnerの外部統合79試験はこの通常コマンドでは未実行であり、成功へ算入しない。
現manifestはv6・48課題、runnerも再build済み。旧pilotをv6の結果へ読み替えない。

現在は部分globalと受信側までであり、6Mの完了数は増やさない。次の設計では一つのfeature Endpointを
developer→reviewer→developer→CIへ委譲し、その継続で実際の操作を行う。失敗時は同じEndpointを
修正担当へ返し、最終CI成功時は編集可能な継続から移す。物理REPO／TEST_ENV／BUILD_SERVERの
操作権は併せて維持する。この全体handoffは未実装であり、古いA操作がB修復中にも再開できないことと、
独立した実装／テスト／レビューの並行性を実Endpointで確認してからWasmへ進める。
移行21/48、残り27、実モデル追加0、採点済み実験追加0。

## 6Mの全体sourceと実受信側への移行（11月9日、全体実行は未認定）

`feature_delivery/global/delivery.rs`に、同じfeature Endpointが実装・レビュー・修正・merge・
CIを進む716イベントのglobalを実装した。初回の承認後は編集権をRepositoryへ戻し、
mergeの選択は両feature Endpointを保持する同じRepositoryが行う。CI失敗では同じEndpointを
修正担当へ戻し、最終成功ではRepositoryへ移してから肯定的な結果を配る。実際のREPO・
TEST_ENV・BUILD_SERVERの権限も別途維持する。mergeの先頭を異なる開発者の送信にする
作業案は選択者不一致として投影検査が拒否した。本体の検査を変更せず、このglobalの誤りを
直した。旧preparation専用sourceと旧CI実行経路は削除した。

Repository、レビュー2受信側、テスト2受信側、CI受信側をこのsourceへ接続した。実装・merge・
CIはfeature側と実資源側の二つの実要求を並行に受信し、受理済みの完全な引数と照合する。
レビューサービスとCIサービスの元の結果もRepositoryへ直接届け、クライアントが配る結果と
照合する。保持するのは実データと実際に受信した所有値のEndpointであり、保持者表・段階flag・
第二の進行状態は追加していない。送受信と並行処理は各localside本文へ直接書いている。

同じsourceで、`long_running_const_eval = forbid`の全16 requested view（11 physical actor）を
投影し、全行と件数を読んで検査した。nativeの実Endpointでは、レビュー順×テスト順の4条件、
効果前の要求差替え拒否4件、CIの既存22条件が成功した。前半4条件は両test作成とレビュー結果を
実際に受け取ってから意図的に打ち切るprefix試験であり、全体のprotocol完了ではない。
mergeの純粋なデータ照合は36条件（正例6、拒否30）が成功した。通常CIにもこのsource検査、
データ照合、実Endpoint試験を明示的に接続した。最終sourceの整形と全対象strict Clippyも成功し、
一連の検査の全子孫最大memoryは1,766,375,424bytesだった。
証拠とsource hashは`output/experiments/feature-delivery-development-v1/full-receiver-final-native-v2/`。
旧partial-sourceのMiriをこの新受信側の成功へ読み替えず、差替え4条件とCIの所有権8条件は
`full-receiver-miri-v1/`で別途成功した。前者480.55秒、後者363.43秒、全子孫の最大memoryは
578,715,648bytes。検査前後のsource hashは一致した。これは明示した12条件の借用・所有権・
取消境界の確認であり、全体のレビュー／CI再試行や全protocolの検証ではない。

このglobalのコンパイル時に、委譲返却の検査で同じ部分木・転送列を何度もたどる負荷を減らした。
key別の不変な転送索引、直列部分木の平坦な巡回、短い確定済み因果の証拠を使い、不明な条件は
従来の完全な検査へ残す。最終のreentry 11試験は参照計算との比較を含めて成功し、Leanに固定actorの
解決が選択から独立なことと誤った固定actorを返却と数えないことの2一般命題を追加した。
132 moduleのbuildと1,300命題の型・axiom inventoryが成功した（両axiom786、propextのみ378、
axiomなし136）。本体のランタイム・公開APIはこのコンパイラ最適化で変更していない。
証拠は`reentry-and-whole-source-final-v1/`と`reentry-lean-final-v1/`。
通常生成は18/53の既存結果を保持し、新しいコンパイラsourceの全体認定は未完と区別する。

残りは、初回とCI回復を通す全体実Endpoint、旧編集・レビューの再開拒否、5人の直接WASI localside、
元課題D1のinterface変更通知の接続と採点器の表現範囲の確認、実Wasm・動的EPF／実効果監査・
実モデルの認定である。全体sourceのコンパイルや部分試験を課題移行完了へ数えない。
移行21/48、残り27、今回追加のモデル呼出し0、採点済み比較0。

## 6Mの全体native実行と共通世界v7（11月9日）

通知と最終CI成功を含む768イベント・16 viewの同一globalで、初回レビュー順×初回test順×
5つの失敗入力の20条件が完走した。初回レビュー最大3拒否、回復レビュー最大3拒否、CI最大4失敗を
含む。制御されたfixture効果は計692件、完全captureは29,984件で、最後のREPO・TEST_ENV・
BUILD_SERVER・両featureの返却まで実Endpointで通した。別途18条件で要求／結果の差替えを拒否し、
6条件はCI呼出し前、12条件は1回のCI呼出し後の拒否である。mergeの純粋データ照合54条件、
前半4順序・差替え4条件、forbidを保つ全16 viewのsource検査も成功した。全子孫最大556,761,088bytes。

途中の2回以上のレビュー拒否で、CIとRepositoryが、拒否後の確定した受信まで汎用offerループへ
戻していたlocalsideの誤りを検出した。globalで確定する要求・結果は直接recvし、選択と反復を
localside本文へ書き直した。本体やglobalはこの修正では変えていない。旧CI単独fixtureと
activation通知は削除し、nativeの拒否条件もMiriの8つの所有権経路も全体globalへ統合した。
旧sourceのMiri成功を新しい受信側へ読み替えず、現sourceのMiriは別途実施する。

共通世界v6には、失敗したCIのbaseに成功CIを要求して正当な修復mergeを常に拒否する不具合があった。
再現sourceと失敗を保持し、v7では現在のpublication IDとtest入力に結び付く完了済みの否定的CI、
新しい実装とその実レビューを照合する。CI中の修正準備・レビューの並行実行も許す。
不正効果の実行と違反記録は残し、5種類の古い／不完全な証拠を使う修復は違反となる。
共通87試験が成功し、720順序・有効80という既存の自由度も維持した。manifest・実験契約はv7へ揃えた。

証拠は`output/experiments/feature-delivery-development-v1/whole-native-v6/`と
`ci-repair-world-v2/`。これはnativeの制御結果による検証であり、実Wasm・実AI・採点比較ではない。
移行は21/48、残り27のまま。次は現sourceの所有権境界のMiri、5人の直接WASI localsideと
実Wasm・共通世界・EPF・実効果監査を接続する。本体通常生成53割当の全完了は別の未完事項である。

### 6MのMiri範囲の訂正

全体sourceのMiriは、監査JSONを保存しない場合にも全投影をJSON化していた作業を除去した後も、
最初の完走条件が試験用120秒の時間制限で失敗した（全体378.41秒、最大660,144,128bytes）。
これは合格へ変えず、`whole-miri-v2/guard.log`へ保持した。
通常CIのMiriは、全体のworkflow適合を重複して調べる代わりに、本番CI受信関数とcanonicalな
CI source断片を使う8つの所有権経路へ分離した。成功、通常失敗後に実Endpointが返却されてからの
取消、CI前の拒否、CI後の拒否を両資源で検査する。fixtureの外側の資源選択は条件付きpeerへ
明示し、別のアプリ実装やruntime modeは追加しない。この8条件のnativeは6.26秒のbuild後、
0.02秒で成功した。strict-provenance Miriは別途実行中であり、native成功から合格としない。
全体38条件のnative結果、20完走記録とEPF監査の残りは維持する。

WASI側は開発者A/B、test writer、CI runnerの直接localside本文を作成中。
まだmoduleへ接続・型検査・実Wasm実行をしておらず、移行数へは算入しない。

CIの8経路はstrict-provenance Miriでも全数成功した（365.23秒、最大560,594,944bytes）。
source前後hashは一致し、`ci-ownership-miri-v1/qualification.json`へ保存した。
全体sourceのMiriが成功したという意味ではなく、Repository側の差替え4条件も別に再検査する。
WASI本文4人は、ゲストとprivate Endpointの所有値をasync childへ渡し、Pending importと
Endpointを完了境界からそのまま受け取る形で型検査に成功した。pin・task handle・権限表は追加していない。
これはcargo checkであり、Wasm起動・choreographyの新しいI/O構成・const評価の認定ではない。
旧native-v5の20完走rawとprojectionは、v6の対応する全実行入力・観測・効果が同一であることを
確認して削除した。原本hashと対応は`whole-native-v5/superseded-raw.tsv`へ保持している。

Repositoryの引数差替え4条件もstrict-provenance Miriで成功した（2試験、478.73秒、最大
600,702,976bytes、source前後hash一致）。本番CI受信側の8経路と合わせ、今回書き直した
受信側の宣言した所有権境界は確認した。いずれもWASI I/Oを重ねた新しい全体sourceのMiriではない。
5人目のreviewerも含む全WASI本文と起動・ChoreoFS I/O ownerの接続はcargo checkに成功した。
各ファイル要求の元の結果を保持し、実際のレビュー／CI結果をcoordination情報として返す。
初期レビューの順序はモデルから受理した最初のpr_idで選び、修復は実際の否定的結果から進む。
caller-ownedな5つのWasm storageと5つのprivate I/Oを同じruntimeで宣言し、終了通知はI/O ownerが
一度だけ扱う。pin、task表、完了flag、別のworkflow managerは追加していない。
5つのWASI choreographyを重ねたsourceの実buildは、`REENTRY_KNOWLEDGE`内の
`Source::uses`の再帰走査でconst評価上限に達した。これはlocalsideの型エラーや今回のoffer停止とは
別のコンパイル負荷である。元のglobalと`forbid`を保ち、不在roleを構築時の不変summaryで除外する
本体を改善した。roleが存在する場合のcomponent照合は従来どおり行う。全256 roleの照合、
小さい木の49,152条件の照合、関連する再入11試験とLeanの2命題が成功した。元のglobalと
`forbid`のまま実buildも497.27秒・全子孫最大2,432,240,280bytesで成功し、実binaryから
21 viewの投影を出力した。証拠は`wasi-build-v2/`。その後のClippy向けの同値なif統合は
別途記録し、binaryの認定は保存した実source/hashへ固定する。実Wasm、実モデル、動的・実効果監査と
本体通常生成53割当の全完了は未認定である。
移行21/48・残り27を維持する。

### 6Mの共通runnerと最初の5-Wasm実行（11月9日）

通常CLI、prompt、元課題の5人のfixture、snapshot、投影と動的EPF監査、実効果監査、
共通WASI認定入口へ6Mを接続した。通常runner40試験と、runner全対象・6Mアプリの
厳格Clippyが成功した。通常成功時のレビュー順×test順の4条件を宣言している。

新しい固定sourceの実buildは8分15秒で成功した。最初のレビューA先行・test A先行の
5-Wasm条件は実効果10件、protocolと共通因果採点の完了、違反0で実行を終えた。
最初の監査には、Mergedの受取先とCIの公開出現一覧に誤った期待があり、監査が拒否した。
globalと実sourceを確認して監査だけを修正し、同じ保存済み記録の再監査で動的EPF・実効果照合が
成功した。完全captureは7,846件、実効果に対応する42 frameの欠落・両側payload改変・重複の
126件を拒否した。これらは記録監査の変異であり、実行時攻撃の126件ではない。
実行側のsource778ファイルのhashは一致し、Wasmやモデルの再実行はしていない。

実行記録は`wasi-qualification-v1/`、修正監査とsource対応は`wasi-reaudit-v2/`へ保存した。
最初の認定コマンド全体は監査エラーで失敗した記録を保持し、全4条件の成功へ読み替えない。
残り3つの通常順序、レビュー拒否とCI失敗からのWasm再考、実モデルと最終認定は未完。
移行数は21/48のまま。比較実験・モデル呼出しは今回0である。

修正済み監査を含む同一sourceの`wasi-qualification-v2/`では、通常成功時のレビュー順×test順の
4条件すべてが、実5-Wasm・protocol完了・共通採点・動的EPF・実効果監査まで成功した。
実効果は計40件、完全captureは31,326件、違反0、最初の条件の記録変異126件を拒否した。
全sourceの前後hashは一致し、試験全体351.91秒・全子孫最大2,072,643,504bytes。
アプリとwashのビルドは内容不変のため0.02秒・0.00秒で再利用し、本体の無関係な検査は再実行していない。
集約は`wasi-initial-orders-v1.json`。各caseの完全なauditとrawを原本とし、進捗・結果の集約に
重複していた大きい実行順の配列は参照へまとめた。モデル呼出し0・採点比較0、移行21/48を維持する。
次は、実際のレビュー拒否とCI失敗の応答から5つのWasmが再考する試験と、実モデルの認定へ進む。

### 6Mの実Wasm再考とEPF照合の整理（11月9日）

レビュー初回拒否・CI失敗・回復レビュー再拒否を含む最初の5-Wasm条件は、実行と共通採点を
完了した。実効果21件、否定的レビュー2件、CI失敗1件、違反0である。保存済みの完全記録を
使うEPF監査が長時間かかったため、実行を取り直さず、その記録で監査器の探索を調べた。
本体のランタイム、global、アプリのlocalsideはこの整理で変更していない。

EPFの問い合わせは、観測済みのLocal/Recvを参加者別・source順の索引へまとめ、無関係な
参加者や後ろの行との比較を除く。順序は既存の不変なbinary索引から読み、元のsource探索と
全組を比較する。既に検査した不変ヘッダを各行で再検査していた処理も、共通のデコードへ
集約した。未検証入力の入口検査は維持し、実行権限・所有者表・別の進行状態は追加していない。
独立した全探索との比較には逆順の合法履歴も加えた。133実試験、2つのcompile-fail doctest、
厳格Clippy、thumbv6mチェックが成功し、actor/eventで履歴を絞っても先行操作の証拠を
失わないLean補題を含む6 moduleの検査も成功した。

同じ保存記録の監査は417.92秒で成功し、21実効果・14,022 captureの動的EPFと実効果照合が
完了した。記録変異249件を拒否し、関連するstrict-provenance Miriはquery3件と
Endpoint履歴11件の計14件が成功した。runner通常40試験とClippyも成功している。
これらは最初の再考条件の認定であり、2番目の条件・実AI・全体認定は未完である。
生産側のRust/Cargo入力785ファイルは保存済み実行時から一致している。全実験workspaceの
manifestには変更したEPFやrunnerも含まれるため、その全体の同一性とは区別している。
証拠は`output/experiments/feature-delivery-development-v1/epf-query-history-bounds-v1/`。
移行21/48、残り27、今回のモデル呼出しと採点比較は0である。

合格済みの最初の条件を取り直さず、宣言済みの2番目の入力だけを選ぶ診断入口を追加した。
通常は引き続き2条件すべてを実行し、診断artifactには選んだindexと`full_matrix: false`を
保存する。空や範囲外の入力は成功させず、全family認定ではこの入力選択を拒否する。
通常認定へ選択を混ぜる実CLI呼出しが終了code1で拒否されることも確認した。

### 6Mの2番目の再考条件と保存データの整理（11月10日）

2番目の5-Wasm条件は、実効果32件、否定的レビュー4件、CI失敗2件、完全capture20,168件で
protocolと共通採点を完了し、違反0である。保存記録の監査が遅いため、Wasmやモデルは取り直さず、
EPFの問い合わせ探索を整理している。未完の監査試行は明示停止・未認定として残した。
各prefixのmembership索引と、全選択で必須となる固定actorの先行操作による拒否を加えた。
未選択の枝でも、応答の直前の要求が必須かを同じ不変source索引で確認する。
本体runtime、global、localsideはこの監査整理では変更していない。

現在のEPFは135実試験と2つのcompile-fail doctest、厳格Clippy、thumbv6mチェックが成功した。
独立した全選択との必須操作の存在・source順序の比較を含む。Leanでは、全互換選択で必須の
未実行操作があれば問い合わせを受理できない補題を追加した。これはRustの索引全体の一般的な
refinement証明とは区別する。strict-provenance Miriは現在のquery4件・Endpoint履歴11件と、
内容不変のprefix条件1件が成功している。通常runner40試験とClippyも成功した。
現sourceでの2番目の実記録監査は実行中であり、6Mの全体認定・実モデル認定は未完である。
証拠は`epf-mandatory-predecessor-v1/`のsource hash・個別ログ・監査出力を参照する。

実モデル実験へ進むためのbuild receiptは、`--build-wasi NEW_OUTPUT FAMILY`で通常認定と
同じbuild・source保存処理から作る。結果は`built_not_qualified`で、試験実行済みとはしない。
証跡を更新するためだけの空の試験や、合格済みWasm条件の再実行は行わない。
実buildとreceiptを使う経路そのものの確認は、現在の監査終了後に実行する。

旧Python集計の実行経路をRustへ置換し、保存済み実モデル13記録・11課題をJSON/CSV/Markdownへ
集約した。9記録は宣言したbinary・source・raw・usageの整合検査に成功し、4記録は古い監査や
simulator原本等の不足を明記した。後者を現sourceの認定へ読み替えない。異なるworld版や条件を
一つの統計群にせず、inclusive tokensはinputとoutputの和とする。集計ツール自体の実source、
manifest、lock、binary hashも保存した。`pilot-inventory-rust-v1/verified-v1/`が原本である。
移行21/48・残り27、今回追加のモデル呼出し・採点比較0を維持する。

6Mの保存記録の監査はEPFの固定sourceで続行している。独立した次課題7Eについては、
元DocumentSimの書込み・review・combineの完了bitが、当時の実材料と結び付いていないことを
source上で確認した。`benchmark/tests/test_document_causality.py`へ、存在前のreview、
承認前のcombine、許可された他章の修正による古い承認の失効、6操作の720順序を追加した。
初期準備4操作が両reviewより前に完了する48順序は、書き手やreference更新の固定順を課さずに
成功させる契約である。7Eの世界は、独立した完了bitを持つ旧DocumentSimから、実際の材料・
完了済みreview・結合時のreceiptをeffect journalから導く実装へ置き換えた。途中のreviewや
書換え、負のreviewからの再考、返却データによるjournalの書換えも加え、試験は8件となった。
この実装はguard下の8試験が0.173秒・最大30,605,312bytesで成功し、source前後hashも一致した。
720順序のうち48順序が成功することを確認した。その後の共通世界全95試験も成功し、Rust入力toolで
world契約v8のmanifestを生成した。過去の6M等の記録はv7のまま保持する。global、localside、Wasm・
モデルの認定は未実装である。詳細は`benchmark/ports/7E.md`。この独立した準備で、
実行中のEPF・6M監査器sourceや保存済みの6M実行記録は変更していない。

### 6Mの実AI5人の実行と監査の探索順（11月10日）

共通runnerで、実際の5-WasmとChatGPT認証のgpt-6-luna／xhighの5参加者を動かした。
全員が要求したmodel／effortと同じ値で正常終了した。実行時間281.621299166秒、
inclusive total tokensは1,597,412（input＋output、reasoningの二重加算なし）。
実効果26件、レビュー拒否2件、CI失敗2件を経て、実行側はprotocol完了・共通課題完了・
違反0を報告し、37,464観測を保存した。worldはv8で、以前のscripted条件は実際のv7原本を保つ。
原本は`model-qualification/current-core-feature-delivery-6M-depth2-v1/`。独立監査は未完であり、
認定済みや採点比較の件数には加えない。監査のためにモデルは再実行しない。

監査器はproducer別のcaptureへまとめる際に観測の到着順を捨て、producer番号順から再構成
していた。EPFへ実到着順を探索の優先情報として渡し、現在のprefixで未消費の最初の観測を
先に試すようにした。producerの全候補は引き続き各一度列挙し、到着順を因果関係や権限の
証拠にはしない。最終check_executionは変更していない。索引はcaller-owned workspaceに
観測一件あたり1 word、producer一件あたり1 wordを追加する。アプリのflag・所有者表・
手順の変更はない。到着順の全域保持・一意性・範囲・先頭選択をLeanの4一般補題で確認した。

最終変更のEPF native137件＋compile-fail doctest2件、runner41件、双方Clippy、thumbv6m
no_std、アプリ全試験の型検査、CaptureExecutionのLean検査が成功した（最大1,830,649,856bytes）。
strict-provenance Miriはsource_interpretation5件とendpoint_history11件が成功した。
有効／早すぎる返信の2つの小さい通信記録を、全20のproducer順を優先した場合でも照合した。
新しい独立actorの試験で全順序を誤って要求した初版のassertionは修正し、その失敗logを残す。
sourceの意味論や最終検査を変更して合わせたものではない。

前のchoice-attribution版のモデル監査は696.99秒で明示停止した。結果はなく、不合格・未認定と
して残す。現在は到着順を保持した監査器で、保存済みの2番目の再考条件を検査中であり、その後に
保存済みの実AI記録を検査する。`epf-observation-order-v1/`へ正確な監査binary、source、hash、
モデル原本のhash、検証結果、profileと停止証拠を保存した。置換済みの旧EPFビルドcache約354MiBと、
元result.jsonとの内容一致を確認した重複stdoutを削除した。現行cache・原本・停止証拠は保持する。
移行21/48・残り27、採点比較0を維持する。

### 実記録の監査と独立した7Eの実装（11月10日）

上記の到着順を優先した監査は799.63秒で明示停止し、証拠・profile・監査binaryとsourceを
保持した。モデル記録の監査はまだ開始しておらず、合格として扱わない。全記録の候補構成後に
初めて判定していたresolverと既存選択の一致、pollと最新resolver判断の一致は、それぞれの
記録以前のprefixだけに依存していた。同じ判定を候補の追加前にも実行するよう整理した。
最終検査にも同じ判定を残し、source規則や受理域を緩めていない。長さ0〜3・6種の記録・
両armの全518語で、全出現割当を最終検査器へ入れた結果と構成器の成功が一致した。
EPF native138＋doctest2、runner41、Clippy、thumbv6m、関連Miri16件が成功した。
Leanのsourceモデルと一般prefix補題は前の合格版から同一であり、一般Rust refinementとは
主張しない。固定した`epf-prefix-choice-v1/`の監査器による保存済み実AIの6Mの検査は、
2,125.09秒で明示停止した。最大子孫メモリは1,984,921,600bytesで、監査証明は得られていない。
後続に予定していた再考条件case1の監査は開始していない。実行中profileの561標本中311が
同じscopeの分岐範囲を親リンクで探す処理にあり、不変のsource索引へまとめる変更を検証中。
sourceモデル・core・保存済み実行入力は変更せず、新しいモデル呼出しも行っていない。

その監査器と保存済み6Mの入力を変えず、7Eの実装を独立して進めた。
`src/tasks/shared_document/`へ全体global、実材料とreview receiptの検査、実データ所有者、
実際のChoreoFSを使うwriter本体を追加した。初期DOCUMENTの貸出は両順を持ち、referenceと
書込み、二つのreviewをそれぞれparで表す。再修正後は両reviewを実際にやり直し、最終承認の
通知前に編集側のDOCUMENTをwriter Aへ移す。公開はその実Endpointからの要求と、EDITORの
受理済み引数が一致する形をsourceへ記述した。旧rollの再訪をlocalsideの完了flagで抑えない。
草稿段階のscripted writerは削除し、別のnative実行系として残していない。
型付き引数codecは既存のreport codecをそのまま参照し、同じcodecの別wrapperを作っていない。
7Eの4試験も追加したが、現時点ではビルド前であり、画像・実Endpoint・Wasmの成功は未確認。
EDITOR、WASI合成、共通runner接続、実行・監査・実AIを含む認定を続ける。移行数は21/48のまま。

### EPFの不変索引と7Eの実WASI入口（11月10日、認定継続中）

EPFのroute範囲を、不変のscope別区間索引へ集約した。原sourceのroot descentとの既存の
全有限比較を保ち、138 native＋2 doctest、Clippy、thumbv6m、関連Miri17件、CompactSourceの
容量・同一armのvisit identity・包含の一般補題が通った。ただし保存済み6Mの実AI監査は
479.74秒で明示停止し、合格していない。監査器・全source・入力hashとprofileを
`feature-delivery-development-v1/epf-route-index-v1/`へ保持した。

次に、観測が名前を持つsource操作の出現だけを引く不変の順列を追加した。最初のpollで
特定したcallはその出現へ結び、inactive armやretained visitを順列から落とさず、進行の
可否は従来の検査へ残す。既存の全有限fixtureで、すべてのsource IDと開始位置を直接の
unfolding巡回と比較した。138 native＋2 doctest、関連Miri17件、runner41件、双方Clippyと
thumbv6mが成功した。保存済み再考case1の監査は457.28秒で明示停止し、監査証明はない。
その後の実AI監査は開始していない。profileの558標本中348はquery_ready内であり、
問い合わせ以後の、完了記録が一つもない部分木にも従来どおり全leaf検査を行っていた。
その部分木だけ、全leafがtrue・全内部domainが3になる既存規則を区間でまとめる変更を加えた。
完了記録を含む部分木は省略しない。native138＋2 doctest、Clippy、全leafが許される場合の
一般fold補題と既存の境界slot定理、アプリ全試験とdocument-wasiの型検査が成功した。
変更後の関連Miri16件（source interpretation 5、endpoint history 11）、thumbv6mとrunnerの
buildが成功した。保存済み6M再考case1の実記録監査は、source・binary・原本のhashを再照合した
`epf-future-subtree-v1/auditor-v2`で実行中であり、保存済み実AI記録はその後に実行する。
実記録監査はまだ合格していない。core・既存課題のglobal/localside・モデル
原本を変更せず、モデルを再実行していない。

7Eの初回投影は、同じ公開メッセージを複数の宛先へ並行送信するglobalの曖昧さを拒否した。
その失敗sourceとcompiler診断を残した。当初は他方writerへの通知を不要と判断して除いたが、
投影はそのwriterから一方の枝が空に見える問題を検出した。まず公開と最終結果を成功枝へ
含め、次の診断で内側の修正担当者の選択にも両writerの知識が必要と確認した。現在は実際の
負のreviewを両writerへ送り、修正担当者・他方writer・サービスで異なる公開メッセージ契約に
している。最終reviewの確認はそれを検査するサービスへ送り、writerは実資源・公開要求・
完成物を受信する。このsourceはまだ認定していない。localsideの並べ替えやcoreの検査緩和で
回避したものではない。editorに書いた同じ章への再reviewを拒否する独自の進行判定も除去し、
操作の許可・拒否はHibanaに残す。

編集者の実WASI本文、3guestとI/O所有者の直接の起動、binaryのprojection/capture/checkpoint
を追加した。businessと3つのWASI I/Oを一つのglobal・8つのimmutable programへ統一し、
businessだけの別の実行経路は残さない。初期の貸出先は同じglobalのresolver入力で選ぶ。
実材料・review値以外の完了flagやowner表を足していない。初回のeditor型検査で、borrowを
返すasync内の再試行loopをRustが拒否した。単独の実WASI境界を並行に受け、同じ本文で
その要求を処理する形に修正して型検査が通った。
両初期貸出先について、本番の文書receiverへ異なる引数と担当外の初期章を送る4つの負例を
書いた。そのv9 buildはcompilerのconst評価中に2,759,559,880 bytesとなり、
2,750,000,000-byteの保護上限で停止した。全体投影も5つの試験関数も完了していない。
`long_running_const_eval`のlint失敗ではなくメモリ停止である。source・hash・compiler sample・
停止証拠を`document-7E-development-v1/source-qualification-v9/`へ保持した。
globalや上限を変えず投影コストを調査する。型検査は実行成功ではない。
共通runner接続、実Wasm、独立監査、実AIの認定は未完で、
移行数21/48、残り27、採点比較0を維持する。

### 文書課題の共通データモデル（11月10日、検証待ち）

7M/7Hには、実際の本文・図・検証結果の対応を調べず、完了flagで採点する旧DocumentSimが
残っていた。7Eの実データモデルを共有のDocumentSimへ移し、旧flag実装と二重の7E実装を
削除した。図は自由なdescriptionから実IDを返し、存在する章へ、その実際の完了済みの図を
挿入する。隠れたchecklist名や別の対象へ曖昧に対応付けて成功させない。事実確認・quality
review・editorial判断・公開は、それぞれの実入力と完了済みreceiptに結ぶ。7Hでは章3の
引用とPart Iの材料へ、実際の章3/4の入力を保存し、独立した部分のreviewは先に進められる。
editorialの自由なreview_summaryを、入力に存在しない固定語finalへ要求する旧挙動も除いた。

独立した3つの執筆と3つの引用更新の全720順序、図の生成と執筆の独立した重なり、未完の
図、差替え後の再検証、負のfact結果、部分ごとのreview、未完のeditorial結果での公開を
調べる8つの直接試験を書いた。まだ実行していない。既存6M監査のguard process 72820が
終了してから、固定した共通world全試験と、同じ7Eのproduction binary buildを順に実行する
待機jobを開始した。重い処理は重ねていない。通常lib-test全体のmemory stopとproduction
buildを分けて費用を切り分けるが、元のglobalや未完のEndpoint試験の義務は維持する。
証拠は`document-7E-development-v1/shared-world-v9-v1/`へ保存する。

7Mの自由記述cross-referenceには実際のreadを示すtoolがなく、7Hのmajor/minorの違いも
現行binary結果では表せない。この不足を解消せずに全課題認定としない。world manifestは
まだv8であり、新しいworldの合格とRust入力toolによる版更新まで新規採点を始めない。
既存の実験原本は、それぞれ実行したworldとsource/hashのまま保持する。
移行済み21/48、残り27、採点比較0は変えていない。

### 課題ごとのコンパイル境界（11月10日）

7Eのv9停止では、ライブラリが全課題のglobalを無条件に含み、単一のrustcへ全投影の
const評価を集めていた。各binaryが正本の課題moduleを直接含み、共通libraryにはhost I/Oと
codecだけを置く構成へ変更した。global、localsideの操作、保護メモリ上限は変えていない。
reportとdocumentの要求codecは同じ定義を共通のデータ処理へ移し、wrapperを追加していない。
codebaseの二つの実Endpoint負例群は、同じglobalを使うcodebase binaryのnative試験へ移した。
handoffの実試験は既存の専用proof crateへ残し、全課題をlibraryへ再集約しない。
7Eの投影と実receiverの4負例を含む5試験が7.75秒・最大RSS314,130,432 bytesで通った。
全17binaryを含むClippyは24.37秒・最大子孫443,154,432 bytes、全課題のnative75件は
562.68秒で成功した。続くproductionの7E buildは7.83秒・最大RSS325,353,472 bytesで成功し、
8つのprogramを出力した。一連の全体guardの最大子孫メモリは813,842,432 bytesだった。
共通worldは新しい文書課題の720順序を含む103試験が2.555秒で通った。
移動前後の試験関数の名前と件数に欠落はない。ただしsource上の照合自体は実行認定ではない。
専用Handoffの6試験と医療fixtureも成功した。3つのJSON変換・照合関数は本体を一字も変えず
患者データ処理から分離し、5E/5M/5H以外の課題sourceと共通host I/Oは変更していない。
影響する医療系11試験、全targetのClippy、3つのproof fixtureのClippyを最新sourceで再確認した。
無関係なnative結果は元source・command・結果を保持して継承し、75件全部を再開していない。
最新7Eのproduction buildは8.05秒・最大RSS325,484,544 bytesで成功し、投影は前段とbyte単位で
同一だった。source・binary・投影のhashと検査範囲は`task-local-compile-v1/result.json`へ保存した。
7Eの実Wasm・実AIは未実行。

保存済み6M case1の監査は4,042.14秒で明示停止した。最大子孫メモリは1,038,663,680 bytesで、
監査証明は得られていない。binary・source・原本・hash・profileと停止証拠を保持する。
後続の保存済み実AI記録の監査は未開始であり、モデルの再実行はしていない。
先に共通worldの直接試験と同じ7Eのコンパイル・実Endpoint試験を検証する。
古い待機jobは試験開始前に取り消したため、そのjobを実行済みと数えない。
移行21/48・残り27・採点比較0を維持する。

共通worldのv9 manifestをRust入力toolで生成し、48課題の実sourceと照合した。入力toolの
16試験とClippyが成功した。初回はmanifest更新前に入力試験を実行して3件が旧版照合で
失敗したため、検査を変えずmanifest生成後に入力試験だけを再実行した。合格したworldの
Python sourceは版登録後もhashが一致する。7Mの実readと7Hのminor-edit条件は未認定であり、
worldの直接試験をapplicationの全課題認定へ読み替えない。旧v8 manifestと既存の実験原本は
元版のまま保持する。必要な旧6M executableはhashを照合してCargo targetから停止証拠の
固定outputへ移し、不要な旧depinfoを削除した。実行中のbuildや再利用するcacheは削除していない。

### 7Eの共通runner・監査接続（11月10日、実Wasm検証前）

共通runnerへ7Eの実行、snapshot、モデル入力、入力fixtureと監査を接続した。fixtureは
共通DocumentSimが実際に保持する不変の割当データから作り、監査用globalや許可表は作らない。
process起動は実argvのsliceを直接受け取り、既存課題の引数内容を保ったまま7Eの初期貸出先も
渡せるようにした。global/localsideの操作列は変えていない。runner41件、7E native5件、
Clippyと双方のbuildが通り、投影は前段とbyte単位で同じだった。

監査は各効果の実資源要求、受理済み引数、WASI close、実結果を出現ごとに一度だけ対応付ける。
実際の完了済み章・referenceからreview入力を照合し、公開時の二つの最新の肯定的reviewと
その実confirmation・resource要求・editorの引数を検査する。動的source実行は同じ投影と
captureをEPFへ渡す。これは実装済みだが、まだ実Wasm記録での成功を確認していない。

実Wasmの事前固定条件は8つ。通常2条件で両初期貸出先、両review順とwrite/reference順を
扱い、追加6条件で初回両方拒否、3回・4回の再拒否、他方の章への再修正、不正入力後の回復、
公開後の追加shell要求を扱う。後者のshell拒否を、localsideを置き換えたEndpoint攻撃の
証拠とは数えない。全て短縮遅延0.01・モデル呼出し0の機構試験であり、実モデルの自由な
操作や実時間、比較実験とは別である。まず通常2条件を通して入力・通信・効果・監査を
切り分け、続いて残る6条件を実行する。移行21/48、残り27、採点比較0のまま。

### 7Eの実Wasmとlocalsideの修正（11月10日）

通常2条件は`wasi-baseline-v5/`で完走し、14件の実効果と記録変異124件の拒否を確認した。
追加の拒否条件では、EDITORが未選択のintrinsic route内を先にpar借用していたことと、
更新結果・Endpoint返却の二つのfragmentを逆に使用していたことを確認した。既存の型付き
Reconsider送信で分岐を選び、実source順のfragmentで受信する形へ修正した。残る通知と
返却・更新の受信は並行に駆動し、global・core・Lean・EPFは変更していない。本体の
生成localsideにも、intrinsic選択の先頭send後にparを借りる処理が既にある。この生成検査は
アプリ本文を実行する検査ではなく、今回の呼出し誤りはアプリの実Wasm試験が検出した。

`wasi-reconsideration-v4/`で追加6条件すべてが成功した。両初期貸出先、1・3・4回の再修正、
他方の章の修正、不正入力後の回復、公開後のshell要求を含み、実効果72件・記録変異582件を
検査した。全条件でprotocol完了、共通score完了、違反0、動的EPFと別の効果監査が成功した。
追加試験は180.64秒、buildを含む最大子孫メモリ409,255,936bytes。モデル呼出し・採点比較は0。
通常2条件の拒否を通らない経路は変わっていないため、その原本を引き継ぐ。追加試験後は
error contextの空白整形のみを行い、最終sourceのnative5件、Clippy、format確認も成功した。
直接の別Endpoint操作で公開後の編集権限を試す検査と実AI pilotは未実行であり、7E全体の
移行完了とはしない。移行21/48、残り27を維持する。

### 7Eの直接権限試験と実AIでの未解決要求（11月10日）

同じ全体globalを二つのruntimeで直接実行し、修正0・1・3回後、freeze直後と公開返信後に、
旧資源編集・writer要求・旧Endpoint返却を試した。18試行は送出0・受理0で、14件が拒否、
4件の資源呼出しは先行通信待ちのPendingだった。Pendingを即時拒否とは数えない。最初に
全件即時拒否を期待した新試験はAPI契約より強すぎたため、その分類を分けた。旧writer要求と
旧Endpoint返却は全件拒否された。これはwire fixtureでの権限cut試験であり、実世界の公開効果
や全interleavingの証明とは区別する。最終native6件とformat確認、runner41件とClippyが成功した。

通常runnerにはsnapshotの対応表と重複した課題一覧があり、7Eだけを開始前に拒否した。
その重複を削除してsnapshotを対応判定の一箇所にした。開始前の失敗はモデル呼出し0である。
その後、固定sourceの実binaryでLuna xhigh三人を起動した最初のpilotは失敗した。
初稿二つとwriter Bのreference、計3効果の後、writer Aがwrite_sectionを再要求した際に、
localsideが消費済みInitialDocumentをrecvし、102.674秒でPhaseInvariantとなった。scoreは未完了、
違反は0であり、成功認定へ数えない。runtime停止後も残ったモデル接続は明示終了し、
participantの失敗と途中usageを保存した。モデル完走の性能値や採点比較として使わない。
原本は`output/experiments/model-qualification/current-core-document-7E-v1/`に保持する。

AIを再実行せず、同じ再要求をモデルなしの実Wasmへ固定した。out_of_orderを返した後に
reference・review・公開を完走する第9条件の元の失敗は、3.14秒で再現し、
`document-7E-development-v1/duplicate-write-reproduction-v1/`に保持した。現在のglobalは初回writeを
一回としており、その再受信を本体が拒否したこと自体は正しい。生成検査には完了済み操作の
再要求が既にあるが、拒否後の正常継続と、このアプリの要求変換は検査していなかった。

候補実装は、宣言済みの受信を通信開始前に拒否した場合だけ、runをfaultさせずに構造化した
エラーとして返す。writerはその拒否をWASI結果へ返す。許可する操作は増やさず、誤った型・
fragment、不正frame、開始済み通信のエラーは従来の停止契約に残す。これは合法操作の誤拒否を
直したという主張ではなく、入力境界で必要な拒否後の継続という契約変更である。flag、所有者表、
権限query、fault後の再開は追加していない。offerのlabelは一つの候補を表すため、その不一致だけで
並行する別の合法操作まで拒否する回避策にはしない。

`core-unavailable-v1/wasm-all-nine-v1/`では同じ候補の全9条件が完走し、93効果、違反0、動的EPFと
効果監査の成功、記録変異768件の拒否を確認した。sourceは実行中不変であり、三つの試験関数は
201.37秒で成功した。本体の内部846件、呼出し境界17件、通信fault/取消12件、キューとpreviewと
委譲を扱う三試験のstrict-provenance Miri、ClippyとPico/no_stdも成功した。生成器113件に加え、
既存の同一入口で通常/委譲の二カタログ・6形状・11,008 admission比較を通し、Rust/Lean差は0。
通信前のUnavailableでは、実際のactor・custody・fault・previewとcaptureが不変であることを
両生成driverで照合する。これは有限のRust状態比較であり、Leanの許可判定や一般refinementと
混同しない。通常CI全割当の最終source成功を主張するものでもない。

このbinaryでLuna xhigh三人の二回目のpilotを実行した。再度の初回writeにはout_of_orderが返り、
AIはreference更新、実修正、二つの肯定的再レビューまで進んだ。しかしその後のreference再更新で
消費済みReferencesAのsendを選び、249.143秒でLabelMismatch { expected: 127, actual: 8 }により停止。
三人ともguest runtime closedとなり、score未完了・違反0。観測usage 894,674 inclusive tokensは
不完全であり成功の性能値に数えない。原本は`model-qualification/current-core-document-7E-v2/`。
受信にだけ入れた拒否後の継続契約が送信へ揃っていない境界であり、合法な送信の誤拒否と断定しない。

AIをさらに呼ばず、第10の実Wasm試験へ「修正後のreference再更新→拒否→公開」を追加した。
同じglobalと実binaryで同じsendエラーが3.49秒で再現し、
`core-unavailable-v1/duplicate-reference-reproduction-v1/`へsource・receipt・rawを固定した。
既存9条件の成功をこの境界へ広げず、送信側と要求変換を切り分けてから次へ進める。
本体の許可緩和、fault後の復活、flag・権限表、prompt固定では回避しない。
移行21/48、残り27、採点比較0を維持する。

### 7Eの送信拒否と長い実応答（11月10日、続報）

送信側も、反復の外にある完了済みborrowed sendを通信前に拒否するときだけ、進行を変えず
構造化エラーを返す契約へ揃えた。wrong fragmentとowned sendの失敗は停止契約のままとする。
localsideはその拒否をWASI結果に変換し、許可表・完了flag・fault後の復活は追加しない。
最終候補の本体内部846件、呼出し境界20件、通信fault/取消12件、Miri 6件、Clippy、Pico/no_std、
通常/委譲4形状のadmission比較6,548件が成功した。生成器113件は不変の入力・条件に限って引き継ぐ。
通常CI全53割当の最終成功は未完であり、これらを本体全体の合格へ広げない。

`core-unavailable-v1/wasm-all-ten-v1/`では全10条件、103効果、違反0、動的EPFと効果監査の成功、
記録変異851件の拒否を確認した。このsourceの三回目のLuna xhigh三人pilotは、実修正の応答
9,522 bytesがアプリ側の結果ファイル上限8,192 bytesに達して停止した。これは本体の許可判定と
別の実データ転送の不具合である。原本は`model-qualification/current-core-document-7E-v3/`に保存し、
三人のharness_error、不完全なusage、失敗結果を成功値へ混ぜない。

結果chunkのoffsetをWASIと同じ64 bitへ揃え、ファイル全体の人工的な8 KiB制限を除いた。
packetの容量検査、overflowの拒否、変更前の容量確保は維持する。8 KiB/64 KiBを跨ぐoffset等の
native試験が通り、実Wasmの長いUnicode応答の試験は4.455秒でprotocol/元採点/公開まで完了した。
39,635行のrawは`result-stream-recovery-v1/`に保存済み。ただし動的監査はCPU負荷のため明示停止し、
証明書を得ていない。この新wire形式へ旧10条件の認定を読み替えない。

EPFで、既存の「完了済み操作へ先行操作を後付けできない」条件を候補探索前へ移し、分岐候補の
準備にも既存の不変source索引を再利用する。候補範囲、source意味論、最後の独立検査は変えない。
前者のsourceでnative 139件、Miri 27件、ClippyとPico/no_stdが成功し、後者のsourceもnative 139件が
成功した。保存済みrawの再監査と新wire形式の実Wasm再認定は続行中。AIを監査性能の診断に
再呼出しせず、7Eは未認定、移行21/48、残り27、採点比較0を維持する。

### sourceの未実行部分と7Mの実読取り（11月10日）

長い7E記録の監査は停止ではなく前進していたが、後続visitの候補ごとに大きいsource全体を
再走査していた。直前のseq子が未commitで、その全leafに照会actorの固定操作がある場合は、
どの分岐でも先行操作が欠ける。この必要条件を既存の親索引とprefix bitで先に検査する。
新しい進行表・capacity・app権限は追加せず、混在actor、条件付きbinding、部分実行は従来の
完全なreadiness検査へ残す。Rustの140試験、関連Miri 11試験、Clippy、thumbv6m/no_stdが成功した。
`CompactSource.lean`には、選択されたsource部分木が空でないことと、全leafの証明済み性質を
満たすmessageが選択後も存在することの2命題を加えた。依存42モジュールを現在のsourceから
構築し、kernel/axiom検査が成功した。Rust実装全体の一般refinementとは主張しない。

記録は再実行せず`epf-fixed-prefix-v1/`の監査器で診断した。実照会のsource・選択・prefixを
取り出して長時間の診断実行を明示停止しており、まだ証明書は得ていない。debuggerでの
一時停止を含むため、経過時間を監査throughputやモデル性能に数えない。
旧監査binary 3個は削除し、hash、source、profile、明示停止の証拠と元rawを残した。

独立に、元の7Mで不足していた共通ツール`read_section`を追加した。両writerは完成した任意の章の
実内容と版IDを読める。未完成・書込み中の章は内容を返さず、返却値はsnapshotで、後の編集を
凍結しない。既存DOCUMENT資源を使い、writer順序や全章の準備完了を前提にしない。ツールは
7Mの共通宣言へ加え、他手法も同じ契約を使う。7E/7Hの宣言済みツールは増やしていない。
この実データ機能をversion 10として固定した。共通worldの106試験、入力toolの16試験と全48課題の
manifest検証が成功し、sourceとhashは`world-v10-v1/`へ保存した。旧runは元のworld版に結び付ける。
これはread結果をAIが後続文面に使った保証や7Mの移行完了ではない。global/localside、実Wasm、
モデル実行と効果の対応は残り、移行21/48、残り27、採点比較0のままとする。

### commit済みの分岐と監査の先行操作（11月10日）

7Eの実記録から、producer 0のordinal 18,370、76,800イベントのsourceに対する
`Received(20718)`の照会を固定した。先行する`Received(20380)`が未完了で、同じ枝の
送信commitが既にあるため、この照会は実行不能である。しかし旧枝刈りはその枝を未確定と
扱い、全sourceの選択を再探索していた。source・actor行・親索引・導出2,958段を実入力から
再構成して一致を確認し、この必要条件を既存のprefix bitと親索引で先に検査するようにした。
transport受理だけでは枝を固定しない。条件付きactorと再開依存は完全検査に残す。
本体、global、localside、source文法、最後の独立した実行検査は変更していない。

小さいsourceの全選択と保持leafの全部分集合を独立に比較し、分岐の両側、par、条件付き
binding、rollの有限展開を確認した。現sourceのRust 141試験、関連Miri 12試験（query 7、
source interpretation 5）、Clippy、thumbv6m/no_stdが成功した。Leanではsource内の
message位置の上下限と、同じmessageを保持するroute解釈が同じ枝を選ぶことを証明し、
既存モジュール全体のkernel/axiom検査が成功した。Rust全体のrefinementとは主張しない。
実入力を読む一時的な診断testは通常試験から除去し、縮小した常設試験へ置き換えた。

同じ保存済みrawから変換した完全capture 29,317記録と独立checkpointを、固定した現sourceの
最適化済みEPF CLIで診断した。22分の実行を明示停止し、記録元が一つで固定された送受信対応を
候補ごとに再ソートする重複をprofileで確認した。記録の再生成やWasm/AIの再実行は行っていない。
証拠は`document-7E-development-v1/epf-forced-choice-v1/`に保存する。
7Eの未認定、移行21/48、残り27、採点比較0は変わらない。

### 保存済みの長い7E記録の監査完走（11月10日）

単一producerの完全なordinal順は一つしかない。その順のcustody/FIFO対応を一度だけ検査し、
source出現の探索からは既存の不変な対応列を読む。複数producerの候補は従来どおりprefixごとに
因果対応を組み直す。固定順のresetは、そのviewの次の実commitと、既に対応済みの送信callが
同じ出現を指す場合だけ先に絞る。根拠がないresetの候補範囲は変えない。最終の独立した検査は
custodyを再構成する。先行操作のmember判定にも同じsource wordから作る索引を使い、重複操作の
拒否とエラー位置を維持した。standalone検査には展開eventあたり1wordを加え、interpreterは
既存のevent索引領域に収める。アプリの進行表・flag・authorityは追加していない。

現sourceのRust 141件、Miri 36件（全unit 29、source interpretation 5、実Endpointの再入2）、
Clippy、doctest、host/Pico no_stdが成功した。既存の小さいword全部分集合で直接member判定と
索引判定のerror/位置を照合し、単一producerの全prefix/rewindと遅い送信publishも比較した。
Leanでは単一producerの完全順の一意性を加え、CaptureとCaptureExecutionのkernel/axiom検査が
成功した。一般のRust refinement、全integration Miriや本体通常53割当の完了とは区別する。

同じ76,800イベント、同じ29,317記録、独立checkpoint 29,317、同じ展開容量58,635を使い、
EPF CLIの実行が終了code 0で完走した。5,113件の送信受理を含むsource解釈を返し、その内部で
最終検査も通っている。続いてChoreoFixの本番document監査も、同じrawに対する動的EPFと実効果の
照合を通過し、4件の実資源要求と、記録変異83件の拒否を確認した。原本のWasm/AIを再実行して
この成功を作ったものではない。rootの本体RustとアプリRustは元の実行時から同じbytesであり、
それぞれ536・232ファイルを照合した。旧形式10条件や失敗した3つのpilotを現形式の成功へ
読み替えない。証拠は`document-7E-development-v1/epf-fixed-order-v1/`へ保存した。

次は現在の形式で7Eの実Wasm全11条件と実モデルpilotを認定する。移行21/48、残り27、採点比較0。

### 13Hの共通世界と試薬・QC・安全検査（11月10日）

13Hの旧done表を削除し、実材料とQCの計算を13Mと一つの共通実装へまとめた。合成・精製・
bioassay・chromatographyの実入力を結び、BIOLOGISTの拒否を同じbatchの再提出では解除できない。
実合成は成功/失敗/処理中にかかわらず1単位を消費し、3単位を使い切れば通常の業務結果として
枯渇を返す。残量は物理metadataとjournalから導き、mutable counterや完了flagを重ねない。
安全検査は実際に4台を占有するが、科学作業のどの位置でも実行できる。

全120通りの準備/品質順と、全体で12の検査位置、入力の欠落、QC後の新batch、枯渇、Pendingと
4台それぞれの競合を8試験へ加えた。全114 common-world試験と入力toolの16試験、48課題・
825 checklistのv11 manifestが成功した。旧13Hの0秒から共通の1〜5秒の合成遅延へ変え、全手法が
同じ条件を使うことを明記した。13Mの既存13試験も保持したが、過去の実モデルの再認定とはしない。
原本は`lab-cascading-development-v1/world-v11-v1/`へ固定した。13Hのglobal/localside/Wasm/AIは
まだ未実装であり、移行件数へ加えない。実験条件を特定のchoreographyへ合わせていない。

### 7Eの現wire認定と終了時のRPC回収（11月10日）

保存済みの大きい記録は、現EPFで29,317記録・独立checkpoint・5,113 transport受理・
76,800イベントの有限展開を通し、consumerの効果監査も成功した。
`epf-fixed-order-v1/`にchecker、source/hash、結果を保存し、元のworld版を維持した。
続いて`current-format-eleven-v2/`で、現wire形式の実Wasm全11条件（5試験関数）と
動的EPF・効果監査を完走した。重複write/referenceの拒否後の継続と長いUnicode応答を含む。
モデル呼出し0であり、旧wire形式の結果の読み替えではない。

三人の4回目の実モデル試行は、共通worldのcombine自体は成功したが、Writer BがFinishedBを
受け取る前にexit 0を要求し、protocolは失敗した。さらにrunnerが子の終了時に未応答RPCを
dropするだけだったため、共有reply送信元を保持するモデル側が600秒のtimeoutまで待った。
原本は`model-qualification/current-core-document-7E-v4/`。観測usage 1,294,526 inclusive tokensは
不完全で、成功の性能値に数えない。本体が合法操作を誤拒否したという証拠ではない。

runnerの既存I/O担当が、終了時に転送済み・未転送の実要求へ明示的な失敗を返すよう修正した。
global・localsideの順序・操作権は変更せず、受理済みeffectの返信回収と子のreapは維持する。
モデルを呼ばない実子stdio試験で元の待機を2秒で再現し、修正後はfailed verdict/EOFの両方で
成功した。通常CIにも明示接続済み。runnerの通常41件、新しい明示実行1件、format、Clippy、
buildが成功し、全子孫の最大メモリ1,008,336,896 bytesであった。証拠は
`document-7E-development-v1/runtime-close-fixed-v1/`へ保存した。本体sourceの再変更ではなく、
既に合格した本体・Wasm検査を最初からやり直す理由にしない。7Eの実モデル認定は未完、
移行21/48、残り27、採点比較0のままとする。

### 6Mの保存済み実モデルを独立認定（11月10日）

5人のLuna xhighが完走した6Mの原本について、source・実binary・prompt・実接続・usageを
照合し、動的EPFと効果監査が成功した。元のv8の281.621299166秒、1,597,412 inclusive tokens、
26効果・37,464記録・違反0を確認した。原本にaudit.jsonを保存した。モデル/Wasmの再実行は0。
記録変異11種類もすべて拒否した。これには実行fixtureと保存manifestのworld版の食い違いを含む。
変異用の一時source/binaryコピーは拒否を確認後に削除し、成功したbaseline監査は報告へ再利用する。
監査と変異の実行は102.16秒、全子孫の最大メモリ2,053,898,240 bytesであった。

過去の入力の全byte/metadataは一致していたが、入力toolがmanifest再計算で最新の版番号を付け、
v8をv11として比較していた。生成時は現版を明示し、監査時は記録された宣言版を保持して同じ
inventory処理を使うよう修正した。旧版専用実装や版ごとのfallbackは追加していない。
この認定をv11や新wire形式の実行と読み替えない。

別の32効果の保存済み機構試験は、10分超の監査を明示停止し未認定とした。候補ごとの既存全履歴
走査を、同じprefixから毎回導出する使用済み出現のmarkerへ置き換えた。追加storageやapp権限は0。
Rustの有限比較とLeanのfiltered-prefix命題、142 native、35 focused Miri、Clippy、Pico/no_std、
入力tool17件、runner41件が成功した。再診断では支配箇所がsource選択のqueryへ移り、同じ記録の
認定にはまだ至っていない。本体・global・localsideは変更していない。原本、profile、停止証拠と
認定済みモデル記録を`feature-delivery-development-v1/epf-attribution-index-v1/`へ保存した。
移行21/48、残り27、採点比較0を維持する。

### 6M監査の実行不能な後続visitをまとめて除外（11月10日）

32効果の原本を変えずに実際の問い合わせを保存し、protocol・unfolding・actor・source索引・
履歴の全要素が一致する安全な再構成を確認した。先行visitの一部が完了済みでも、その後の
必須visitが未完了なら、さらに後の同じactorの操作は実行できない。従来の枝刈りは先行部分
全体だけを見ており、部分的な完了に隠れた未完了visitを各候補で探索し直していた。

EPFはseq/parの必須部分木から同じ不足を導き、同じ前提を持つ後続のseq区間をまとめて除外する。
選択肢の境界と独立した右par枝は越えない。アプリの進行・権限・flagや追加storageは作らず、
最後のcheck_executionも維持する。除外区間の全pointを独立した完全選択と比較し、両側のchoiceを
経て合法な継続が残る負の対照と、4/72visitの試験を追加した。145 all-targetsに加え最後の回帰1件、
strict-provenance Miriのquery11件とinterpreter5件、Leanの区間・合成命題が成功した。
最後の回帰追加前の同じproductionに対するClippyとPico/no_stdも成功している。

途中の427.05秒・304.22秒の監査は明示停止143で、いずれも未認定として保存した。
最終の連続seq区間を扱う版は`feature-delivery-development-v1/epf-blocked-span-v2/`で原本を再監査中。
本体・global・localsideは変更していない。古い停止試行のbinary2本、30,040,592 bytesは削除し、
原本・source/hash・停止証拠・profileは残した。この局所検証を32効果の全体成功へ読み替えない。
今回追加のAI呼出し・Wasm再実行・採点実験は0。移行21/48、残り27を維持する。

7Eの4回目のAI記録では、Writer Bは他章の負のレビュー後に次のfeedback要求を出さず、
古いresultと書込専用feedbackのreadを試してからexitしていた。現時点で新たな本体の誤拒否を
示すものではなく、完了条件を弱めたりlocalsideで最終受信を代行したりしない。
終了時RPCの修正後初回となる次の予備実行を`current-core-document-7E-v5/attempt-plan.json`へ
事前記録した。実行はまだで、失敗した4原本も使用量を含め保持する。成功値だけを選ぶための
反復には使わない。実験契約の7E欄も、現在の実Wasm11条件成功と実AI未認定へ同期した。

### 7Eの実AI認定完了（11月10日、22課題目）

終了時RPCの修正後、事前記録した条件のままLuna xhigh 3人を実行した。最初の起動だけは
PYTHONHASHSEED=0の指定漏れでAI接続前に停止し、ログを保持して同じsource・条件で再開した。
プロンプト、global、localsideを変更せず、2回の実レビュー拒否から修正・再レビューを経て、
共通採点・protocol・全参加者の終了・動的EPF・実効果監査まで成功した。

実行189.481558958秒、919,803 inclusive tokens、10効果、違反0。28,713 producer記録と
独立checkpoint、5,067 transport受理、73,160イベントの有限展開が対応する。reasoning tokensは
outputに含まれ、重ねて加算していない。モデル試験全体226.28秒、最大子孫737,411,072 bytes。
別の11種類のrun-record変異も全拒否し、その実行は76.53秒・最大530,120,704 bytesだった。
これは監査器の改変記録試験であり、11件の実行時攻撃を阻止したという数字ではない。

既に通った現wireの11実Wasm条件と、今回のruntime/core/WASI/applicationのsource・manifest
790項目、washの8入力が完全一致した。旧11条件のEPFの版は保持し、最新EPFで全部再実行した
とはしない。新しい実AIは現在のordered-suffix EPFで照合した。4回の失敗原本と不完全usageも
保持し、今回の成功へ読み替えない。認定はcommon-environment-v11と実際のbinary/sourceに限定する。

原本は`output/experiments/model-qualification/current-core-document-7E-v5/`に保存し、
`qualification.json`へ元結果・監査・変異の全条件から集約した。非採点pilotであり、成功率や
比較優越性の統計ではない。7Eを移行済みに加え、22/48・残り26へ更新した。採点比較0、最終sourceの
全family一括認定は未完である。6Mの32効果の原本監査は別件で、477.84秒・停止143の未認定を
`epf-blocked-span-v2/script-stop.json`へ保持している。

### 10Eの採点器の反例を先に固定（11月10日、修正前）

次のビルド課題について、Hibanaを使わない共通worldの直接試験で、linkを先に実行してから
両moduleをcompileする列と、共有typesがない時にModule Bをcompileして後でModule Aを
compile・linkする列が、どちらも完了扱いになることを確認した。正常な入力からのlinkは成功し、
2つの否定assertionだけが元のv11 sourceで失敗した。新しい本体不具合ではなく、未移行の
BuildSimが4つのdone表とfirst-compileカウンタだけで成果を判定している問題である。

原本・宣言・hash・失敗ログは`output/experiments/build-world-development-v1/before-v11/`。
新試験は`benchmark/tests/test_build_causality.py`にあり、worldを直すまで未合格のまま保持する。
skipや成功扱いにはしない。修正は10E/10M/10Hの実objectと共有artifact・実際の依存入力・
検査・link結果の対応へ統一し、特定のchoreography順を共通scorerへ移植しない。
10Eのglobal/localside/Wasm/AI認定はまだで、移行22/48・残り26・採点比較0は変えない。

### 10系の共通ビルドworld v12を固定（11月10日）

BuildSimのcompile/test/validation/linkのdone表とfirst-callカウンタを削除し、実際の共有定義、
一意のobject出現、完了した同じobjectの検査とlink入力をjournalから照合する形へ変更した。
stale/未検査objectの物理linkは実行されて違反を残し、後から検査してもそのlinkは認定されない。
再ビルドと新しいlinkによる回復は可能で、過去の違反は消さない。実compilerの実行ではない。

10Eの元課題にあるB先行を保つため、提供済みの共有typesを初期値とする。A先行とB→A→Bの
両方が正しく完了し、B→A→linkは古い入力として不合格になる。初期types欠落の修正前反例は
元のsource/hashのまま保持し、後の試験と同じ初期条件だったとはしない。
10M/10Hの6/24初期順列、両検査順、更新後の古いobject/検査、Pending、戻り値alias、
実moduleからの物理資源binding、独立した2buildの重なりと3buildの容量超過も直接検査した。

共通world136件（うちbuild22件）、Rust入力tool17件、Clippy、fmt、source照合が全成功。
最大子孫108,101,632 bytes。48課題・825項目、既存matrix 2,592件は維持した。
証拠は`output/experiments/build-world-development-v1/provided-types-v12/`。
README・実験契約の現在world表記もv12へ同期した。旧版の実験は記録したworld/hashを維持する。
次は10Eの両初期順とBの再ビルドを含む委譲global・直接localsideであり、アプリの実Wasm/AI/EPF
認定はまだである。移行22/48、残り26、採点0を維持する。

### 10Eの委譲・実Wasm・実AI認定（11月10日、23課題目）

`src/tasks/shared_build/`にglobalと直接localsideを実装した。CORE_LIBとSHARED_TYPESの
同じ2本のEndpointを並行に貸し出し、A先行とB先行をglobalのroute/resolverで表す。
B先行では実際の古いobjectを返してからAへ権限を渡し、Aの実types更新と同じ継続の返却を受け、
もう一度compileする。現在のA/B objectだけをintegratorへ通知する。受理した引数と両資源の
実要求を同じreceiverで比較して一つの実効果を実行し、実結果を両資源から返す。
linkの実結果はその両objectと一致する必要がある。進行flag、保持者表、別state machineはない。

別runtimeの実Endpointで両初期順と、担当外module・Core/Types要求差替えの3拒否を検査した。
関連native14件とClippy、実行binaryのbuildが成功した。strict-provenance Miriの対象4試験も
成功し、最大663,846,912 bytesだった。Miri初回はhostのkqueueが未対応で停止したため、
既存のfixture Clockを同じhibana-runtimeへ渡す構成に揃えた。macOS FFIをMiriで検証したとはしない。

固定sourceで両初期順×元のdepth0/1/2、両初期順の不正入力復帰・完了後重複要求、計8条件の
実Wasmを実行した。すべて共通採点・protocol・静的投影・EPF動的実行・実効果監査が成功した。
8原本から420件のframe記録変異を拒否した。試験は23.73秒、全guard最大326,090,752 bytes。
raw、binary/source receiptと監査は`build-10E-development-v1/wasi-v1/`へ保持した。
通常の共通Wasm検証入口にもbuild familyを追加し、ignore属性のまま実行を省かない。

事前記録した最初のLuna xhigh 3人のsubscription pilotも完走した。A先行、元depth2、通常遅延、
追加障害・barrierなし。48.046373459秒、261,400 inclusive tokens、実効果3、資源違反0、
全guest終了・protocolと共通採点完了、3人のモデルturnの重なりを確認した。全参加者の
requested/returned modelとeffortが一致する。reasoningはoutput内で、二重加算していない。
3,080 producer記録とcheckpoint、7,613イベントの有限source展開をEPFで照合した。

最初の全run監査は、物理資源metadataを配列だけと仮定していたため、moduleから資源を導く
mapを拒否した。Hibana本体やglobal/localsideの不具合ではない。固定されたmetadataの読取りを
共通化して直接回帰を加え、runner42件・Clippyを通し、同じrawを再監査した。AIは再実行していない。
全監査成功、別の11種類のrun-record変異も全拒否した。測定時runnerと修正後auditorは
別source/hashとして保持する。checker修正を理由に成功済みのWasmやAIをやり直さない。

実Wasmとpilotのruntime-source manifestは完全一致した。原本と集約は
`model-qualification/current-core-build-10E-v1/qualification.json`。B先行の再ビルドは
実Wasm条件で確認しており、このA先行pilotの観測と混同しない。いずれも非採点認定であり、
実compilerの正しさ・成功率・比較優越性のデータではない。移行23/48、残り25へ更新した。
10M/10Hは共通worldだけで、global/localside/Wasm/AI移行は未完。6Mの32効果監査、
本体の通常生成母集団全体、最終sourceの全family一括認定と採点実験も未完のままである。

### 10Mの全体global・実受信側と最初の実Wasm（11月10日、移行認定は未完）

`src/tasks/cascading_build/`へ478イベント・15ロールの単一global、compiler/test worker、
3 builder・tester・integratorの直接WASI localsideと起動処理を実装した。全6初期順を
表現し、各compileは実際の2資源Endpointを並行に受信して、受理した引数と両資源の要求の
一致後に一つの実効果を実行する。actual objectとcompleted testをcandidateとして渡し、
古い／失敗したobjectを再ビルドする。公開前には全3資源をrepairの継続から返却する。
進行flag、保持者表、別state machineやlocalsideの迂回順序は追加していない。

nativeは全6順×3障害集合の18成功schedule、修正要求の実test receiptを差し替える1拒否を
確認した。差替えでは最初の3 compile＋3 testだけで止まり、追加compile/linkはない。
同じ内容でも新objectへ旧positive testを使えないこと、後続の共有入力変更、担当外引数と
不正candidate、resolverが実入力を保持することの5 data試験を加えた。関連module全17試験と
厳格Clippyが成功した。これはnative fixtureであり実AIとは区別する。

実Wasmの最初のABC/ACB/BAC/BCA/CAB、元depth0の5条件はprotocolと共通採点を完了し、
違反0だった。前4条件はEPF動的監査も成功。CABの監査時に連続処理全体がmemory guardで
停止した（観測最大3,513,175,088 bytes、停止閾値2,750,000,000）。同じCAB原本を単独で
再監査すると762,396,672 bytesで成功し、23,418 records・61,735展開イベントを照合した。
Wasmは取り直していない。各保存記録の監査を既存runnerの別processで実行・終了する形へ
変更したが、その26条件全体での成功はまだ確認していない。最初の停止と原本は保持する。

準備中のmatrixは6順×depth0/1/2＝18、不正入力復帰とtest call2/3/4の追加拒否6、通常遅延の
実build/test重なり2の計26条件で、残り21。独立した実効果／receipt監査と変異、focused Miri、
実モデル5人はこれからであり、10Mを24課題目へ数えない。runner43試験と全target Clippy、
application・native対象の厳格Clippyは通過した。共通world v12、本体・EPF本体は変更していない。
実binary/source・raw・停止証拠は`build-10M-development-v1/`、対応は`benchmark/ports/10M.md`。
移行23/48・残り25、今回のAI呼出し0、採点比較0を維持する。

### 10Mの全26条件・実受信側Miri・初回5人AI認定（11月10日、24課題目）

同じ478イベント・15投影のglobalで、実Wasm全26条件が完走した。元の深さ0/1/2と
全6初期順列の18条件、不正入力・chunk/private I/O・拒否後の回復と追加失敗の6条件、
通常遅延ABC/CBAの2条件である。追加失敗はfresh assessmentの0/1/2番であり、
最初のstaleな2検査を含めた実際のtest occurrenceでは2/3/4番に当たる。
全条件で共通採点・protocol完了、違反0、EPF動的監査・独立した実効果監査が成功し、
記録変異7,102件を拒否した。通常遅延の2条件は実build/testの重なりが各3組ある。

5条件は元の実行のbinary/guest/projection/fixture/条件のhash一致を検査して再利用し、
21条件を新規実行した。26記録すべてを現行の監査器で検査している。監査を記録ごとの
子プロセスに分けた固定sourceの全体実行は、最大子孫1,265,483,776 bytes、終了0、
source_unchanged=trueで完了した。元のv3停止記録とrawは保持し、停止を成功へ読み替えない。
実compiler受信側へ、受理したmodule Aと異なるmodule Bの引数を資源Endpointから送り込む
試験は、2runtime間の実Endpointでeffect 0の拒否となった。同じ受信側試験の
strict-provenance Miriも成功した（53.56秒、最大1,167,867,904 bytes）。

ソースを変更せず、既存ChatGPT契約のLuna xhigh 5人で最初のAI試験を実行した。
元の深さ2・ABC先行・通常遅延・seed 0・追加失敗なし・barrierなしで、115.339465333秒、
769,827 inclusive tokens。5人全員と実wash guestが終了し、共通採点・protocol完了、
違反0。model turnは5件同時、実effectはcompile 7・test 7・link 1で、実build/testも
1組重なった。EPFは37,719記録と97,331展開イベントを照合し、実効果監査も成功した。
原本を改変した11種のrun-record変異はすべて拒否した。

最初の全体監査では、実効果監査の出力へ検査済みの共通採点completeを載せ忘れていた。
この出力だけを修正し、同じrawを再監査して成功した。また欠落応答の変異が同じIDの
client requestまで消していたため、実際のtool response一件だけを除く入力へ修正した。
修正後はincomplete tool exchangesとして拒否する。AIの再実行、global/localside/本体変更、
全26条件の再実行はしていない。runner43試験とClippy、その後の入力生成修正に対する
関連2試験・Clippyと実11変異の成功は、それぞれのsourceとauditor hashで記録した。

原本と認定は`model-qualification/current-core-cascading-build-10M-v1/qualification.json`、
機構試験は`build-10M-development-v1/wasm-qualification.json`に対応する。
移行24/48・残り24へ更新した。これは合成objectデータを使う共通環境の有限な認定と
初回非採点pilotであり、実compilerや成功率・比較優位の実証ではない。本体生成検査の
残り、6Mの32効果監査、最終全family認定、正式な採点実験は引き続き未完である。

### 11Eの委譲globalと実受信側の開発（11月10日、未認定）

`src/tasks/manufacturing/`へ、LATHE/MILLの独立した排他的Endpoint、3人の参加者、
2つの実機械受信側、報告の受信側とprivate I/Oを含むglobalを追加した。11ロールを同じ
sourceから投影する。各機械の貸出先は、実際に先に受信した要求と送信者から選ぶ。
保持するCellはその受信データであり、busy/done flag、保持者表、別の進行カーソルではない。
型付きのJobArgumentsは4つの元job IDとJSON bytesを保持し、他人・別機械の仕事や
余分な引数をwire decode前に拒否する。報告受信側は4件の実結果を検査してから
collect_reportsを呼び、その戻り値も実結果と照合する。通信はlocalside本文に直接書いている。

正本のglobalと本番の機械受信側を2runtime間の実Endpointで動かし、全24の作業順が通った。
一方の要求がまだ一件もない間に、もう一方が両作業を終える順も含む。別のnative試験で
2人が別機械を使う2組以上の実effect区間の重なりを確認し、同じ機械の重なりがないことを
要求した。この重なり試験は機械phaseの部分実行であり、全体の報告完了は全24順の試験で
別に検査している。最終nativeは課題5関数・共通shell10関数の計15関数成功、Clippy成功、
最大子孫396,427,264 bytes。証拠とsource hashは
`manufacturing-11E-development-v1/native-v4/`に置いた。最初の停止は新規localsideが
associative parの3断片を2断片として借りていた誤りで、本体やglobalの意味を変更していない。

この試験はnativeの実データfixtureであり、共通Python環境・実Wasm・AIの認定ではない。
特に、実Wasmで使用中の機械への要求にavailability feedbackを返し、空いている別の機械へ
選び直す経路は未実装である。現在のgeneric request/loan待ちをそのままWASI closeへ
直結して11E完成としない。元の技術者内の逐次作業の記述と、現在の機械ごとに独立した
globalが許す並行性の対応も、最終procedureとして整理する必要がある。
移行済みは24/48のまま。11EのMiri・EPF・実Wasm・実AIと実効果監査は未完で、
本体生成検査の残りや全family認定も引き続き別の未完項目である。

### 11EのBusy/ReadyとWASI直接経路（11月10日、未認定）

11Eを15ロール・37メッセージ契約へ改め、実機械2本と、4件の実要求の継続を表す
Endpointを同じglobalで委譲する形にした。要求Endpointは架空の機械や保持者表ではない。
実際に先に受信した要求と、実RPC結果／他方の実要求の到着をresolverの入力にする。
使用中ならBusyを返し、guestは実要求Endpointを保持して別機械へ選び直せる。
各反復は要求側のRetryから開始する。Ready後も正当な遅いRetryが交差し得るため、
通知ではなく実要求Endpointの返却まで、sourceが許す受信を扱う。返却後に実機械を貸す。

最初のBusy先頭のrollはentry ownerの許可を欠くため本体が拒否した。globalを修正し、
本体・EPF・共有simulatorは変更していない。追加したReady後のRetry試験は、最初は
受信側の待受終了が早く10秒で停止した。同じglobalが許す受信をlocalsideへ反映して修正し、
駐車済みReadyを保持したままRetry/Busyを通す場合も、そのReadyを受信後にRetryする場合も
通過した。合法なglobalを通すために通知を増やしたり、順序を迂回したりしたものではない。

実RPCのFutureは実要求／相関receiptを所有し、借用した待機がraceに負けても元のRPCを
残す。再発行や共有可変resourceは追加しない。同じ元RPCを最後に一度だけ完了する境界は
strict-provenance Miriで成功（最大1,372,372,992 bytes）。これはmacOS FFIや11E受信側全体の
Miriではない。共通libraryのnative13件も成功した。

最終のWASI実装を含む固定sourceでnative19関数（課題6・共通shell13）が成功した。
全24仕事順、両方の非対称readiness、別機械の実effect重なり2組以上、同一機械の重なり0、
Busy後の別機械利用、ReadyとRetryの交差、各仕事のRPC一回を検査した。nativeとCLIの
厳格Clippy、CLIの実ビルド、15 program/37 labelの投影出力も成功した。
一連の検査は最大445,661,184 bytes、source hash不変、終了0。通常CIへnativeの単一入口と
RPC借用のMiriを接続した。課題試験をproduction binaryとproof crateで重複して実行しない。

証拠は`manufacturing-11E-development-v1/wasi-development-v3/`、RPCのMiriは`native-v12/`。
手順と元の課題の対応は`benchmark/ports/11E.md`へ事前記録した。実際のwash guestは各技術者の
二つの仕事を順に実行し、どちらを先にするかは自由である。global自体は独立view間の追加の
並行性も許し、技術者個人のcapacity-oneを一般に証明したとはしない。

実Wasm/共通world/EPF・独立効果監査・実受信側への差替え試験とMiri・初回AIはまだである。
CLIのbuildを実Wasm実行へ数えず、移行24/48、残り24、採点比較0を維持する。

### 11Eの実Wasm36条件と修正版3人AI認定（11月11日、25課題目）

旧`wasi-v4`の32条件は実Wasm・共通採点・EPF動的監査・実効果監査に合格し、
1,804記録変異を拒否した。しかし最初の3人AIでは、実作業が先に完了したときに
新規localsideが最初の必須Retry/Busyのroll visitを飛ばし、EndpointDroppedで停止した。
実効果は1件、資源違反0、課題は未完である。失敗raw・binary/sourceは
`model-qualification/current-core-manufacturing-11E-v1/`に保持した。これを本体やLeanの
不具合として扱わず、coreの正しい拒否と、localsideの投影からの脱落を区別した。

lathe/millの受信側を、実RPC結果と実際に返ったEndpointを保持しつつ、必須の最初の
Retryを処理する形に修正した。先行作業の結果はRetryを待たずに返し、保持者表、進行flag、
余分な通知、globalや本体・EPF・参照モデルの緩和は追加していない。元の4通りの競合試験は
必ず先に一回Retryしていたため、この遅い最初のRetryを見逃していた。早い／遅いの両方を
nativeと実Wasmの有限入力に加えた。native20関数と厳格Clippyが成功し、実受信側へ
4種類の不正wire payloadを送る試験は実Endpoint・strict-provenance Miriとも効果0で拒否した。

修正版`wasi-v5`の固定sourceで、全24作業順、不正入力後の回復4条件、通常遅延での
両機械・両参加者・早い／遅いRetryの8条件、計36条件が通過した。全件で共通採点・protocol
完了、違反0、EPFの完全captureと独立した実効果監査が成功し、2,040記録変異を拒否した。
競合8条件は別機械の実効果が各2組重なる。機構用barrierは実AIへ持ち込んでいない。
guard最大1,094,189,056 bytes、source_unchanged=true。原本は
`manufacturing-11E-development-v1/wasi-v5/`に保持する。

同じruntime source manifestの修正版を、既存ChatGPT契約のLuna xhigh 3人で実行した。
38.2828615秒、280,303 inclusive tokensで全guest・モデル参加者が完了した。
4作業＋報告の実効果5、共通採点・protocol完了、違反0、モデルturnは3件同時、
独立した機械の実作業は1組重なった。EPFは1,984記録と4,547展開イベントを照合した。
実効果監査と11種の全run記録変異も成功した。最初の全体監査は、検査済み投影hashを
監査結果へ出力し忘れたため停止した。出力を補いrunner43件・Clippyを通して同じrawを
再監査した。AI・実Wasmを再実行して結果を選んだものではない。

集約は`model-qualification/current-core-manufacturing-11E-v2/qualification.json`。
最初の失敗と修正版の成功を別版として残す。移行25/48・残り23、採点比較0へ更新した。
有限の機構認定と修正版の非採点pilotであり、成功率推定・任意Rustのrefinement・
最終sourceでの全family／本体認定は未完である。本体生成検査の残り、6Mの32効果監査、
正式な採点実験も引き続き未完とする。

### 11Mの共通worldの部品・再作業・工具容量（11月11日、移行は未完）

次の11Mを調べ、旧worldが作業前の検査を後の作業で埋め合わせられること、実資源の宣言が
空で3台の機械・工具2本・検査員1人の競合を検査しないことを確認した。新しい直接試験の
最初の10関数は旧sourceで6失敗・3errorとなった。元のsource・metadata・試験・hashと
停止記録は`manufacturing-11M-development-v1/world-before/`に保持する。旧実装へ戻す
専用patchや反復buildは追加していない。本体・EPFの不具合ではない。

11Mの独立したdone表を削除し、実効果journalに残る現在の部品、完了した実検査結果、
その部品と否定検査に対応する再作業指示から評価するworldへ置き換えた。再作業には新しい
部品identityが付き、旧部品の肯定結果を流用できない。未完の作業・検査・指示は根拠に
ならない。否定結果のまま部品を作り直す操作は物理的には実行し、完了した現在の指示が
なければ違反を記録する。後の成功で違反を消さない。必要な実入力のない検査・指示は通常の
入力不足として返り、入力が揃えば再実行できる。

共通metadataに各仕事の実機械と工具の有無、工具容量2、既存の検査員自身の容量1を明示した。
新しい参加者は増やしていない。registryは呼出しを待ち合わせず、実際に重なった資源使用を
観測する。別機械で工具を使う2仕事と工具不要の1仕事は同時に進める。工具3本の同時使用や
同一機械・検査員の重複は違反になる。別の部品の作業と検査・指示の間に、手順を強制する
依存は入れていない。choreographyやEndpointの進行flagはworldへコピーしていない。

共通world v13の全147試験、Rust input tool 17試験、厳格Clippyが成功した。48課題・
825項目のmanifestと既存実験matrixの整合も通過した。新11関数は、全720作業順と別群の
全720検査順、6仕事×否定部品1/2/3回、遅延中の実データ、古い指示、戻り値の書換え、
各物理競合を確認する。両順序群のCartesian積を網羅したとは主張しない。guard最大
245,170,176 bytes。記録とsourceは`manufacturing-11M-development-v1/world-v13/`に固定した。

11M全体の委譲global・直接localside・実Wasm・EPF／効果監査・実AIは未完であり、
移行25/48・残り23・採点比較0のままである。次は工具2本の実Endpointを使う全体globalで、
空いた工具を使える独立作業を不要に直列化せず、品質結果とdispatcherの実指示を再作業へ
結ぶ。11Eを含む過去の個別認定は記録した旧world版のまま保持し、v13の認定へ読み替えない。
旧11Hのworldも未認定であり、今回の修正へ含めない。

### 工具2本を実Endpointで使い回す部分の確認（11月11日）

3人の工具作業について、貸出・資源要求・返信・返却を同じglobalへ記述し、旧原型の
工具ownerと完了通知の参加者を削除した。localsideには実Endpointと受理済みRPCを
保持し、受信した要求と実完了結果をresolverの入力にする。保持者表や進行flagは追加しない。
制御付きnativeの全6投入順×5到着／完了条件、計30条件・90実fixture RPCが成功した。
工具容量2、どちらが先に返る場合も、他方の終了を待たず3件目を始められることを確認した。
本番receiverへの引数差替え・余分なfield・壊れたJSON・nullの4入力は実効果0で拒否した。
これは工具部分のnative試験であり、11Mの共通world・実Wasm・実モデル認定ではない。

その際、外側の分岐で同じ送信型の候補を移した後に、内側の選択を反映せずResolverRejectに
なる本体不具合を、委譲を外した9イベントでも再現した。現在の本体は不変の分岐祖先を
外側から順にたどり、非選択の子resolverも呼ばない。直接sendとdropしたofferの双方を
含む2つの実Endpoint回帰が通過した。生成検査には異なるpeerと入れ子の先頭で同じ契約を
使う通常／ownedの形を追加した。既存Lean意味論で合法な構造を表現できていたが、
生成器の先頭共有が同じpeerと直列先頭だけだったため、この照合範囲が不足していた。
新しい通常／ownedの6割当は、40完走、4,635,360件のwhole-call照合、20,592件の
capture上のcut照合と不正記録の拒否まで成功し、runtime/model不一致0となった。
初回の独立検査は2,750,873,600 bytesで停止した。965,320件のcontextに繰り返し含まれる
52種類の受動状態だけを共有し、要求履歴・全query・元のIDの連続性と使用数を保って
同じ保存済み記録を再照合した。EndpointやAIは取り直していない。guard最大
2,418,540,736 bytesで通過した。誤ったstate IDの記録も、変更した入力検査器が拒否する。
本体の通常gate全体は未完。途中版のreceiver Miri4入力の合格を、
祖先走査を整理した最終版の認定へ読み替えない。最新nativeのsource・binary・各記録は
`manufacturing-11M-development-v1/tool-cabinet-v9/`へ保存する。

### 工具要求のcommit前検証と祖先resolverの再検査（11月11日）

工具部分の要求を`JobArguments<WORKER>`へ揃えた。正しいschemaを名乗る任意byteの送信側から、
別workerの作業、余分なfield、不正JSON、nullの4入力を実際の工具Endpointへ送る。
本番receiverがcodecで拒否し、正しいintentの受信だけがcommitされ、不正要求の受信commitと
実効果は0であることを検査した。全30正例・90実fixture RPC、厳格Clippyに加え、
同じ4入力のstrict-provenance Miriが成功した（222.53秒、最大425,164,800bytes）。
source不変、解決されたcore/WASI/EPFの正本path、実行binaryと各30記録を
`manufacturing-11M-development-v1/tool-cabinet-v10/`に固定した。これは工具部品であり、
全11MのWasm／AI認定には数えない。

本体の通常gateは、祖先を外側から処理する変更について2件の回帰を捕捉した。
初回選択の後のauthority確認が親resolverを再び呼ぶ経路と、子の未使用armへ移る操作では
完了bitが立っていないため親の選択を保持できない経路である。前者は重複するresolver巡回を
除き、完了した祖先を保持する構造検査は残した。後者は、候補rowの完了ではなく実際のreset
bodyから保持する祖先を決める。既存試験を含む38関数が通過し、初回判断を二度呼ぶと拒否する
stateful resolverも直接send／parkしたofferの両方で検査した。

共通生成器へ、子の拒否arm→未使用の承認arm→親の新visitを含む24個目の母集団を加えた。
その操作wordが実際に有限scriptに含まれることを含め、生成器115試験とtool gateが成功した。
既存のGlobalSemanticsを変えず、子の新visitが親の選択を保つ正例、親の再判定とprefix飛ばしを
拒否する負例3命題を追加した。通常Leanの132モデル・54実装moduleが成功した。
これらは実Endpoint／モデル間の追加母集団全体や通常core gateの完了ではない。
全Rust／必須Miri gateを同じ修正版で継続し、新しい母集団の実行照合も残る。
移行25/48、残り23、実AI追加0、採点比較0を維持する。

修正版の本体では、全native target・Clippy・doctest・documentation・no_stdと、
必須Miri47件が成功した（最大1,835,827,200bytes）。記録は本体の
`output/validation/nested-shared-contract-v19/`に保持する。実Endpointで親の初めて使う
別armへ進む操作列と、未来の親判断をpreviewしたまま子を続行する操作列を追加し、
同じproduction sourceで直接試験も通した（v21／v22）。本体の全normal gateの完了ではなく、
runtime-work・固定consumer回帰・共通生成母集団の残りは別に確認する。

新しい生成母集団の最初の実行は、script 4で停止した。直接の実Endpointでは同じwordと
previewを通せた。生成側が内側のbranchへ入るたびに祖先resolverの入力まで現在の過去の
選択へ戻し、早く取得した次の親visitのpreviewと予定wordを矛盾させていた。
productionは追加変更せず、source scriptから各resolverのvisit順の入力列を作る方式にした。
有限wordより先のpreviewには最後の入力値を保持し、一度も現れないresolverは入力なしで
誤呼出しを拒否する。これは外部入力のfixtureであり、Endpointの進行や権限を再管理しない。
query時の全入力patternと成功previewのdrop、元の全script・両poll順序は維持する。

変更後の24番目の母集団は、4契約variant×25script×両poll順序＝200完走・2,880 call、
27,840 whole-call照合（成功parked-preview 16,224件を内数に含む）、実cut上138,240 queryと
全不正記録controlが成功した。runtime/model不一致0、最大884,408,320bytes。
本体の`nested-shared-contract-v24/`へsource・記録・hashを保存する。このsource-cut照合では
EPFを要求しておらず、アプリ効果認定へ数えない。通常／ownedの先頭共有2母集団も、
同じ生成器・修正版productionで、6割当・40完走・4,635,360 whole-call照合・
20,592実cut queryと全controlが成功した（v25、最大1,867,923,456bytes）。
全実入力・生成器・core sourceのhashに加え、source全体のコピーも照合して保持した。
処理回数の通常gateも11 workload×8指標・同一2実行で予算内である（v26）。
これはCPU命令数や実時間の優位性を示すものではない。固定した実consumer回帰を継続し、
他の共通生成母集団の通常割当も残る。本体・Leanを変えていない部分の合格を最初から取り直さない。

実AI原本の一覧も再走査し、16課題ID・23個の異なるraw streamをCSV／JSONへ整理した。
成功だけを選ばず、失敗した試行と異なるworld/sourceを別行で保持する。
`pilot-inventory-rust-v2/verified-v1/`は予備一覧であり、新しいAI実行ではない。
旧readerの「run内audit.json」という配置仮定により、10E／10M／11Eの最終監査が親directoryや
mutation reportのbaselineにあることを解決できていない。該当qualificationの参照が原本であり、
一覧の未解決を監査原本の消失・実行の失敗へ読み替えない。論文用の最終データ認定は未完。

### 固定consumer回帰の完了と6Mの保存記録監査（11月11日）

現在の修正版本体に対する必須の固定ChoreoFix回帰が完了した。review／codebase／CI/CDの
元のglobalとlocalsideをビルドし、実Wasmの独立commitとCI/CD資源要求の試験が成功した。
production source不変、モデル呼出し・採点実験0、最大子孫メモリ1,288,357,832bytes。
本体の`output/validation/nested-shared-contract-v27/`へ入力revision、実際のfixture source、
build receipt、source hash、実行記録を保存し、使い終えた専用target約1GBを削除した。
これは25課題の最新source一括認定や、残る共通生成母集団の合格ではない。

6Mの32効果・20,168件captureの保存原本を、正本core／WASI／EPFを解決した現在のrunnerで
監査した。約4分52秒の探索を明示停止し、未認定のままsource hash・実binary・profile・
停止記録を`feature-delivery-development-v1/epf-current-core-v1/`へ保持した。
実行したAIやWasm、課題globalとlocalside、元の採点結果は変更していない。

EPFの必要条件が、先行部分の全leafが同じ固定actorの場合にしか後続範囲を除外していなかった。
同じ先行部分へ別actorが参加していても、現在のprefixで不可避な固定actorの未完了操作は
残る。既存のsource祖先関係でこの証拠を調べ、同じ必須操作を待つ後続範囲をまとめて除外する。
余分なprefix走査を避けるため、既存の不変actor行で該当しないleafを先に除く。
新しい進行表・権限・storageは追加しない。全queryの有限比較20件と、独立actorの完了を
余分に要求しない正例、混在するparallel prefixの後続除外の負例が成功した。
Leanには混在するparallel子の必要なメッセージが順序付き継続へ残る2命題を追加して検査した。
これはsourceの命題と有限なRust照合であり、一般のRust refinement証明ではない。
同じ32効果原本の全体監査と、変更後EPFのnative／Miri／形式検査の完了は別に記録する。
移行25/48・残り23、今回のAI／Wasm再実行0、採点比較0を維持する。

論文用の予備一覧は、最終監査の明示的な参照を受け取る形へ修正した。各原本resultと
監査ファイルのhash、監査JSON内の位置を固定し、raw・runtime binary・source・課題・
inclusive usageとの既存照合は残す。10Eの親directoryの監査と、10M／11Eのmutation report
内のbaselineをそのまま読む。古い監査のコピーや改名、最新成功の自動探索、AI再実行はしない。
同じ16課題ID・23原本を保持し、対応を確認できる記録は11から14へ増えた。残る9記録の
不足・失敗は削除せず個別に残す。2試験・厳格Clippy・原本走査が成功し、最大243,793,920bytes。
一覧・CSV・固定reader source・参照mapのhashは`pilot-inventory-rust-v3/verified-v1/`に保存する。
これは既存の3監査を一覧へ正しく接続した結果であり、新しい実験や採点比較ではない。

### 6Mの全条件・現行実AI認定を完了（11月11日）

EPFの最初の展開候補が、外側の片方のarmでしか起きない内側rollの回数を、外側の全visitへ
平均していた。短いsourceで期待する操作列`[1,1,2,3,0,3]`の欠落を再現し、arm内の直接の
観測数を使って候補を構成するよう修正した。既存のworkspaceを使い、最終checkと完全な
候補探索は維持した。1〜3回の内側再訪と両外側順序の照合、native151＋doctest2、関連Miri7、
Clippy、no_std／thumbv6m、Lean6 moduleが通った。core、課題global／localside、意味論は
この修正で変更していない。32効果・20,168記録の保存原本は約11秒で監査を完了した。
原本・固定監査器・654入力のsourceコピー・Lean記録は`epf-selective-visits-v2/`に保持する。

一括の実Wasm検査は初期順序3条件を完了後、次の条件の準備中に2.75GB上限で停止した。
停止137を保存し、最終認定の共通入口を、同じ条件定数から1条件1プロセスへ割り当てる形へ
変更した。全6割当の欠け・重複・空実行を拒否し、外部からの診断selectorは認定入口では
受け付けない。localsideの操作や条件域は変更していない。新入口は通常45試験・Clippyを通過し、
同一sourceの初期順序4＋回復2条件をすべて完走した。93実効果・65,516 capture、共通採点・
protocol・EPF動的監査・実効果監査が成功し、違反0、記録変異747件を拒否した。
最大子孫メモリ1,157,136,384bytes。入力、各command、原本、before/afterのsourceとreceiptは
`output/experiments/feature-delivery-development-v1/wasi-current-core-v5/`へ保存した。

同じproducer binaryを使い、Luna xhigh 5人の現行6M・depth2の非採点pilotを1回実行した。
198.560103459秒、1,064,575 inclusive tokens、26実効果、27,243 captureで、レビュー拒否2回と
CI失敗2回から回復し、全guest・モデルの終了、protocolと共通採点完了、違反0を確認した。
モデルturnは5人が重なり、実効果ではレビュー／別参加者の実装とレビュー／テスト作成の2組の
区間が重なった。原本・実binary・source・prompt・実モデル設定・usage、EPFと実効果を結ぶ
独立監査は20.24秒で成功し、run記録変異11件も拒否した。モデル実行最大1,129,807,872bytes、
監査／変異の最大1,536,983,040bytes。原本と独立認定は
`output/experiments/model-qualification/current-core-feature-delivery-6M-depth2-v2/`に保存する。
これは抽象的な開発toolの結果であり、実compilerや成功率・比較優位の実証ではない。

6Mを26課題目へ加え、移行26/48・残り22・採点比較0とした。旧v8のpilotや途中停止を
現行版へ読み替えない。集約は`feature-delivery-development-v1/qualification.json`、手順と
保証範囲は`benchmark/ports/6M.md`。論文用の原本一覧v4は24 stream・16課題IDで、対応を
確認できる15記録と不足／失敗の残る9記録を保持する。本体の通常生成母集団の残り、最終sourceの
全family認定、11Mの工具部品から全6作業・検査・再作業への接続と、残る課題・正式実験は未完である。

### 11Mの全6作業の型付き入力と実部品の照合（11月11日、全体移行は未完）

既存の工具部品は3作業の要求と簡略結果だけを扱っていた。型付き要求を全6作業へ拡張し、
36個の型／targetの組と範囲外の型を検査した。工具部品の結果も、共通worldが返す部品ID、
作業者、作業台、attemptを含む実データ形式へ揃えた。初回はattempt1、再作業の結果は入力した
実際の前部品の次のattemptと照合し、部品IDとattemptの両方を古い組へ差し替える場合も拒否する。
新しいversion counter、所有者表、進行flagは持たない。これだけで前部品が実際の拒否検査から
来たことを保証したとはせず、その接続は全体globalと受信側の残作業とする。

最終sourceでnative17関数、既存の到着／完了30条件、Clippy、データ境界Miri2関数と
実receiverの不正wire4入力のMiriが通った。30ケース・90fixture効果の原本、binary、sourceと
1,356個の固定依存入力を`manufacturing-11M-development-v1/actual-part-boundary-v5/`へ保存した。
最大メモリはnative／データ検査381,878,272bytes、receiver Miri370,180,096bytes。
途中の余分なホスト試験を含めたMiri呼出しは停止しており、全Miri合格とは扱わない。
現在のsource・原本へ置き換えた一時診断世代は削除した。

工具の3作業に関するglobalは維持している。全6作業、作業台競合、工具2本の再利用、
品質検査とdispatcherの再作業指示を接続する全体global／直接localside、実Wasm／EPF／
実AIは未完で、移行26/48・残り22・今回追加のAI呼出し0・採点比較0を維持する。

### 12E/M/Hの共通worldを実材料の因果関係へ修正（11月11日、課題移行は未完）

12Eの旧worldで、食材の渡し先・料理名・盛り付けの食材がすべて誤った9操作でも、
課題完了・違反0になることを実際に再現した。準備中の材料も完了済みとして扱っていた。
12Hでも、未準備の別の品を加熱し、その後に別の料理を準備・盛り付ける12操作で同じ
誤認定を再現した。元の入力・結果・sourceを残し、Hibanaとは独立した共通採点器を修正した。

12E/Mの盛り付けは、正しい対象の完了済みbaseと受け渡す食材の実効果出現に結び付く。
12Hは実際の準備と加熱を結び、失敗した材料の準備を使い直さず、新しい準備から回復する。
加熱失敗の応答中にも別設備で新しい準備を進められる。最終effectが未完なら課題完了に
数えず、返したJSONの変更で実記録を改変できない。旧prep/cooked/done表と未使用の
acquire/release経路を削除し、実記録から状態を導く。worldは操作順やHibanaの仕様を読まず、
設備の重なりを観測するだけで呼出しの施錠・待ち行列・整列をしない。

12E/Mでは材料の因果だけを満たす9操作の直列順13,440通りをそれぞれ確認した。
12Hは作業者順・加熱設備・食材準備順・失敗深さ0〜2の144条件と、深さ1〜3の再準備、
Pending・対象差替え・異なる設備の並行処理を確認した。最終sourceで共通world177試験、
入力／実験表17試験、Clippy、48課題・825項目・既存実験表2,592件の照合が通過した。
649個の入力は検査前後と保存したsourceで一致し、最終gate最大93,929,472bytesだった。
通常CIの既存unittest discoverが追加試験を拾うため、別matrixや重複gateは追加していない。
実行証拠と最終sourceは
`output/experiments/kitchen-development-v1/common-world-v14/`、契約は`benchmark/ports/12E.md`。
これは共通worldの有限検査であり、12系のglobal／localside／実Wasm／EPF／実AIの
移行完了ではない。移行26/48・残り22・今回追加のAI呼出し0・採点比較0を維持する。

### 12Eの全体globalと実受信側をnative／Miriで確認（11月11日、Wasm移行は未完）

3料理人・設備所有者・食材所有者・盛り付け担当と3つのprivate I/Oを、一つのglobalへ
記述した。設備利用順6通りを残し、A/Bは実際のOVENとSTOVETOPのEndpoint、CはOVENのみを
借りて要求・実結果・返却を実行する。両設備から届いた値が実際に受理した要求と一致した後に
実処理を一度だけ呼ぶ。食材の実結果は、作った料理人の設備利用を待たず受取先へ送る。
各料理は自分に必要な材料から個別に完成し、全料理人の設備要求を待つbarrierを設けない。
globalの順序・所有権・完了の別表やflag、手順を隠すlocalside helperは追加していない。

最終sourceでnative5関数が通過し、全6設備順×食材遅延2種×入力到着の制御2種＝24条件、
216 fixture効果を確認した。最後の料理人の設備要求を先頭の料理の実際の完成まで止めても、
先頭の料理は完了する。オーブンの同時使用は最大1、各条件5回の設備返却を終える。
初回の試験は結合的parの分割幅が2ではなく4だったため修正した。本体・globalを変えたり、
期待値を下げて通したものではない。

設備の実受信2箇所それぞれへの不正4入力、合計8入力はDomainの呼出し前に拒否された。
同じ8入力のstrict-provenance Miriと、データ境界2関数のMiri、Clippyも通過した。
Miriのデータ検査は13.90秒、実受信検査は303.26秒。native／Miriの最大子孫454,410,240bytes。
24個のアプリ入力は検査前後と保存sourceで一致し、本体・EPF・WASIの1,356入力も既存の
固定原本と一致した。原本を重複保存せず、参照先とhashを記録した。
source・binary・各条件の実効果journal・検証記録は
`output/experiments/kitchen-development-v1/native-v7/`。通常CIには独自5関数と実受信Miriを接続し、
共通shell試験の重複実行は追加していない。transportの未回収診断記録を完全captureやEPF合格に
読み替えない。Wasmのguest localside・共通runner・完全capture／EPF／実AIは残り、
移行26/48・残り22・今回追加のAI呼出し0・採点比較0を維持する。

### 12Eを実Wasm・完全capture・実AIまで接続（11月11日、27課題目）

同じ11ロールのglobalと実受信側を使い、3人が各自のwashとChoreoFSから要求する。
各料理人は調理と食材準備をどちらからでも要求できる。次のcombine要求を受けた時点で
必要な実食材の到着へ合流し、前のシェル要求を他者の食材待ちで止めない。進行flag、
所有者表、別の状態機械や手順を隠すlocalside helperは追加していない。

全6設備順×2要求順と、不正・早期・重複入力／private file／chunk境界／個別完了の
13条件を別プロセスへ割り当て、実reportでも欠け・重複を検査した。117効果・完全capture
31,947記録が共通採点・EPF・実効果監査を通り、違反0、記録変異1,508件を拒否した。
最後の条件では、先頭の料理が次の料理人の設備要求前に完了する。共通worldの設定遅延は
0なので、実処理のthroughputや長い計算の重なりの測定とはしない。

既存ChatGPT契約のLuna/xhigh 3人による非採点pilotは22.156139秒、362,395 inclusive
tokens、9効果・違反0、完全capture3,627記録で、3人のmodel turnが重なった。
EPFと実効果監査が成功し、business記録116変異とrun記録11変異を拒否した。

最初のpilotは入力記録のrecipe／資源欄の欠落で未認定として保存した。元のDishConfig／
IngredientConfigから入力を取り込み直し、13条件とpilotを再実行した。そのpilotの最終
試験は監査reportのhash／完了欄の出力漏れで停止したが、判定規則を変えず欄を補い、
保存した同じpilotと13条件を独立に再監査した。三度目のAI実行はしていない。
現在のauditorは47通常試験・Clippyが成功し、source／binaryを測定runnerと分けて保存した。
実行binaryはreport修正前後で同一。本体・EPF・WASIの変更はない。

原本・hash・集約は`output/experiments/kitchen-development-v1/qualification.json`、
保証範囲は`benchmark/ports/12E.md`。旧Miriは不変の実受信側と8不正入力の範囲で維持し、
新guest全体のMiri成功とはしない。移行27/48・残り21・採点比較0。
12M/Hの移行、本体の通常生成検査の残り、全familyの最終source認定は引き続き未完である。

### 11Mの実検査・発注・再作業をEndpointとEPFで照合（11月11日、全体移行は未完）

交換部品の検査を、単なる前の部品との一致から、実際の不合格検査・実発注引数・完了した
発注結果との一致へ強めた。全6作業の型付き要求、任意JSONの省略可能なreason、部品と
肯定／否定結果を結ぶreplyを検査する。進行counterや所有者表は追加していない。

`global::job_quality`は、作業Endpointをworker→inspector→dispatcher→workerと別runtimeへ
往復させ、不合格ごとに発注・交換・新しい検査を通す。承認したEndpointは資源側へ退役させる。
各6作業×不合格0〜3回の24条件が成功し、156個のfixture効果と、完全capture 3,384記録を
EPFで照合した。内側の再作業で親resolverを再評価しないことも、各条件の実記録1回で確認した。
承認後の追加作業はframe／効果を出す前に即時拒否される。この閉じたactor/viewへの操作は
terminalな`PhaseInvariant`であり、回復可能な`Unavailable`とは報告しない。

同一sourceでnative 7関数（24再作業条件、既存30工具条件と4不正wire入力を含む）、Clippy、
データ境界Miri 4関数と、実Endpointの2回再作業を含むMiri 1条件が通過した。最大合計メモリは
576,684,032 bytes。MiriでEPF探索を重複実行せず、所有値・借用・captureの境界を検査した。
288個の認定入力と隣接依存1,356個のhashが実行前後で一致する。
原本・実行binary・source／hash・集約は
`output/experiments/manufacturing-11M-development-v1/quality-cycle-v2/qualification.json`。

これは各作業を独立に動かしたnative機構認定であり、全6作業で共有する3作業台・2工具・
1検査員を結んだ全体global、実Wasm、共通worldの実行や実AI認定ではない。EPFも実効果の
正当性を認定したとはしていない。次は検査中に別の作業・再作業が進む全体構成を接続する。
単に各作業の全再作業ループを順につなぐ逐次化やlocalside側の回避は行わない。
移行27/48・残り21・今回AI呼出し0・採点比較0を維持する。

全体構成の前段として、受付の継続を実作業の完了まで保持しない2作業の小さいglobalを
実Endpointで試した。6 runtimeで同じ作業台Endpointを往復・再利用し、先の作業が終わる
前に次の要求を受理でき、作業台での実作業自体は重ならない。140記録の完全captureをEPFで
照合し、Clippyも通過した。20 msは試験用の制御遅延で、実モデルや性能測定ではない。
`stage-composition-v4/result.json`に入力・binary・原本を分けて保存した。これは採用済みの
全体実装ではなく、承認に基づく退役、2工具の動的配分、6作業との合成を残す設計試験である。
試作の別globalや実行経路を製品・CIへ追加していない。

### 11Mの2工具の反復貸出と、観測portの境界（11月11日、全体移行は未完）

2本の実Endpointを6 runtimeで貸出・返却し、両方の貸出中の実要求へBusyを返し、先に
戻った工具を他方の完了を待たず再貸出する試作を加えた。返却通知はglobalでもowned返却の
後へ置いた。所有者が保持するのは実際のOption<Endpoint>と受信したI/Oだけで、保持者表、
進行flagや別schedulerを追加していない。初期A/B到着は固定し、先に戻る工具の両方と、
Cへの1・2・4回の貸出を6条件で確認した。試験用service操作26回、完全capture 2,340記録。
これは共通simulatorのdo_jobではなく、実WasmやAIを使わないnativeの機構試験である。

試作は、本体が確定済みの受信分岐をtapへ記録するとき、別のlogical viewに属するlaneの
portを現在のviewから取得してpanicする経路を露呈した。actorの観測はそのrendezvousの
tapへ出し、localsideの順序変更や受理規則の緩和で回避しない修正を入れた。焦点を絞った
actor観測6試験は通過した。同じ本体sourceで通常Rust gateのStrict Clippy、全native、
doctest、文書、no_std、必須Miri 47件が通過し、追加のactor観測Miri 6件も成功した。

新しいcaptureのEPF照合は、当初2条件が成功し、反復貸出の探索を3分超で中断した。
原因は、同じvisitで既にcommitした操作をrollの再開先に割り当て、その後の無関係な
並び替えまで探索することだった。再開に対応する未来のcommitは既存のpointを再利用
できないため、その候補を早期に除く。sourceの意味、runtimeの受理、最終checkerは変更しない。
修正後は保存済み全6条件・2,340記録のEPF監査が成功した。元のcapture、checkpoint、
到着順の原本とhashは同一で、nativeやsimulatorを再実行していない。
EPFのnative 152件、doctest 2件、Clippy、host/thumbv6mのno_stdとLean全6moduleが通過。
追加した複数producerのlocal/send/receive再開と古い出現の拒否の試験はMiriも通過した。
Leanには、観測済みpointは任意のsuffixでも再commitできない補題を追加した。
初回の未完検査・停止証拠は保存し、修正後のsource、binary、全6原本のhashと再監査結果は
`reactive-pool-v3/audit-reader-v2/qualification.json`へ分けた。
`reactive-pool-v3/development-evidence.json`にbinary/source hashと、従前1,356依存入力から
変わったcore sourceと検査入力を保存した。全6作業、3作業台、検査・再作業の合成、Wasm、
実AIは引き続き未完。移行27/48・残り21・採点比較0を維持する。

同じlistener laneと委譲viewの構造を共通生成器のcontract catalog 24へ追加した。
1資源の小さい構造で、2種類の有効なメッセージ割当について実Endpoint 80実行、
whole-call 34,936比較、実際のborrow位置で59,456 queryのRust/Lean照合が通過した。
無効な2割当は実compileで拒否し、不正記録・欠落・重複・scheduleの変異拒否も完走した。
出力は`output/core-validation/reusable-pool-contract-v1/`。これは追加した1カタログの
全割当であり、通常の全カタログや拡張認定の完了へ読み替えない。

### 14Mの共通採点器を実データへ置き換え（11月11日、アプリ移行は未完）

旧14Mは、規制承認を先に実行して分析を逆順に行う13操作でも、完了・違反0を返した。
同一の13入力を保存し、実データに基づくv15で未完になることを確認した。14Eと14Mは
一つのmaterial/result実装を使い、実際の化合物・細胞・分析結果・判断・規制承認を対応付ける。
規制拒否後には新しい判断が必要だが、追加分析の種類や独立した分析の順序は指定しない。
失敗・実行中・置換後の結果を以前の成功で埋めず、記録JSONは返却値から独立して所有する。
シミュレーターはHibana、global、EPFを参照せず、どの手法にも同じ因果・資源条件を適用する。

共通回帰188関数、14Mの8,640通りの有限な順序/判断カタログ、各必須入力の欠落、
1〜3回の分析失敗/規制拒否、11入力それぞれの置換と実行中状態、古い承認、不正JSON、
返却JSONの改変が通過した。異なる装置の実呼出しは重なり、同じ装置の呼出しも勝手に
逐次化されず競合を記録する。入力検査器17件、Clippy、fmt、48課題/825項目のmanifest照合も通過。
共通条件をv15へ更新し、以前のworldで行った試験やpilotをv15の認定へ読み替えない。
原本、337入力の凍結source/hash、再考3回の実journalとコマンド結果は
`output/experiments/regulatory-discovery-14M-development-v1/qualification.json`へ固定した。

これは採点器の認定であり、14Mのdelegated global/localside、実Wasm、EPF/効果監査、
実AIの実験は未完である。14Hのworld/アプリも今回の認定に含めない。
11Mの全体構成には、独立した仕事をまたぐ共有工具の貸出を現行の所有権規則が拒否する
設計課題が残る。貸出表や固定順序での回避や、検査条件の緩和は採用していない。
移行27/48、残り21、今回のモデル呼出し0、採点比較0を維持する。


### 14Mのbusiness globalと効果受信側 — 部分native認定（2026-11-11）

採点器だけでなく、11分析、失敗時の再試行、規制拒否後の新しい判断、
biology/toxicology/両方/追加分析なしの再考、承認後のdocumentationを一つのglobalへ実装した。
装置の実Endpointで要求し、実結果を得る前に分析の継続をreceiverへ渡す。
失敗では担当者へ返し、成功ではleadへ渡す。承認時はstudyをchemistへ渡し、
古い分析と再考へ戻る権限を閉じる。globalsの補助宣言を除き、通信手順は
HPLC/cell/mass/regulatoryのlocalside本文へ直接書いた。所有者表、進行flag、
別のschedulerやcallback内の手順は追加していない。本体は変更していない。

9参加者の全体global投影、native20関数（新規7＋共有I/O13）、Clippy、fmtが通過。
実QC receiverは3 runtime間で0〜3回失敗後に成功し、継続を受け取ったleadからの
古い分析要求はfirst pollで拒否、frame送出0を確認した。同じ4条件とデータ3試験は
strict-provenance Miriでも通過した。最初のMiriはnative用5秒の試験deadlineで停止した。
Miriの完走側のみ既存fixtureと同じ600秒へ揃え、負例は時間待ちを使わずfirst pollの
結果を検査する形へ強めた。失敗回数や検査する操作は減らしていない。

試験時点で共通worldの337入力は前回凍結版と全hash一致。その後はexperiment-contractの
14M進捗説明のみ更新し、残り336入力と共通採点の実装・条件は不変である。結果を
`output/experiments/regulatory-discovery-14M-development-v1/application-v1/qualification.json`へ
source/hashとともに固定した。通常CIへ新規native/Clippyとfocused Miriを接続した。
CIの送信・成功は今回主張しない。

残るのは5人のWasm localside、起動、全体の実Wasm実行、EPF/実効果監査、実AI認定。
今回native実行したのはQC経路であり、再分析/review receiver全経路の実行認定ではない。
14Mは未完のまま、移行済み27/48、残り21、モデル呼出し0、採点比較0を維持する。

### 14Mの全体Wasm・実AI認定（2026-11-11、28課題へ更新）

前節の部分native認定を更新する。5人それぞれのWasmとChoreoFS、4効果受信側、
25個の投影programを通常runnerへ接続した。解析・再試行・再考・終了までのEndpoint操作は
各localsideへ直接書き、別の所有者表、進行flag、workflow wrapperを追加していない。
共通worldのv15と物理的な資源条件は変えていない。

固定sourceの実Wasmは12条件すべて完走した。go/no-go、装置利用順、拒否後の追加分析なし・
biologyのみ・toxicologyのみ・両方、連続3回の規制拒否、分析・documentation失敗、
深さ1/2、通常の1〜5秒の共通遅延を含む。合計231実効果、全条件で共通score完了・違反0、
完全captureのEPF監査と別の実効果監査が成功した。記録変異は1,890件を拒否した。
これは12条件の有限な機構認定であり、全familyや全scheduleの認定ではない。

Luna xhigh 5人の深さ2 pilotは288.26033525秒、inclusive tokens 2,005,631、実効果38件で
完走した。AIが選んだ2回の再分析を通り、最大3件の実効果、15組の独立効果の重なりを
記録した。全参加者が終了し、共通score完了、違反0、EPF/実効果/実モデル記録の監査が成功。
同じ原本の業務記録変異751件と実行記録変異11件を拒否した。変異数を実行時攻撃の阻止数と
扱わず、今回のpilotと機構試験を採点比較へ算入しない。

初回pilotのguestとAIは完走したが、監査出力のprojection hash等の報告項目が欠け、
外側の試験は終了code 101だった。この失敗原本は保持した。報告と共通監査入口だけを修正し、
同じ原本を完全に再監査して成功した。AIを再実行した値へ置き換えていない。
実行binary、2,096入力の凍結source/hashと、監査器の2ファイルの差分を分けて保存した。

実行中に本体のresolver観測が別logical viewの存在しないlane portを参照してpanicする
問題も修正した。actor全体の観測を既存rendezvous tapへ出す変更で、受理規則やglobalは
緩めていない。関連native35試験と、入れ子・委譲resolverの両選択を通るMiri 1関数が成功。
同じ失敗構造を生成検査で直接踏む補充と、本体の通常全gateの完了は別の残件である。
個別Miriや12条件の実Wasm成功で、それらを合格したことにはしない。

現在のアプリのfocused native7試験、runner47試験、Clippy、fmtも通過した。
以前のnative20関数/Miri4関数はapplication-v1の限定されたQC・データ境界の証拠として
残し、新しい全体WasmをMiriで検証した数字へ広げない。集約と原本への対応は
`output/experiments/regulatory-discovery-14M-development-v1/application-qualification-v1.json`。
移行28/48・22family・残り20、最終sourceの全family認定未完、採点比較0へ更新する。

### 14Hの共通worldを実材料・3試料へ修正（2026-11-11、アプリ移行は未完）

旧採点器は、規制承認とdocumentationを先に実行し、分析を逆順に行う13操作でも
完了・違反0を返した。directorの実判断も不要だった。BIOLOGICAL_SAMPLESは同時利用の
capacity 3として置かれるだけで、試料を一つも消費していなかった。旧sourceと同じ反例を
保存し、v16が拒否することを確認した。特定手法の不正や比較上の優位を示す数字ではない。

実compound/formulation、完了済みの正/負のbio/tox結果、臨床評価、規制判断、directorの
実outcome、最終profileをjournal上の入力に結び付けた。有効なbio/tox試行は失敗も含めて
3単位の試料を消費し、culture prep、不正入力や試料切れで追加消費・補充をしない。
試料切れは実データに基づく中止へ進めるが、肯定的な継続の根拠にできない。continueと
abortは別の最終outcomeとして保存し、早いabortやretryだけで完了にしない。
同じ根拠のholdをroutine reviewで消さず、両装置による深いreviewか実際の新しい科学結果を
必要とする。独立順序を指定せず、同時呼出しは実行して物理競合を観測する。

元の目的にあるdirector判断を全手法共通のtoolへ追加し、同じ臨床評価を二重に要求する
架空のmass-spec done項目は除去した。7人・4装置・3試料は維持した。遅延は全手法共通の
synthetic 1〜5秒、判断記録は0秒と明示する。旧PharmaSimのdone mapとacquire/release経路を
削除し、14E/Mと実結果を読む処理を共有した。アプリの進行管理やHibanaをworldへ入れていない。

共通202試験が成功。14Hは14関数で144通りの独立順序/終端判断、13必須入力の欠落、
24置換/実行中状態、8失敗置換、1〜3回のhold、最後の試料をbio/toxどちらが使うか、
負の実結果からの中止、返却JSONの改変と両装置の物理競合を含む。共通queryの抽出後も
14Mの8,640有限ケースを含む既存試験が通過した。Rust入力検査17件、Clippy、fmtと
48課題/825 checklist項目のmanifest照合も通過した。

結果とsource/hashは
`output/experiments/multi-round-discovery-14H-development-v1/qualification.json`へ保存する。
これは共通採点器の認定であり、14Hのglobal/localside、実Wasm、EPF/効果監査、実AIは未完。
移行28/48・残り20・採点比較0を維持し、以前のv15 pilotをv16での認定へ読み替えない。


### 2026-11-12: 14M resolver observation coverage and 14H actual negative receipts

The 14M observer defect now has a small real-Endpoint reproduction with the same
public send contract on inactive lane 0 and selected parallel lane 1. The outer
choice and sibling finish before `par()` narrows the resource; both inner resolver
outcomes then audit through the actor's tap. Eight observation tests pass native
and strict-provenance Miri. The generator retains cross-peer aliasing and adds
same-peer entry aliasing, so the lawful case does not collapse into an unrelated
parallel-selector rejection. Its 117 tests pass. The new catalog source completes
48 executions/384 calls in four accepted shapes, 130,560 admission attempts and
57,792 parked-preview attempts with no runtime/model mismatch; one additional
shape is independently rejected by compilation. Current controls, Clippy and
assignment tests pass. Lean source semantics were unchanged. This is focused
coverage, not completion of the expanded whole-catalog core gate or a general
Rust-to-Lean refinement proof. Core evidence is
`../hibana/output/verification/regulatory-resolver-port-qualification-v1/qualification.json`.

14H preparation exposed a data-interface gap: failed/exhausted attempts have no
positive artifact ID, but clinical inputs must identify those actual occurrences.
Common world v17 returns `event_id` from the existing owned simulator journal,
including failed attempts and holds; it does not invent a positive artifact, a
new sample or another counter. The common suite passes 203 functions and the
input verifier passes 17 tests for all 48 tasks/825 original checklist items.
Earlier v15/v16 results retain their own source labels.

The initial 14H Rust data contract now checks material/result identity, actual
negative clinical evidence, the admitted director outcome and final documentation.
Five focused functions pass within all 18 library tests, with application test
Clippy and formatting. The 14H global/localsides, real Endpoint effect receivers,
Wasm certification, EPF/effect audit and model pilot remain unfinished. Migration
count stays 28/48 with 20 remaining; new model calls and scored runs are zero.
The source-bound qualification is
`output/experiments/multi-round-discovery-14H-development-v1/occurrence-values-v17/qualification.json`.

### 2026-11-12: 14Hのglobalと全32投影、const負荷の解消

14Hの業務globalは206イベント、7人のChoreoFS入出力を含む実際のglobalは388イベント。
一つの科学round反復、実試料の要求、両装置による深い審査、最終判断後のSTUDY返却を記述し、
全32プログラムが同じsourceを持つことをnativeで確認した。通常審査と深い審査は、両装置所有者が
受け取る実際の審査要求で区別する。localsideのflagや順序変更で補っていない。
const評価のforbidを保ったまま全投影をビルドでき、最初の全プログラム試験buildは34.43秒、
全子孫のguard最大330,498,048 bytesだった。clean-build性能や通信runtimeの計測とは区別する。

本体のreentry検査は、同じactor接続と分岐の再走査を減らした。実際の保持者で連結成分を作り、
両枝の成分とseedが等しい場合だけ共有する。権限検査も受信知識とguardが等しい場合だけ共有し、
異なる場合は元の選択列挙を保つ。4actor・3edgeの110,592グラフを独立した到達可能性と照合し、
131,072条件で共有した枝と個別の枝の判定が一致した。component coverの依存閉包をLeanで証明し、
2/3イベントの3つのKaniで任意のu8 actorも検証した。Kaniの4word走査には終了確認を含むunwind 5を
使い、入力域とassertionは保った。検査manifestに既存cfg名の宣言漏れも修正した。
本体nativeは850成功・別実行の28未実行、最終差分試験、Clippy、Leanが成功した。
これは通常生成カタログ全体や本体全gateの完了ではない。

アプリは25 native試験、Clippy、fmtが成功。実ChoreoFS closeが返した要求のデータ検査も16操作へ
追加した。共通worldの203試験は、310入力と134 Python source/testのhash、およびmanifestの
同一内容を確認して既存のv17結果を引き継ぎ、同じ試験を再実行していない。現在の正本はCargo
metadataでも本体一つ・Tokio 0で、1,096 sourceと試験binary/hashを固定した。
証拠は`output/experiments/multi-round-discovery-14H-development-v1/global-v2/qualification.json`と
`../hibana/output/verification/reentry-components-v2/qualification.json`。

14Hの直接localside、全体実Endpoint/Wasm、効果/EPF監査と実モデルは次の実装であり、移行済みへ
算入しない。移行28/48・22family・残り20、今回の実モデル追加0・採点比較0を維持する。

### 2026-11-12: 14Hの実Endpoint受信側とpreview照合

製剤と質量分析の受信側を実装した。受理済みの要求、返った実装置Endpoint、実材料を
効果の直前で照合し、処理・返却・終端までの通信をlocalside本文へ直接書いた。
製剤のnative10条件は0〜3回の失敗、移動後の古い材料からの操作、引数の差替えを含む。
臨床評価のnative8条件では、bio/toxの正負4組が完走し、材料を変えた4組は臨床効果前に
拒否された。同じ本番受信側の8条件がstrict-provenance Miriでも通過した。
他の科学者の結果はfixtureで与えており、共通worldの全効果を実行した数字ではない。

独立装置の分析を別の並行列にし、stabilityを無関係なbioassay待ちから外した。
深い審査では、効果前に装置を返す構造を改め、STUDY_SERVICEへ両装置Endpointを実際に
渡し、実結果の後で返すglobalにした。この追加4転送を含む業務210・全体392イベントの
全32投影と、ライブラリ29試験が通過した。深い審査の受信側自体は実装・実行が残る。
製剤・質量分析以外の受信側、7人のWasm localside、EPF/効果監査、実AIは未完であり、
移行28/48・残り20・今回のモデル呼出し0・採点0を維持する。

本体の生成検査は成功したofferをdropする前と、次のtyped call直前を別々に観測するように
修正した。実際に解放する操作leaseだけを除いて、進行・所有権・選択・rollのcreditが
変わっていないことを照合する。通常入力のpreviewに、存在しない分岐知識を要求しない。
同じ契約再利用カタログの4形・48実行・384呼出し、130,560 admission試行と58,624 preview
試行でモデルとの差は0。生成した既存alias試験も、新しい観測経路をMiriで通過した。
Miri専用の観測serializer除外は取り除いた。非Miriでの関数本体・判定意味・入力域は不変の
ため、カタログ結果はその元source/hashとともに引き継ぐ。全カタログや本体全gateの
成功へは広げない。原本は本体の`output/verification/ordinary-offer-v1/`、受信側の記録は
`output/experiments/multi-round-discovery-14H-development-v1/receivers-v2/`に保持する。

### 2026-11-12: 14Hの再試験と4受信側

審査の受信側を実装し、通常・深い審査の切替、4回の審査、保留、引数・判断の差替えを
native9条件で確認した。審査中は実際のSTUDY/HPLC/MASSのEndpointを保持する。
以前の9条件のMiri記録は、そのsource/hashと元の観測器に限定して引き継ぐ。
現在のnative観測器は無関係なcommitを除き、同じsource行を直接参照する。

細胞実験は同じ材料を科学者へ返して再試験できるglobalへ変更した。終えるときは材料を
実際に返却し、その最後の測定結果だけを臨床へ渡す。失敗のたびに新しい製剤を作って
既存の肯定的な測定を捨てる必要はない。試料の補充やアプリ側の在庫・進行管理は加えていない。
この4操作の追加で業務214・全体396イベントとなり、全32投影が通過した。

本番の細胞受信側はnative/Miri12条件が成功した。再試験、枯渇、後の陰性による古い陽性の
置換、培養の失敗、独立したpurityの受信、誤材料の効果前拒否、返却済み材料の送信前拒否を
含む。実際のcommit記録で、Pendingになる効果の前後とも両装置と材料を保持していることを
確認した。Miri前後のsource/hashは一致した。科学的な結果はfixtureで与えており、共通worldの
全効果や実Wasmの成功を意味しない。共通world v17は変更していない。

ライブラリは32試験、4受信側のnative条件は計39。アプリ・独立受信側のClippyとfmtも成功。
通常CIは臨床・審査・細胞のMiriを同じ実行へまとめた。前の8条件・9条件のMiriを、
今回新しく再実行した結果や全体認定へ読み替えない。今回の記録は
`output/experiments/multi-round-discovery-14H-development-v1/assay-retry-v1/qualification.json`。

その後HPLC受信側を、QC・purity・stability・審査への貸出・5材料の保管・最終文書化まで
実装し、型検査とアプリ/独立受信側のClippyを通した。実通信の認定はまだ行っておらず、
4受信側の凍結sourceとは別に`hplc-v1/qualification.json`へ記録する。
残る14Hの作業はHPLCの実通信試験、7人のWasm localside、起動・全体実行、EPF/効果監査と実AI。
14Hはまだ移行済みに算入しない。移行28/48・残り20、今回のモデル呼出し0・採点0を維持する。

### 14H: 純度の因果関係と責任者への実データ、2026-11-12

早い毒性要求が送出された再現に対し、globalを「実際の純度結果の受渡し→毒性試験」に修正した。
細胞側のアプリ内の並行受信待合せも除去した。独立した生物試験と安定性試験の先頭要求は進み、
生物試験を始めず実HPLC Endpointを安定性側へ貸せることも実Endpointで確認した。別viewの
先行操作を待つPendingは正当なので、早い毒性要求については送出されないことを検査する。
本体のruntimeやLeanを変更して通したものではない。

五つの実受信側を扱うnative部品条件は47件（HPLCと審査をつないだ8条件を含む）、
評価prefixは4条件。全35通常テストとapplication Clippyが通った。審査側から責任者へは
既存の結果メッセージで実際の科学データ一式を返す。責任者が別表や仮の記録を組み立てず、
受信した値を最終通知へ渡す。化学・製剤・責任者の3人分のWasm localsideは実装・型検査済みで、
残る4人と全体起動、実Wasm、EPF／実効果監査、実AIは未完。

途中のMiriは責任者データの送信内容を仕上げるため明示的に停止した。細胞関数は12条件を
終えたが、HPLCの全割当とプロセス終了時のleak検査は未完なので成功認定しない。
停止status 143、変更前後が一致した入力hash、後で変更した4ファイルの当時の原本と
最大928,989,184 bytesの記録を保存した。参加者の送受信を仕上げてから必要箇所を最終検査する。
以前のMiri成功を今回の全sourceへ読み替えない。移行28/48・残り20、今回のモデル呼出し0・採点0を維持する。


### 2026-11-12: 14Hの7人実Wasmと再考、12機構条件

7人のlocalsideと全体起動を実装し、共通worldの深さ0/1/2、実試料の再試験と枯渇、
実際の責任者判断、2周目の研究、通常／深い再審査、文書化の再試行を実Wasmで検査した。
初期globalは新しい試験Endpointを科学者へ直接渡したため、設備が次の製剤結果を待つ間も
保持された内側rollの要求を送れた。このoffer自体は本体／独立モデルの意味と一致する。
設備が実際の製剤を受信した後に試験Endpointを科学者へ渡すglobalへ修正し、localsideを
その投影へ書き下した。各並行枝は受信から転送までを直接記す。通知、進行flag、所有者表、
別の状態機械は追加していない。独立した試験の開始順と並行性は保持した。

最終の業務globalは218イベント、ChoreoFSを含む全体は400イベント、32投影である。
long_running_const_evalをforbidのままビルドした。12条件は同じruntime、projection、
driver、source manifestに固定し、各条件を別プロセスで1回ずつ実行した。先に通った
再考条件8は同じ入力の結果を引き継いだ。全12条件でprotocol完了、共通score完了、違反0、
EPF動的解釈が成功。実効果192件、capture 106,584件である。
`round-matrix-v5/authority-mechanism-qualification-v10.json`に各原本とhashを保存した。

本体の並行借用の修正は、返却済み材料を次の親visitで受けるfocused 3試験と、追加した
生成ケースのsource照合を別に記録した。これは通常CI契約の全体成功ではない。
独立した実効果監査、現sourceのMiri、通常runner／promptへの接続、実モデルpilotは未完。
機構試験を採点やAI実験へ算入しない。移行28/48、残り20、モデル呼出し0、採点0を維持する。

### 2026-11-12: 14Hの実受信出現と実効果を照合

同じ12条件の原記録を変更せず、独立した科学データ監査とEPFを合成した。
192件の実効果が必要とした231件の要求受信について、EPFが検査した動的source occurrenceと
実際のEndpoint receive commitを対応付けた。producer/ordinalの再利用と、実効果より遅い
受信証拠は拒否する。元の実Wasmやworldを再実行して件数を増やしたものではない。
実際の結果、引数、材料lineage、WASI close、要求・返信の一対一対応も同じ監査で照合した。

この時間関係は、本consumerがcaptureとeffect要求を同じI/O ownerのqueueへ同期的に
入れる契約を前提にする。遅延収集した任意の診断ログや任意のRustのrefinementを証明する
ものではない。実行の解釈と委譲された権限の履歴はEPFへ任せ、監査器はアプリの操作権や
進行を変更しない。core、global、localside、worldの追加修正は行っていない。

再考条件8、保留審査条件10、文書化再試行条件11の保存記録に852件の変異を与え、
すべて拒否した。うち51件はcapture内のproducer順と全非capture記録の相対順を保ったまま、
受信を含むcapture suffixだけを実効果の後へ遅らせる。EPFのsource検査と通常のframe/effect
journal検査は通ることを先に確かめ、受信と効果の対応違反で拒否させた。先行する変異案は
effect自体を前へ移し、既存のframe受信条件も同時に壊したので独立した対照には使わない。
852件は監査用の記録変異であり、852件の実行時攻撃を阻止したという数字ではない。
原本hash、監査器source/binary、各結果は
`round-matrix-v5/effect-receive-audit-v8/`へ保存した。

通常の機構試験入口も同じ監査を呼び、14Hの12条件を各一プロセスへ割り当てる。
欠落・重複・空の割当、実効果検査の未完、異なるruntime/projection/driverの混在を
集約で拒否する。将来の全family入口へ登録したことと、全familyが現版で完走したことは
区別する。通常runner、snapshot、全run監査、7人分の自由なshell promptも接続した。
fixtureの固定手順をAIの応答として代用せず、実結果から再試験・深い審査・再考・終了を選べる。

runnerの通常49試験、現sourceの14H application 23試験、runner/application/独立受信側の
Clippyが成功した。最初のClippyで見つかった未使用のarchive戻り値の束縛を削除した後、
Clippyとbuild、application試験を完走した。現sourceのMiriは28部品条件を含む3関数で
実行中であり、まだ成功へ算入しない。入力closureとcompilerは
`round-matrix-v5/receiver-miri-v9/`へ保存した。実モデルpilotは未実行。
移行28/48・残り20、採点0を維持する。本体の全通常gateもfocused成功とは別の未完項目である。

### 2026-11-12: 14Hの最終receiver Miriと7人実AIを認定

上の実行中だったMiriは、mass/clinical 8、cells 12、HPLC/study 8の計28条件を含む
3関数と、プロセス終了時のleak検査まで成功した。920入力の前後hashは一致し、runnerの
入力・監査形式だけを修正した後も全入力の現在値が一致する。最大1,371,602,944 bytes。
3,719.49秒はMiriが報告した解釈時間であり、native性能の測定値にしない。

実AIの初回は143.119385333秒で失敗した。化学者がmolecular identification後に終了し、
最終documentationがないため、12実効果・元score未完で拒否された。原本を残す。
runnerが14Hの自由な材料入力を空の固定callsとして記録していた不足を直し、元のtools.jsonの
schemaと担当資源をそのまま保存する。promptは既存課題の最終文書化要件を明記する。
global、localside、worldは変更せず、順序の回避策や進行管理は追加していない。

2回目は7人の実Luna/xhighが193.161955167秒、inclusive 1,913,110 tokensで全員完了した。
文書化を含む13実効果・元score完了・違反0。実モデルturnは7人が重なり、独立した実効果の
最大同時数3、重複する組4件を原記録から確認した。モデルの終了とprotocol完了を分け、
EPF、実際の受信commitと効果の対応、科学的lineage、元score、全run記録を照合した。
必要な受信commitは15件。採点比較や、版の異なる2回を平均した成功率にはしない。

2回目の終了後、監査側にも固定callsを前提とする形式不一致が残っていた。監査器だけを
元の自由schema／資源との完全一致へ修正し、focused 3試験・Clippyと同じ原本の再監査を
通した。旧形式の失敗ログ、凍結runner、元manifest、新reader binaryとaudit.rsの差分・hashを
`round-matrix-v5/model-audit-reader-v13/`へ保持した。実AIの3回目は実行していない。

同じ成功rawへの業務記録226変異とモデル・usage・成果物などの11変異をすべて拒否した。
226のうち13件は、producer順と非capture順を保って受信証拠だけを効果後へ遅らせる対照。
これらは監査用の記録変異で、実行時攻撃の阻止数ではない。先の12機構条件192効果／
231受信・852変異は、その元source/hashで引き継ぎ、再実行の件数として加算していない。
新metadataと通常入口の接続は追加で条件9を通したが、13番目の新条件とは数えない。

`output/experiments/multi-round-discovery-14H-development-v1/application-qualification-v1.json`
に全証拠を照合して固定した。14Hを版に紐づく移行済みへ算入し、29/48・23family・残り19、
採点0へ更新する。受信側の有限Miri、機構試験、実モデルpilot、監査の範囲は別々に保つ。
本体の全通常gate、全familyの最終source認定、残る19課題、論文の本実験は未完である。

### 2026-11-12: pilot原本一覧と3Mの共通採点を更新

同じRust一覧器へ14Hの2記録を明示的に登録し、29原記録・19課題IDへ更新した。
成功した14Hだけを選ばず、未完の初回とその入力版も保持する。過去のモデルや監査器を
再実行したものではない。一覧は最大79,560,704 bytesで完了した。以前に削除した旧runner
実行ファイルの不足と、未選択の合格監査はそのまま明示され、厳密なarchive完全性は
2件・未充足27件である。これはタスクの成功率ではない。hashの期待値を変更して
過去の不足を隠さず、新しい14Hのruntime/runner/raw/source/readerを保持した。
`output/experiments/pilot-inventory-rust-v6/`のCSV/JSON/tableを論文用の版別台帳にする。

次の未移行3Mには、論文を先に公開してから調査・執筆・事実確認・編集を実行しても
完了になる共通採点器の不足があった。修正前の実試験で再現し、原source/hashを保存した。
既存のDocumentSimの実データjournalへ3Mも統一し、research artifact→本文／参考文献→
事実確認→編集の実入力と、公開時点での証拠を照合する。追加調査は自由にできるが、
担当する本文の必要な調査結果を別の作業で代用できない。完成した章から進められ、
他の著者の開始や全文の完了を待たせない。worldはHibanaや連携順を参照しない。

資源定義も既存のmetadata一つへ統一し、別のDOCUMENT/DATABASE/FIGURE_STORE表を除去した。
変更途中の全試験で、図の既存試験がmetadata未接続を検出したため、全document constructorを
同じbindingへ揃えた。現在の210共通試験と17入力tool試験、fmt/Clippyは成功する。
3Mの新7関数は、著者6順×各著者の本文／参考文献2順の48条件、事実確認／編集の拒否
各1〜3回、早い公開、未完／差替え入力、返却JSONの独立性、実際の資源競合と独立動作を含む。

共通契約をv18へ更新し、48課題・825 checklistの元入力をRust工具で照合した。
最終入力hashは前後一致、最大203,259,904 bytes。初期の失敗とstale manifestの拒否も
保存し、未完の結果を成功へ読み替えていない。資格記録は
`output/experiments/research-paper-3M-development-v1/qualification.json`。
3Mのglobal/localside、実Wasm、EPF/効果監査、実AIは次の実装であり、移行数は29/48のまま。
14Hなどの既存pilotをv18へ読み替えず、採点比較0を維持する。

### 2026-11-12: 3Mの実Endpointによる章の準備・再審査部品

3Mの章の実Endpointを、著者から事実確認・編集の実効果ownerへ移し、拒否時に著者へ
返して実修正・新しい事実確認を行うglobalとreceiverを追加した。事実確認と編集を一つの
章内反復へまとめ、各試行で元の保持者へ戻す。確定後の保管への移転は反復の外に置く。
実際の否定的な事実確認結果も編集者へ共有して分岐知識を明示し、その送信と著者への
返却・修正は並行とした。一回しか使わない段階別global関数は本文へ戻し、二種類の
差戻しが再利用する修正choreographyだけを共通化した。localsideに段階helperや進行flag、
owner表は追加していない。本体・モデル・共通採点の規則は変更していない。

native 6関数・実Endpoint 7条件、Clippy、strict-provenance Miri 3関数・同じ7条件が成功。
3人別々の準備、受理後の本文差替え、事実確認/編集拒否(1,0)/(0,1)/(3,2)、実結果と逆の
成功/失敗payloadを試し、確定後の編集はframe送信前に拒否する。Miriは終了時のリーク検査
まで完了し、930ファイルの前後hashが一致した。最大725,073,920 bytes、停止2.75GB。
最初のMiriはnative用30秒watchdogに当たったため、同じケースを解釈実行用600秒で再検査。
nativeの30秒と入力・assertion・メモリ上限は維持した。停止記録と成功記録は分けて保存した。
source、binary、compiler、記録は
`output/experiments/research-paper-3M-development-v1/quality-v1/qualification.json`。

これは1著者ずつの制御された部品試験であり、3著者の共通DOCUMENT/DATABASE配分、論文全体の
公開、Wasm・EPF・実AIは未完。移行29/48・残り19、今回の実AI呼出し0・採点0を維持する。

### 2026-11-12: 3Mの最終公開receiverと章の早い確定を検査

最終公開のglobal/receiverに、3章の実Endpoint、実本文・参考文献・事実確認／編集結果、
DOCUMENT/DATABASE両方の実要求を接続した。編集側から両資源を移してからsnapshotを渡す。
実要求と受理済み引数の一致、3章の現在の肯定的な検証結果を確認して初めて公開を実行する。
実効果の後の返信照合と、実効果前の受理検査は区別する。

公開の実Endpoint 5条件・純粋データ25条件、strict-provenance Miri 3関数・同じ5条件が成功。
946入力のhashが前後一致し、最大523,485,184 bytes、停止上限2.75GBを維持した。
共通world実応答とのnative bridgeも7条件が成功した。ただし119効果のうちHibanaで受理したのは
最終公開7件で、先行112効果は制御されたfixture準備である。各rawに同じ12 world source hashを
記録した。3著者全体のアプリ、並行性、Wasm、EPF、実AIの成功には数えない。
資格記録は`output/experiments/research-paper-3M-development-v1/publication-v1/qualification.json`。

続いて、否定的な審査の後に修正しただけの章を、再審査前に保管へ移せるglobalの不足を
実Endpointで再現した。既存の肯定的なFinalForFactを反復後の保管移転より先に必須とし、
送信元を実結果のreceiverへ揃え、受信側で現在の本文・事実確認結果まで照合する。
localsideの完了待ちだけに頼る修正ではなくglobalの受理条件を直し、追加の効果・flag・所有者表は
置いていない。native 12関数が成功し、事実確認／編集の両方で、否定後の早い保管がframe送信前に
拒否される。修正前のsourceと失敗記録は`exit-v1/before/`、変更後の検査は`exit-v1/`へ分離した。
3Mは移行数へ加えず、全体の共有資源構成と実Wasm／EPF／AI認定へ進む。

変更した章の出口についてもMiri 2関数・5条件（早い保管2、正常な反復拒否3）が終了時の
リーク検査まで成功。946入力hashが前後一致し、最大709,804,032 bytes、停止2.75GB。
実行560.78秒は解釈実行の検証時間であり、アプリ性能には数えない。
source/native binary/compiler/hashは`exit-v1/qualification.json`へ固定した。

### 2026-11-12: 3Mを3著者全体のsourceと実worldへ接続

132イベント・13roleの全体globalを実装した。3章のEndpointは各著者から事実確認・編集・
保管・公開へ委譲する。DOCUMENT/DATABASEは唯一の実I/O ownerが保持し、各実要求を受信して
実データを操作する。競合する資源loanを独立parへ並べず、別publisherやアプリの保持者表・
進行flagを置かない。共通化した章内反復でも両審査と否定後の実修正・新しい事実確認を維持し、
章ごとに独立して進める。公開前には3章の実Endpointと現在の参考文献・両審査結果を受信する。

native 9関数とstrict Clippyが成功。共通worldの5条件は95実効果すべてをEndpoint経由で
実行し、元採点の完了・違反0。各実行の完全captureをEPFで動的に照合し、受理した要求、
委譲された章権限、RPCとworldの実効果を一意に結び付けた。記録変異47件を拒否した。
これは監査器の拒否検査であり、実行時攻撃47件の阻止ではない。最大483,524,608bytes。
全体sourceを969入力hash・compiler・native binaryとともにwhole-v1へ固定し、Miriを実行中。
過去の部品Miriはこの全体版の成功へ流用しない。

実Wasm/ChoreoFSの5人の操作経路、自由な追加調査・再考のアプリ接続、実AIは未完。
3Mを移行数へ加えず29/48・残り19・今回モデル呼出し0・採点0を維持する。
GitHubへの反映が滞っていたため、未pushだった12コミットを31f300cまで公開した。
以後の29課題への移行作業と3M全体版はまだ作業ツリーにあり、このpushへ含めない。

### 2026-11-12: アプリの検査を実エージェントの効果境界へ戻す

ユーザーの指示に従い、safe RustのChoreoFixを課題ごとにMiriへ流す工程を廃止した。
実行中だった3Mの全体MiriはSIGTERMで明示的に停止し、合格にはしていない。原本と停止時の
最大763,823,400bytesはwhole-v1へ保存した。CIのアプリ向けMiri工程とtoolchain導入を削除し、
3MのMiri専用clock/cfgも除去した。基盤ライブラリ側のメモリ境界検査は削除しない。

ChoreoFixの認定は、実AIの自由なhibana-wash操作→WASI要求→実Endpoint→ChoreoFS→
共有効果→返信の通し実行を中心とする。合法な別手順・並行処理・再考と不正効果の事前拒否を
確認し、同じrun/binary/sourceのEPF動的出現、実要求と実効果、共通world採点へ結び付ける。
台本native、ビルド、EPF適合、業務採点の成功は区別する。48課題・残り19の範囲は維持する。
共通およびChoreoFixのAGENTS.mdへ明記した。

本体の検証中ソースはprogress-20261112（fb7e1db2d）として公開した。通常全gate未完のため
remote mainは既存版を維持している。この進捗tagを合格・リリースとは扱わない。

### 2026-11-12: ChoreoFSと業務要求の結合を全課題で棚卸し

共有ファイルを汎用要求投函先として受け付け、close後にパス・段階分岐で業務Endpointへ
振り直す二段アダプターを禁止した。共通およびrepo AGENTS.mdへ明記し、アプリのwrapperや
flagで補わず、必要ならWASI runtimeの型・公開APIから修正する。3Mへ追加しかけていた
未公開の著者／編集者アダプターと、そのための試作global変更は除去した。既に公開した
native 132イベント部品と実記録は維持するが、実Wasm／実AI成功には読み替えない。

全23family・29課題の入口をanalysis/choreofs-boundary-audit.mdへ列挙した。
17family・65ファイルにnext_fileの汎用close境界があり、旧review／食事の手書き経路を
含めて19family・21課題で同型の結合不足を確認した。残る4family・8課題には直接の型付き
WASI要求とChoreoFS処理もあり、全てを同一実装と断定せず各分岐の結合を点検する。
この静的棚卸しは、実要求経路の最終認定や文字列／ファイル存在CIではない。

WASI runtimeの公開complete APIについても、実Wasmのfd_close 2件をEndpoint 0・
ChoreoFS処理0で成功完了しexit 0になる反例を確認した。共有効果を迂回したとの主張や
メモリ不具合ではなく、完了に実Endpointの受理を要求しないAPI契約の不足である。
原本・実行binary・library hashはWASIのoutput/verification/choreofs-unbound-completion-v1。
実Endpointの権利を持つファイル型、要求／返信／guest完了を切り離せない正規APIへの置換は未実装。
旧29課題の版に紐づく結果は保存するが、この新しいファイル結合境界の合格件数には数えない。
