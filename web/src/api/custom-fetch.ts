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
        throw new Error(`HTTP error! status: ${response.status}`);
    }

    // 204 No Content の場合は null を返す
    if (response.status === 204) {
        return null as T;
    }

    return response.json();
};

export default customFetch;
