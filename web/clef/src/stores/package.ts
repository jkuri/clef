import { create } from "zustand";
import type { PackageResponse } from "@/types/packages";

interface PackageStoreState {
  data: PackageResponse | null;
  isLoading: boolean;
  error: string | null;
  lastUpdated: string | null;
  packageName: string | null;
}

interface PackageStoreActions {
  setData: (data: PackageResponse) => void;
  setLoading: (isLoading: boolean) => void;
  setError: (error: string | null) => void;
  setPackageName: (name: string) => void;
  clearData: () => void;
  reset: () => void;
}

export const usePackageStore = create<PackageStoreState & PackageStoreActions>((set) => ({
  // State
  data: null,
  isLoading: false,
  error: null,
  lastUpdated: null,
  packageName: null,

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

  setPackageName: (name) => {
    set({ packageName: name });
  },

  clearData: () => {
    set({
      data: null,
      error: null,
      lastUpdated: null,
    });
  },

  reset: () => {
    set({
      data: null,
      error: null,
      lastUpdated: null,
      isLoading: false,
      packageName: null,
    });
  },
}));
