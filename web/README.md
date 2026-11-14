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

画面の組み立ては `src/pages/`、編集状態と更新処理は `src/hooks/` に置きます。
モーダル・保存ボタン・エラー表示は `src/components/Modal.tsx` で共有し、通信・表示形式・キャッシュ更新は `src/lib/` にまとめています。
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
