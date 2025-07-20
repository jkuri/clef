import { useAuthStore } from "@/stores/auth";
import { API_URL } from "./config";

export const api = {
  async request<T>(url: string, options: RequestInit = {}, skipAuth = false): Promise<T> {
    const fullUrl = this.getFullUrl(url);
    const authStore = useAuthStore.getState();

    if (authStore.token && !skipAuth) {
      options.headers = {
        ...options.headers,
        Authorization: `Bearer ${authStore.token}`,
      };
    }

    const response = await fetch(fullUrl, options);

    if (!response.ok) {
      let errorMessage = `API error: ${response.status}`;

      // Check if response has content before trying to parse it
      const contentType = response.headers.get("content-type");
      const hasContent = response.status !== 204 && response.headers.get("content-length") !== "0";

      if (hasContent && contentType && contentType.includes("application/json")) {
        try {
          const errorData = await response.json();
          // Check if the response contains an error message
          if (errorData && typeof errorData === "object") {
            if (errorData.error) {
              errorMessage = errorData.error;
            } else if (errorData.message) {
              errorMessage = errorData.message;
            }
          }
        } catch (parseError) {
          // If JSON parsing fails, use the default error message
          console.error("Error parsing error response:", parseError);

          // Try to get text content if JSON parsing fails
          try {
            const textContent = await response.text();
            if (textContent) {
              errorMessage = textContent;
            }
          } catch (textError) {
            console.error("Error getting response text:", textError);
          }
        }
      } else {
        // For non-JSON responses, try to get the text content
        try {
          const textContent = await response.text();
          if (textContent) {
            errorMessage = textContent;
          }
        } catch (textError) {
          console.error("Error getting response text:", textError);
        }
      }

      throw new Error(errorMessage);
    }

    // Check if response is empty or not JSON
    const contentType = response.headers.get("content-type");
    if (response.status === 204 || !contentType || !contentType.includes("application/json")) {
      return {} as T;
    }

    return response.json();
  },

  async get<T>(url: string, options?: RequestInit, skipAuth = false): Promise<T> {
    return this.request<T>(url, { method: "GET", ...options }, skipAuth);
  },

  async post<T>(url: string, data: unknown, options?: RequestInit, skipAuth = false): Promise<T> {
    return this.request<T>(
      url,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
        ...options,
      },
      skipAuth,
    );
  },

  async put<T>(url: string, data: unknown, options?: RequestInit, skipAuth = false): Promise<T> {
    return this.request<T>(
      url,
      {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
        ...options,
      },
      skipAuth,
    );
  },

  async patch<T>(url: string, data: unknown, options?: RequestInit, skipAuth = false): Promise<T> {
    return this.request<T>(
      url,
      {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
        ...options,
      },
      skipAuth,
    );
  },

  async delete<T>(url: string, options?: RequestInit, skipAuth = false): Promise<T> {
    return this.request<T>(url, { method: "DELETE", ...options }, skipAuth);
  },

  getFullUrl(url: string): string {
    return url.startsWith("http") ? url : `${API_URL}${url.startsWith("/") ? "" : "/"}${url}`;
  },
};
