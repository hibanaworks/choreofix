# ChoreoFix 研究開発計画

更新: 2026-11-16。受領した「ChoreoFix 研究開発計画」計画版1.0を反映した。
入力文書のSHA-256は[実験契約](benchmark/experiment-contract.json)に記録する。
研究の中心は、**実資源の唯一の操作権、残存手順、実際の共有効果を結ぶ保証**である。

現在は直接ファイル境界の27/48課題をsource別に認定し、残り21課題である。
次の13Hは4装置の安全検査と、試薬EndpointをQCへ渡す合成componentを実装した。
合成は実結果を使うnative fixtureの5経路（初回成功、1/2回失敗後の成功、3回失敗後と
初期の枯渇）を確認し、同じ5実行の完全646記録をEPFで照合した。試薬残量はアプリで
保持しない。2回目の要求で露呈した別viewの返却待ちは本体の待機分類で修正中であり、
本体の最終全gate成功や13H全体の認定とはしない。直接委譲globalと2つの検査localsideでは、
要求到着の全24順序・不足要求の待機と再開・引数差し替え4件の効果0での拒否・実結果と
4本すべての返却をnativeで確認した。科学処理全体と実Wasm・EPF・実AIへの接続は未完で、
13Hを移行済み件数へ加えない。詳細は`benchmark/ports/13H.md`に記録した。
13Mは776イベント・29 viewの直接global、7つのlocalside、4 Wasmと共通runnerを実装した。
全8条件の実Wasm（112効果・37,128記録）と実AI4人のrunについて、それぞれ同じ実行記録で
EPF・実要求／材料／効果／返答・共通採点まで成功した。実AIは113.926317333秒、872,510 inclusive tokens、40 shell calls、
17効果・完全5,873記録・違反0である。保存済み記録を固定した監査器で再検査し、科学的な記録反例
27件とrun証拠の反例11件も拒否した。原本は`lab-quality-direct-20261116/qualification-live-v94.json`。
EPFの完了済みroll訪問からの遅れた受信の扱いを修正し、全gateが成功した。本体の実行仕様は
変更していない。WASIの実パス容量不足は64バイトへ拡張し、長すぎる入力はEndpoint操作前に
ENAMETOOLONGを返して正当な再試行を可能にした。WASIの全gateも成功しmainへpush済みである。
13EはLuna/xhigh 3人が76.462663417秒で完走し、同じrunの完全EPF4,044記録、
8実効果・科学的入力・返答照合・共通採点が成功し違反0。inclusive tokensは564,435。
25条件の実Wasm（200効果・55,648記録）、本番receiverの15負例と監査記録の9反例も確認した。
原本は`output/experiments/lab-synthesis-direct-20261116/qualification-live-v12.json`。
同じ実行原本のhashを再照合し、最終整形・入力manifest検査・runner 53件・Clippyも成功した。
その後の差分は文書／help、module宣言の整形と反例試験の不要なJSON変換の除去であり、
実Wasm・実AIの結果は保存した元binary/sourceへ結び付けたままとする。
4MはLuna/xhigh 5人が79.728423秒で完走し、同じrunの完全EPF5,624記録、
12実効果・返答照合・共通採点が成功し違反0。inclusive tokensは687,694である。
8条件の実Wasm（96効果・25,720記録）、本番receiverの12負例、監査記録の9反例も確認した。
認定原本は`output/experiments/shared-refactor-direct-20261116/qualification-live-v20.json`。
13Eの直接ファイル経路を実装した。装置の使用順を固定せず、独立した2本の準備列と
独立した品質測定を同じglobalへ置いた。3人の実Wasmの最初の条件は8実効果・完全EPF・
実要求／材料／返答と共通採点まで成功し、実装置receiverへの15負例は効果0で拒否された。
24通りの因果順序・入力と独立装置の並行実行1条件、その後の実AI認定まで完了し、
新しい直接境界の26件目へ加えた。共通環境v24の13Eは変更していない。
全schedule・最終sourceの全family・本実験の認定ではなく、以前の結果は各元sourceへ結び付ける。
6Hの`live-directory-v101`はLuna/xhigh 7人、深さ2を517.343330458秒で完走した。
同じrunの完全EPF50,851記録、53実効果・返答照合・中立な共通採点が成功し違反0。
レビュー拒否・CI失敗・staging失敗を各2回、実rollbackを2回含み、inclusive tokensは
8,533,959。認定原本は`qualification-live-v102.json`。全生成入力、最終sourceの全family、
本実験の認定ではない。以下の段階別経過と失敗記録は、それぞれ元のsourceへ結び付ける。

2026-11-16の追加進捗: 6Hの7人分の実Wasm実行・直接ファイルEndpoint・native側の
実資源処理を接続した。現行globalは1,237イベント・46 viewであり、通常の返信を
`offer()`から取得しようとしたアプリの誤りを除去した。重複するmerge-ready通知を外し、
CI所有者が実際の検証済み入力をstagingへ渡す。連携順序・修正・rollbackはglobalが守る。
実Wasmの深さ0で、実効果21件、完全capture、EPF動的監査、独立した効果・返答照合と
共通採点が通り、早期merge・配備後の再編集の拒否後にも続行した。通常CIの共通入口へ
通常・回復の2条件を接続し、現在の結果は末尾の2026-11-16 staging節に記録した。
これは実AIの成功や全interleavingの認定ではない。
4Mの新しい直接ファイル経路を実装した。5人の実Wasmの最初の条件では12実効果と
共通採点が成功し、完全captureのEPF・効果照合も通過した。候補通知を通常の`offer()`で
選ぼうとしたアプリ側の誤りを除去し、既存の所有者の通常viewと実REPO viewへ責務を分けた。
古い承認・否定・種別違い・対象違い・偽の計算出力・欠落出力を実Endpointから渡す
本番receiverの12負例は、公開効果0で拒否された。その後8条件の全機構試験と実AI認定を完了し、
4Mを25件目へ加えた。共通環境v24は4Mの実refactor応答へ現在の内容hashを追加するだけで、
操作の自由度・採点・資源制約は変えない。旧v23等の結果は元のsourceへ結び付けたままにする。
Luna/xhigh 7人の深さ2は2,751,791,104 bytesでguardが停止したため未完である。
観測記録の自己起床と大きなJSON値の保持をI/O所有者側で整理し、共有I/O17件・runner53件・
Clippyと修正後の実Wasmを確認した。再試行は最大2,601,713,664 bytesで上限内だったが、
実レビューの成功後にアプリが拒否用の返信型を選び、203.794633333秒で停止した。
reviewerとCIを実結果に対応した型を直接送る形へ直した。本体は変更していない。
本番のprepare sourceとreviewerによる「初回承認」「拒否→拒否→承認」の実Endpoint試験、
Clippyと修正後の7人実Wasm深さ0（21効果・15,186 capture記録・EPF/効果/共通採点）を
確認した。修正後の実AI認定は残り、23/48課題・残り25課題は変えない。
その後、受信候補の再試行ごとに同じpayloadを観測へ再追加するフィードバックを除去した。
実transport受理は一度だけ記録し、実受信は完全EPF記録のcommitへ対応付ける。
二重のpreview/requeue再生状態機械を監査器から外した。本体の進行検査は変更していない。
共有native19件、runnerとClippy、7人実Wasm（`wasi-v68`）、記録反例443件を確認した。
固定600秒の実AIは深さ2（`live-v70`）と深さ0（`live-v73`）で時間切れとなった。
最大メモリは各1,389,854,720／1,410,449,408 bytesに収まり、失敗原本を保持している。
後者ではCが他featureのreviewを読まないままmergeを再試行していた。
`wasi-v74`はその未読入力による拒否を再現し、同じguestで入力を消費すると完走した。
既存2件を含む拒否4件の後に21実効果・EPF・共通採点・違反0を確認した。
コマンド列は固定せず、ファイルのreadが各通知を消費するという説明を明確にした。
この説明変更後の`live-v78`はLuna/xhigh 7人が236.742336042秒、3,378,451 inclusive tokensで
完走した。同じrunのEPF 23,887記録、21実効果、実要求・返答照合、共通採点を通過し違反0。
`live-v80`の深さ2はレビュー拒否と2回のCI失敗後に再修正・検証を通り、配備失敗後の
実rollbackまで進んだが600秒で終了した（cleanup込み620.091017167秒、6,857,423 tokens、
最大1,410,138,112 bytes）。これはAIの時間予算切れで、本体の反例として扱わない。
実AIの再試行全体と全familyの最終source再認定は未完であり、課題数は増やさない。
2026-11-16、採点実験開始前に今後の全条件の時間予算を一律1800秒へ改定した。
旧600秒の全試行は元条件のまま保存し、新条件へ読み替えたり同一条件の統計へ混ぜない。
失敗注入、200 tool calls、モデル・推論強度、domain delay、choreographyは変更しない。
また、分割JSONの途中で文字列・objectを閉じた後へ追記する入力ミスに、washがerrnoだけを
返していたことを確認した。操作とpathを表示し、JSONの組立てとshell quotingの契約を説明する。
変更後のwashは関数テーブル131枠を使うため、WASI runtimeの容量を128から256へ拡張した。
区間ごとに関数一覧の巨大配列を持つ構造が7人起動時のホストstackを圧迫したため、検証済みの
不変Wasm原本を借りる形へ整理した。native107件・Clippy・対象Miri4件とPico 2向けlinkを確認。
guest保存領域45,000 bytes、stack24KiBを含む予約RAM152,296 bytes、flash333,516 bytesである。
`wasi-v86`では7人を起動・完走し、21実効果、完全EPF15,571記録、共通採点、違反0を確認した。
不正JSONのclose時に操作・path・元のWASIエラーを返し、同じguestで修正後に続行する試験を含む。
拒否後の復帰は計5件。本体の進行意味論と業務choreographyは変更していない。
公開CIに残ったEPF旧引数はhandoff・tool-cabinet・research-paper・pool fixtureから除去し、
native handoff 6件と現行APIでのClippyを確認した。アプリ用Miri時計と実行分岐も削除した。

現在はChoreoFS境界の是正を優先している。全26 taskディレクトリと共通I/Oを点検し、
旧29課題・23系統のうち20系統・24課題で、汎用close後に別業務Endpointへ振り直す
経路を確認した。WASI runtime側を実Endpointを所有するファイル型へ改めた。
新しい境界では9E/9M/9Hを、実Wasm 6条件とLunaの3/5/7人実行、完全なEPF記録、
実要求・実効果・返答・共通採点まで再確認した。8Eも実Wasm 3条件とLuna 3人の
成功・終端失敗の2経路で再確認した。4Eと16Eも下記の新経路で再確認した。
7E・3Eと2E／2M／2H・1E／1M／1Hも下記の新しい実ファイル投影で再確認し、10E・10M・11E・12Eと5E・5M・5H・6E・6Mも加えて再確認済みは23/48課題で、残り25課題の
新境界での認定と本実験は未完。本体の全gate・最終公開artifactの認定とは区別する。
採用する実行の成果・採点は元sourceへ結び付け、新経路の成功へ読み替えない。

2026-11-15の整理で、使用しない旧境界の試作記録、古い診断・停止出力、旧Rustのbuildを
削除した。本文の過去の進捗説明に現れる保存先が、現在も残ることは意味しない。
現行23課題の認定が実際に参照する原本・source・binaryは保持する。現在の開発用Rustは
最新stable 1.100.0であり、停止した1.99.1の生成検査を最終合格へ数えない。

2026-11-16には、未移行の4M・13E・13M・14E・14M・14Hに残っていた旧closeアダプター、
旧WASI入口、旧監査・機構試験を99ファイル削除した。廃止したWASI APIへの参照が
公開CIを停止させていた。実AI認定済みの17系統を保持し、Wasm機構試験の共通入口には
6Hの深さ0条件を加えた18系統を登録する。
元の48課題の仕様・共通simulatorと採用済み実験原本は保持し、未移行25課題は減らさない。
破棄したアダプターを互換APIで復活させない。

6Hは1,237イベント・46 viewのglobal、24種類の型付き実要求、7人分の
Wasm bindingと全localsideを持ち、実Wasmの通常・回復2条件と実AI深さ2を認定した。
Aの統合・stagingはB/Cの準備と並行に進め、stagingのA→B→C順序をsourceに記述する。
Bの統合はAの成功とB/C準備のjoin後、Cの統合はBの成功後に進む、という現在の範囲である。
実CI・候補・先行deploymentの検証を効果前に照合し、否定的なstaging結果では
実際に拒否されたdeploymentをrollbackする。10件のnative試験には実Endpointを
通る肯定・否定の受信側試験と、古いCIで効果0となる試験を含む。
新しい共通1,800秒条件で7人の実AI・完全EPF・効果照合・共通採点まで成功した。
旧600秒条件の時間切れ、guard停止、診断停止は消さず、新しい完走へ読み替えない。
6Hを24件目の認定へ追加した。有限の範囲と原本は[6H](benchmark/ports/6H.md)に記録する。

11Eは5本の操作ファイルを実Endpointへ結び、Ready/Busy通知とclose後の振り直しを撤去した。
実Wasm36条件とLuna xhigh 3人の実行が、同じrunのEPF・実効果・共通採点まで成功した。
実AIは約32.08秒、321,179 inclusive tokens。旋盤へのNONBLOCK要求がEAGAINで戻った後、
AI自身がフライス盤を選び、後から旋盤を再試行した。違反0、実作業の重なり1組である。
12Eも9本の実操作ファイルへ移し、全51条件とLuna xhigh 3人の通し実行を完了した。
実AIは約23.99秒、332,800 inclusive tokens、共有効果9件、器具Endpoint返却5件、違反0である。
元の調理simulatorの処理時間はゼロなので、実AIの効果時間の重なりは主張しない。
処理時間を明示した追加3条件では調理と材料準備がそれぞれ3組重なり、他の料理の調理と
自分の担当材料の準備より先に一皿を完成できた。制御した機構試験と実AIの測定を区別する。
12Eで露呈した本体のforeign-tail並行借用は、通常通信と実委譲を含む生成入力へ追加した。
その入力で見つかった貸出前の受信試行によるrun故障も修正し、関連native 58件、Miri 3件、
追加生成2構造の実行と正負の公開操作照合、Leanの局所射影補題まで成功した。
以前のcontracts-31反例は、古い受信を残した次周回の受信が実際にcommitすることを確認した。
反復分岐の先頭を区別できないglobalは投影時に拒否し、受信ACKや実FIFOのある合法なglobalは
保持する修正を加えた。分岐知識の起点の精密化後にも、最新sourceの生成31〜36が完走した。
11構造・188実行、23,616公開操作照合、8,798 parked preview、3,000負例を確認し、不一致は0。
通常試験で見つかった大規模レーン割当のconst上限は、集合比較の重複と探索済み候補の
再走査を除去して元の大規模7試験が通った。その後、後続の完了を同一レーンだけで探していた
再レビュー2周目の停止をsourceの因果関係で修正した。元のglobal・localsideを保った回帰と
Miriが成功し、本体nativeはlib869件・57 integration binary・112 UI入力が通過した。
生成器125件、割当計画4件、Clippy、thumbv6mのno_std buildも成功した。
新しい生成37は検査記録の重複保存で空き容量を圧迫したため、合格扱いせず停止した。
source/hash・実binary・生記録の抜粋・停止理由を保存し、巨大な未完の中間生成物を削除した。
入力範囲を減らさず保存の重複を解く作業と、生成38の実行、通常全gateは残っている。
本体の最終全gateは未完であり、12Eの保存済み実AI結果を後の本体変更へ読み替えない。
その後、現在の正本を使う`consumer-kitchen-v38/`で、12Eの実Wasm全51条件を再認定した。
source不変、全51条件成功、モデル呼出し0であり、上記の旧binaryの実AI結果とは分ける。
本体の同じ実装は`b7e4609fa`として公開した。これは生成全体と残る通常gateの成功を
意味せず、未完の生成37・38と既存カタログ・構造32分割の最終照合を継続する。
12Mの仕様整理は始めたが、新課題の認定はまだ増えていない。既存の課題別sourceとrawを保持し、
出力・記録形式だけの変更で無関係な本体検査や実AI試験を最初からやり直さない。

10Mは、開始順をCLIで先決めせず、実ファイル要求でimplicit routeを選ぶ形へ是正した。
`cascading-build-direct-files-20261114/wasi-v132/`の実Wasm 26条件が、全6開始順、深さ0/1/2、
不正入力からの回復、追加の実テスト失敗、通常遅延を含めEPF・実効果・返答・共通採点まで成功した。
通常遅延のABC/CBAはそれぞれ2組の実build/test重複があり、記録変異11,456件を拒否した。
同じ実行binaryのLuna xhigh 5人による自由なshell操作も、C→B→A開始から再ビルド・リンクまで
274.684265042秒で完了した。inclusive total_tokensは2,698,413、shell呼出し105件、同時model turnは
最大5件、実効果13件、producer記録24,051件、違反0件で、同じrunのEPFと実効果・共通採点を照合した。
原本は`live-10M-v146/subscription-10M-1794682675773695000/`に保持する。これは有限な機構試験と
非採点pilotであり、成功率・他手法との性能比較・実コンパイラの品質を示す結果ではない。
10Eとrunnerだけの変更に伴うsource保存の更新では、10M binaryのbyte同一性と入力差分を照合し、
既に成功した26条件を再実行していない。元source・command・hashは`10M-evidence-continuity-v147.md`に対応付ける。

10Eは実ファイル経路の試験は成功したが、開始順をCLIからresolverへ渡す先決めが残っていたため、
2026-11-15に自律的な実要求による選択の認定対象から外した。過去のWasm・モデル・効果・採点
の原本は維持し、開始順の自由度まで認定済みとはしない。現在は10Mと同様にCLI/resolverを
撤去し、実要求で選ぶglobalとlocalsideへ書き換えた。新sourceのnative 4試験は成功したが、
先決め除去後の実Wasm 8条件は一度成功したが、
`live-10E-v149/`の実AIではB先行の後、Aの更新をAIが読めるファイルへ投影していない不足が露呈した。
Bが再ビルド前に終了し、実行は`SessionFault(EndpointDropped)`で失敗した。原本は成功へ置換しない。
現在はAの実`TypesUpdated`をBのファイルEndpointへ送り、`/service/updates`の`read_as`へ直接投影した。
native側でだけ通知を消費する経路を削除し、AIが受信した実データから再ビルドを選べる形にした。
この通知を含む新sourceのnative 4試験、runner 54試験、Clippyと、`10E-wasm-v156/`の実Wasm
8条件が、両開始順・深さ0/1/2・不正入力を含めEPF・実効果・返答・共通採点まで成功した。
記録変異480件を拒否した。`live-10E-v157/subscription-10E-1794684056703719000/`のLuna xhigh
3人も、A開始から135.807130833秒で完了し、inclusive total_tokens 726,990、shell呼出し32件、
実効果3件、producer記録5,529件、両資源の要求4件と最終返却2件を同じrunへ照合した。違反は0件。
B先行の再ビルドは実Wasmで確認し、このpilotでは実行していない。台本の先読みをブロッキング
読み取りにした旧`10E-wasm-v152/`の停止記録も保持する。製品の待機を変えず、台本の任意の
先読みを通常の非ブロッキングI/Oへ直した。pilotの説明に残った旧`/service/result`を同じ操作
ファイルから読む記述へ訂正した。説明を訂正した`live-10E-v160/subscription-10E-1794684481272681000/`
も、同じ製品binaryの3人で112.97762875秒、inclusive total_tokens 613,335、shell呼出し27件、
実効果3件、producer記録4,616件、違反0件となり、同じrunのEPF・実効果・共通採点を照合した。
入力説明が違うので、二つのpilotの時間差を性能改善と扱わない。製品binaryと8条件の検証義務が
不変であることを確認し、既存8条件を再実行していない。集約は`10E-qualification-v161.json`、
10Mは`10M-qualification-v153.json`に、元source・input・binary・監査のhashを対応付けている。
10Eは、コンパイルとリンクのファイルが実Endpointを所有する形へ移行した。
汎用closeの後に業務要求を選ぶlocalside、要求投函用fd、段階別のエラー返信と結果転送を撤去した。
3つのファイルEndpointを最初に同じbuild-service actorへ委譲し、それぞれを各guestの
実操作ファイルへ直接渡す。A/Bどちらの実compile要求が先に受理されたかでimplicit routeが
決まり、その受理後に実資源Endpointを貸し出す。私有I/Oとメタデータの通信は選択の外に
一度だけ置く。開始順のCLI引数、resolver override、分岐を強制するファイル貸出はない。
各builderは実ファイル要求と両資源からの実要求を受信・照合してからコンパイル効果を呼び、
両資源と元ファイルへ実結果を返す。integratorは実A/Bオブジェクトとファイル要求を受け、
共有型の一致を効果前に確認してリンクする。終了や進行を別flagへ置き換えていない。

以下は先決めを含む旧v22 sourceの記録であり、上記の新sourceの検証結果ではない。
同じ資源Endpointを使うA開始とB開始・B再ビルドのnative経路、入力検査、古い型を本番receiverへ
渡してリンク効果を拒否する試験の計4件とClippyが成功した。nativeの簡略化したI/O側で露呈した
失敗は、必須のrollを0回で終える試験入力と、出力ownerを駆動せずterminal応答を待つfixtureに
起因した。失敗のログは保持し、正しいメタデータ操作と終了まで駆動する入力へ修正した。
Hibana本体Rust・WASI runtime・EPF・共通build worldは変更していない。アプリMiriは実行していない。

`output/experiments/build-direct-files-20261114/build-v22/`の同一binaryで、A/B開始、元の深さ0/1/2、
不正引数・重複要求の実Wasm計8条件が、完全capture、EPF、実要求／効果／返答、共通採点まで成功した。
記録変異480件を拒否した。これは監査の変異試験であり、480件の実行時攻撃ではない。
同じbinaryを使うLuna xhigh 3人の自由なwash操作も54.695354709秒で業務完了true・違反0となった。
inclusive total_tokensは446,583、shell呼出しは22件、同時モデルturnは最大3件。
4,136 producer記録、実効果3件、両資源からの要求4件と最後の返却2件を同じrunへ照合した。
このpilotはA開始であり、B開始の再ビルドの確認は実Wasmの別条件として区別する。
採点付き比較・成功率の推定ではなく、共通worldの合成オブジェクトと型データの試験であり、
実コンパイラや生成コードの品質評価ではない。source 2,204入力とbinaryのhashを照合した。
集約は`output/experiments/build-direct-files-20261114/qualification-v25.json`、機構試験は`wasm-v23/`、
実AI原本は`live-v24/subscription-10E-1794654350677273000/`へ保存する。
8Eは現在検査中の本体と拒否を修正したSDKで再実行し、実Wasm 3条件と実AIの成功・
終端失敗の2経路を同一binaryで監査まで確認した。4Eは共通採点器v19と実Endpointを
所有するファイルへ移行した。API・合意・候補・実テスト結果は実メッセージの受信を
ChoreoFSの読み取りへ直接投影し、hostの次操作選択、終了段階、保持者表は置かない。
両実装順、1回・3回のAPI改訂、通常遅延の実Wasm 5条件が、完全なEPF記録、
実要求・実効果・返答の照合、共通採点まで成功した。最初の3条件の記録を保持して
監査器の出現照合を修正し、同じ記録を再監査した。失敗した旧コマンドを成功へ
書き換えず、残る2条件だけ追加実行して5条件を満たした。
同じruntime binaryでLuna xhigh 3人が自由にshellを操作した予備実行も成功した。
39.862秒、inclusive total_tokens 352,337、実効果4件、違反0件で、同じrunの
EPF 3,186記録と実効果・返答・共通採点を独立に再監査した。記録は
`output/experiments/coordination-direct-projection-20261113/qualification-summary.json`と
`output/experiments/coordination-free-shell-current-20261113-live/`へ保持する。
これは4Eの有限な機構・実AI予備試験であり、採点付き比較や全課題認定ではない。
16Eは汎用WASIアダプターと要求準備・検証flagを除去した。各操作ファイルのcloseが
実要求となり、実際のBUILD_SERVERまたはDEPLOY_ENVの要求・結果・返却へつながる。
ビルド完了・テスト合格・配備完了を含む7件の実メッセージを、同じファイルの読み取りへ
直接投影する。2件の最終通知は既存の独立viewから並行に送り、別の保持者表は置かない。
実資源の要求を偽造したnative試験は本番receiverで効果前に拒否され、深さ0/1/2の
実Wasm 3条件と早期終了1条件が成功した。通常遅延、6件の不正引数・担当外対象の拒否、
早すぎる要求・重複要求のエラー後の続行を含み、3実効果と4通知・3結果をEPFと
独立した実効果監査・共通採点へ照合した。記録変異は3記録×16種類の48件を拒否した。
共通採点器v20は16Eの完了flagを実ビルド・テスト・配備済み成果物へ置き換えた。
全6操作順は呼出し可能で、未検証の配備も実際に反映し、その違反を後の修復で消さない。
世界の試験は16E 10件と4E 10件が成功し、他47課題の世界・825 checklist項目は不変である。
最後に未検証配備の違反保持を補強した際は、Rust本体・SDK・runtime・runnerが同一である
ことを確認して既存の境界試験を引き継ぎ、新しい採点器の通常Wasm 1条件を追加確認した。
初回のLuna 3人実行（38.038秒、351,565 tokens）と区別し、最終の世界でもLuna xhigh
3人を一度実行した。32.079833秒、inclusive total_tokens 386,756、実効果3件、
EPF 2,375記録、違反0件で、同じrunの要求・実効果・返答・共通採点を再監査した。
双方とも非採点のsource付き予備実行であり、時間差を性能改善や同条件比較としない。
証拠は`output/experiments/cicd-direct-projection-20261113/qualification-summary.json`と
`output/experiments/cicd-free-shell-current-20261113-live-v2/`に保持する。
ChoreoFSはshellが自由に使うchoreographyの投影であり、hostが次の操作や貸出を先に
決めるworkflowではない。SDKでは受理前の拒否をWASIエラーへ戻し、同じguestで
別操作・再試行できることを実Wasmで確認した。受信メッセージをそのままファイルの
読み取りへ投影する`read_as`も実装した。分割読み取り・拒否後の再試行・再open・
他viewへの分岐知識・容量不足と取消の実Wasm 5試験、SDK通常検査
（lib100件・example8件・doctest13件、Clippy、no_std、既存Lean照合）と
SDKのファイル境界18件のMiriが成功した。記録はSDKの
`output/verification/incoming-file-projection-20261113/`に保存する。
SDKの境界検査を全課題の認定へ読み替えない。4Eで不採用になった旧globalの
投影拒否と途中の失敗記録は保存し、上記の新しい実要求経路の結果と区別する。
委譲、投影、並行処理、prefix safety、版の照合をそれぞれ単独の新規性とはしない。

既存A比較の採点対象は **0 / 2,592実行**。追加B–Gは **0 / 1,368実行**で、
最大600実行の予備枠を含む総上限は **4,560実行**である。
追加課題、比較接続、精度設計は未認定であり、この上限を凍結・実行済みmatrixとは扱わない。
予備試験、部品試験、ビルド成功を採点済み実験へ数えない。

### 2026-11-14: 1E／1M／1Hの共通worldを実データへ是正、アプリ移行は未完

コード開発3課題のWASI段階dispatcherとfd入りClose envelopeを除去した。
設計・実装・コミット・テストの4ファイルが実Endpointを所有し、localsidesへ実受信、
両モジュールの実要求、実効果、結果、返却を直接書いた。モジュールはテスト前に返す。
1Hのresolver入力は実テスト結果そのものであり、進行flagや保持者表を追加していない。
各モジュールの受信側も実結果に従う同じ投影を書き下す。共通結果の通知に揃えて、
成功用・失敗用の別ラベルや開発者ごとの処理コピーを除去した。
初期の実Endpoint試験では早すぎる実コミット要求の拒否と正当な設計要求による続行が成功した。
その後の静的な観測対応と並行準備を含む実Wasmは、予定した15条件すべてが監査まで完了した。
3／5／7人それぞれの深さ0／1／2・追加テスト失敗・通常遅延が対象で、3課題の実AIも下記のとおり成功した。
同じ実行binaryと入力を保持し、EPFで停止した深さ1／2はWasmをやり直さず再監査した。

7人の深さ1では、EPFの初期展開がすべてのroll訪問を継続より前へ置き、合法な再訪を
監査できていなかった。継続への挿入を、先行部と継続の別々の再構築から、既に展開した
全体への合成へ変更した。古い訪問が新しい挿入を根拠にしたり、未来の訪問を過去の
再訪へ流用したりできないことも検査する。Hibana本体のRust実装、global、localsideの
通信順はこの修正で変更していない。Leanの132モジュールとEPFの6証明モジュールが通り、
実Endpointの両再訪順・継続途中／継続後を含む試験とstrict-provenance Miriも成功した。
これは任意のRust実行に対する完全なrefinement証明という意味ではない。

EPFの再訪に関する依存照合は、その再訪の影響を受けない操作でも履歴を再走査していた。
既存の展開情報から対象の継続を判定する形へ変更し、一般命題と有限な構造・操作列で照合した。
同じ完全capture 51,020記録・9,335送信受理を26.98秒、最大1,315,897,344 bytesで検査した。
以前の停止した実行を成功や速度比較の測定点に数えない。ChoreoFixの実効果監査も同じ記録で
成功し、49実効果、資源要求／返却各28件、14テスト判断、独立commit最大3件、業務完了true、
違反0を確認した。記録変異552件も拒否した。先行11条件の実施版の証拠は別に保持する。
証拠は`epf-compose-v66/`、`1H-depth-1-recheck-v69/`、`epf-compose-miri-v70.log`
（いずれも`output/experiments/codebase-direct-files-20261114/`配下）に保存する。

深さ2では、複数のrollの最終訪問だけでなく各訪問を継続中へ配置する初期候補に改めた。
候補を構成した後のsource・所有権・実行の検査は維持した。実Endpointの2／3／5訪問の
記録を自動監査し、全native検査、Clippy、Picoビルドが成功した。保存済み69,036記録・
12,631送信受理は63.73秒、最大1,779,418,792 bytesでEPFの最終検査まで成功した。
同じ実行の70実効果、資源要求／返却各42件、21テスト判断、独立commit最大3件、
業務完了true、違反0を照合し、804件の記録変異も拒否した。検査前後で1,119入力のhashと
使用binaryのhashが一致した。証拠は`epf-all-visits-v81/`と`1H-depth-2-recheck-v83/`へ保存する。
これはモデル呼出し0の機構試験であり、実AIや採点付き比較の完了へ読み替えない。

外部テスト失敗を追加した7人条件も、同じbinaryの実Wasmは業務完了true・違反0で完走した。
105,068記録を保存したが、後段のEPF監査が3,010,005,464 bytesでguardに停止された。
停止した検査は成功へ読み替えない。探索中だけ使う分岐候補配列を独立した出力から除去し、
既存のcaller-owned作業領域へ統合した。展開イベントの上限、探索範囲と選択・最終実行検査は
維持した。EPFのnative158件・doctest2件、Clippy、no_std/Picoビルドが成功した。
証拠は`wasm-v85/explicit-memory-stop-v86.json`と`epf-workspace-native-v88.log`へ保存する。
保存済みの実行記録を新しい検査器で再監査し、Wasm自体は繰り返していない。
`1H-external-recheck-v91/`で動的source・112実効果・資源要求と返却各70件・35テスト判断が
照合され、業務完了true・違反0を確認した。各人が4回の否定的テストの後に実修正し、
5回目に成功した。1,308件の記録変異もすべて拒否し、最大メモリは1,138,900,992 bytesだった。
実行前後で2,203入力のhashと使用binaryのhashが一致した。モデル呼出し0の機構試験として記録する。

最後の通常遅延条件も`wasm-v93/`で通過し、独立commit最大3件・違反0を確認した。
現在のrunnerの通常native検査54件も通過した。15条件は同じruntime binaryを用いたが、
各条件の監査器の版は保存したsourceへ個別に結び付け、全条件を同じ最終検査器で再確認したと
読み替えない。過去の記録の照合と、同じbinaryでの実AI確認を続ける。

`live-v95-1E/subscription-1E-1794644139956580000/`では、既存ChatGPT接続のLuna xhigh
3人が各自のwashを自由に操作し、同じbinaryで業務完了true・違反0を確認した。
12実効果、実資源要求／返却各6件、独立commit最大1件を完全なEPF記録と実結果へ照合した。
inclusive total_tokensは540,934で、cached inputを含みreasoningを二重加算しない。
これは採点付き比較ではない。1Eを新境界での認定へ追加した。

`live-v96-1M/subscription-1M-1794644342265683000/`でも、Luna xhigh 5人の自由なshell操作が
36.283251958秒で業務完了true・違反0となった。9,756記録をEPFで検査し、20実効果、
資源要求／返却各10件、独立commit最大2件を照合した。inclusive total_tokensは666,008。
全5人の完了とモデル・effort、同じrunの採点・実効果・動的sourceが一致し、1Mも認定へ追加した。
7人の実AIは次の保存記録と再監査で確認した。採点付き比較は未完のまま区別する。

`live-v97-1H/subscription-1H-1794644479037445000/`では、Luna xhigh 7人が各2回の
否定的テストを受けて実修正し、58.034685917秒で業務完了true・違反0となった。
inclusive total_tokensは1,857,691。旧候補は再訪をソース順に置き、実記録のB→Cの
順序に合わず、監査だけを明示的に停止した。モデルやWasmの成功を停止したtestの成功へ
読み替えない。候補の構成順と、継続のどの訪問の前へ置くかを観測された入口で選ぶよう改めた。
検査条件・sourceの文法・完全列挙の範囲は維持し、Hibana本体のRust、global、localsideの
通信順を変更していない。実Endpointの5つのrollを異なる順で再訪する2経路を加え、
EPFのnative159件・doctest2件、Clippy、no_std/Picoビルドが成功した。
調査用testと出力は製品sourceから除去した。証拠は`epf-entry-hints-v109.log`に保存する。

同じ30,316記録を通常のconsumerで再監査した`live-v97-1H-reaudit-v111.json`は、
11.76秒・最大307,953,664 bytesで成功した。70実効果、実資源要求／返却各42件、
21テスト判断、独立commit最大3件を同じrunの実結果・採点へ照合し、1Hも認定へ追加した。
旧停止と新しい監査結果を別々に保持し、AIを再実行していない。
3課題の実行binaryは同じ`b3f97377865e127e7b4b16f28304c38141a854655e04e3740197e44a67a15363`。
15機構条件・4,754記録変異と3実AI pilotの集約を`qualification-v112.json`へ保存した。

その後、同じ検査器での再確認で1Hの深さ1の記録が13分以上の探索に入り、明示的に停止した。
別々の送信者が反復へ入った順序を、共通の受信者の受信順序にも当てはめていた候補構成が原因だった。
同じproducerの同じactorが両方の反復を観測した場合だけ順序の手掛かりとし、情報がない場合や
食い違う場合はsourceの順序を使う。候補は引き続き独立した実行検査を通す。判定条件・展開の
全列挙域・本体Rust・アプリのglobal／localsideは変更していない。
送信開始と受信の順序を両方向で変える回帰試験、別session・別producer・欠落したvisitの確認を加え、
既存の実Endpointによる異順再訪も維持した。EPFのnative160件・doctest2件、Clippy、no_std／Picoの
確認と現consumerのbuildが成功した。調査用の中断・出力コードは製品sourceから除去した。

`build-v121`の検査器で15機構条件と3実AI実行の保存記録、計18件を通常のconsumer経路で再監査し、
すべての動的source・実要求・実効果・返答・共通採点の照合が成功した。実AI3実行の業務完了はtrue。
1E／1Mの外部失敗注入2条件は元から業務完了falseを期待する負例で、その結果も保存済みの採点と一致した。
深さ1の1Hは32.96秒、実AIの1H記録は12.31秒、105,068記録の最大条件は197.33秒だった。
最大条件のCPU採取は候補後の独立した実行検査を示し、全18件のguard最大値は753,041,408 bytes。
前後で2,203入力のhashと使用binaryのhashが一致し、実行binaryは同じ`b3f9737…a15363`のままである。
WasmやAIは再実行していない。`current-audits-v122/summary.json`と`qualification-v123.json`へ保存した。
4,754件の記録変異の既存結果は各元sourceへ結び付け、最新版で全変異を再実行したとは扱わない。
この1E/1M/1H再監査時点の認定数は14/48、残り34、採点付き比較0だった。

先に共通worldの4つのdone mapを除去した。完了した設計text、その設計と実際の否定的な
テスト結果を入力とする合成source、実コミットのsnapshot、そのsourceを実際に調べた
テスト結果を、既存のevent journalへ保持する。Hibanaのglobal・Endpoint・許可表や次操作の
cursorをworldへ置かない。各操作の呼出し・再試行、別featureの独立作業は自由である。
これは抽象的なコードデータのworldであり、実コンパイラや修正コードの品質評価ではない。

`common-environment-v23`の11試験で、3／5／7人の全24ローカル操作順、著者順の回転、
同じ内容の再作成、設計変更、受信済みデータの分離、未完入力、古い合格結果の流用拒否、
否定的テスト後の実修復・再コミット、1Hの0／1／2／4回の失敗を確認した。
独立コミットは実時間で重なり、同じモジュールの競合は観測される。
共通world全258試験と入力工具17試験が成功した。他45課題のworld、48課題の公開入力、
825 checklist項目は不変である。旧worldの認定は元のsource／hashへ結び付けたまま保持する。

監査側の手順counter・保持者／移譲表も撤去した。EPFが検査した実受信と、元要求、
両モジュール要求、実効果、返答、共通採点を一対一で照合する。1Hは、実際のテスト結果と
その結果の後に呼ばれたresolverの判断も照合する。通信用のwire契約とresolver IDは、
実際にattachする同じprogramsから保存する。監査用globalや別のworkflow仕様は作らない。
テスト専用の環境側コミット保留gateも除去し、通常遅延の実効果から並行性を測る。
最終試験は深さ0／1／2、1Hの3／4回目の否定的テスト、終端失敗、不正入力と正当な続行、
通常遅延での1／2／3件の独立コミットを含む。準備の待合せは試験刺激にだけ明記し、
productionや実AI実行へ入れない。

証拠とworld sourceは`output/experiments/codebase-direct-files-20261114/world-qualification-v4.json`
と`world-v4/`に保存した。実Wasm15条件と3課題の実AIを確認し、
この1E/1M/1H確認時点の認定数は14/48、残り34、採点付き比較0だった。最終公開artifactと本体の全gateは別の未完項目である。

### 2026-11-14: 2E／2M／2Hを直接ファイル投影と実AIで認定

文章作成の旧WASI段階dispatcherを除去した。4つの操作ファイルが実Endpointを持ち、
そのcloseが実要求、同じファイルのreadが実結果となる。localsidesに実際の受信、
2つのsection Endpointによる要求と結果、返却を直接記述し、次操作や許可を決める
ホスト層・汎用の要求投函ファイル・進行flagは置かない。washは自由なshellのままである。
3／5／7人の全globalはforbidのままビルドでき、runner通常54試験と両側Clippyが成功した。
同じ最終binaryで実Wasm 15条件が成功し、完全capture・EPF・実要求と実効果・返答・共通採点を
照合した。審査の終端失敗はprotocol成功と業務上の失敗を区別し、不正入力後の正当な続行も含む。
記録変異2,730件を拒否した。これは監査器の検査であり、実行時攻撃の阻止数ではない。
通常遅延の試験では、テスト側が全員の実準備結果を待ってから書込みを出し、実効果の重なり
1／2／3件を確認した。このテスト用の待合せをproduction localsidesや実AI試験へ入れていない。

Luna xhighの3／5／7人が自由なshellを操作した3実行も、同じbinaryで完走し再監査した。
2Eは31.01519625秒・367,675 inclusive tokens・12実効果・6資源要求・EPF 5,945記録、
2Mは33.482359秒・636,160 tokens・20実効果・10資源要求・EPF 9,954記録、
2Hは30.908882667秒・930,270 tokens・28実効果・14資源要求・EPF 12,003記録である。
全員の実返答はgpt-6-luna／xhigh、ChatGPT認証、業務完了true、違反0である。
実AIで観測した書込みの重なりは1／2／2件で、7人の試験上の最大3件と混同しない。
これらは各1回の非採点予備試験であり、性能優位・成功率・同条件比較を示すものではない。

共通world v22は完了flagを実文献、文献を含む原稿、実際の文書内容と審査snapshotへ
置き換えた。任意の呼出し順・再試行は全手法に許し、後から得た材料で過去の不正な
操作を正当化しない。同内容の再作成は同じ内容の審査を無効にしない。全24操作順、
独立実行、物理section競合を含む9試験と共通world全247試験が成功した。
他45課題のworld・48課題の公開操作・825 checklist項目は不変である。
既存8課題の認定は各々の実sourceへ結び付けたまま、新worldの成功へ読み替えない。

初回のWasm試験は、発行元actorより先に委譲file viewを作るintegrationの誤りで、効果0のまま
停止した。初期actorの構築順だけを直し、通信順、globalの許可条件、localsideの手順を変えて
回避していない。失敗記録`wasm-v11/`と旧sourceは保持し、成功へ読み替えない。
最終binaryは`03d89156e536dfe2668860d9081e5c71f409e196e4617b19cf431d9efa3289f9`。
証拠は`output/experiments/writing-direct-files-20261114/qualification-v16.json`、
`build-v12/`、`wasm-v13/`、`live-v14-2E/`・`live-v14-2M/`・`live-v14-2H/`と各独立監査に固定した。
本体の全gate・生成入力域の補充・最終公開artifactとは区別する。認定済み11/48、残り37、採点比較0。

### 2026-11-14: 3Eを直接のChoreoFSと自然な研究者の終了で認定

共通world `common-environment-v21` は、担当topicの完了済み研究、実原稿・参考文献、
各sectionの現在の内容に結び付くレビューと、公開前に完了した肯定的な結果を採点する。
独立した著者順、執筆／参考文献順、審査順とDOCUMENT／DATABASEの同時利用は自由で、
採点器へHibanaの手順やEndpointを入れていない。3Eのworld試験8件、共通world全238件、
入力工具17件を確認した版である。他47課題と825 checklist項目はこのworld変更の対象外。

3Eのclose後のpath／phase振分けを除去した。ファイルは最初から実Endpointに結び付き、
localsidesが実要求・資源Endpoint・実結果を直接送受信する。公開は両sectionの現在の承認と
実資源要求の照合後に実行する。ホストの次操作選択、貸出表、進行flag、終了managerは置かない。
この3E認定時点の点検では、他17系統・76ファイルに旧方式の参照が残った。試験・補助参照も含む件数で、
76個の独立した欠陥や全課題の移行完了を意味しない。

初回の実AI試験は、共通採点complete=true・違反0・13実効果でも、研究者Bが承認後に終了し、
まだ要求されていた最終結果のファイル受信を残してprotocolが失敗した。元課題にないshellの
終了条件をglobalへ増やしていたため、公開結果を著者のshellへ転送する2通信を削除した。
両承認、資源の移譲と、担当native localsideへの公開通知は維持する。研究者は自分の承認後に
終了でき、残る公開処理はlocalsidesが続ける。本体の変更や自動drain、終了flagによる回避ではない。
旧失敗の113.028秒・788,439 tokensとrawは `live-v24/` に保持し、新版の成功と混ぜない。

新版binary `7572940734d40e95115a6f8bdef29d5b6dfd7b84df975e4246f1f03ec833ebb8` で、
通常18条件、3／4回目の否定的レビュー8条件、不正入力・公開後要求1条件、計27条件が成功。
研究者の承認後の早い終了、独立処理の重なり、不正要求後の続行、公開後の再編集の拒否を含む。
同じ実行の完全なEPF記録、実受信・引数・効果・返答、共通採点を照合し、違反0である。
記録変異6,579件を拒否した。これは監査器の検査であり、実行時攻撃の阻止数ではない。
監査の順序はEPFの受信commitを使い、遅延したtransportログの行順で再判定しない。
委譲順序の二重検査と、同じ記録にEPFを二回実行する重複は除去した。

同じbinaryの深さ2をLuna xhigh 3人が自由なshell操作で完走した。106.091916458秒、
inclusive total_tokens 806,895、13実効果、4レビュー（否定2回）、7資源要求、違反0。
全員exit=0、要求・実返答ともgpt-6-luna／xhighで、完全な23,012記録をEPFと実効果監査へ
接続した。実効果の時間的な重なりはこのpilotでは観測していない。3人のmodel turnの重なりや
機構試験の並行性と混同せず、性能比較・成功率・採点付き比較へ数えない。

証拠は `output/experiments/research-report-direct-files-20261114/qualification-v30.json`、
`build-v27/`、`wasm-v28/`、`live-v29/`、`live-v29-audit.json` に固定した。
本体の全通常gate・生成入力域の補充・最終公開artifactは未完であり、この有限の認定へ含めない。
この時点の再確認済みは8/48、残り40課題、採点付き比較は0件だった。

### 2026-11-13: 7Eの貸出選択を実ファイル要求へ移行中

ホストの`FIRST_WRITER` resolverとCLIの初期著者指定を撤去し、編集者が
`/service/allocate_document`へ書いた`first_writer`が、そのファイルの実Endpointから
`Allocate`としてdocument serviceへ直接届く形にした。SDKの`write_as`がordinary messageの
送信へ投影するため、close受付後にStartA／StartBへ振り直す役と専用close roleは除去した。
ファイルデータ担当はmetadataと実bytesだけを扱い、貸出先や進行を判定しない。
document serviceは実際に受信した選択へ従い、元の唯一の文書EndpointをAまたはBへ渡す。
要求bytesは既存の`ReportArguments<7>`で保持し、選択だけの独自enum／codecも残さない。

正本のglobal、貸出ファイルlocalside、本番document receiverを直接コンパイルし、
10投影の構築と8試験が成功した。実Wasmの有限入力はA/B×通常／1万bytesの空白を加えた
JSONの4条件である。不正入力のEINVAL後、同じguestの正しい要求と実返信、読み取り専用の
close、消費済み操作へのEACCESと正常終了まで確認した。FileDataの容量は固定8KiBから
設定済みguest memory容量に基づく値へ直し、10,026-byte入力も縮めず検査した。
値・codec・引数差替えの負例と、公開後の編集権限の18条件も含む。strict test-target
Clippyも成功した。以前のFileExit失敗は、fixtureが一度もmetadata rollを実行していなかった
準備の誤りだった。実ChoreoFSのmetadata要求を一度通して修正し、元の禁止条件を維持した。
このconsumer修正では本体の実装・意味論を変更していない。失敗ログは成功へ書き換えず残す。

SDKでは`write_as`の通常検査104 lib tests・8 example tests・15 doctests、Clippy、no_std、
既存poll／path-rightsのLean検査が成功した。ファイル境界22件のstrict-provenance Miriと、
その後に変更したoutgoing fixture4件のMiriも成功した。Pico 2の既存poll fixtureは
flash331,416 bytes、stack込みRAM154,568 bytesでlinkした。これはwrite_asを用いる
任意アプリの物理サイズや、ファイル投影全体の一般証明ではない。SDKのsource/hashと結果は
`hibana-wasip1-runtime/output/verification/outgoing-file-projection-20261113/`へ保存した。

著者の編集と編集者の審査・再考には旧WASIアダプターが残り、7E binaryの移行は未完。
アプリ全体のtest buildも未移行APIで失敗しており、部品検査の成功とは区別する。
今回のWasmは境界fixtureで、実wash全体・実AI・完全EPFと実効果の通し認定ではない。
新しい貸出部分の成功を7Eへ加算せず、保存済みの認定は各sourceに紐づく6/48、残り42のまま。
SDK変更後の全課題再認定を済ませたとも扱わない。記録と検査対象sourceは
`output/experiments/document-allocation-20261113/direct-projection-result.json`と
同directoryの`direct-projection-source/`へ保存する。
本体は同じsourceの生成検査を継続しており、このconsumer変更で再開し直してはいない。

### 2026-11-13: 複数ファイルを同じEndpointへ投影するSDKの整理

一つの参加者が複数の操作ファイルを使うために、ファイルごとに別のlogical viewを要求する
SDKの制約を除いた。`ChoreoFsFile::file`は、既に所有する一つのEndpointへ別のファイル投影を
追加する。各ファイルは実bytes、offset、codec、必要なら実際に受信した所有値を保持し、
通信は常にその同じEndpointの継続で受理する。アプリのパス／段階による業務振り分け、
進行表、別の所有者表、追加の発行権、互換modeは導入していない。

実Wasmで同じEndpointの三つのファイルを使い、二つの合法な操作順、まだ許されない要求の
拒否後の続行、書き込み用に再openした消費済み要求の再送拒否、各実返信を確認した。
一度閉じたfdへの通常のmetadata closeと、業務メッセージの再送は区別して検査する。
宣言の重複fd／重複path／非正規path／別Endpointの投影との衝突は5入力で事前拒否し、
どの場合もprotocol frameを出さない。未完のprefixを検査するfixtureは、回収した実Endpointを
他参加者の完了まで保持する。試験の準備とfd操作を修正したもので、本体の受理は変更していない。

最終SDK sourceで106 lib tests・8 example tests・15 doctests、strict Clippy、no_std／
thumbv8m build、既存のpoll／path-rightsのLean照合が成功した。ファイル境界24件の
strict-provenance Miriも成功し、最大合計メモリは1,278,918,656 bytesだった。
誤って未固定のnightly aliasを使った先行Miriはsysroot準備中に予算で停止し、試験は未実行。
その停止を保存し、既存の検証用`nightly-2026-05-28`で同じ予算・全24入力を完走した。
このライブラリの検証を7Eの実wash・実AI・EPF／効果採点の認定へ数えない。
SDKのsource、依存source hash、compiler、結果は
`hibana-wasip1-runtime/output/verification/grouped-file-projection-20261113/`へ保存する。
同じSDKを実際に選ぶ7Eの貸出部品8試験とstrict test-target Clippyも成功した。
アプリの検査対象sourceが前回の部品検査から不変であることと、Cargo metadataの
core／runtime／WASI／EPFが全て正本の隣接pathであることも確認した。実Wasmは前節の
4入力、実モデルは0件で、結果は
`output/experiments/document-allocation-20261113/grouped-sdk-result.json`に保存する。
この部品検査時点では7Eの著者・編集者の旧WASIアダプターの置換は未完だった。
その結果を、次節の全体書き換えと混同しない。

### 2026-11-13: 7E全体を実ファイル投影へ書き換え

著者・編集者の`next_file`／`ClosedFile`による受付と業務振り分けを除去した。
実要求は各操作ファイルの`write_as`、実応答は同じファイルの`read_as`へ投影する。
編集者AIの実要求が初期著者、再考、候補確定を選び、nativeの各localsideは
そのglobalのEndpoint操作を直接実行する。別のホスト制御、段階flag、保持者表はない。
パス／fd／offset／bytesを扱うmetadata localsideには業務実行や次の操作の判断を置かない。
ホストが連携を判断する、という説明は採用しない。連携仕様はglobal、実行はそのlocalsideである。

17 roleの全体を`forbid(long_running_const_eval)`のままビルドし、実binの8試験と
アプリ／runnerのstrict Clippyが成功した。washのRust標準ライブラリが使うfd照会も
WASIのglobalとその直接の応答に含めた。著者の通知と最終結果は一つの受信ファイルへ
投影し、アプリやシェルが別ファイルを選び直す処理を除去した。肯定的な実審査からの
自主的な再考も、実際の現在の審査との一致を守ったまま可能にした。

実washの12条件が完走した。A/B先行、両審査順、反復拒否、他章への修正、
不正入力・重複要求後の続行、公開後の拒否、大きなUTF-8結果、肯定的な審査からの
再考を含む。同じrunの完全captureをEPFの動的source検査、実要求／実効果の照合、
共通world v20の採点へ結び付け、全12記録が成功・違反0件だった。
8-bit labelだけでメモリ拡張を業務結果と誤分類した監査器は、通信の両端も含めて修正した。
実Wasmを再実行せず、保存済み12記録を修正した監査器で再照合した。
記録改変1,425件も拒否した。この件数は監査器への記録変異で、実行時攻撃数ではない。
source、binary hash、旧失敗と訂正後の監査は
`output/experiments/document-direct-files-20261113/mechanism-result.json`から辿れる。

これはモデル呼出し0件の機構認定で、7Eの現版実AI認定はまだである。保存済み認定は
引き続き各source付きの6/48、残り42とする。本体sourceは変更・再起動していない。
他の18系統・73 production filesには旧受付方式の参照が残る。全体を除去済みとは言わず、
一覧を同directoryの`remaining-adapters-inventory.json`へ保存した。

### 2026-11-14: 7Eを直接の公開要求で認定

別の承認投函ファイルと要求型を削除した。`combine_sections`の実要求そのものが
globalの確定分岐となる。編集者localsideは、実際に受信した現在の両審査と成果物を
照合し、肯定的な審査に限って実文書Endpointを移す。公開のreceiverも、実際の両審査・
現在の成果物・実資源からの要求値を独立に照合してから効果を実行する。
AIへ審査JSONの診断用フィールドの丸写しを要求しない。再考の入力は現在の実審査IDのみで、
対応する実審査はlocalsideが受信した値を使う。新しい保持者表・進行flag・workflowは置かない。

最終版で実binの8試験、アプリとrunnerのstrict Clippy、runnerの54通常試験が成功した。
実washの同じ12条件も完走し、完全なEPF記録、実要求／実効果、共通world v20の採点を
同じ各runで確認した。早すぎる正しい型の公開要求はWASIエラーとなり、同じguestが続行する。
記録改変1,503件を拒否した。実験の原本とsourceは`document-direct-files-build-20261114-v9`と
`document-direct-files-20261113/wasm-v9`、集約は同directoryの`qualification-result-v9.json`へ保持する。

その同じruntime binaryでLuna xhigh 3人を深さ2・通常遅延で実行した。AIは自分で一覧を
調べ、章の内容・shellコマンド・再考と修正を選んだ。123.033290083秒、inclusive total_tokens
727,242、実効果10件、審査4回、修正1回、違反0件で完了した。3人のmodel turnが重なり、
同じrunのEPF・実効果・モデル記録・共通採点の独立監査も成功した。
これは非採点の有限な認定で、性能比較や全課題・全入力の保証ではない。
記録は`output/experiments/document-free-shell-current-20261114-live-v3/`に保存する。

旧入口の実AI予備実行2件は失敗として残す。1件目は著者の早期終了、2件目は正しい審査IDでも
診断フィールドの丸写しを要求する余分な入口による拒否だった。どちらも成功へ数えない。
最終runの監査も最初の2回は本体と合算したメモリ上限で停止した。二重に保持していた
解析済み履歴を解放し、本体のメモリ使用が小さい区間に同じ全記録を再監査した。
上限2,750,000,000 bytesは変えず、成功時の最大合計は1,368,080,384 bytesだった。
本体・SDK・実行binaryを変えず、実AIを再実行して監査失敗を隠したものではない。
source付き認定は7/48、残り41、採点付き比較は0件。旧方式の参照が残る18系統の移行を続ける。

### 2026-11-16: 6Mを直接ファイル投影と5人の実AIで認定

6Mの汎用close後の振り分け、段階別adapter、操作ごとのloan brokerと旧native workflowを撤去した。
614イベントの一つのglobalから31programを作り、6つの実ファイルEndpointを5guestへ直接渡す。
実REPO／TEST_ENV／BUILD_SERVERは各効果ownerが一度だけ受け取り、実要求・実データを扱う。
両featureの実装・review・testは並行に準備し、最初の実承認receiptがmerge順を決める。
各mergeのCIと修復が完了してから他方のmergeへ進む。この段階的な仕様を、任意のmerge/review
pipelineの実現とは表現しない。拒否されたreviewと失敗CIは新しい実装・test・reviewで再考できる。

実Wasmはreview順×test順4条件と、CI修復中の再拒否を含む2条件、計6条件が成功した。
実効果93件、完全producer記録53,335件、実不正要求20件の拒否後の継続、記録変異1,149件の
拒否を確認した。変異は監査の試験であり、実行時攻撃の件数ではない。
同じproducer binaryのLuna xhigh 5人も、自由なwash操作と私有ファイルの利用で482.10365525秒、
inclusive 3,854,537 tokens、shell 137呼出し、実効果26件、記録32,775件、違反0件で完走した。
review拒否2回とCI失敗2回から回復し、実review/test効果の重複3組と最大5model turnの同時実行がある。
同じrunのEPF・実要求／効果／返答・共通採点を照合し、外側hashを再計算する11記録変異も拒否した。

途中の送信拒否はlocalsideが並行testの返答前に通知へ進んだ誤りで、本体の拒否が正しかった。
実reviewを保持し、次にenabledなofferの操作を直接駆動するよう直した。進行flagは追加していない。
EPFには、入れ子のrollで継続と重なる合法な受信を最初の候補へ入れない不足があった。
既存のalongside規則を候補生成へ使い、完全capture・source readiness・最終検査は維持した。
EPF native161件、Clippy、doctest、no_std/Pico build、Lean6module、新しい回帰のMiriが成功した。
実AIで露呈した私有closeの誤拒否は、監査器に残っていた旧応答長の仮定を実WASI codecへ合わせて
修正した。同じ原本を再監査しており、AIを再実行していない。アプリMiriは実行していない。
監査の重複呼出し除去とcodec変更の入力別の継承根拠も記録した。

原本は`feature-delivery-direct-20261115/wasi-v62/`と`live-v52/`、集約は同ディレクトリの
`qualification-v70.json`。common-environment-v23のsource/hashへ結び付けた有限認定と非採点pilotであり、
旧境界の6M実験や他sourceの結果は混ぜない。本体RustとLeanの判定規則は変更していない。
新境界の認定は23/48課題となり、残り25課題、通常生成全体、最終sourceの全family認定、本実験は未完。

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
受信側の一覧と各課題の移行記録を対応付ける。旧境界では29課題・23familyを移行し、
19課題が未移行だった。新しい型付きChoreoFS境界で実AIまで再確認したのは4/48課題であり、
旧認定や最終sourceでの全family認定と区別する。
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
今後の1runは1800秒、参加者200tool calls、domain delay1.0、同時run1とし、run内部の参加者は並行実行する。
時間予算の正本は[実験設定](experiment.json)である。旧600秒の非採点試行は元条件へ固定する。
期限到達だけでchoreographyやlocalsideを不正と判定しない。

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

以下の一覧は旧境界を含む各時点のsource記録である。過去の課題番号・Miri・認定数を、
現在の実ファイル境界の認定へ読み替えない。現在の正確な課題集合は実験契約の
`migration_status`と本書冒頭へまとめる。

| 対象 | 現状 | 残る確認・変更 |
| --- | --- | --- |
| 1E/1M/1H | 実資源Endpointの要求・結果・返却。1Hは7人が実結果で再考。実Endpoint・実Wasm試験と実モデルの予備記録あり | source固定後の全条件の認定と比較 |
| 2E/2M/2H | 両sectionの実Endpoint要求・結果・返却。実Wasm15条件と3/5/7人のLuna予備試験が通過 | source固定後の全条件の認定と比較 |
| 6E | 実Repo Endpointによるglobal。各拒否後に修正・再レビューを行い、承認後の並行な後続へ進む | 通常Git pinで実Wasm34条件、投影に基づくwire監査と928記録変異の拒否。旧sourceの3人の実モデル予備試験あり。現版の実モデル・全条件の認定は未完 |
| 8E | DB・test環境の実Endpoint委譲、単独所有の結果ファイル、実Wasm3条件、Luna成功・終端失敗の予備試験 | source固定後の認定と比較 |
| 9E/9M/9H | 実fork Endpointを両方受け取り、実要求・結果・返却。3/5/7人の実Wasm・実モデル予備試験あり | source固定後の認定と比較 |
| 16E | build/deploy環境の実Endpoint委譲。実Wasm3条件、早期終了拒否、3人のLuna予備試験あり | source固定後の認定と比較 |
| 3E | 旧版の機構・AI記録は当時のsourceに固定。現行は二段アダプターを除去し、直接Endpointのnative 7関数とClippyが成功 | 全体buildは受信lane割当のconst上限で停止。新しい実Wasm・AI・EPF／効果監査は未認定。現行7/48へ追加しない |
| 4E | 実REPO／TEST_ENV Endpoint、API再合意と両実装、編集権返却後のテスト。native22条件＋不正引数3条件、実Wasm5条件と実AI3人の予備試験が元採点・EPF／実効果監査まで成功 | 14/48へ算入。採点比較と最終sourceでの全family認定は未完 |
| 4M | 直接Endpointの実Wasm8条件・96効果、本番receiverの12負例、監査記録の9反例、実AI5人がEPF・実要求／返答・共通採点まで成功。実AI79.728423秒・687,694 inclusive tokens・5,624記録・違反0 | 新境界の25課題目、残り23課題。旧アダプターの成績とは別source。最終sourceの全family認定・本実験は未完。範囲は`benchmark/ports/4M.md` |
| 14E | 実装置Endpoint3本、実受信側の負例2件、実Wasm3条件と実AI4人のpilotが共通採点・EPF・実効果監査まで成功。simulatorは必要な入力の因果関係を検査し、独立した順序を許す | 16課題目として算入済み。最終sourceの全family認定・本実験は未完。範囲は`benchmark/ports/14E.md` |
| 13E | 直接Endpointの実Wasm25条件・200効果、本番receiverの15負例、監査記録の9反例、実AI3人がEPF・実要求／返答・共通採点まで成功。実AI76.462663秒・564,435 inclusive tokens・4,044記録・違反0 | 新境界の26課題目。試料・試験・品質結果を実データで結ぶ。最終sourceの全family認定・本実験は未完。範囲は`benchmark/ports/13E.md` |
| 13M | 直接境界の776イベント・29 view・7 localside。実Wasm全8条件（112効果）と実AI4人の同じrunがEPF・実効果・共通採点まで成功。実AI113.926秒・872,510 inclusive tokens・17効果・違反0 | 新境界の27課題目。科学的記録反例27件とrun証拠反例11件を拒否。最終sourceの全family認定・本実験は未完。範囲は`benchmark/ports/13M.md` |
| 5E | 実Endpointへ直接結合した9ファイル。実Wasm9条件と、同じbinaryによるLuna xhigh 3人の自由なwash操作が、完全EPF・実効果・共通採点まで成功。48.27秒、377,070 inclusive tokens、6効果、違反0。アプリMiriは使わない | 新境界の19課題目。残り29課題、最終sourceの全family認定・本実験は未完。範囲は`benchmark/ports/5E.md` |
| 5M | 直接ファイルの実Wasm9条件と同じbinaryのLuna5人pilotが、共通採点・完全EPF／効果監査まで成功。pilotは75.24秒、776,651 inclusive tokens、12実効果、違反0 | 新境界の20課題目。古い7-viewのtimeoutは成功へ含めない。範囲は`benchmark/ports/5M.md` |
| 5H | 直接ファイルの実Wasm11条件と同じbinaryのLuna7人pilotが、共通採点・完全EPF／効果監査まで成功。pilotは68.85秒、1,081,788 inclusive tokens、16実効果、違反0、実効果の重なり7組 | 新境界の21課題目。両方向の患者独立性と機構試験のCT/MRI並行効果を確認。アプリMiriなし。範囲は`benchmark/ports/5H.md` |
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
| M3 課題と接続 | 全48課題と採用追加課題のglobal/localside・実Wasm・共通採点・接続を認定する | 旧境界の移行は29課題・23family。新しい直接ファイル境界で1E/1M/1H・2E/2M/2H・3E・4E/4M・5E/5M/5H・6E/6M/6H・7E・8E・9E/9M/9H・10E/10M・11E・12E・13E/13M・16Eの27/48課題を、各sourceの実Wasm・実AI・EPF・実効果・共通採点で確認。残り21課題、最終sourceの全family認定・本実験は未完。比較接続の開発は保留 |
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
全runが1800秒を使う逐次実行の上限はAが1,296時間、最大4,560件で2,280時間である。
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

### 2026-11-12: ChoreoFS の型付き結合を実装中

全23系統の再点検では、codebaseのdesign経路も汎用close/mailboxの後に
別task Endpointへ要求しており、確認済みは20系統・24課題へ更新する。
残るwriting/API/CI-CDの3系統・5課題にも公開raw completionへの依存があり、
ファイルと実Endpointの結合としての認定は0件。これは既存の要求の実装不足であり、
新しい条件を後付けしたものではない。

WASI runtimeの作業版は、共有ファイルが実Endpointを所有するChoreoFsFilesを
guest起動前に渡す形へ変更した。raw pending/completionとnext_fileは公開経路から
削除し、実要求・実返信を同じ結合で処理する。完了したguest終了時だけ実Endpointを
回収でき、取消後の別filesystemへの差替えを許さない。実Wasm/Endpointの5試験、
公開型境界を含むdoctest9件、全target Clippyが通過した。

組込み・ライブラリMiri等の全gateと課題global/localsideの移行はまだ完了していない。
削除APIに依存するアプリのビルドが自動的に成立したとは扱わない。旧29課題の結果は
元source・元範囲に保持し、今回の結合の成功へ読み替えない。48課題の移行・実AI/実Wasm・
EPF・実効果・共通simulator採点という目標は維持する。

### 2026-11-12: 型付きChoreoFSライブラリの通常認定が完了

`hibana-wasip1-runtime`の`0362074`をmainへpushした。生のWASI完了値と
汎用close/mailboxを公開APIから削除し、ファイルが実Endpointを所有する経路に統一した。
独自payloadを任意のWASI要求へ読み替えるChoreoFsRequest実装も外部へ開放しない。
preopenのfd実データを保持し、fd全u8域の直接索引と成功したguest writeback後の
定数時間更新へ整理した。pollの16件という既存入力域は変更していない。

同一sourceで通常gateが完走した。nativeライブラリ88件＋例8件、compile-fail10件、
Miriはライブラリ86件＋例8件、Leanは20＋14定理と実VM520・ChoreoFS160判定の
照合、Pico 2の実Endpoint poll・リンク、Clippy/doc/配布アーカイブが通過した。
Pico 2はflash325,152bytes、24KiB予約stack込みRAM154,568bytesであり、実機と
worst-case stackの実測までを主張しない。全gateの最大メモリは702,464,000bytes。
完全な記録と検査source hashはWASIの`output/verification/choreofs-bound-files-final-source`
へ保持した。公開CIの完了待ちはローカル合格へ積み上げない。

アプリ・runner・handoff fixtureもruntimeの正本の隣接worktreeを参照するよう揃え、
manifest/lock/Cargo metadataの3箇所でcore/runtime/macros/WASIが各一つに解決することを
確認した。これは依存の整合の確認であり、削除APIに依存する既存課題のビルド成功や
実AI再認定ではない。課題側のglobal/localsideの置換と通し実行は未完であり、この境界の
認定数は0件のまま保持する。旧APIへのfallbackは作らない。

### 2026-11-13: EPFと重複する旧監査の継続・保持期間判定を除去

1/2/9系統に続き、6E/3Eの監査から独自の貸出窓、返却順、制御Endpointの保持期間の
再判定を削除した。8E/16Eの独自フレーム／貸出parserも共通journalとEPFへ統一した。
EPFが確認したreceive commitと、実際の要求値、共有効果、返答、元simulatorの採点を
結ぶ検査は保持する。この実データの照合をEPFだけで代替したとは扱わない。

共通journalは各frameを一意のEPF receive commitへ結び付ける。transportのreceiveは
previewであり、queueからの出力がcommit captureより遅れる場合もあるため、その到着位置を
受信の因果順に代用しない。旧6E/16Eの「loanの輸送ログだけを早める」負例は合法なcaptureを
変更しておらず、保持期間の違反を作っていなかったので削除した。6Eと3Eの早い実効果の負例は、
EPFが実際に確認した受信commitより前へeffectを移す。前後の同一引数の要求・close・結果の
欠落／重複を含む負例は維持する。

保存済み6Eの14実効果と3Eの17実効果を新readerで再監査し、EPFと実データの照合が成功した。
同じ原本への6Eの30変異、3Eの345変異も全件拒否した。これらは保存記録の監査試験であり、
実行時攻撃の阻止件数や新しい実Wasm／実AI実験には数えない。実モデル呼出し0、採点比較0。
記録・reader・source/hash・コマンドは`output/experiments/epf-audit-cleanup-v2/`へ保持する。

最初の変異バッチは同一process内で探索領域を繰り返し確保し2.75GB guardで停止した。
欠落frameに対応するcommitの件数を探索前に照合し、同じproduction readerを入力ごとに
子processで実行して領域を回収する形へ修正した。sourceの復元・clone・再buildは行わない。
停止は未完として保存し、最終の6Eは最大824,016,896 bytes、3Eは409,944,064 bytesで通過した。

8E/16Eへのcapture/checkpoint接続も記述したが、実Wasmの再認定は未完である。
現WASIライブラリから削除済みのraw completion、古いFdRequest、PrivateFs呼出しを使う
既存localsideがbuildを拒否されたため、旧API／旧監査へ戻さない。これらは型付きChoreoFSへの
課題移行が必要な既知の範囲であり、この監査整理を課題の完成・48課題の完了へ読み替えない。

### 2026-11-13: codebase／writing監査に残る重複を整理

貸出先・Endpoint封筒の独自再判定と貸出状態の集計を削除した。compiled projectionとEPFが
検査した受信commitだけを実データの照合に使い、送信ログで同じ値をもう一度処理しない。
輸送ログだけを移動するwritingのoverlapping_loansとcodebaseのreturn_before_resultは
source違反を作っていなかったので削除した。codebaseの早い実効果の変異は、対応する実受信の
EPF commitより前へ移動する。要求・実効果・返答・採点の照合、および途中経過を完了と誤認
しないための実際の資源返却の確認は保持する。別の保持者表や進行再生は追加しない。

最終reader sourceで通常テスト53件成功、98件の統合／モデル試験は未実行。最大メモリは
1,130,004,480 bytes。Clippyは移行中の食事課題の未使用close role定数14件で失敗しており、
抑制していない。新しい実Wasm、実モデル、採点比較は0件。この整理を型付きChoreoFS境界の
全課題認定へ読み替えない。コマンド、source/hash、失敗ログは
`output/experiments/epf-audit-completion-v1/`へ保存した。

### 2026-11-13: 食事課題の型付きclose境界と並行継続

9E/9M/9Hの各ファイルを実Endpointへ結び、成功したcloseでは実際の応答Endpointをguestへ
moveしてからWASI importを完了する形へ移行した。通常I/Oと実効果の応答を別の論理viewにし、
保持者・完了flagは追加していない。WASI参加者の送受信は各localsideへ直接記述し、旧guest
手順helperを削除した。WASI境界の実Wasm 11試験と同じ11試験のstrict-provenance Miriは成功。

最初の製品版は正常入力の3/5/7人版でprotocol・EPF・実効果・共通採点が成功し、並行する食事を
1/2/3件確認した。各20件、合計60件の記録変異も拒否した。ただし9Eの不正入力後の経路は、
thinkの再試行成功後に本体のparがPhaseInvariantで停止し、6条件全体の認定は失敗した。
元記録と製品sourceは`output/experiments/dining-close-authority-v1/`へ保存している。

同じ構造を本体の実Endpointへ縮小し、完了したrollの別候補として残るframe-free previewを
並行枝へ強制的に移す処理が原因と確認した。previewを親に保持し、既に受信した実frameが
借用の外へ取り残される場合は引き続き拒否する。localsideの手順や期待値は変更していない。
呼出し回帰26件、対象Miri 3件、共通生成器へ追加した4形状・32実行・公開操作5,040件・
保持preview 2,320件のRust/Lean照合が成功した。library 853件はframe保持条件を狭める前の
候補での成功であり、最終sourceの全通常gate・拡張認定の完走とは区別する。本体のClippyは成功。

監査のbaselineを変異検査から呼出元へ返し、同じrunを二度EPFへ通す重複も削除した。
未使用close role定数14件によるrunner Clippyの停止は解消済み。修正本体を選んだ製品と
6条件の実Wasm・実モデルによる再認定を続ける。ここまでの新しい実モデル呼出しと採点比較は0。

### 2026-11-13: 通信順序の重複監査を除去

EPFが検査する操作間の順序と委譲封筒を、食事課題の実効果監査でも再判定していた箇所を
削除した。共通journalのclose照合からも、独自の前回reply位置とimport順の判定を除いた。
fd・内容・要求／返答の一対一対応、実要求受信後かつ実返答前にある効果、実際の物理actor、
全資源返却、共通simulatorの採点は別の保証なので保持する。監査用の第二の進行状態は作らない。

最終readerの通常テスト53件とClippyは成功。98件の統合／モデル試験はこの通常テストでは
未実行であり、実Wasm 6条件による再検証と新しい実モデル実行は別途記録する。
ログは`output/experiments/dining-close-authority-v2/audit-cleanup-*.log`へ保存した。

整理後のv3製品では、正常入力の9E/9M/9Hが実Wasm・EPF・実効果・共通採点に成功し、
並行する食事は1/2/3件、各20件の記録変異も拒否した。9Eの不正入力後の再試行も、6実効果・
protocol完了・共通採点・EPFに成功した。この後の負例検査は、既に上書きされた古い入力を
改変して必ず拒否されると期待していたため停止した。これはruntimeの失敗ではない。

負例の対象を、その実効果に使われたファイルの最後のwriteと実際の結果readへ修正した。
既存の9E再試行記録を使い、最終readerで20変異を全件拒否した。うち5変異はEPF適合を
保ったまま実データ・効果との不一致を作る。上書き済みの入力だけを変更する元の記録は、
最終readerでも正しく受理することを別に確認した。実Wasmを再実行して成功数を水増しせず、
原本・reader・hash・変異と結果を`output/experiments/dining-close-authority-v3/`へ保持した。
最終Clippyも成功。9M/9Hの不正入力後の条件は未実行であり、6条件全体の完了とはしない。
この更新での実モデル呼出しと採点比較は0件。

### 2026-11-13: 食事6条件を条件ごとの試験processへ分割

v4の単一processでの再試験は、正常3条件と9Eの再試行条件に成功した後、9Mの監査変異中に
全子孫2,754,199,552 bytesでguardが停止した。9Mの実Wasmは10実効果・protocol完了・
共通採点に成功しているが、全体検証は未完として保存した。同じ記録の単独監査は
最大416,612,352 bytesでEPF・実効果とも成功した。

条件間に残る探索領域を避けるため、同じ6入力を6つの独立したRust試験へ分け、共通qualifierが
それぞれ別processで呼び出す。全6報告の課題・正常／不正入力・EPF・実返却・20変異の拒否と
同じruntime hashを集約で要求する。入力を減らすselector、上限緩和、アプリの手順変更はない。
このハーネス修正のClippyは成功。最終の6条件と実モデルは続行中であり、合格には数えない。

EPF適合を保つ5種類の実データ変異も、既存のproduction `--audit-protocol`を入力ごとの
子processで実行するよう統一した。実効果監査での拒否とEPFでの受理は両方要求し、探索領域を
条件や入力の間で親processへ蓄積しない。別のモデル・旧実装・追加のworkflow仕様は作らない。

### 2026-11-13: 型付きChoreoFSの食事3課題を実AIまで再確認

9E/9M/9Hの正常入力と不正入力後の再試行、全6条件で実Wasm・EPF・実効果・共通採点が成功し、
各20件、計120件の記録変異を拒否した。実効果は計60件、違反0。正常・再試行の両群で
並行する食事は3/5/7人それぞれ1/2/3件だった。
同じ実行ファイルを使用し、通過済み4条件は検査本体・実入力域・仕様・実行条件の一致を
確認して引き継いだ。5/7人の再試行条件はprocess分離後に実行し、最大メモリはそれぞれ
1,132,822,528 / 1,167,409,152 bytes。上限は2,750,000,000 bytesのまま。

同じbinaryでLuna xhighを動かし、全15参加者のrequested/returned model・effort、完了と
inclusive tokensをrawから独立監査した。モデルはコマンドを自分で選び、実wash・実Wasmから
要求し、各runのEPF・要求値・実効果・返答・元simulatorの採点が全て成功した。

| 課題 | Luna参加者 | 実行時間（後段の監査を除く） | Inclusive tokens | 実効果 | 並行する食事 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 9E | 3 | 21.8805秒 | 255,101 | 6 | 1 |
| 9M | 5 | 23.2486秒 | 470,767 | 10 | 2 |
| 9H | 7 | 29.3417秒 | 683,158 | 14 | 3 |

ChatGPT接続を使用し、APIキーへのfallbackはない。これは各課題1回の非採点pilotであり、
統計的な優位性・競合比較・全48課題の完成を意味しない。新境界での再確認は3/48課題、
残り45課題を継続する。最終本体の全通常gate・拡張認定も別の未完事項である。
全入力・source・binary/hash・完全capture・モデルraw・採点を
`output/experiments/dining-close-authority-v5/`へ保存した。4条件の引継ぎ根拠、
6条件の対応、実モデル集計とCSVも同じ場所に置く。元の停止記録はv4へ保持した。

次は8Eの旧ABI駆動・RequestReady/Validate等の自己通知・汎用close後の業務転送を除く。
移行の単位はglobalとlocalsideの全体とし、3つの操作ファイルと2つの結果ファイルへ最初から
実Endpointを持たせる。成功したcloseの権利は実Endpointとしてguestへ移し、DATABASEと
TEST_ENVの実要求・実結果・返却を同じglobalへ結ぶ。結果ファイルは実際のtest結果を所有する
参加者が直接応答する。既存の独立したprivate I/Oと入力の自由度を維持し、旧completionや
旧アダプターを復活させない。8Eの新経路はまだ実装・認定済みとはしない。

### 2026-11-13: 料理・製造課題の重複する手順監査を削除

12Eの貸出期間表と期間の重なり判定、11Eの貸出・返却・Readyの終了・Busy/retry列の
再判定を削除した。両readerは実効果の照合より先にEPFを一度実行し、その検査済みの
receive commitを共通journalへ結び付ける。検査後に同じrunをもう一度EPFへ通さない。
実際の要求値、WASIの入力、実効果、返答、必要な最終返却と共通simulatorの結果の照合は
別の検査として維持する。実際の機器使用時間の重なりも、権限の保持期間とは区別する。

最終readerで通常テスト53件・Clippy・buildが成功した。統合／モデル用103試験はこの
通常テストでは未実行。保存済み12Eの9実効果、11Eの5実効果とBusy/retryを含む別の5実効果を
再監査し、EPF・実データ・共通採点との対応が成功した。同じ原本に対する12Eの116変異と
11Eの56変異、計172件をすべて拒否した。空のReady/Busy返答のpayload検査はデータの検査として
維持した。検査中の最大メモリは1,132,036,096 bytes、原本と変異の監査はそれぞれ最大
56,180,736／194,183,168 bytesだった。

原本は変更せず、新reader・差分・hash・command・結果を
`output/experiments/epf-audit-pruning-v2/`へ保存した。置き換えられた中間readerと成功済みの
重複する出力は削除する。新しいWasm・AI実行と採点比較は0。
保存済み記録の監査成功を、型付きChoreoFSへの移行や48課題の再認定へ読み替えない。

### 2026-11-13: 14Mに残る機器貸出期間の再判定を削除

14Mの監査から、貸出・返却が要求を挟むことや、返却が返答の前後のどちらにあるかを
再判定する処理を削除した。操作順序と委譲の保持は、同じcompiled sourceと完全captureを
検査するEPFへ任せる。実要求・WASI入力・科学データ・実効果・返答・共通採点との照合、
実機器使用時間の重なりは維持した。有限prefixの受理を課題完了と扱わないため、最終返却の
件数も確認する。アプリの進行状態や新しい監査用workflowは追加していない。

最終readerで通常テスト53件、Clippy、buildが成功した。保存済みの通常経路14実効果と、
再考3回を含む24実効果の二記録がEPF・実データ監査に成功し、通常記録への283変異を
全件拒否した。変異は実効果監査の有限な検出試験であり、実行時攻撃の阻止数ではない。
通常テストの統合／モデル用103件は未実行。新しいWasm・AI実行・採点比較は0である。
最大メモリは通常テスト1,102,692,352 bytes、二記録の監査285,835,264 bytes、
変異検査81,100,800 bytesだった。

原本を変更せず、readerのsource hash・差分・binary・結果を
`output/experiments/epf-audit-regulatory-v1/`へ保存した。これは保存記録の監査整理であり、
14Mの型付きChoreoFS移行や48課題の再認定の完了には数えない。

### 2026-11-13: 8Eの型付きChoreoFS移行とoffer候補の修正

8Eのglobalと直接のlocalsideを25roleの実ファイル・close権限・資源要求へ組み直し、
旧WASIアダプターを削除した。ビルドは通ったが、実Wasmでは三つの実効果の後、結果通知の
parで停止した。古いrollの再訪候補が、未実行の後続resolverを隠していた本体の問題だった。
localsideの逐次化や操作順変更では回避していない。元の停止記録は
`output/experiments/api-direct-choreofs-v1/`へ保持する。

本体の候補選択を、sourceで後続する未完了のローカル操作を優先する形へ修正した。
実Endpointの再現を含む関連47試験、strict-provenance Miriの再現1試験、Leanの選択条件の
証明が成功した。同じ構造を共通の生成カタログへ追加し、生成器120試験と、4variantの
64実行・60,480公開操作probe・28,864個の保持済みpreviewのprobeでRust/Leanの不一致は0。
別のalias variantは独立したcompile-failで拒否を確認した。
これは修正範囲の検証であり、最終sourceの全gate成功とは扱わない。
結果通知まで進んだ実Wasmでは、8Eのglobalにguest終了通知がないことも判明した。
各guestの実ファイル返却後の通知をglobalへ明記した。localsideの操作順は変えていない。

修正後の同じbinaryで実Wasmの障害条件0/1/2が完走し、EPFと実効果監査が成功した。
三条件で実効果9件、不正なclose 9件の拒否、記録変異48件の拒否を確認した。
成功条件の共通scoreはcomplete=true、二つの終端失敗条件はcomplete=falseを保つ。
最大memoryは477,200,384 bytes、実Wasm機構試験全体は62.97秒だった。

同じbinaryでLuna xhigh 3人を成功・終端失敗の二経路で実行し、完全captureのEPF、
実要求・実効果・返答・共通scoreと、実モデル・effort・usageの独立照合が成功した。
各runで実効果3件、資源Endpoint二本の要求・返却、結果ファイルを両参加者が読み終える
ことを確認し、違反は0件。終端失敗を業務の成功へ変更していない。

| 条件 | 実行時間（後段の監査を除く） | Inclusive tokens | 業務完了 |
| --- | ---: | ---: | --- |
| 成功・障害0 | 30.882653625秒 | 254,868 | true |
| 終端失敗・障害2 | 40.944399458秒 | 254,291 | false |

実モデルは計6人、inclusive tokensは計509,159。API課題は必要なデータ因果に従う直列の
三効果であり、独立効果の重なりは主張しない。比較採点0の非採点pilotである。
binary SHA-256は`a2520751c09e2da033541576c7eefa264a31b9dcefb5b0ec74e4e73691ff87b4`。
凍結source・receipt・完全raw・各監査結果は`output/experiments/api-direct-choreofs-v3/`へ保存した。
再認定を4/48、残り44課題へ更新する。本体の最終全gate・全familyの最終認定は未完である。

### 2026-11-13: 本体のoffer修正で露呈した回帰の是正と全体検査

上の8E認定後、本体のRust全体検査で、先の候補順位の変更が拒否経路の既存2試験を
壊していたことが判明した。単なる受信済み要求から、後続の承認候補まで優先していた。
4Eの新境界への移行と追加のAI実験を一旦止め、この回帰の修正を優先した。

現在の修正は、後続routeの直前にある実際の送信／ローカル操作のcommitとsource順序を
必要とし、受信だけを根拠にしない。sourceのlaneではなく、投影済みの実laneで完了を読む。
別の後続操作がcommitした際には、同じviewの古い送信previewを解放する。実受信frameや
commit済みの分岐知識は捨てない。進行flag、保持者表やアプリ側の回避策は追加していない。
既存の拒否2件と8Eの再現を含む関連73試験、および選択条件のLean検査が成功した。

全体検査では、宣言済みの借用payload送信を通信前に拒否する`is_unavailable()`に
追随していない既存試験も見つかった。許可条件を変えず、拒否後の記録不変と正規の
継続を確認する試験へ改めた。受信laneの8試験と、残る5群56試験が成功した。
本体を追加変更していない範囲の合格結果は、元のlog・実入力・commandを保存して保持する。
結果は本体の`output/verification/api-offer-final-rust-20261113-v2/`へ保存する。

最新sourceのnative検査の残り、Clippy・文書・no_std、通常Miri 50件と検証toolのgateが
成功した。Miriには既存の拒否2経路を含むcloseの3試験も含め、無視された入力は0。
未変更のnative結果は元のcommand・sourceとともに保持し、変更した拒否試験を追加検査した。
共通生成検査はcanonicalな全62割当を開始し、新しい28/29番の2群が成功した。
残りの割当とconsumerの再確認は進行中であり、全体成功とは扱わない。
既に保存した8Eの実行結果を、この修正sourceの再認定へ読み替えない。

native検査はCargoのtarget一覧と各実行記録を照合し、通常の1,295試験、二つのbenchと
二つのexampleを含む全対象に実行漏れがないことを確認した。最初の停止と失敗logは残し、
修正した試験の再実行結果を対応付けた。libraryの拡張用28試験は通常実行ではignoreであり、
拡張認定の合格へ数えない。一つの全gate commandが成功したという記録にはしない。
`native-target-coverage.json`へ対応を保存し、凍結した差分・未追跡sourceとの一致を確認した。
生成用の入力manifestとは別に、integration testと単独Lean証明も含む
`all-verification-inputs.sha256`を保存した。検査中の本体sourceは変更していない。

全生成割当の完了を確認してから、同じsourceで残るLean model gate、portable処理量予算、
固定した実consumerの回帰を一つずつ実行する。生成検査が停止・失敗した場合はそれらへ
進まない。これは実行の予約であり、未実行のgateを合格としていない。

### 2026-11-13: 本体を固定したまま4Eの直接ChoreoFS経路を実装中

本体の生成検査では、追加した28/29番に加えてconstructorの0/1番が完走した。
同じ本体で残りを継続しており、この時点の合格は4/62割当である。本体のsourceは変更していない。
未実行の割当や後続gateを成功へ数えない。

独立したconsumer作業として、4Eを8個の操作／結果ファイル、33roleのglobalと直接の
localsideへ書き換えている。旧`FileBoundary`、汎用`next_file`とパスによる業務転送を除き、
各fileはWasm開始前から実Endpointを所有する。API定義・両実装・最終判断・テストの
closeは別々の実Endpointで受ける。actual resource requestと受理した入力を照合した後に
物理ownerが実効果を実行し、実結果を同じファイルと結果readerへ返す。

合意入力の`first`で両方の実装順を選べ、最終判断の`action`で`revise`か`test`を選べる。
これらはエージェントが渡す実データであり、アプリの進行flagや保持者表へ写していない。
複数回のAPI改訂、両実装順、最後のREPO／TEST_ENVの二つの実要求をglobalに残した。
共通simulatorの採点や障害条件は変更していない。既存の5条件のWasm fixtureも、新しい
各操作ファイルと結果ファイルを読む入力へ更新中である。

最初のビルド要求は本体検査との重複を防ぐresource guardに拒否され、compilerは
起動しなかった。その後、各生成割当に既存のguardを一つずつ適用する実行へ変更した。
合格した四割当はそのまま保持し、停止した未完のconstructor 2番だけを、旧入力・観測を
上書きしない別の出力先で再実行している。これは実行制御の変更であり、入力域・検査・
compiler・本体sourceを変えたものではない。

割当間にconsumerの開発用検査を実行し、4Eの型推論と返値の借用に関するコンパイルエラーを
保存した。該当する型の指定を修正し、次の割当間で再確認する。監査readerとモデル向けの
操作説明も実装したが、4Eの実Wasm・実AI・EPF／実効果監査は未認定である。
runnerはClippyと通常53試験が成功した。103件の統合／モデル試験は未実行であり、
その成功へ数えない。本体の生成検査の全完了を独立したconsumer実装の待機条件にしない。
4Eの古いnative専用workflow試験は削除し、旧経路を新しい境界の認定に使わない。
記録は`output/experiments/coordination-direct-choreofs-v1/`と`v2/`へ保存し、
再認定数は4/48のままとする。過去の四課題の実行を、現在の修正済み本体の再認定へ
読み替えない。


### 2026-11-13: 回帰確認と全体認定を分け、consumerを並行して開発

今回の既知の回帰は、修正後の再現・既存の拒否二経路・関連73試験と追加した生成28/29群で
確認済みである。通常の全62割当は本体全体の認定として継続するが、独立したconsumer開発の
再開条件にはしない。ユーザーが並行実行を許可したため、consumer側の監視器は本体の検査
プロセス群と自分の子孫を合算し、2,750,000,000 bytesでconsumer側を停止する。各仕事が
別々に上限まで使う方式にはしていない。本体の検査入力manifestとの一致も再確認した。

4Eは型検査に成功した後、実ビルドでglobal案の不整合を検出した。受理した本体を変更せず、
WASI側の操作Endpointの貸出・受渡し・返却と、確定時の編集権限の移動をglobalへ明記した。
初回と再考のために同じ業務手順を二重に展開せず、一つの反復を使う。エージェントに属する
ファイルEndpointはrun中そのエージェントが所有し、実終了通知と共有操作権の移動を区別する。
確定時の移動には、通常の貸出と異なる実Endpointメッセージ`Retire`を使い、公開操作が別の
出現を区別できるようにした。正の完了応答は入力判定の対応するarm内に残す。

同じ4E sourceを診断用のnative投影入口でも確認し、再入検査の拒否とconst評価の停止を
区別した。失敗した案と入力hashは保存し、検査の無効化や本体の変更で解消していない。
現在の案は通常の`forbid(long_running_const_eval)`で実ビルドし、コンパイル済みの投影を
出力できた。記録は`output/experiments/coordination-global-authority-v5/`へ保存した。

これはglobal案のビルド確認だけである。実WASIのcloseより前のEndpoint受渡しをSDKと
直接のlocalsideへ接続し、resolverの登録、現在の型付き実効果監査、fixtureとモデル説明を
このglobalへ合わせる作業は残る。既存のclose後のEndpoint所有契約を壊す変更は入れていない。
このbinaryを現状のまま実験に使わない。実Wasm・実AIは0件、4Eの認定はfalse、再認定数は
引き続き4/48である。基盤全体・48課題の完成とは報告しない。

### 2026-11-13: ホストが先回りして貸出を選ぶ案を撤回

4Eのcompile済みglobal案には、ホストが先に操作権を貸してから、その操作をAIが選ぶのを
待つ問題があった。これではAIが再考と確定を自分で選ぶことを保証できない。ビルド成功を
実行設計の成功へ読み替えず、この案を実験に使用しない。4Eの現在のlocalside／監査の
移行は未完であり、以前の33role案やcompile済みbinaryを実験用の完成版へ昇格しない。

SDKに試作した`close_via`は14個の境界試験を通ったが、この設計上の問題を解消する証拠に
ならなかった。未使用の公開APIと専用試作実装を削除した。既存のclose後の所有値受信を
含む11試験は、削除後にも成功した。原本と結果は
`hibana-wasip1-runtime/output/verification/close-via-v1/`に保存し、SDKの最終認定や
ChoreoFixの実験成功には数えない。ここでのWasmは境界fixtureであり、実AIは0件である。

wash本体、モデルのshellツール入口、WASI標準入力までの経路を点検した。コマンド名や
課題段階によるホストの許可表はなく、渡されたscriptを4096 bytesの上限内でそのまま
Wasmへ渡している。washには変数、条件分岐、for／while、パイプ、ファイル読み書き、
保存scriptの実装がある。Bash/POSIX全体ではなく、外部プログラム、関数、command
substitution等は未対応である。今回の4Eの詰まりを、その未対応機能の不足だと示す
再現はない。nativeのparser 3試験の成功も、実Wasm全機能の認定とは区別する。
点検原本とhashは`output/experiments/wash-freedom-inspection-v1/`へ保存した。

ChoreoFSはchoreographyの操作をシェルへ提示する投影の入口である。AIが実際の要求を
選び、受理前の拒否はWASIのエラーとして受け取り、別操作や再試行を選べなければならない。
ホストが次の貸出や操作を先回りして選ぶ機構を足す方向へ進まない。ファイルの実データと
Endpointの結合は保持し、連携・権限の仕様はglobalだけに置く。

この境界の具体的な欠陥をSDKの実Wasm試験で再現した。本体が早すぎるpath_openを
Unavailableで拒否した後、SDKがその拒否をWASI errnoへ戻さずguest全体の失敗にしていた。
SDKでは受理前のUnavailableをEACCESへ、read/writeの未結合fdをEBADFへ戻す。成功応答を
生成せず、fd binding・Endpointの継続を保持する。受理後の失敗、壊れた応答、所有値の移送失敗は
再試行へ変換しない。対応する20種の宣言済みWASI要求、二回の拒否、不適切なfd操作、
必要な操作後の同じopenの成功を実Wasmで検査し、境界13試験とSDKの通常ゲートは成功した。
記録は`hibana-wasip1-runtime/output/verification/unavailable-wasi-v1/`へ保存した。
これは実AIや4Eの通し実行ではなく、SDK境界の修正・検証である。4Eのglobal／localside移行は
依然未完で、実AI・EPF・実効果／共通採点の認定までは加算しない。目標は48課題、再認定済みは
元のsourceで4課題のままである。

### 2026-11-13: ChoreoFSの投影とshellの自由な試行を再確認

ChoreoFSを利用するためにホストが次の操作や貸出を選ぶ必要はない。AIがshellから
操作を選び、投影された実Endpointが受理前に拒否した操作はWASIエラーとして返し、
同じguestで別操作・再試行を可能にする。SDKの実Wasmでの拒否・再試行の検証は前節の
記録を保持する。washの通常コマンドエラーもstatusへ戻し、入力ループを続行する実装である。
このコード確認を、新しい実AI認定の成功とは数えない。

4Eで投影拒否を解くために追加していた、定義後に三つの操作Endpointを配り直す未検証の
global変更は撤去した。これらはlocalsideにも実装されておらず、成功した提案ではない。
終了時に使用しないファイルEndpointを返す通信と、そのためのrootのpar・受信待ちも除去した。
実際のFileExitとprocess exitは維持する。変更前のsourceと今回の検査記録は
`output/experiments/coordination-projection-correction-20261113/`へ保存する。

4Eには外側のrollとDEFINE_CLOSEの所有権に関する投影拒否が残っている。これは未解決で、
4Eの実Wasm・実AI・EPFと実効果の通し認定には進めていない。型検査・formatの成功を、
投影のconst評価や実行の成功へ読み替えない。本体は固定したsourceの生成検査を別途継続し、
今回のconsumer修正のために最初から再実行しない。

### 2026-11-13: 4Eの共通採点を実際のAPI・実装・統合試験へ結合

4Eの旧simulatorはAPI定義・二つの実装・試験のdone mapだけを見ていたため、先に試験し、
後から必要な呼出しを済ませる列でも完了になり得た。共通環境をv19へ更新し、API文書、
backend/frontendの実ソースと、そのAPI入力・実試験snapshotから完了を判定する。
統合試験では実際のPythonコードを負数・0・正数で実行する。定義前の実装もファイルを作り、
早すぎる試験はその時点の失敗を返す。呼出し順やchoreographyでsimulatorを制限しない。
後の作業で過去の失敗を合格へ変えず、同じbytesの再定義・再実装は無意味に無効化しない。
物理REPO競合は別に観測する。アプリに新しいphase・permission・done flagは追加していない。

新しい4Eの10試験は全24通りの呼出し順、両実装順、早すぎる実装からの修復、実コードの
誤出力、古いsnapshot、Pending入力、同一内容の再実行、呼出し側の結果改変、実REPO競合を
検査する。共通worldの全220試験と入力工具の17試験が成功した。入力manifestの差分は
4Eのsimulator・説明だけで、他47課題と共有simulator本体は不変である。48課題・元toolの
signature・参加者・物理resource metadata・825 checklistは維持する。
証拠は`output/experiments/coordination-world-causality-20261113/`へ保存する。以前の4E pilotを
新しいworld／新しい直接ChoreoFS経路の認定へ読み替えず、新しい実AI・採点比較は0件である。

4Eの投影拒否も四つの小さい型付きsourceで切り分けた。遠隔fileからrollを再訪できるまま
受信resourceを永久に移す形はreentry検査がfalseになり、実file Endpointを先に返す形、
独立に戻る一時貸出、その保護と独立parの組合せでは同じ検査がtrueになった。全投影の
成功を主張するものではない。実際の同じ検査をnativeで呼んだ結果であり、runtimeの
受理条件やLeanの意味論は変えていない。記録は
`output/experiments/coordination-authority-reproducer-20261113/`にある。これは診断用sourceで、
consumerの依存先にも4Eの実Wasm認定にも使わない。現在の4Eの拒否は引き続き未解決である。

### 2026-11-13: 現行本体・SDKで8Eの実Wasmと実AIを再確認

修正済みのcanonical sourceを変更せず、通常の`forbid(long_running_const_eval)`で8Eを
ビルドし、完全なsource/hashと同じbinary `301d5de741d25df1087b826bd8bb39d24da8f31dfc1ff282aa77d91011cdfee4`
に固定した。実Wasmの深さ0/1/2が成功し、計9効果、9回の不正closeの拒否、結果の早いopenと
繰返しread、48件の記録変異の拒否を確認した。変異は監査器の試験であり実行時攻撃の件数ではない。
記録は`output/experiments/api-free-shell-current-20261113/`へ保存した。

同じbinaryで各人の実Wasm shellをLuna xhighが操作する予備実行を二つ実施した。

| 条件 | 実AI参加者 | 時間 | inclusive tokens | 業務結果 | 監査 |
| --- | --- | --- | --- | --- | --- |
| 深さ0 | 3人 | 27.441037秒 | 254,534 | complete=true、違反0 | EPF・実効果・モデル記録が成功 |
| 深さ2 | 3人 | 27.336787041秒 | 271,209 | 想定した終端テスト失敗、complete=false、違反0 | 同じ監査が成功 |

両方とも全参加者が正常終了し、要求・実返答とも`gpt-6-luna`/`xhigh`、protocol完了、
runtime exit=0、基盤errorなし。各3件の実効果を別々に照合した。業務失敗を成功へ変えず、
protocol完了と業務採点を区別できている。EPFはそれぞれ完全な2,141／1,789記録を検査した。
inputとoutputの合計525,743 tokensを使い、reasoning分を二重加算していない。

実行原本は`output/experiments/api-free-shell-current-20261113-live-success/`と
`output/experiments/api-free-shell-current-20261113-live-domain-failure/`、再監査と小さい集計は
`output/experiments/api-free-shell-current-20261113-live-driver/`にある。model呼出しを伴わない
実Wasm試験と実AIの二実行は別に記録する。全て非採点pilotで、比較実験には算入しない。
本体は生成検査の未完割当を継続中であり、この結果を本体の最終全gate成功へ含めない。
8Eを現在版で更新した証拠であり、未移行／未認定44課題を完了へ数えない。

4Eの次の設計判断は、shellが終了する前に操作serviceを終了・退役させる必要が本当にあるか、
実際の課題の仕様から見直すことである。候補確定や公開がない4Eに、他課題の永久freezeを
持ち込まない。現在のAPI・実装・試験snapshotの対応は共通採点で守り、serviceの実I/Oの寿命を
runの実際の終了まで保つ案を評価する。これはまだ設計案であり実装済みとはしない。権利の
二重発行、受理済み要求の中断、古いsnapshotによる成功を許す形なら採用しない。

### 2026-11-14: 3Eの直接ファイル経路と、本体の終了後要求の待機を切り分け

連携仕様はglobalだけに置き、要求の受信と実効果はlocalsideが直接実行する。別のホスト
workflow、次の操作の選択、段階表は置かない。3Eの共有操作はChoreoFsFileのwrite_as/read_asへ
実Endpointを結合し、ClosedFile受信後のパス分岐で業務Endpointへ振り直す経路を撤去した。
他の17family・76ファイルには旧語彙の参照が残る。これは試験と補助参照も含む手動点検の
件数であり、76個の独立した欠陥や全課題の修正完了を意味しない。原本は
`output/experiments/research-report-direct-files-20261114/remaining-adapter-source-review-v1.json`。

3Eの複数回の否定的レビューを通る経路では、継続をelastic rollの既完了に依存させず、
実際のcombine要求と確認・資源移譲を同じglobal上で接続し直した。旧globalの成功記録は
この版の認定へ引き継がない。新globalの実Wasmでは早すぎるcombineを同じguestで拒否し、
二回の修正後に実際のcombineまで実行できた。一方、公開完了後の著者の再修正要求が
エラーにならずPendingを続け、試験は失敗した。3Eは依然未認定である。

この最後の待機はWASIを使わない実Endpointの最小例でも再現した。本体に、既存の不変な
受領索引へ永久移転を示す実受信の位置を加える修正を実装した。source上で受取actorから
その資源を送る経路がなく、移譲に因果的に後続する反復・選択外の通知を受信済みの場合に
限り、古い受領を待つ要求を送信前に拒否する。アプリの保持者表・終了flagは追加しない。
永久移転後の直接／借用操作の拒否と別の合法な継続、一時貸出のPendingと実返却後の再開を
確認した。本体の858ユニット試験、追加の異常画像試験、対象2試験のstrict-provenance Miri、
132 Leanモジュールのビルドが成功した。sinkの所有権foldの補題は前提付きの一般命題であり、
Rust全体のrefinement証明とはしない。全通常gate・生成入力域への補充は未完である。

記録は本体の`output/verification/retired-file-view-20261114-*`へ保存した。この修正を選ぶ
新しい3E binaryの実Wasm・EPF・実効果／共通採点・実AIはこれからであり、今回の本体の
個別成功を新しいアプリ認定や採点実験へ加算しない。全48課題の目標を維持する。

### 2026-11-14: 3Eの実Wasm完走と、実効果監査の境界の修正

通常flagsのbinary `fbcf9f959682211c505e728fb16fd4e15e511d9b5611fbc382fbf7f1c3d8b201`
で、通常18条件と不正入力・公開後要求の1条件が実Wasmを完走した。受信commitを含む完全な
EPF記録、実効果と共通simulatorの完了・違反0を確認した。既存の監査版で記録変異は
それぞれ3,870件／257件を拒否した。モデル呼出しと採点比較は0で、3Eはまだ認定へ加算しない。

追加の3回目・4回目の拒否条件は、先頭の2条件で実処理まで完了した後、4回目の拒否を含む
記録の監査で停止した。これは、遅れて出力されるtransport観測の行番号を、別の受信の実行順の
上限にしていた監査器の誤りだった。EPFが検査した実受信commitを、要求・効果・実返答の境界に
使うよう修正した。同じ4回拒否のrawで17実効果・9資源要求・6レビューを再監査し、違反0で成功、
343件の記録変異を拒否した。追加2件は実受信前の効果／実結果前の返答という境界を越え、
内部的に有効なeffect intervalを保っても、実効果照合が拒否することを確認している。

委譲の順序とenvelopeの二重検査は削除し、動的手順はEPFへ集約した。通常のWasm試験で
EPFを同じ記録に二回実行していた重複も除いた。監査器の判定を変更したため、保存済みの
通常18条件と不正入力1条件は新しい監査で再検査し、追加8条件の全実行と実AIの予備試験を
完了してから3Eを数える。globalやlocalsideの操作順、共有効果の受理条件はこの監査修正で
変更していない。本体の最新個別検査は861件のnative試験、Clippy、対象Miri2件、
no_std／thumbv6m／thumbv8mが成功。全通常gateと生成入力域の補充は引き続き未完である。

### 2026-11-14: 10Mの直接ChoreoFS経路と投影負荷の改善

10Eの実Wasm・実AI・完全EPF記録・実効果と共通採点を結ぶ認定は
`output/experiments/build-direct-files-20261114/qualification-v25.json`に保存した。
この時点の認定は15/48、残り33課題、採点付き比較は0件である。後続の本体変更を
以前の全課題の再認定へ読み替えない。

10Mは、各Wasmのファイル操作を貸し出された実FILE Endpointの型付き要求・返答へ直接
結び付け、ビルド・実オブジェクトの試験・再考・公開を一つのglobalと直接のlocalsideに
書き直した。汎用close後の業務振り分け、次のコマンドを教えるホスト通知、旧native台本は
使わない。テスターへは実際に受信したオブジェクトを提示し、公開側へは実際の検証済み候補を
送る。引数と実資源要求、実結果の一致検査は効果の前後に維持する。

このglobalのビルドで露呈した投影処理の再走査を減らした。受信レーンの構築と索引の更新、
固定actorの分岐知識、passive childの包含関係、画像の参加者・最初の委譲位置の導出が対象で、
Endpointの実行時進行規則はこの変更で修正していない。`forbid(long_running_const_eval)`を
保った実アプリのビルドは197.09秒、全子孫の最大503,922,688 bytesで成功した。
これは診断ビルドであり、実験用source receiptの認定ではない。

本体のnative867件とClippy、レーン割当の3つのLean補題が成功した。有限な追加確認は、
2,304通りのレーン分離、容量不足時の変更なし、固定actorの17,496比較、参加者と最初の委譲
位置の独立走査との一致を含む。Lean補題はパレットと色選択の性質であり、Rust全体の
refinement証明とはしない。runnerの通常54試験とClippyも成功し、Cargo metadataで本体・
runtime・WASI・EPFが隣接する正本を各一つ参照することを確認した。

記録は`output/experiments/cascading-build-direct-files-20261114/`。10Mの実Wasm26条件、
実AI、完全なEPF・実効果・業務採点の通し認定は未完であり、15課題へ加算しない。
本体の全通常gate・関連する拡張認定も未完である。

### 2026-11-14: 10Mで露呈した別契約の受信と、生成検査の拒否条件を修正

10Mの実Wasmは、初期actor発行順と台本側の拒否後の再試行を直した後、5ビルド・5試験・
1リンクまで進んだが、`FileReconsidered`を待つ間に到着した合法な`FileFinished`を
本体が破棄し、PhaseInvariantで停止した。原本は同じ証拠rootの`wasi-v54/`へ保持する。
本体は別契約の合法なframeを最後のlaneでも保持し、通常のtyped recvへ再試行可能な
Unavailableを返すよう修正した。不正frameの検査は維持し、localsideの順序変更で回避しない。

未到着・到着済み・offerの先読み保持済みを実Endpointの一つの回帰試験で確認した。
関連27試験、本体867試験、Clippy、3状態のstrict-provenance Miriが成功した。
生成検査は全Endpointの受信拒否を破壊的faultと同一視していたため、Unavailableと
進行・保持frameの保存を要求するよう補強し、同じ構造を共通カタログへ追加した。
`receive-generated-v68/`では2契約variant、144個の実source cutの公開問い合わせを
RustとLeanで照合し、不一致0だった。これを全割当の成功へ読み替えない。

Lean検査器は依存moduleごとに構築し、古いSourceUnfolding constructorの呼出しを現行の
完全な展開証拠へ直した。意味論と入力域は変えず、15個の検査器制御入力を保ったまま、
最大2,435,465,216 bytesでビルドが成功した。先の容量停止と型エラーも証拠として残す。
修正版を選ぶ10Mの実Wasm26条件、実AI、同一runのEPF・実効果・共通採点はこれからで、
認定数は15/48、残り33、採点付き比較0件のままである。本体の全通常gateも未完である。

### 2026-11-15: 10Mの待機を通信の破棄と区別し、非ブロッキングの実WASI読出しを実装

受信修正版の実binary `cc5a6d175b431c03b389a6eec8685c3729438ec041b88fabc34f815eae4dcca8`
をsource receiptへ固定した。`wasi-v72/`の最初の条件は実リンクまで進んだが、B/Cが
到着しない再考通知へのblocking readを継続し、180秒のdeadlineで未完となった。
この結果は26条件の成功ではない。別レーンの排他的な分岐を誤って待ち続ける問題は、
3参加者の小さい実Endpoint試験でも再現し、独立した受信を保持する対照試験と共に修正した。
29実Endpoint試験とその追加2件のMiriが成功した。反復visitの異なる選択を、headerだけで
同じvisitと推測して拒否しない。追加生成ケースでは、先に届いた別visitのpreviewと台本の
分岐期待が一致しない停止が残る。原本は`receive-multilane-generated-v77/`と`v78/`で、
全通常gateの未完項目として保持し、生成入力を削除して成功にはしない。

washから複数の通知を自分で選んで読めるよう、標準WASI NONBLOCKを実装した。
fdのflagsは既存のbindingと同じ一つのslotで保持し、権限やworkflow状態には使わない。
`dd if=FILE iflag=nonblock`は未到着ならEAGAINを返す。自動で次の操作を選ばず、通常の
blocking readも残す。実WasmでEAGAIN後の再試行、同じmessageの一回限りの受信、EOF、
close後に通常flagsで開き直すことを確認した。WASIのnative105件と新しい実Wasm fixtureの
strict-provenance Miriが成功した。記録は同じ証拠rootの`wasi-native-v83.log`と
`wasi-nonblocking-miri-v84.log`。10Mの新しいbinaryでの26条件・実AI・EPF・実効果照合は
依然未完で、認定数と採点件数を増やしていない。

### 2026-11-15: 10Mの18条件の完走と私有fdの衝突を修正中

`build-v91/`を固定した`wasi-v92/`では、全6開始順×深さ0/1/2の18条件が、実Wasm・
完全なEPF・実要求と効果／返答・共通採点まで成功した。その後の不正入力復帰条件で、
`/work/notes`のfd 6と投影ファイルのfdが衝突し、`ReservedFileDescriptor(6)`で停止した。
32分8秒、最大775,405,568 bytesで終了し、26条件の成功や実AI認定ではない。

問題はアプリ共通I/Oの固定fd配置で、本体の進行やWASIの衝突拒否を緩めて直さない。
私有オブジェクトの実fdを、実際の投影ファイルが使わないfdから構築時に割り当てる。
offset／open flagsは安定したChoreoFS objectに対応させ、guest fdと同一視しない。
これはfdの実データであり、権限表・保持者表・次の操作の管理ではない。
同じ全26条件の実行順だけを不正入力復帰から始める形へ変え、長い通常条件の前に
失敗箇所を検査する。18条件の原本は保持し、新binaryの成功へ無条件に読み替えない。
10Mの認定、実AI、追加生成ケース、本体の全gateは引き続き未完である。

修正後のapp lib 15件（投影fd 4..255の全数と複数fd配置を含む追加2試験）とClippy、
runnerの54通常試験とClippyが成功した。原本は`private-fd-checks-v97.log`と
`private-fd-runner-v99.log`。新しい実Wasmの通し認定はこれからである。

論文用の保存済みAI実行の一覧も、既存のRust readerを変更せず更新した。
`output/experiments/pilot-inventory-rust-v7/verified-v1/`に51個の異なるraw stream・
29 task IDを保存し、21 archiveのsource／binary／raw／usage／監査の整合性を確認した。
30件は過去の実行ファイル・sourceの欠落や監査未選択を明記して残す。旧7E・3Eの失敗や、
8Eの正当な終端失敗も除外せず、別source・simulator版の結果を成功率へ混ぜない。
新たなmodel呼出し・過去の監査の再実行・採点比較は0件であり、認定15/48とは別の集計である。

### 2026-11-15: 10Mの外部開始順の先決めを是正する

fd修正版`build-v100/`の`wasi-v102/`は、不正入力復帰の3条件で、各3回の実試験失敗から
再ビルドし、完全なEPF・実要求／効果／返答・共通採点まで成功した。検証中の設計点検で、
CLIのorder配列を初期resolverへ渡し、実際のfile要求より先に貸出順を決めていることを確認した。
これは自律的なshell操作の認定条件に合わないため、guardへ明示的な停止を通知した。
終了143、最大807,452,672 bytes。3条件の原本と未完のcaseを保持し、26条件成功とはしない。

この版で実AI認定へ進めず、実ファイル要求によって順序を選ぶglobalと直接のlocalsideへ
置き換える。hostの要求分類器、貸出表、progress flagや同じCLI入力の別名への移動では補わない。
現在は同じactorが所有する複数のlogical file viewで、先着する実要求が暗黙のrouteを選べる
構成を小さいsourceで確認している。実装案であり、投影・実Endpointでの検証はこれからである。
10Mは依然未認定で、認定15/48、残り33課題、採点比較0件を維持する。

### 2026-11-15: 実要求による10Mの順序選択を実装、通し再認定は未完

開始順のCLI引数と5個のresolver、compilerから各VMへのFILE再委譲を削除した。
DATAから受け取ったFILEは同じcompiler actorの別々のaffine viewのまま、それぞれの
ChoreoFSへ直接結び付ける。実際のtyped file要求が暗黙のrouteを選び、その後に必要な
実資源を貸し出す。共通のmetadata/private-I/Oは開始順の外へ一度だけ配置した。
次の実要求の後に、既存の成果物通知をその枝から送り、前の成果物のテストと次のビルドを
並行に進める。新しい通知、保持者表、選択済みflagや業務要求を振り直すadapterは置かない。

compiler localsideは、初回3回を数えるループと別の再ビルド実装を廃し、投影が示す
貸出・実成果物の配送・テスト集約を一つの直接のEndpointループで処理する。
`request-driven-app-v114.log`の7試験と`request-driven-runner-v115.log`の54通常試験、
両Clippyが成功した。改訂した26条件はモデルなしfixtureの実shell要求で順序を変え、
製品へorderを渡さない。現版の実Wasm・実AI・同一runのEPF／効果／採点は未完である。

本体は867内部試験、30実Endpoint試験、18投影診断、追加境界のstrict-provenance Miri、
Clippy、no_std/Thumbが成功した。新しい共通生成case32は4実行・130 admission・
40 parked-preview probeで不一致0、独立したsource-cut検査は136公開queryを確認した。
同一actorの暗黙選択を許しても、resolverのlogical view権限は拡張しない。候補保持者集合が
同じだけの異なる条件付き所有者も拒否した。記録は同じ証拠rootの`co-resident-*`にある。
Kaniの2起動試行は固定compilerの実体欠落で停止しており未検証である。本体全gate、
既存の生成case31、10M認定と本実験を完了扱いにせず、認定数・採点件数を増やさない。

その後、固定Kani compilerを復元し、検証専用のrange引数とimport gateの追随漏れを修正した。
`co-resident-kani-v130.log`はu16 controller mergeと条件付き所有者の比較・参加者unionの
3 harnessすべて成功、最大908,787,712 bytes。全Kaniの成功とはしない。

`wasi-v122/`は試験側が3人目の更新通知を駆動する前に2人目のビルド完了を待ったため停止した。
実際の更新をshellから受信する試験へ直した後の`wasi-v125/`は、成果物通知と貸出の
外側／内側parをglobalとcompilerが取り違えた不一致で終了した。現在は既存の成果物通知を
test/buildのfork前へ置き、通知の後にテストと次ビルドを並行化する。追加通知やflagで回避せず、
直接localsideとglobalの階層を一致させた。`request-driven-fork-v126.log`の7試験とClippyは成功。
停止原本を保持し、現在版の実Wasm全26条件と実AIは未完である。

同じCLIによる開始順先決めが、以前認定に数えていた10Eにも残っていた。実ファイル・効果の
過去の成功は維持するが、自律的な要求選択まで認定済みとはせず、現在数を14/48、残り34へ
訂正した。10Eも本体の同じ能力を使って修正し、実Wasm・実AI・EPF／効果／採点を再認定する。

### 11Eの実ファイル投影・NONBLOCK・実AI（11月15日、17課題目）

旧Ready/Busy・汎用close後の振り直しを除去し、2人の旋盤／フライス盤の4操作と
supervisorの報告操作を、それぞれ実Endpointを所有する5つのChoreoFSファイルへ結び直した。
資源Endpointの貸出、実要求の受信・照合、実効果、結果、返却をlocalsideへ直接書く。
2台の順序は実要求から選び、結果の読取りを次の資源利用の待合せ条件にしない。
保持者表、busy flag、到着flag、第二の進行管理は追加していない。

最初のbusy試験は通常のblocking書込みへ即時拒否を期待していた。実際には別viewの
先行操作を待ってから受理されており、それだけでcoreの不正な許可とは言えなかった。
一方、AIが待たずに別操作を選ぶため、標準NONBLOCK出力が必要だった。既存send Futureの
Preparing／Sendingを一つのまま使い、同じFutureで送信準備を確認してから送出する境界を設けた。
準備待ちならtransport poll・通信commitなしでEAGAINを返す。準備後は同じFutureの送信を完了し、
受理済みI/Oや所有権移送を未実行として再試行しない。通常send/recv/offerの呼出しは維持し、
アプリにpinや準備手順を書かせない。ChoreoFSが標準WASI NONBLOCK指定を処理し、washの
`dd of=FILE oflag=nonblock`から利用する。通常のredirectionはblockingのままである。

`manufacturing-direct-files-20261115/build-v21/`の同一app binaryで、全24作業順・
不正入力からの修正4条件・通常遅延のbusy切替8条件、計36条件が実Wasmで成功した。
各条件をEPF、実要求と効果・返答の照合、共通simulator採点へ接続し、違反0。
busy 8条件は各2組の異なる機械での実作業の重なりを持つ。記録変異2,448件を拒否した。
これは監査記録の変異であり、2,448件の実行時攻撃を阻止したという意味ではない。
source・binary・projection・完全capture・採点は同一runに結び付く。

続く`live-v23/subscription-11E-1794690488182584000/`では、実Luna xhigh 3人が
32.079503584秒、321,179 inclusive tokens、15 shell callsで完了した。
Aが使用中の旋盤への要求で実際のEAGAINを受け、フライス盤を選び、後から旋盤を再試行した。
実効果は4仕事と1報告、資源の実返却4件、producer記録1,499件。実作業の重なり1組、違反0で、
同じrunのEPF・実効果監査・共通採点が成功した。3人のmodel turnも重なった。
source保存と監査を含む試験時間は40.70秒。旧38.28秒のpilotとは条件・sourceが異なり、
速度改善の比較結果とはしない。API課金は使わず既存ChatGPT subscriptionで実行した。

基盤側は前段のcore lib 867試験、最終carrier境界の関連public 43試験・Miri 7試験、
WASI outgoingのnative/Miri各4試験、strict Clippy、thumbv6m no_stdが成功した。
3件の関連Kaniは、既存の入力保持と送信完了に加え、準備だけではtransport poll・公開を
起こさないことも検査して成功した。全321 harnessの最終認定を意味しない。
UIは107入力とparallel borrow 5入力、doctest 12件が成功した。拒否診断の期待lane値に
古い0が残っていた1件は現在の1へ修正し、拒否自体と診断の残りは維持した。
生成検査も同じ入口でco-resident choiceのcatalog 32を再実行し、4 scripts/runs・24 calls、
130 admission probesと40 parked previewsがruntime-only/model-onlyとも0。
独立checkerは18実cut・136公開queryを照合した。これはこの1 catalogの有限検査であり、
新しい準備APIの全生成入力、全population、以前のrank-31反例の解決とは区別する。

集約は`manufacturing-direct-files-20261115/qualification-v28.json`。
初回auditorの既定引数照合失敗、blockingへの誤った拒否期待、projection hashの保存漏れも
それぞれの元記録を保持し、新しい成功へ読み替えていない。全gate・最終全family source認定・
採点比較は未完。新境界の通し確認は17/48課題、残り31課題。次は12Eへ進む。

### 2026-11-15：12Eの実ファイル境界・51条件・実AI3人

`output/experiments/kitchen-direct-files-20261115/qualification-v25.json`へ集約した。
新境界の確認済みは18/48、残り30課題、採点比較は0件である。旧27課題目の12E認定は
旧境界の履歴へ分け、現在の認定に読み替えていない。

9本のChoreoFS操作ファイルが実Endpointを所有する。close後にパスで業務要求を振り直す処理と、
要求値を共有して器具の順番を選ぶCell/resolverを撤去した。全6器具順序をglobalに記述し、
実際のファイル要求がimplicit routeを選ぶ。localsideは器具Endpointの受信、実要求、実効果、
実結果の返信と返却を直接駆動する。材料は実際の完成結果を相手へ届け、一皿の完成を無関係な
他の料理や自身の担当材料の完成で止めない。追加の権限表、進行flag、所有者表はない。

`build-v21/`はruntime `49e68feee11c71afd2fcac5d6d5c37b65eb08b4118f943e82b8f40606462cc33`、
wash `90e6e07e240258d3363636b5edbb8a67513146267a7734f81513b7ddc6d4a8ec`、
source manifest `78db306e1292a36dbd6bf4fee9654611c1ebd493ca97a329db2667bc2061ef00`を保存する。
依存は検査中の正本に揃え、32ロールの投影を`long_running_const_eval=forbid`でビルドした。

`wasi-v22/kitchen-wasi-1794694767769241000/report.json`は、器具の全6順序と各料理人の
材料／調理の全8組、追加3条件の計51条件を完走した。共有効果459件、違反0、記録変異4,845件を
拒否した。全条件で同じrunの完全なEPF記録・実要求と実効果と返答・共通採点を照合した。
269.21秒、最大470,368,256 bytesである。記録変異は監査器の試験で、実行時攻撃の阻止件数ではない。

元の調理simulatorは処理時間ゼロであり、そのまま正の効果時間重複を要求した最初の追加試験は
誤りだった。`wasi-special-v19`は全効果と監査・採点が成功しても、試験としては失敗のまま保存した。
追加3条件だけ、test-onlyの環境接続から調理150ms・材料100msの明示した処理時間を設定した。
通常の実モデル接続にはこの設定APIがなく、元の採点・引数・資源制約は変えない。この3条件は
それぞれ実効果が3組重なり、他の料理の調理と自身の担当材料より先に一皿を完成できた。
不正な早い要求、型／引数不正、chunked入力、私有計算、重複からの回復も同じ実Wasmで確認した。

`live-v24/subscription-12E-1794695054473189000/`はLuna xhigh 3人の自由なshell操作で成功した。
23.9857885秒、inclusive total_tokens 332,800、shell 16回、同時model turn最大3、共有効果9件、
実ファイル要求9件、器具実要求と返却各5件、producer記録4,006件、違反0である。同じrunの
EPF・実効果・共通採点を照合した。テストの処理時間設定は0件で、元の処理時間ゼロの環境を使う。
これは非採点pilotであり、成功率・処理速度の優位性や全入力の保証へ拡大しない。
result SHAは`5902fe013d61d6d65fd8d8c0ed7b799e30bf87a6c152fc123f1ef242ff9d99c6`、
audit SHAは`e146c517bc3ee3bfa8c8a5811bb98f9d19508b062d0fe87664bd982131104ddb`である。

本体では`par(Reply, seq(par(First, Second), Foreign))`のForeignに自分の操作がない場合まで、
globalの範囲の末尾が異なるという理由で3分割を拒否していた。3ロールの最小再現を固定し、
現在のviewの操作がない後続区間だけを消去する修正を入れた。自分の実際の後続操作とrollの
境界は残す。localsideの順番変更で迂回していない。関連6 native targetの57件と、追加した
正負2例のstrict-provenance Miriが成功した。ChoreoFix自体はMiriへ流していない。
この保存済み12E binaryの後に、同じ構造を通常通信と実委譲の生成入力へ追加した。
追加入力で貸出前の受信試行がrunを故障させる問題を切り分け、別の本体修正で
利用不可として返し、正当な貸出と応答が続くことを確認した。
`generated-v33.log`では2構造・16完走・1,374公開操作試行と374 preview試行を照合し、
runtime-only/model-onlyは0、局所不正操作280件と独立compile-fail 4件を拒否した。
関連native 58件とstrict-provenance Miri 3件、foreign suffix/空区間のLean補題も成功した。
これは任意のRust実行のrefinement証明ではない。追補は`core-boundary-followup-v35.json`へ
固定し、保存済み実AIのsourceと区別する。本体の最終全gate、以前のcontracts-31反例、
12M/12H、最終全family認定は引き続き未完である。


### 2026-11-15：5Eの直接ファイル9条件・実AI3人

`output/experiments/medical-direct-files-20261115/qualification-v11.json`へ集約した。
新しい実Endpointファイル境界の認定は19/48課題、残り29課題である。
旧5Eの汎用close実装とそのMiri・AI記録を、今回の成功へ読み替えない。

9本の実操作・データファイルを、それぞれのEndpointへ最初から結び付けた。
共有効果の要求をclose後にパスや段階で振り直す処理を撤去し、紹介・所見・相談結果も
実メッセージのread-onlyファイルへ投影した。完了した両panel Endpointと実データは、
completed-record担当が一度だけ受け取る。そこから再貸出せず、実際の完了データと一致する
相談だけを公開する。二重の進行管理・所有者表・完了flagは追加していない。

`wasi-v10/`の全9条件は、分割・逆順・反復・combined panel、CTの挿入位置、不正入力、
panel完了後の重複要求拒否を確認した。全条件の完全EPF、実要求／実効果／返答、共通採点が
成功し、違反0。記録変異153件を拒否した。これは監査の負例であり実行時攻撃153件ではない。
実AIと同じruntime `b2373202d4878dc539143f85b0258751f59bd5ad67ab082c0154ccae23fc1877`を使い、
56.96秒、全子孫の最大1,151,254,528 bytes。入力・実データのnative 2件、runner通常54件、
両者のClippyが通過した。アプリMiriは実行していない。

`live-v9/`はLuna xhigh 3人が自由にwashを操作し、48.271922459秒、377,070 inclusive tokens、
shell呼出し18件、実効果6件、2,831 producer記録、違反0で完了した。同じrunの共通採点・
完全EPF・実要求と実効果と返答が一致し、両panelの実効果の重なり1組を確認した。
CTとpanelの重なりはこのpilotでは0であり、通常遅延の機構条件で2組を観測した。
共通worldは`common-environment-v23`。実データのshapeを扱う抽象課題であり、臨床上の
診断正しさ、存在しない処方・投薬効果、成功率、採点付き比較の結果とは区別する。

`live-v8/`ではER担当がCT未実施のままsummaryをblocking readし、他担当もその所見を
待ったため停止した。成功や完走したtimeout条件へ数えず、rawと停止理由を保存した。
通常readは自分の未完作業を待つことと、既存のnonblocking readを説明へ明記した。
コマンド列・操作順は固定していない。条件の違う二つのpilotの時間を改善率へ使わない。

本体の現在の正本では通常Rust・必要Miri・Lean/model・runtime-workの局所全gateが通過したが、
通常生成検査の全71割当は未完。最終sourceの全family認定・本実験・採点比較も未完である。


### 2026-11-15：5Mの直接ファイル9条件と実AI5人を認定

`output/experiments/multi-specialty-direct-files-20261115/qualification-v9.json`へ
同じruntimeの証拠を集約し、新境界の認定を20/48課題・14family、残り28課題へ更新した。
38 roleの一つのglobalで、23本の実操作・データファイルを5個のEndpoint viewへ投影する。
各viewはglobal内の委譲で実guest VMへ渡す。要求のclose後に業務Endpointへ振り直さず、
ファイル操作がそのままtyped業務要求になる。実資源の貸出・要求・応答・返却は各localsideへ
直接書き、完了したpanelのEndpointと実データは会議担当へ一度だけ渡す。

旧7-view案の実AIは、ERが自身の検査前にcandidateをblocking readしてtimeoutした。
その失敗とrawは`live-v5/`へ残し、成功や速度比較へ数えない。新案はguest自身が一つの
実操作viewを所有し、自分の未実行操作を必要とする先読みをWASIエラーで返す。
最初の新globalでは完了応答をrouteの外へ置いていたため、guestが片側の分岐しか観測できず
投影時に拒否された。完了panelの引渡しと実ファイル応答を完了の枝へ含めて修正した。
通知・flag・command recipe・別の許可判定を追加せず、本体・runtime・WASIのproduction
sourceも今回の課題対応では変更していない。

`wasi-v7/`は6通りのCT/MRI因果順、combined panel、不正入力と反復partial、独立CT/MRIの
並行要求の全9条件が完走し、同じ各runの完全EPF・実要求／実効果／返答・中立な共通採点が
通過した。検査前のcandidate・findings・summaryの読み出しからの回復と、完了後の重複要求
拒否を維持した。通常遅延の並行条件では実performと実interpretにCT/MRIの重なり各1組が
あり、全条件の違反0。効果監査の記録変異360件を拒否した。これは実行時攻撃360件ではない。
9条件の試験関数は185.50秒、guardの全子孫最大1,420,148,736 bytesである。

`live-v8/`はLuna xhigh 5人が自由にwashを操作し、75.243086458秒、776,651 inclusive tokens、
shell呼出し35件、実効果12件、producer記録7,504件、違反0で完了した。AI自身が早すぎる
所見readのエラー後に検査を要求し、正当な処理を続行した。同じrunのEPF・実効果・共通採点が
一致し、実効果の重なり4組、同時model turn最大5件を観測した。実AIでのCT/MRI同種効果の
重なりは0であり、機構条件の重なりを実AIの結果へ読み替えない。モデル実行の最大は
1,026,211,840 bytes、run記録変異11件も拒否した。2つの実入力・実データ試験、runner通常54件、
アプリとrunnerのClippyが通過し、アプリMiriは実行していない。認定後の文書更新前に、
build前後と検査後の2,212入力が一致することを照合した。

これらは保存sourceに紐づく有限の非採点認定である。臨床的な正しさ、存在しない処方・投薬効果、
全interleaving、成功率や比較優越性、本体の生成全71割当・最終全family認定は未完である。


### 2026-11-15: 5Hの直接ファイル投影を実Wasm・実AIまで認定

`output/experiments/critical-care-direct-files-20261115/qualification-v14.json`へ、
同じsource・runtimeの認定を保存した。50 role、23本の実操作／データファイル、7本の
VM所有Endpoint viewを単一globalで表し、旧close後dispatcherを撤去した。実際のtyped要求、
資源の貸出・要求・返却、結果とread-onlyの所見／診断／最終summaryをlocalsideへ直接書いた。
完了panelは自分のERへ移り、準備済みORは麻酔から手術へ移り、ICUは両資源の実要求を消費する。
放射線の担当は要求された患者のassessmentだけを受け取り、他方患者の受付を待たせない。
本体・runtime・WASI・EPFのproduction sourceは今回変更していない。

`wasi-v11/`の2試験関数・11条件は、6通りのCT順、partial／combined／反復panel、
不正入力と早すぎるreadからの回復、CT/MRI並行効果に加え、どちらの患者も他方のassessment前に
治療まで完了できる2条件を含む。全runで完全EPF・実要求／実効果／返答・中立な共通採点が
成功し、違反0。実CT/MRIのperformとinterpretにそれぞれ1組の重なりを観測した。
効果記録変異354件を拒否した。これは記録監査の検査であり実行時攻撃354件ではない。
試験関数は182.45秒、全子孫最大1,888,583,680 bytes。全90画像順の実Wasm全数検査とは言わない。

`live-v12/`は同じruntimeでLuna xhigh 7人が自由にwashを操作し、68.851604666秒、
1,081,788 inclusive tokens、shell呼出し43件、実効果16件、producer記録8,803件、違反0で完了した。
同じrunのモデル記録、完全EPF、実効果、共通採点を照合し、run記録変異11件も拒否した。
実効果の重なり7組、同時model turn最大7件を観測した。実AIの同種CT/MRI効果の重なりは0であり、
機構条件の並行性をAIの結果へ読み替えない。モデル実行の全子孫最大1,398,833,152 bytes。
実データ試験2件、runner通常54件、アプリとrunnerのClippyが成功し、アプリMiriは実行していない。
build前後および文書更新前の2,212入力を照合した。runtime SHA-256は
`c40de4b6f8d8c1ce6d14328690ee723dfd4d02d752a841ce4e4e44538501bc7c`である。

旧`wasi-v9`／`wasi-v10`の失敗は、アプリのprojection形式、case receiptのhash、A/B panel返信の
移行ミスを修正し、元の全11条件を`wasi-v11`で再実行した。失敗を合格へ読み替えていない。
認定は21/48課題・15family、残り27課題へ進んだ。本体の生成全71割当と最終全family認定、
残る課題移行・採点実験は未完。共通worldにない投薬・監視・退院等の実効果や臨床的正しさ、
成功率・比較優越性もこの有限の非採点試験からは主張しない。


## 2026-11-15: 6Eを独立したfeatureの直接ファイル投影で認定

6Eの外側rollが承認後の継続まで包んでいた問題をglobal側で解消した。
二つのfeatureをparにし、「拒否結果→新しい実装とレビュー要求」だけを反復する。
承認とmergeは反復の外へ置く。reviewerの一つのWasmは二つの実ファイルEndpointを
所有し、それぞれの実要求と実装結果を直接localsideが受け取る。repositoryの唯一の
Endpointは実処理担当へ初期委譲し、run終了まで保持する。二段要求、操作ごとの貸出broker、
全体の手動interleaving列挙を撤去した。承認flagや別の状態機械は加えていない。

本体とWASI runtimeを変更せず、元の34条件すべてが実Wasm・完全EPF・一対一の
要求／効果／返答・共通採点を通過し、違反0、記録変異3,080件を拒否した。
3回目・4回目の拒否、早過ぎる要求、型／対象違い、承認後の重複を維持した。
通常遅延では全条件で実装／レビューとmerge／レビューの重なりがあり、両初手の
深さ2交互経路では実装／レビュー3組が重なる。検査器の入力では不正要求probeを先に実施し、
最初のレビュー対象の実装を先に送ってから、両実装を並行に進める。期待値とsimulatorの
遅延は変えていない。production localsideの順序変更で回避した修正ではない。

同じbinaryのLuna/xhigh 3人・障害条件2のpilotは164.669516334秒、976,699 inclusive
tokens、shell 43回、レビュー4回（拒否2）、効果10件、違反0。4,880記録のEPF・効果監査と
元の共通採点が通り、run-record変異11件も拒否した。pilot自身の重なりは実装／レビュー1組で、
merge／レビューは観測していない。実AI turnは最大3人同時。機構試験の重なりをこのpilotへ
読み替えない。常駐するホスト判断層、操作権表、進行flag、代替filesystemは追加していない。

sourceは19 program・181イベント（前の試作は1,461）。記録した開発buildは11.90秒、
最大415,105,024 bytes。制御したコンパイル性能比較ではない。task data 2件、runner 54件、
両Clippy、共有／data fixture 17件も成功。application Miriは実行していない。
採用記録は `output/experiments/review-direct-files-20261115/qualification-v45.json`、
binary SHA-256は `833ce2cf007c04bf6cda7597a9f0afb07268a429029c4b3b0599aa19cc3ccc6c`。
使用する共通worldはv23（manifest 90c71ae26686bed3365453d97ed76ffd08b3d8eb2001103de9ba04675a8ba59f）
で不変。移行22/48・残り26。採点済み比較は追加0、全familyの最終公開認定は未完である。


## 2026-11-16: 6Hの成功後の継続を同じ資源Endpointの委譲で区切る

7人の実Wasmでレビュー拒否2回・CI失敗2回・staging失敗2回から回復する入力を追加した。
旧版は実際の肯定的validationを返した後にも、同じ担当のofferが旧StageFailedAを候補にした。
旧版の反復をlocalsideの都合で無視せず、成功後に同じSTAGING_ENVをA担当からB担当、
B担当からC担当へmoveするglobalへ変更した。受渡すhistoryは実際のdeployment／validation
データであり、保持者表・進行flagではない。新しいnative担当2人はAI参加者を増やさない。

1,237イベント・46 viewの全source検査、実際の否定的結果2回からのrollbackと成功、
Endpointの受渡し、B継続、移譲後の旧A操作のframe送出前の拒否、4種の不正CI入力による
効果0件、focused Clippyが成功した。本体のproductionは今回変更していない。
この変更後の実Wasm全2条件も`wasi-v92`で52.53秒、最大798,523,392 bytesで通過した。
通常条件は21効果・15,587 EPF記録、深さ2の回復条件は53効果・32,522記録で、両方とも
完全EPF・実要求／効果／返答照合・中立な共通採点が成功し違反0。レビュー拒否・CI失敗・
staging失敗を各2回、実rollbackを2回含む。同じruntime hashは
`c17c574596ea3f48d01e84b3ca2ca6fd8ba52162c16f9a730cc70e3bacf2cbe2`。
モデル呼出し0、採点比較0。派生集約は`mechanism-v93.json`、原本は`wasi-v92`へ残す。
旧版live-v78の成功を新しいglobalの実AI認定へ読み替えず、6Hは未認定、
移行23/48・残り25のままにする。

別のlive-v88は、TEST_WRITERのB review待ちとREVIEWERのA candidate待ちが循環した
状態を記録し、診断のため明示的に停止した。controller終了143、最大1,409,597,440 bytes、
設定budgetは1,800秒。完了した実験やtimeoutへ数えず、partialなモデル記録と停止理由を
保持する。モデルのterminal usageがないため、使用token数を0とも推計値とも報告しない。


## 2026-11-16: ChoreoFSの階層表示と一重のファイル記述

`live-v95`は全7guestがmerge前の読出しで待ったため診断停止した。完了・時間切れ・
使用token数0へは数えない。最初に疑った無条件拒否の前提は誤りだった。elasticな再訪前に
送信済みの継続は、後の未完の交換中にも到着し得る。実Endpointと既存Leanのqueued-
continuation定理で確認し、本体の進行規則は変更していない。非ブロッキングの機構probeは
4回のEACCESと、その後のEAGAINを確認して53実効果まで完走する。

実AI記録で別途確認した`ls /service/review`等の失敗は、親パスを作らず末尾名だけを
一覧化したファイルデータ側の不具合だった。実際の投影パスから親ディレクトリと直下の子を
導出し、WASI要求→実Endpoint→ChoreoFSの境界を保って修正した。WASI側もpath・fd・
materialの三重配列を廃止し、呼出側のarray/sliceを直接借りる。未使用の記述用generationを
削除し、実行時のbinding generationは維持した。別の権限表・workflowは追加していない。

変更後のWASI全ローカルgateが成功した。native lib 110件・example 8件、compile-fail
15件、library Miri 108件・example Miri 8件、Leanのpoll 520入力・path権限160入力、
Pico 2 link、実Wasm shell例、Cargo packageを確認した。Miriで解釈しない2件はLeanへ
データを出力する試験であり、native側では実行済み。Pico 2測定fixtureは予約stack24KiBを
含むRAM152,296 bytes、flash333,524 bytes。実機実行・最大stack・wash全体の実機容量の
認定ではない。ユーザーの許可に従い、必要なWASI容量はPico 2の実容量内で拡張できる。

`wasi-directory-v97`は同じ変更後runtimeで7人の実Wasm全2条件に成功した。
通常21効果・EPF15,906記録、回復53効果・EPF32,918記録で、完全EPF、要求／効果／返答、
中立な共通採点は両方成功し違反0。実washの`/service`・`review`・`interface`一覧も検査した。
試験53.68秒、build込みの全子孫最大1,067,155,456 bytes。runtime SHA-256は
`295b8bdf4ae086fd2e22cced011c66d531954799fd7ec13794ffdd63b5edcfc2`。
集約は`mechanism-directory-v98.json`と`wasi-storage-v98.json`。モデル呼出し0、採点比較0、
現在sourceの実AIと全family再認定は未完。23/48課題・残り25課題を維持する。


## 2026-11-16: 6Hの実AI・完全監査を認定、24/48課題

`live-directory-v101`は7人のLuna/xhighが自由なwash操作で、深さ2の修正・再考・rollbackを
含む全53実効果と正常終了まで完走した。517.343330458秒、8,533,959 inclusive tokens、
256 shell calls。レビュー拒否・CI失敗・staging失敗を各2回、実rollbackを2回含む。
完全EPF50,851記録、受理済みの要求／効果／返答と現在素材の照合、中立な共通採点は、
別実行を寄せ集めず同じrunで成功し違反0。7つのmodel turnは重なったが、実効果の
重なりは0件であり、並行性能の優位性へ読み替えない。

実行時の全子孫最大1,423,425,536 bytes、保存記録の監査は425,148,416 bytes。
実Wasm全2条件と同じruntime SHA-256は
`295b8bdf4ae086fd2e22cced011c66d531954799fd7ec13794ffdd63b5edcfc2`。
その間に変えた3つのbuild入力は記録説明のみで、通常build入口から正しいreceiptを作り、
sourceの不一致を無視して起動してはいない。raw・source・hash・最終usageを保存した。
集約は`output/experiments/staged-integration-direct-20261116/qualification-live-v102.json`。

直接ファイル境界は24/48課題・18系統、残り24課題。採点比較は0件のまま。
本体の全通常生成入力、最終sourceの全family認定、残りの移行と論文用本実験は未完。
使用したcanonical coreの基点とdirty patchのhashも記録し、最終本体合格と混同しない。
次は4Mの直接ファイル投影を実装する。元の自由なworldと公開前の内容一致による採点を守り、
受理した実要求から実効果までを、別のhost workflowや進行flagで補わない。

## 2026-11-16 13M直接境界：科学データと実2資源QC

13Mの旧汎用closeアダプターは復活させない。新しい`lab_quality`のQC localsideは、
受理した要求、SPECTROMETER・FUME_HOODそれぞれから実Endpointで受信した要求、
所有する実材料データを照合してから、一回の`verify_result`を呼ぶ。実承認・実拒否を
同じ科学的入力へ結び付け、両資源と呼出元へ実結果を返す。別の許可表・進行flag・
所有者表・業務ステートマシンはない。保持するのは実要求・材料・測定・返答だけである。

`proofs/endpoint-handoff/tests/lab_quality.rs`はこの実sourceをnativeで検査する。
5関数が成功し、別runtime間の2資源貸出・返却と両到着順／両subject／両verdictの8条件、
片方だけの要求では効果が起きないこと、差し替え14条件の効果前拒否を確認した。
最終検証のデータ照合は、否定・古い承認、別batch／別trial／別品質測定の12反例を拒否した。
自由なbatch名・culture文字列をcodecで固定せず保持する。共通simulatorは変更していない。
通常CIへ同じnative入口とClippyを接続し、アプリMiriや旧実装の復元検査は追加していない。

これはQC部分の制御されたnative fixtureで、完全な13M choreographyの実行ではない。
次は科学的候補の実Endpointを審査中に保持し、拒否なら返して再測定／再試行を可能にし、
承認なら古い反復から使えない保持先へ移す。QCが相手を切り替える選択と、独立した
生物試験・品質測定をglobalへ書き、各操作ファイルを実Endpointへ直接結び付ける。
その全体を実Wasm・実AI・完全EPF・要求／実効果／返答・共通採点で確認するまで、
13Mを新境界の認定件数へ加えない。26/48・残り22、採点比較0件を維持する。

13Mの全体sourceを776イベント・29 viewとして投影した。FILEとWASI rootの実Endpointは
最初の所有権移動で各科学者のactorへ渡す。nativeの通常view／候補viewとファイルviewが
同じactorのsemantic progressを使い、別の段階フラグや通知で分岐知識を同期しない。
sealed imageに科学者の通常受信とFILE送信が含まれることを実際に確認した。
初回に審査する相手と切替えは、QCの実ファイル要求による選択とし、旧ホスト側の
選択入力や汎用close分岐は復活させていない。承認した候補はINITへ移し、残る候補の
再審査中に、古い反復から承認済み候補を再編集する権限を戻さない。

3科学者のlocalsideには実Endpointの操作を直接書いた。品質測定2件は独立した枝とし、
要求・実資源の貸出／実要求／実結果／返却をそれぞれの枝で完了する。初期準備は現在2段、
品質測定開始は両QC承認後というsource上の制約が残る。共通worldの唯一の正解順とは言わず、
この制約と改善余地を認定前に記録する。
新たな入力エラー専用rollは、別の生物試験訪問とoutbound selectorが衝突したため採用しない。
動的なbatch名の不一致は、既存の失敗返信経路で効果前拒否と再試行を表す。
入力拒否と実生物試験の失敗を偽って読み替えないデータ検査も追加した。

最終sourceの9 native試験・Clippyは全子孫最大463,175,680 bytesで成功し、
`long_running_const_eval = forbid`を維持した。本体・WASI・runtime・simulatorは変更していない。
残るQC制御／実装置3系統／Wasm・runner・監査を実装し、実4 Wasm・実4モデル・
完全EPF・独立した実効果照合・共通採点まで成功することが13M全体の認定条件である。
現在のsource／partial boundaryの原本は`output/experiments/lab-quality-direct-20261116/source-v15.json`。

## 2026-11-16: 13Mの直接localsideと4 Wasm起動口を実装

QC制御、SPECTROMETER、CENTRIFUGE、FUME_HOOD/QCの実効果所有者を書き、7者の
localsideを直接Endpoint操作でつないだ。QCは実ファイル要求から相手を選び、両資源の
実結果と実材料を照合する。拒否では候補を返して再測定・再試行を待ち、承認した候補は
INITへ移す。実装済みという範囲であり、この全経路の実行認定はまだしていない。
保持するのは実際の合成・測定・試験・審査データであり、別の許可表や進行flagはない。

4つのWasm起動口はFILEとWASI rootの実Endpointを各科学者actorへ受け渡し、操作ファイルを
対応する実メッセージへ直接bindする。生物試験のresolverは委譲されたBIOLOGY view、
QCのresolverは実効果所有者の実結果を参照する。汎用closeアダプターと旧QC試作用の
メッセージ・source経路は残していない。

本番QC受信側の合成・所有権prefixを通す9 native試験が成功した。両subject・両verdict・
両到着順の8条件と、差替え14条件を検査する。入力拒否時のQC効果は0件であり、正当な
合成prefixの1効果をQC効果へ混ぜない。これらは候補の提出前で止める有限のnative prefixで、
最終返却・全体完了の認定ではない。試験入力で誤っていた分岐の分け方、材料の形、並行fragmentの
対応と候補の早期dropを直し、本体の進行規則や期待する拒否件数を変更して通してはいない。

固定sourceのnative試験・Clippy・`lab-quality-wasi`ビルドとClippyが成功し、全子孫最大
663,650,304 bytesだった。実バイナリの`--projection`も776イベント・29 viewを出力した。
binary SHA-256は`c1fdeae7413301547a4aeb5e7b4b84651de925a17c44531d9961ca39dcda5aef`。
原本は`output/experiments/lab-quality-direct-20261116/source-v29.json`、検証ログは
`devices-v26.log`。Cargo metadataは本体・runtime・WASIの正本となる隣接worktreeだけを
選択している。本体・WASI・runtime・共通simulatorはこの作業では変更していない。

ユーザーが許可したPico 2内でのWASI容量拡張は引き続き可能だが、今回は容量不足が
露呈していないため固定容量を増やしていない。ホスト上の4 Wasm用メモリ使用量を、
Pico 2への収容実測と取り違えない。残りは共通runner・EPF／実効果監査への接続と、
実4 Wasm・実4モデル・共通採点の同一runでの確認である。13Mは未認定、26/48課題・
残り22課題、採点比較0件を維持する。

## 2026-11-16: 13Mの全8実Wasm条件とWASIパス容量

共通runnerの`lab_quality_boundary::`へ8条件の有限入力域を接続した。深さ0/1/2、初回QCの
両subject、同じ相手の再審査と相手の切替え、品質測定の両順序、任意／空batch名、
省略／Unicode cultureを含む。各条件で誤ったbatchを実効果前に拒否してから同じguestが
続行する。通常遅延の条件ではQC／生物試験と、分析者の独立した2装置の実効果が重なる。
全8条件が112実効果・37,128完全capture記録でEPF、要求・実材料・実効果・返答と共通採点まで
成功した。全case index、深さ別の効果数、各監査成功を共通認定入口が要求し、部分実行を
全条件合格へ読み替えない。深さ0と2の各9件の記録反例も拒否した。実行時攻撃18件という
意味ではない。runner 53 native試験と全target Clippyも成功した。

実Wasmで旧40バイトのパス容量不足が露呈した。アプリの名前を切り詰めず、WASI runtimeを
64バイトへ拡張した。超過するopen/statはEndpoint操作前にENAMETOOLONGを返し、guestの
出力領域を変更せず、その後の正当なopen/stat/closeが成功することを実Wasmで検査した。
WASI runtimeの全gate、109件のMiriと8例、Leanのpoll 20定理・520実判断、path-rights
14定理・160実判断、no_std・Clippy・packageが成功した。最大子孫メモリ702,726,144 bytes。
Pico 2 fixtureは24KiB予約stackを含むRAM152,296 bytes、flash333,580 bytesで、
実機実行・最大実stackの測定とは区別する。library commit `34c17e6`をmainへpushした。

監査実装では、実transport受理とEndpointの受信commitを区別した。生物資源の移譲送信は
intentの受信より先に受理され得るが、実効果の前にはその資源の受信commitを要求する。
別subjectへの選択通知の受信完了を、独立した選択先の実効果の前提にする誤期待も除去した。
必要な実要求の受信条件は維持し、global・localside・本体の意味を変更して合わせてはいない。

同じruntime binaryを使う`live-v52`のLuna/xhigh 4人は113.926317333秒、872,510 inclusive
tokens、40 shell callsで全員正常終了した。元深さ2の生物試験失敗2回・QC拒否2回から回復し、
17実効果・業務完了・違反0となった。完全な5,873記録の独立監査は実行中であり、ここでは
13Mを認定数へ加えない。26/48・残り22、採点比較0件のままとする。
原本は`output/experiments/lab-quality-direct-20261116/qualified-v51`と`live-v52`。


## 2026-11-16: 13Mの保存済み実AI記録を監査し、新境界の27課題目を認定

前節の`live-v52`を新たにモデル実行せず、そのまま再検査した。問題は、現在の反復のFILE viewが
未受信でも、前の反復が完了していれば遅れた継続の受信が可能である、という本体の既存仕様を
EPFの通常反復展開が表していなかったことだった。本体の公開Endpoint試験8件では、この受信を
許可し、新しい送信は現在のFILE受信までPendingとする区別が成功した。globalやlocalsideの
順序・通知・flagを足して合わせる変更はしていない。

EPFは同じsourceの順序境界・同じ外側訪問にある前の選択済み訪問について、実行actorの必要な
全commitと非空性を確認する。受理だけ、別actor、別の外側訪問、未完了の訪問は流用しない。
新しい送信、ローカル操作、同じ出現の受理と同居送信の依存は維持する。別選択を試す場合も、
既存の受信の正確なprefixを再検査する。EPF `009735d`はnative/doctest165件、Miri140件、
全6 Lean module、Clippy、no_std・thumbv6mの全gateに成功した。最大子孫メモリは
1,379,024,896 bytes（上限2,750,000,000）。追加した8つのLean命題は完了と依存のcomponentの
性質であり、Rustによるsource範囲の抽出全体や一般的なRust/source refinementの証明ではない。

同じ監査器で全8 Wasm記録と実AI記録を再照合し、科学的な記録反例は機構2条件と実AIで
各9件、計27件を拒否した。実AIのrun証拠の反例11件も拒否した。実AIのEPF・実効果・モデル・
source・共通採点の通し監査は約11.99秒で成功した。この時間はAIの実行時間113.93秒と区別する。
全再監査の最大子孫メモリは224,051,200 bytesで、モデルの再呼出しは0件である。

認定原本は`output/experiments/lab-quality-direct-20261116/qualification-live-v94.json`。
同じruntimeのSHA-256は`cd6b1523d7cb0f1a80289cf6c3f0f7547910b9c25ea127db5fa9b5d370e3d93c`。
監査器のsource基点・dirty patch・1,555入力のhashとcompilerは`auditor-source-v92/`、
実際の依存解決は`auditor-metadata-v95.json`に保存した。coreは正本1個へ一致し、現在の本体の
production sourceも実Wasmを認定した保存版と一致した。これは通常生成検査71割当の全完了、
一般refinement、最終sourceの全family認定や採点比較の成功ではない。それらは別の未完項目とする。
現在27/48・残り21、採点比較0件で、次は13Hの直接委譲global・localsidesを進める。

### 13H：4装置の安全検査component（11月16日）

`src/tasks/lab_cascading/`へglobalと直接のofficer／effect receiverを実装した。
各装置の「要求→実Endpoint貸出」をparに置き、受信した4台の実要求すべてがファイルで
受理した入力と一致した後だけ、安全検査のDomain効果を呼ぶ。4つの実結果の一致を確認して
各装置へ返却し、ファイルへ実結果を返す。独立した装置取得を待機境界で逐次化せず、
localsideの順序変更、保持者表、進行flagや手順helperでglobalを補っていない。

初期の取得globalは「全要求→全貸出」という2つのparをseqにしていたため、各枝で
要求→貸出を進めるlocalsideと不一致になり、nativeでPhaseInvariantを検出した。
意図した独立取得をglobalの各par枝へ直接表現し直し、localsideを逐次化して合わせてはいない。
Hibana本体には変更を加えていない。旧sourceと失敗logは`failure-v2/`に残した。

最終componentはRust 1.100.0でnative4試験、Clippy（warnings deny）、整形に成功した。
24通りの実要求順序で最後の要求を一度保留し、効果0を確認して同じEndpointで再開した。
各装置への正しい型の別引数4件は、本番receiverが実効果0で拒否した。正・負の実結果2条件では、
本番officerも使い、4台の実Endpointの別runtimeへの委譲・返却と同じ実結果の返送を確認した。
これは合計30条件のnative fixtureである。正・負の本番officer2条件では、実際に使ったsealed
projectionと3producerの完全capture（各107記録、source 23イベント）によるEPF動的検査も
成功した。capture・checkpoint・transport frame・制御した実効果と返答を同じ原本へ保存した。
これは13H全体の実Wasm、実AI、共通simulatorや実効果の一般的な認定ではない。

選択したtest自体のCargo生成は成功したが、その後の無関係な全binary生成は停止した。
以後は同じCargo生成の依存artifactを使って対象testのみを直接rustcでコンパイルし、全4試験を
実行した。停止したCargo全体をビルド成功に数えない。最終対象のコンパイルは最大
292,126,720 bytes、native実行は5,537,792 bytes。最終Clippyと整形も成功した。
原本は`lab-cascading-direct-20261116/component-native-v4.json`とsource／command／log。
使用済みの再生成可能なtarget・binaryはユーザーの削除指示に従って撤去し、原記録・source・
hashを残す。13H全体の科学global・ChoreoFS/Wasm・EPF・実AIを引き続き進める。
現在27/48・残り21、採点比較0件を維持する。

### 13H：合成の再試行と枯渇のcomponent（11月16日）

合成のファイル要求、実装置／試薬Endpointの貸出、両資源からの実要求と実結果、
両Endpointの返却とファイル返答を同じglobalへ置いた。再試行はroll、完了と枯渇は
実結果に対応した型付き分岐とし、同じ試薬EndpointをQCへ渡す。実装側に残量カウンタ、
保持者表、進行flagを追加していない。科学的な引数codecは元の自由な化合物名を保つ。

初回失敗後、File viewの返答は受信済みでも、同じactorの所有者viewに資源返却の受信が
残ると次の要求がUnavailableになった。globalやlocalsideの順序は変えず、本体へ4イベントの
実Endpoint再現を置いた。進行許可の条件は変えず、他viewだけの待機と、自分／借用中の
viewを必要とする待機を分類する修正で、9件の公開回帰試験と13Hの8件が成功した。
自分の返答が未読なら拒否し、別view待ちの間はsend／reentryをcommitしないことも確認した。

合成の5条件はcontrolled native Domainによる機構試験で、実simulator、Wasm、実AIの
認定ではない。同一実行のEPFは74／120／166／212／74記録を受理し、既存の検査component
2条件の107記録ずつも維持した。source・完全capture・失敗時原本とcommandは
`output/experiments/lab-cascading-direct-20261116/synthesis-v5/`へ保存する。
native試験は既存の`proofs/endpoint-handoff`packageへ移し、正本のアプリlibを参照する。
これにより選択した試験だけのために無関係な全アプリbinaryをビルドせず、CIでも同じ
Cargo targetを一度だけ実行する。検査を省略したり別sourceを認定へ使ったりしない。

本体の生成検査・形式モデルとの対応、科学処理全体と検査位置、ChoreoFS／Wasm・実AI・
共通採点は未完であり、移行27/48・残り21、今回AI呼出し0・採点比較0を維持する。
