export const customFetch = async <T>(
    url: string,
    options: RequestInit,
): Promise<T> => {
    // クッキーからCSRFトークンを取得
    const csrfToken = document.cookie
        .split('; ')
        .find((row) => row.startsWith('csrf_token='))
        ?.split('=')[1];

    const headers = new Headers(options.headers);
    if (csrfToken && ['POST', 'PUT', 'DELETE', 'PATCH'].includes(options.method || 'GET')) {
        headers.append('X-CSRF-Token', csrfToken);
    }

    const response = await fetch('/api' + url, {
        ...options,
        headers,
    });

    if (!response.ok) {
        console.error(`Fetch error: ${response.status} ${response.statusText} for ${url}`);
        if (response.status === 401) {
            console.log('401 Unauthorized detected, redirecting to /login');
            window.location.href = '/login';
            // 遷移中のため、それ以上のエラーハンドリングを抑制するために未解決のPromiseを返すか、
            // 直ちにthrowする。ここではコンポーネントの状態更新を避ける意図。
            return new Promise(() => { });
        }
        throw new Error(`HTTP error! status: ${response.status}`);
    }

    const data = response.status === 204 ? null : await response.json();

    return {
        data,
        status: response.status,
        headers: response.headers,
    } as T;
};

export default customFetch;
