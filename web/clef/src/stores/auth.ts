import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { LoginResponse } from "@/schemas/auth";

type AuthStoreState = {
  token: string | null;
  isAuthenticated: boolean;
};

type AuthStoreActions = {
  setToken: (response: LoginResponse) => void;
  clearToken: () => void;
};

export const useAuthStore = create<AuthStoreState & AuthStoreActions>()(
  persist(
    (set) => ({
      token: null,
      isAuthenticated: false,

      setToken: (response) => {
        set({ token: response.token, isAuthenticated: true });
      },

      clearToken: () => {
        set({ token: null, isAuthenticated: false });
      },
    }),
    {
      name: "clef-auth-storage",
      partialize: (state) => ({
        token: state.token,
      }),
    },
  ),
);
