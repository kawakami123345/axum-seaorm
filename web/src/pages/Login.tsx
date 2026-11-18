import { useSearchParams } from "react-router";

function loginError(reason: string): string {
  switch (reason) {
    case "missing_nonce":
    case "missing_state":
    case "invalid_state":
      return "ログイン手続きの有効期限が切れたか、別のタブでログインが開始されました。";
    case "access_denied":
      return "認証がキャンセルされたか、アクセスが拒否されました。";
    case "provider_error":
    case "token_exchange_failed":
      return "認証サービスとの通信に失敗しました。";
    case "session_error":
      return "ログインセッションを保存できませんでした。";
    case "invalid_subject":
      return "このアカウントのユーザーIDを確認できませんでした。管理者に連絡してください。";
    case "no_id_token":
    case "invalid_token":
      return "認証サービスが発行した本人確認情報を検証できませんでした。";
    case "internal_configuration_error":
      return "認証サービスの設定に問題があります。管理者に連絡してください。";
    default:
      return "認証サービスからの応答を確認できませんでした。";
  }
}

const Login = () => {
  const [params] = useSearchParams();
  const error = params.get("error");
  const handleLogin = () => {
    window.location.href = "/api/login";
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-100 dark:bg-gray-900">
      <div className="max-w-md w-full bg-white dark:bg-gray-800 rounded-lg shadow-md p-8 text-center">
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-6">
          BookKeeper
        </h1>
        <p className="text-gray-600 dark:text-gray-300 mb-8">
          書籍管理システムへようこそ。
          <br />
          利用するにはログインしてください。
        </p>
        {error && (
          <p role="alert" className="mb-4 text-red-700 dark:text-red-300">
            {loginError(error)}
          </p>
        )}
        <button
          onClick={handleLogin}
          className="w-full bg-primary hover:bg-primary-dark text-white font-bold py-3 px-4 rounded focus:outline-none focus:shadow-outline transition duration-150 ease-in-out"
          style={{ backgroundColor: "#2563eb" }} // Fallback/Specific color if primary not set
        >
          {error ? "ログインをやり直す" : "ログイン"}
        </button>
      </div>
    </div>
  );
};

export default Login;
