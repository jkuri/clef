import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";
import { api } from "@/api/client";
import { usePackageStore } from "@/stores/package";
import type { PackageResponse } from "@/types/packages";

// Query keys
export const packageKeys = {
  all: ["package"] as const,
  detail: (name: string) => [...packageKeys.all, "detail", name] as const,
};

// API function
const fetchPackage = async (name: string): Promise<PackageResponse> => {
  return await api.get<PackageResponse>(`/api/v1/packages/${encodeURIComponent(name)}`);
};

// Custom hooks
export const usePackage = (name: string) => {
  const { setData, setLoading, setError, setPackageName } = usePackageStore();
  const queryClient = useQueryClient();

  // Set package name in store
  useEffect(() => {
    setPackageName(name);
  }, [name, setPackageName]);

  const query = useQuery({
    queryKey: packageKeys.detail(name),
    queryFn: () => fetchPackage(name),
    staleTime: 5 * 60 * 1000, // 5 minutes
    gcTime: 10 * 60 * 1000, // 10 minutes
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
    enabled: !!name, // Only run query if name is provided
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

  // Refresh function
  const refresh = () => {
    return queryClient.invalidateQueries({
      queryKey: packageKeys.detail(name),
    });
  };

  return {
    ...query,
    data: query.data,
    refresh,
  };
};
