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
