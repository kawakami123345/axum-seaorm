# RustWeb

## アクセス制御

書籍の一覧・年度別一覧・集計は、管理者なら全件、一般ユーザーなら自分の書籍だけを取得します。
個別取得・更新・削除・申請日の変更も同じ条件で絞り込み、対象外の書籍は404を返します。
新規書籍の所有者はログイン中のユーザーです。出版社・店舗はログイン済みユーザーで共有します。

参照範囲の条件は `usecase/src/lib.rs` の `UserContext::book_filter` にまとめています。
書籍のユースケースと年度別集計で共通利用し、DBのユーザーID条件は `infra/src/book.rs` にあります。

セッションはDBの`auth_session`テーブルで管理し、認証のたびにサーバー側で24時間の有効期限を確認します。
ログアウト時はDBのセッションを削除するため、保存したCookieも再利用できません。
従来のユーザー情報入りCookieは受け付けないので、この変更後は再ログインが必要です。

書籍の更新は読み取り時の`revision`が一致する場合だけ保存し、競合時はHTTP 409を返します。
競合した場合は再読み込みしてから操作し直してください。
追加マイグレーションは既存の書籍・履歴を保持したまま、API起動時に適用されます。

## 日時と依存パッケージ

ドメイン・API・SeaORMのDBモデルの日時には `time::OffsetDateTime` を使います。
APIで受け取った申請日はUTCに揃えて保存し、年度もUTCで判定します。
APIはRFC 3339の日時文字列と`null`を扱い、申請日を省略した場合も`null`として扱います。
DBとの日時型変換は不要で、既存の `timestamp with time zone` 列をそのまま使えます。
Chronoの直接依存はありませんが、SeaORMやOpenID Connectの間接依存としては残ります。

TypeScriptは `typescript-eslint` の対応範囲に合わせて6.0系、reqwestはOpenID Connectとの互換性のため0.12系を使います。

フロントエンドのAPIクライアントは `openapi.json` から開発サーバー・ビルドの起動時に自動生成します。
生成先の `web/src/api/` はGitの管理対象外です。
手書きの通信処理 `web/src/lib/custom-fetch.ts` はソースとして管理します。
`mise run orval`はOpenAPIの書き出しからクライアント生成まで実行します。
`mise run openapi`でOpenAPIだけを書き出すこともできます。DBやKeycloakの起動は不要です。

Postgresとの日時の読み書きは、開発DBの接続情報を使って確認できます。永続データの書き込みは行いません。

```bash
cargo test -p infra --features test postgres_timestamp_round_trip -- --ignored
```

## 開発環境

言語と開発コマンドはmise、開発コンテナとサーバーはDev Containerで管理します。

| 設定 | 管理するもの |
| --- | --- |
| `mise.toml` | 言語・ツールのバージョンと設定、開発・検証コマンド |
| `.devcontainer/devcontainer.json` | DB・OIDC接続用ENV（`remoteEnv`）、ポート転送、接続先、miseの導入、VS Code拡張 |
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
mise run openapi       # APIを起動せずOpenAPIを書き出す
mise run orval         # OpenAPIを書き出してAPIクライアントを生成
mise run check         # Rustのチェックとフロントエンドのビルド
mise run test          # Rustワークスペースのテスト
mise run test:frontend # フロントエンドの単体テスト
mise run test:browser  # Chromiumで画面操作を検証
mise run test:database # PostgreSQLの専用スキーマでDB操作・移行を検証
```

`test:database`には起動済みPostgresと`DATABASE_URL`が必要です。
各テストは一意なスキーマを作成し、終了時に削除します。既存の開発用テーブルは変更しません。

`mise.toml`変更後は`mise install`を実行します。Dev Containerの設定変更後は
**Dev Containers: Rebuild Container**を実行してください。

DB・Keycloakの開発用データは名前付きボリュームに保存します。
コンテナを閉じると各サービスは停止し、データは保持されます。
Realmは初回のみインポートされるため、定義変更後は管理画面から既存Realmを更新してください。
