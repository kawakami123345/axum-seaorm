# RustWeb

## 開発環境

言語と開発コマンドはmise、開発コンテナとサーバーはDev Containerで管理します。

| 設定 | 管理するもの |
| --- | --- |
| `mise.toml` | 言語・ツールのバージョンと設定、開発・検証コマンド |
| `.devcontainer/devcontainer.json` | DB・OIDC接続用ENV（`remoteEnv`）、ポート転送、接続先、miseの導入 |
| `.devcontainer/compose.yaml` | 各サーバー自身の起動設定、コンテナ間接続、起動待ち、データ永続化 |
| `.devcontainer/keycloak/realm-export.json` | 開発用Realm・OIDCクライアント・ユーザー |

アプリが参照するPostgres・Keycloakの接続情報は`devcontainer.json`の`remoteEnv`にまとめます。
言語・ツール用のENVが必要になった場合は`mise.toml`で管理します。
`POSTGRES_*`・`KC_*`は各サーバーコンテナの初期化・起動用なので、Compose側で指定します。

## 起動

ホストにはDocker Desktop、VS Code、Dev Containers拡張を用意します。
WindowsではDocker DesktopをLinuxコンテナ / WSL 2で起動してください。

1. このリポジトリをVS Codeで開き、**Dev Containers: Reopen in Container**を実行します。
2. 初回セットアップが終わったら、コンテナ内のターミナルで実行します。

   ```bash
   mise run setup
   mise run dev
   ```

PostgresとKeycloakはコンテナ作成時に自動起動します。Rust・Bun・Node.jsは`mise install`が導入し、
`mise run setup`でフロントエンドの依存を`bun.lock`からインストールします。
API起動時にDBマイグレーションが実行されます。Ctrl+CでAPIとViteを停止できます。

| サービス | URL / 接続先 |
| --- | --- |
| フロントエンド | http://localhost:5173 |
| API / Swagger UI | http://localhost:3000/swagger-ui/ |
| Keycloak管理画面 | http://localhost:8080/admin/ |
| Postgres（コンテナ内） | `postgres://user:password@postgres:5432/rust_web_db` |

アプリとKeycloak管理画面の開発用ログインは、どちらも`admin` / `admin`です。
設定中のパスワードやクライアントシークレットはローカル開発専用です。

ポートは`devcontainer.json`から転送します。VS Codeの接続中に上記URLへアクセスできます。
Postgresのホスト側接続先はVS Codeの「ポート」パネルで確認してください。
作業用コンテナはKeycloakとネットワークを共有し、ブラウザとAPIでOIDC issuerの
`http://localhost:8080`を一致させています。

## 開発コマンド

```bash
mise run dev           # APIとフロントエンドを起動
mise run dev:backend   # APIのみ
mise run dev:frontend  # フロントエンドのみ
mise run setup         # bun.lockから依存を再インストール
mise run check         # Rustのチェックとフロントエンドのビルド
mise run test          # Rustワークスペースのテスト
```

`mise.toml`変更後は`mise install`を実行します。Dev Containerの設定変更後は
**Dev Containers: Rebuild Container**を実行してください。

DB・Keycloakの開発用データは名前付きボリュームに保存します。
コンテナを閉じると各サービスは停止し、データは保持されます。
Realmは初回のみインポートされるため、定義変更後は管理画面から既存Realmを更新してください。
