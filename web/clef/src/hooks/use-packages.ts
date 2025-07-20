import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect } from "react";
import { useSearchParams } from "react-router";
import { api } from "@/api/client";
import { usePackagesStore } from "@/stores/packages";
import type { PackageListParams, PackageListResponse } from "@/types/packages";

// Query keys
export const packagesKeys = {
  all: ["packages"] as const,
  lists: () => [...packagesKeys.all, "list"] as const,
  list: (params: PackageListParams) => [...packagesKeys.lists(), params] as const,
};

// API function
const fetchPackages = async (params: PackageListParams): Promise<PackageListResponse> => {
  const queryParams = new URLSearchParams();

  queryParams.append("page", (params.page || 1).toString());
  queryParams.append("limit", (params.limit || 20).toString());

  if (params.search?.trim()) {
    queryParams.append("search", params.search.trim());
  }
  if (params.sort) {
    queryParams.append("sort", params.sort);
  }
  if (params.order) {
    queryParams.append("order", params.order);
  }

  return await api.get<PackageListResponse>(`/api/v1/packages?${queryParams.toString()}`);
};

// Default state
const DEFAULT_STATE = {
  page: 1,
  pageSize: 20,
  search: "",
  sortField: "created_at",
  sortOrder: "desc" as const,
};

// Custom hooks
export const usePackages = (params: PackageListParams = {}) => {
  const { setData, setLoading, setError } = usePackagesStore();
  const store = usePackagesStore();
  const [searchParams, setSearchParams] = useSearchParams();

  // Parse URL state
  const getUrlState = useCallback(() => {
    const page = parseInt(searchParams.get("page") || "1", 10);
    const pageSize = parseInt(searchParams.get("limit") || "20", 10);
    const search = searchParams.get("search") || "";
    const sortField = searchParams.get("sort") || DEFAULT_STATE.sortField;
    const sortOrder = (searchParams.get("order") as "asc" | "desc") || DEFAULT_STATE.sortOrder;

    return {
      page: Math.max(1, page),
      pageSize: Math.max(1, Math.min(100, pageSize)),
      search,
      sortField,
      sortOrder,
    };
  }, [searchParams]);

  // Update URL with new state
  const updateUrlState = useCallback(
    (
      newState: Partial<{
        page: number;
        pageSize: number;
        search: string;
        sortField: string;
        sortOrder: "asc" | "desc";
      }>,
    ) => {
      const currentState = getUrlState();
      const updatedState = { ...currentState, ...newState };

      const params = new URLSearchParams();

      // Only add non-default values to keep URL clean
      if (updatedState.page !== DEFAULT_STATE.page) {
        params.set("page", updatedState.page.toString());
      }
      if (updatedState.pageSize !== DEFAULT_STATE.pageSize) {
        params.set("limit", updatedState.pageSize.toString());
      }
      if (updatedState.search !== DEFAULT_STATE.search) {
        params.set("search", updatedState.search);
      }
      if (updatedState.sortField !== DEFAULT_STATE.sortField) {
        params.set("sort", updatedState.sortField || "");
      }
      if (updatedState.sortOrder !== DEFAULT_STATE.sortOrder) {
        params.set("order", updatedState.sortOrder || "");
      }

      setSearchParams(params, { replace: true });
    },
    [getUrlState, setSearchParams],
  );

  // Get current URL state
  const urlState = getUrlState();

  // Sync store with URL state
  useEffect(() => {
    store.syncWithUrlState(urlState);
  }, [urlState.page, urlState.pageSize, urlState.search, urlState.sortField, urlState.sortOrder]);

  // Use URL state directly for query parameters to ensure consistency
  const finalParams = {
    page: params.page ?? urlState.page,
    limit: params.limit ?? urlState.pageSize,
    search: params.search ?? urlState.search,
    sort: params.sort ?? urlState.sortField ?? undefined,
    order: params.order ?? urlState.sortOrder ?? undefined,
  };

  const query = useQuery({
    queryKey: packagesKeys.list(finalParams),
    queryFn: () => fetchPackages(finalParams),
    staleTime: 30 * 1000, // 30 seconds
    gcTime: 5 * 60 * 1000, // 5 minutes
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
    // Keep previous data while fetching new data to prevent flickering
    placeholderData: (previousData) => previousData,
    // Ensure query re-runs when URL state changes
    enabled: true,
  });

  // Sync query state with store
  useEffect(() => {
    setLoading(query.isLoading);

    if (query.error) {
      setError(query.error.message);
    } else if (query.data) {
      setData(query.data);
      setError(null);
    }
  }, [query.isLoading, query.error, query.data, setData, setLoading, setError]);

  return {
    ...query,
    data: query.data,
    // URL state management
    updateUrlState,
    urlState,
  };
};

export const usePackagesRefresh = () => {
  const queryClient = useQueryClient();

  const refresh = () => {
    return queryClient.invalidateQueries({
      queryKey: packagesKeys.all,
    });
  };

  return {
    refresh,
  };
};
