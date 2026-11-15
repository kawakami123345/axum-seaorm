# RustWeb Frontend

React・TypeScript・Viteによるフロントエンドです。
開発環境の構築は[ルートのREADME](../README.md)を参照してください。

Dev Container内で、リポジトリのルートから実行します。

```bash
mise run dev:frontend
```

APIも同時に起動する場合は`mise run dev`を使います。
`/api`へのリクエストはViteからAxumへ転送されます。

依存を変更する場合は`web`で`bun add` / `bun remove`を使い、`bun.lock`を更新します。
OpenAPIクライアントの再生成は、リポジトリのルートから`mise run orval`を実行します。

書籍・出版社・予算の実装は、機能ごとに `src/features/` にまとめています。

| ディレクトリ | 役割 |
| --- | --- |
| `src/features/books/` | 書籍画面、一覧、編集フォーム、更新処理 |
| `src/features/publishers/` | 出版社画面、一覧、編集フォーム、更新処理 |
| `src/features/budget/` | 予算表示と集計計算 |
| `src/components/` | 共通の見出し・テーブル・入力欄・フォーム・モーダル・通信状態表示 |
| `src/hooks/` | 複数の機能で使う編集状態 |
| `src/lib/` | 通信・表示形式・キャッシュ更新 |
| `src/api/` | Orvalが生成するクライアントと型。直接編集しません |

各画面はデータ取得と操作を組み立て、一覧の表示は機能内のテーブルコンポーネントが担当します。
共通UIは`features`に依存させず、入力欄のラベル・保存中の操作制御・保存エラー・再試行を共通化します。
フォームを閉じると入力状態を破棄し、保存失敗時は入力内容を保持します。
一覧の編集ボタンはマウスとキーボードの両方で操作できます。
書籍の更新後は一覧・詳細・年度別データと予算集計を再取得してから保存処理を完了します。
出版社の更新後は出版社と書籍のデータを再取得します。
認証切れではログイン画面へ移動し、APIの4xxエラーを自動リトライしません。

検証はリポジトリのルートから実行できます。

```bash
mise run test:frontend
mise run test:browser
```

ブラウザテストの初回は `web` で次のコマンドを実行してください。

```bash
bun x playwright install --with-deps chromium --only-shell
```

ブラウザテストはポート4174で開発サーバーを自動起動し、API応答をモックします。Postgres・Keycloakの起動は不要です。
TypeScriptは `typescript-eslint` が対応する6.0系を使います。
