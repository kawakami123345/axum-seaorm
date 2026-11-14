export class ApiError extends Error {
  readonly status: number;

  constructor(message: string, status: number) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

export function getErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : "処理に失敗しました。";
}

async function readErrorMessage(response: Response): Promise<string> {
  try {
    const body: unknown = await response.json();
    if (
      body &&
      typeof body === "object" &&
      "error" in body &&
      typeof body.error === "string"
    ) {
      return body.error;
    }
  } catch {
    // Non-JSON errors can be returned by the authentication middleware or proxy.
  }
  return `通信に失敗しました (HTTP ${response.status})`;
}

const mutationMethods = new Set(["POST", "PUT", "DELETE", "PATCH"]);

export const customFetch = async <T>(
  url: string,
  options: RequestInit = {},
): Promise<T> => {
  const csrfToken = document.cookie
    .split(";")
    .map((cookie) => cookie.trim())
    .find((cookie) => cookie.startsWith("csrf_token="))
    ?.slice("csrf_token=".length);

  const headers = new Headers(options.headers);
  if (
    csrfToken &&
    mutationMethods.has((options.method ?? "GET").toUpperCase())
  ) {
    headers.set("X-CSRF-Token", csrfToken);
  }

  const response = await fetch(`/api${url}`, { ...options, headers });
  if (!response.ok) {
    const error = new ApiError(
      await readErrorMessage(response),
      response.status,
    );
    if (response.status === 401) window.location.assign("/login");
    throw error;
  }

  return {
    data: response.status === 204 ? undefined : await response.json(),
    status: response.status,
    headers: response.headers,
  } as T;
};

export default customFetch;
