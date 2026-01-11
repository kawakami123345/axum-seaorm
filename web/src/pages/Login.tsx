

const Login = () => {
    const handleLogin = () => {
        window.location.href = '/api/login';
    };

    return (
        <div className="min-h-screen flex items-center justify-center bg-gray-100 dark:bg-gray-900">
            <div className="max-w-md w-full bg-white dark:bg-gray-800 rounded-lg shadow-md p-8 text-center">
                <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-6">BookKeeper</h1>
                <p className="text-gray-600 dark:text-gray-300 mb-8">
                    書籍管理システムへようこそ。<br />
                    利用するにはログインしてください。
                </p>
                <button
                    onClick={handleLogin}
                    className="w-full bg-primary hover:bg-primary-dark text-white font-bold py-3 px-4 rounded focus:outline-none focus:shadow-outline transition duration-150 ease-in-out"
                    style={{ backgroundColor: '#2563eb' }} // Fallback/Specific color if primary not set
                >
                    ログイン
                </button>
            </div>
        </div>
    );
};

export default Login;
