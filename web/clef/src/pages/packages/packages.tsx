import { useCallback, useEffect, useRef, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { usePackages } from "@/hooks/use-packages";
import { usePackagesStore } from "@/stores/packages";
import { createColumns } from "./columns";
import { DataTable } from "./data-table";

export function Packages() {
  const store = usePackagesStore();
  const { data, isPending: isLoading, isFetching, error, updateUrlState, urlState } = usePackages();
  const searchTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Local search state for immediate UI feedback
  const [searchInput, setSearchInput] = useState(urlState.search);

  // Sync search input with URL state when URL changes (e.g., browser back/forward)
  useEffect(() => {
    setSearchInput(urlState.search);
  }, [urlState.search]);

  const handlePageChange = (newPage: number) => {
    updateUrlState({ page: newPage });
  };

  const handlePageSizeChange = (newPageSize: number) => {
    updateUrlState({ pageSize: newPageSize, page: 1 });
  };

  const handleSearchChange = useCallback(
    (value: string) => {
      // Update local state immediately for UI feedback
      setSearchInput(value);

      // Clear existing timeout
      if (searchTimeoutRef.current) {
        clearTimeout(searchTimeoutRef.current);
      }

      // Debounce URL update and API call
      searchTimeoutRef.current = setTimeout(() => {
        updateUrlState({ search: value, page: 1 });
      }, 300);
    },
    [updateUrlState],
  );

  const handleSort = (field: string) => {
    const currentOrder = store.sortField === field ? store.sortOrder : null;
    const newOrder: "asc" | "desc" = currentOrder === "asc" ? "desc" : "asc";
    updateUrlState({ sortField: field, sortOrder: newOrder, page: 1 });
  };

  const columns = createColumns({
    onSort: handleSort,
    sortField: store.sortField,
    sortOrder: store.sortOrder,
  });

  // Create pagination object using current URL state instead of API response
  // This ensures the UI shows the correct page size immediately
  const currentPagination = data?.pagination
    ? {
        ...data.pagination,
        limit: urlState.pageSize, // Use URL state for immediate UI feedback
        page: urlState.page,
      }
    : {
        limit: urlState.pageSize,
        page: urlState.page,
        total_pages: 1,
        has_next: false,
        has_prev: false,
      };

  if (error) {
    return (
      <div className="container mx-auto py-10">
        <Card>
          <CardHeader>
            <CardTitle>Error</CardTitle>
            <CardDescription>Failed to load packages</CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-muted-foreground text-sm">
              {error instanceof Error ? error.message : "An unknown error occurred"}
            </p>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="font-bold text-3xl tracking-tight">Packages</h1>
        <p className="text-muted-foreground">Manage and explore all packages in your registry.</p>
      </div>

      {isLoading ? (
        <Card>
          <CardHeader>
            <Skeleton className="h-4 w-[250px]" />
            <Skeleton className="h-4 w-[200px]" />
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {Array.from({ length: 10 }).map((_, i) => (
                <div key={i} className="flex items-center space-x-4">
                  <Skeleton className="h-12 w-12 rounded-full" />
                  <div className="space-y-2">
                    <Skeleton className="h-4 w-[250px]" />
                    <Skeleton className="h-4 w-[200px]" />
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      ) : (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">All Packages</CardTitle>
            <CardDescription>
              {data && data.total_count > 0 && <>{data.total_count} packages in registry</>}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <DataTable
              columns={columns}
              data={data?.packages || []}
              searchValue={searchInput}
              onSearchChange={handleSearchChange}
              pagination={currentPagination}
              onPageChange={handlePageChange}
              onPageSizeChange={handlePageSizeChange}
              totalCount={data?.total_count || 0}
              isLoading={isFetching}
              showNoResults={!isLoading && !error && data && data.packages.length === 0}
            />
          </CardContent>
        </Card>
      )}
    </div>
  );
}
