import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";
import { api } from "@/api/client";
import { useAnalyticsStore } from "@/stores/analytics";
import type { AnalyticsApiResponse } from "@/types/analytics";

// Query keys
export const analyticsKeys = {
  all: ["analytics"] as const,
  data: ["analytics", "data"] as const,
};

// API function
const fetchAnalytics = async (): Promise<AnalyticsApiResponse> => {
  return await api.get<AnalyticsApiResponse>("/api/v1/analytics");
};

// Custom hooks
export const useAnalytics = () => {
  const { setData, setLoading, setError } = useAnalyticsStore();

  const query = useQuery({
    queryKey: analyticsKeys.data,
    queryFn: fetchAnalytics,
    staleTime: 5 * 60 * 1000, // 5 minutes
    gcTime: 10 * 60 * 1000, // 10 minutes
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
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
  };
};

export const useAnalyticsRefresh = () => {
  const queryClient = useQueryClient();

  const refresh = () => {
    return queryClient.invalidateQueries({
      queryKey: analyticsKeys.all,
    });
  };

  return {
    refresh,
  };
};
