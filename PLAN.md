# EPUB 読書支援アプリ — 要件定義 / 設計（フロント抽出版・MVP）

## 1. 概要
- **目的**: macOS 上で DRM フリーの **EPUB** を読み、**表示中 or 選択中のテキストだけ**を LLM に渡して Q&A する。
- **方針**: **テキスト抽出はフロントエンドのみ**（Ask 時に一度だけ）。バックエンドは LLM 呼び出し / Keychain / DB に専念。
- **スコープ**: EPUB のみ（PDF/DRM/ローカル LLM/OCR/テレメトリは対象外）。UI は **英語のみ**。

---

## 2. プラットフォーム / 技術
- **OS**: macOS 13+
- **App**: Tauri（Rust + WebView）
- **EPUB**: Readium **ts-toolkit**（Navigator / Locator を使用）
- **保存**: SQLite（`~/Library/Application Support/<App>/app.db`）、API Key は **macOS Keychain**
- **LLM**: **OpenAI 互換 API**（`/v1/chat/completions` ストリーミング）

---

## 3. ユースケース（MVP）
1. **読む**: EPUB をインポート → ライブラリ → TOC で移動 → 前回位置に復元
2. **Ask (Selection)**: 選択テキストで質問 → LLM 回答 + **引用**（章 + loc） → クリックでジャンプ
3. **Ask (This Screen)**: 画面に見えている範囲で質問（オプションで隣接セクション ±1 を追加）
4. **注釈**: ハイライト/ノートを CFI 範囲で保存 → Markdown 出力

---

## 4. フロント/バックの責務分担

### 4.1 フロント（WebView / ts-toolkit）
- **表示/ナビ**: ts-toolkit の **Navigator** を使用、現在地 **Locator** を購読
- **Ask 時のみ抽出**:
  - 優先 1) `selectionText`
  - 優先 2) `visibleText`（画面内の段落のみ、整形・上限カット）
- **送信**: `locator` を必ず添えて **Tauri.invoke('qa_ask')**
- **受信**: ストリーミング回答の表示、**引用クリックで `navigator.goToLocation(locator)`**

### 4.2 バック（Tauri / Rust）
- **QA 実行**: Keychain から API Key を取得 → OpenAI 互換 API へプロンプト送信（ストリーム）
- **保存**: 書籍の最終位置 `last_locator_json`、QA ログ、注釈（CFI）
- **イベント**: `qa_stream`（回答）、`navigate_to(locator)`（引用ジャンプ用）

---

## 5. 共有データモデル（Locator / Payload）

### 5.1 Locator（アプリ内共通 DTO）
> `locations` の各プロパティは型上 optional だが、**アプリ運用では最低 1 つ以上の“手がかり”を必須**（後述ポリシー）。

```ts
type Locator = {
  href: string;        // リソース（spine item）
  title?: string;      // 章タイトル
  type?: string;       // "application/xhtml+xml" 等
  locations?: {
    cfi?: string;              // EPUB CFI
    progression?: number;      // 0..1（章内相対位置）
    position?: number;         // 擬似ページ連番（Positions List があれば）
    totalProgression?: number; // 0..1（本全体）
  };
  text?: {
    highlight?: string; // 抜粋（引用表示に有用）
    before?: string;
    after?: string;
  };
};
