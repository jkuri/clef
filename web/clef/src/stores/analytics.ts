import { create } from "zustand";
import type { AnalyticsData } from "@/types/analytics";

interface AnalyticsStoreState {
  data: AnalyticsData | null;
  isLoading: boolean;
  error: string | null;
  lastUpdated: string | null;
}

interface AnalyticsStoreActions {
  setData: (data: AnalyticsData) => void;
  setLoading: (isLoading: boolean) => void;
  setError: (error: string | null) => void;
  clearData: () => void;
}

export const useAnalyticsStore = create<AnalyticsStoreState & AnalyticsStoreActions>((set) => ({
  // State
  data: null,
  isLoading: false,
  error: null,
  lastUpdated: null,

  // Actions
  setData: (data) => {
    set({
      data,
      error: null,
      lastUpdated: new Date().toISOString(),
    });
  },

  setLoading: (isLoading) => {
    set({ isLoading });
  },

  setError: (error) => {
    set({ error, isLoading: false });
  },

  clearData: () => {
    set({
      data: null,
      error: null,
      lastUpdated: null,
    });
  },
}));
