import { create } from "zustand";
import type { PackageListResponse } from "@/types/packages";

interface PackagesStoreState {
  data: PackageListResponse | null;
  isLoading: boolean;
  error: string | null;
  lastUpdated: string | null;
  // UI state
  searchQuery: string;
  currentPage: number;
  pageSize: number;
  sortField: string | null;
  sortOrder: "asc" | "desc" | null;
}

interface PackagesStoreActions {
  setData: (data: PackageListResponse) => void;
  setLoading: (isLoading: boolean) => void;
  setError: (error: string | null) => void;
  clearData: () => void;
  // UI actions
  setSearchQuery: (query: string) => void;
  setCurrentPage: (page: number) => void;
  setPageSize: (size: number) => void;
  setSorting: (field: string | null, order: "asc" | "desc" | null) => void;
  syncWithUrlState: (urlState: {
    page: number;
    pageSize: number;
    search: string;
    sortField: string | null;
    sortOrder: "asc" | "desc" | null;
  }) => void;
  reset: () => void;
}

export const usePackagesStore = create<PackagesStoreState & PackagesStoreActions>((set) => ({
  // State
  data: null,
  isLoading: false,
  error: null,
  lastUpdated: null,
  searchQuery: "",
  currentPage: 1,
  pageSize: 20,
  sortField: "created_at",
  sortOrder: "desc",

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

  // UI actions
  setSearchQuery: (query: string) => {
    set({ searchQuery: query, currentPage: 1 });
  },

  setCurrentPage: (page: number) => {
    set({ currentPage: page });
  },

  setPageSize: (size: number) => {
    set({ pageSize: size, currentPage: 1 });
  },

  setSorting: (field: string | null, order: "asc" | "desc" | null) => {
    set({ sortField: field, sortOrder: order, currentPage: 1 });
  },

  // Sync with URL state
  syncWithUrlState: (urlState: {
    page: number;
    pageSize: number;
    search: string;
    sortField: string | null;
    sortOrder: "asc" | "desc" | null;
  }) => {
    set({
      currentPage: urlState.page,
      pageSize: urlState.pageSize,
      searchQuery: urlState.search,
      sortField: urlState.sortField,
      sortOrder: urlState.sortOrder,
    });
  },

  reset: () => {
    set({
      data: null,
      error: null,
      lastUpdated: null,
      searchQuery: "",
      currentPage: 1,
      pageSize: 20,
      sortField: "created_at",
      sortOrder: "desc",
    });
  },
}));
