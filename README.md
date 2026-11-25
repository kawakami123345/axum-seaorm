# RustWeb

## アクセス制御

書籍の一覧・年度別一覧・集計は、管理者なら全件、一般ユーザーなら自分の書籍だけを取得します。
個別取得・更新・削除・申請日の変更も同じ条件で絞り込み、対象外の書籍は404を返します。
新規書籍の所有者はログイン中のユーザーです。出版社・店舗はログイン済みユーザーで共有します。

参照範囲の条件は `usecase/src/lib.rs` の `UserContext::book_filter` にまとめています。
書籍のユースケースと年度別集計で共通利用し、DBのユーザーID条件は `infra/src/book.rs` にあります。

セッションはDBの`auth_session`テーブルで管理し、認証のたびにサーバー側で24時間の有効期限を確認します。
ログアウトは `POST /logout` で行います。ほかの更新と同じく `csrf_token` Cookie の値を
`X-CSRF-Token` ヘッダーに付けてください。セッションの失効後、応答の `redirect_url` にブラウザを移動すると Keycloak からもログアウトします。
従来のユーザー情報入りCookieは受け付けないので、この変更後は再ログインが必要です。

開発環境では初回起動時に `.data/cookie.key` を生成し、再起動後も同じ鍵を使います。
本番ではランダムな32バイト以上の文字列を `COOKIE_KEY` に設定し、全インスタンスで共有してください。
従来の `COOKIE_KEY` はそのまま使えます。以前の自動生成鍵を使っていた場合は、切り替え時に一度ログインし直してください。

書籍・出版社・店舗は、取得時に返された`revision`を更新リクエストのJSONに必ず含めます。
書籍の申請日変更にも`revision`が必要です。削除は`DELETE /books/{pub_id}?revision=0`のように
クエリで指定します（出版社・店舗も同様）。版が一致しない場合はHTTP 409を返し、変更を保存しません。
DBへの書き込み時にも版を照合するため、リクエストの処理中に起きた同時更新も検出します。
更新のたびにDBが版を1つ進め、申請日変更の応答には保存後の`revision`を返します。
競合した場合、画面は入力を保持して一覧を再取得します。編集を閉じて対象を開き直し、
最新の内容を確認してから操作し直してください。古い版のまま再送しても上書きできません。

各レコードの`id`・`pub_id`は作成後に変更できません。更新リクエストにこれらのフィールドを含めると
拒否され、更新用SQLにも含めません。PostgreSQLのトリガーでも変更を拒否するため、直接SQLで
書き換えることもできません。`publisher_id`・`shop_id`による書籍の関連先変更は引き続き可能です。
追加マイグレーションは既存の書籍・履歴を保持したまま、API起動時に適用されます。

初期版の「申請済み」に日付が記録されていない場合は、閲覧専用の「日付不明」として引き継ぎます。
その書籍の `applied_without_date` は `true` で、正しい申請日を申請日変更APIで設定した時点で年度別集計に加わります。
所有者の情報が欠けている書籍は管理者だけが参照できます。旧版に記録のない作成・更新日時には移行時刻を使います。
旧版で同じ名前を持つ出版社もIDと書籍の関連を保持し、今後の登録・改名では重複を入力エラーとして返します。

## 日時と依存パッケージ

ドメイン・API・SeaORMのDBモデルの日時には `time::OffsetDateTime` を使います。
APIで受け取った申請日はUTCに揃えて保存し、年度もUTCで判定します。
APIはRFC 3339の日時文字列と`null`を扱い、申請日を省略した場合も`null`として扱います。
DBとの日時型変換は不要で、既存の `timestamp with time zone` 列をそのまま使えます。
Chronoの直接依存はありませんが、SeaORMやOpenID Connectの間接依存としては残ります。

TypeScriptは `typescript-eslint` の対応範囲に合わせて6.0系、reqwestはOpenID Connectとの互換性のため0.12系を使います。

`mise run dev`は、OpenAPIの書き出し・OrvalによるAPIクライアント生成・整形を済ませてから、
APIとViteを並列で起動します。生成に失敗した場合はAPIとViteを起動しません。
`dev:frontend`・`check`・`test:frontend`・`test:browser`も、処理の前に同じ生成タスクを実行します。
APIだけを起動する`dev:backend`ではクライアントを生成しません。
Orvalのツール定義と生成コマンドは`mise.toml`で管理し、`web/package.json`には重複して定義しません。
生成先の `web/src/api/` はGitの管理対象外です。
手書きの通信処理 `web/src/lib/custom-fetch.ts` はソースとして管理します。
`mise run orval`はOpenAPIの書き出しからクライアント生成まで実行します。
`mise run openapi`でOpenAPIだけを書き出すこともできます。DBやKeycloakの起動は不要です。

Postgresとの日時の読み書きは、Composeの開発DBを使って確認できます。永続データの書き込みは行いません。

```bash
mise run services:postgres
mise exec -- cargo test -p infra --features test postgres_timestamp_round_trip -- --ignored
```

## 開発環境

開発用ツールとコマンドはmiseで管理し、PostgreSQLとKeycloakはDocker Composeで起動します。
APIとフロントエンドは引き続きホスト上で実行します。

| 設定 | 管理するもの |
| --- | --- |
| `mise.toml` | 開発用ツール、接続情報、開発・検証コマンド |
| `mise.lock`・`.mise/locks/` | miseで解決したツールのバージョン・チェックサムと、Orvalの依存関係 |
| `compose.yaml` | PostgreSQL・Keycloakのイメージ、ポート、データ領域 |
| `config/keycloak/realm.json` | Keycloak初回起動時にインポートする開発用Realm・OIDCクライアント・ユーザー |
| `.vscode/` | VS Codeの推奨拡張とESLint設定 |

Docker Engineと`docker compose`をホストに用意してください。接続先や認証情報はローカル開発用です。
必要ならGit管理外の`mise.local.toml`でmiseの環境変数を上書きできます。

`mise.toml`の`[settings] lockfile = true`により、`mise install`やツール更新時に`mise.lock`を
自動生成・更新します。既存のlockfileがある場合は、`latest`という指定でも解決済みのバージョンを使います。
対応する最新バージョンへ更新するときは`mise lock --bump`、続いて`mise install`を実行します。
`mise.lock`と生成された`.mise/locks/`はGitで管理します。`Cargo.lock`・`web/bun.lock`は
アプリの依存関係を固定します。詳細は[miseのlockfileの説明](https://mise.jdx.dev/dev-tools/mise-lock.html)を参照してください。

## 起動

ホストに[mise](https://mise.jdx.dev/getting-started.html)を用意します。WindowsではRustのビルドにVisual Studio Build Toolsの
C++ツールとWindows SDKが必要です。LinuxではCコンパイラ、`pkg-config`、OpenSSLの開発用
ライブラリが必要です。エディターや`mise exec`を使わないコマンドからもツールを使う場合は、miseの
[シェル設定](https://mise.jdx.dev/faq.html#how-to-get-mise-working-with-my-ide)またはshimsを設定します。

リポジトリのルートで以下を実行します。

```bash
mise trust
mise install
mise run setup
mise run dev
```

`dev`はComposeのPostgreSQLとKeycloakが利用可能になるのを待ってからAPIとViteを起動します。
API起動時にはDBのマイグレーションを適用します。Ctrl+CでAPIとViteが停止します。
サービスの起動・停止には`mise run services:up`と`mise run services:down`を使えます。
コンテナの状態とログは`docker compose ps`、`docker compose logs -f keycloak`で確認できます。

| サービス | URL / 接続先 |
| --- | --- |
| フロントエンド | http://localhost:5173 |
| API / Swagger UI | http://localhost:3000/swagger-ui/ |
| Keycloak管理画面 | http://localhost:8080/admin/ |
| PostgreSQL | `postgresql://postgres:postgres@127.0.0.1:5432/rust_web_db` |

新規環境でのアプリとKeycloak管理画面のログインは、どちらも`admin` / `admin`です。
PostgreSQLとKeycloakのポートはlocalhostだけに公開しています。設定中のパスワードやシークレットと
あわせてローカル開発専用です。

PostgreSQLのデータはComposeの`postgres_data`ボリュームに保存されます。KeycloakはGit管理外の
`.data/keycloak/`を使い、これまでのKeycloak 26.8.2のH2データも引き継ぎます。サービスを停止しても
データは残ります。Realmは既存の場合はインポートをスキップするため、定義変更後は管理画面から
既存Realmを更新してください。PostgreSQLのメジャーバージョンを変える場合はデータ移行が必要です。
以前のmise PostgreSQLデータはComposeへ自動移行されません。必要な場合は旧DBから`pg_dump`を取り、
Composeの空のDBへ`pg_restore`してください。元のmise状態ディレクトリは削除しないでください。

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
mise run test:database # PostgreSQLを起動し、専用スキーマでDB操作・移行を検証
mise run services:up   # PostgreSQLとKeycloakを起動
mise run services:down # コンテナを停止（データは保持）
```

`test:database`の各テストは一意なスキーマを作成し、終了時に削除します。
既存の開発用テーブルは変更しません。

以前のmise PostgreSQLのデータを引き継ぐ場合は、設定を切り替える前に旧環境でダンプを作成し、
PostgreSQLとKeycloakを停止してください。KeycloakのH2データは`.data/keycloak/`からそのまま使います。

```powershell
mise exec -- pg_dump -h 127.0.0.1 -p 5432 -U postgres -d rust_web_db -Fc -f .data/postgres-mise.dump
mise daemons stop keycloak
mise daemons stop postgres
```

新しい設定へ切り替えた後、APIを起動する前に空のCompose DBへ復元します。

```powershell
mise run services:postgres
$containerId = docker compose ps -q postgres
docker cp .data/postgres-mise.dump "${containerId}:/tmp/postgres-mise.dump"
docker compose exec -T postgres pg_restore --no-owner --no-acl --exit-on-error --single-transaction -U postgres -d rust_web_db /tmp/postgres-mise.dump
```

すでにデータのあるCompose DBへ復元しないでください。元のmise状態ディレクトリとダンプは、
移行を確認するまで削除しないでください。
