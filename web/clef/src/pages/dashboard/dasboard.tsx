import { Database, HardDrive, Package, TrendingUp } from "lucide-react";
import { Link } from "react-router";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { useAnalytics } from "@/hooks/analytics";
import { formatBytes, formatNumber, roundNumber } from "@/lib/utils";

export function Dashboard() {
  const { data, isPending: isLoading, error } = useAnalytics();

  if (error) {
    return (
      <div className="flex min-h-[400px] flex-col items-center justify-center space-y-4">
        <div className="text-center">
          <h3 className="font-semibold text-destructive text-lg">Error loading analytics</h3>
          <p className="mt-2 text-muted-foreground text-sm">{String(error)}</p>
        </div>
        <Button onClick={() => window.location.reload()} variant="outline">
          Try Again
        </Button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="min-w-0 flex-1">
          <h1 className="font-bold text-2xl tracking-tight sm:text-3xl">Dashboard</h1>
          <p className="text-muted-foreground text-sm sm:text-base">Package registry analytics overview</p>
        </div>
      </div>

      {/* Overview Cards */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {isLoading ? (
          Array.from({ length: 4 }).map((_, i) => (
            <Card key={i}>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <Skeleton className="h-4 w-20" />
                <Skeleton className="h-4 w-4" />
              </CardHeader>
              <CardContent>
                <Skeleton className="mb-2 h-8 w-24" />
                <Skeleton className="h-3 w-16" />
              </CardContent>
            </Card>
          ))
        ) : data ? (
          <>
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="font-medium text-sm">Total Packages</CardTitle>
                <Package className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="font-bold text-xl sm:text-2xl">{formatNumber(data.total_packages)}</div>
                <p className="text-muted-foreground text-xs">Packages in registry</p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="font-medium text-sm">Total Size</CardTitle>
                <HardDrive className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="font-bold text-xl sm:text-2xl">{data.total_size_mb.toFixed(1)} MB</div>
                <p className="text-muted-foreground text-xs">Cached packages size</p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="font-medium text-sm">Cache Hit Rate</CardTitle>
                <TrendingUp className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="font-bold text-xl sm:text-2xl">{roundNumber(data.cache_hit_rate, 2)}%</div>
                <p className="text-muted-foreground text-xs">Cache efficiency</p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="font-medium text-sm">Metadata Cache</CardTitle>
                <Database className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="font-bold text-xl sm:text-2xl">{data.metadata_cache_size_mb.toFixed(1)} MB</div>
                <p className="text-muted-foreground text-xs">
                  {formatNumber(data.metadata_cache_entries)} metadata files
                </p>
              </CardContent>
            </Card>
          </>
        ) : null}
      </div>

      <div className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Most Popular Packages</CardTitle>
            <CardDescription>Top packages by download count</CardDescription>
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <div className="space-y-3">
                {Array.from({ length: 5 }).map((_, i) => (
                  <div key={i} className="flex items-center justify-between">
                    <div className="flex items-center space-x-3">
                      <Skeleton className="h-8 w-8 rounded-full" />
                      <div className="space-y-1">
                        <Skeleton className="h-3 w-24" />
                        <Skeleton className="h-2 w-16" />
                      </div>
                    </div>
                    <Skeleton className="h-3 w-12" />
                  </div>
                ))}
              </div>
            ) : data?.most_popular_packages && data.most_popular_packages.length > 0 ? (
              <div className="space-y-1">
                {data.most_popular_packages.map((pkg, index) => (
                  <Link
                    key={pkg.name}
                    to={`/packages/${pkg.name}`}
                    className="flex items-center justify-between gap-3 rounded-lg p-2 transition-colors hover:bg-muted/50"
                  >
                    <div className="flex items-center space-x-3">
                      <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-primary/10">
                        <span className="font-medium text-xs">{index + 1}</span>
                      </div>
                      <div className="min-w-0 flex-1 space-y-1">
                        <p className="truncate font-medium text-sm leading-none">{pkg.name}</p>
                        <p className="text-muted-foreground text-xs">
                          {pkg.unique_versions} version
                          {pkg.unique_versions !== 1 ? "s" : ""} • {formatBytes(pkg.total_size_bytes)}
                        </p>
                      </div>
                    </div>
                    <div className="shrink-0 text-right">
                      <p className="font-medium text-sm">{formatNumber(pkg.total_downloads)}</p>
                      <p className="text-muted-foreground text-xs">downloads</p>
                    </div>
                  </Link>
                ))}
              </div>
            ) : (
              <div className="flex h-32 items-center justify-center">
                <p className="text-muted-foreground text-sm">No popular packages data</p>
              </div>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Recent Packages</CardTitle>
            <CardDescription>Latest packages added to registry</CardDescription>
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <div className="space-y-1">
                {Array.from({ length: 5 }).map((_, i) => (
                  <div key={i} className="flex items-center space-x-3">
                    <Skeleton className="h-8 w-8 rounded-full" />
                    <div className="space-y-1">
                      <Skeleton className="h-3 w-32" />
                      <Skeleton className="h-2 w-20" />
                    </div>
                  </div>
                ))}
              </div>
            ) : data?.recent_packages && data.recent_packages.length > 0 ? (
              <div className="space-y-1">
                {data.recent_packages.slice(0, 5).map((recentPackage) => (
                  <Link
                    key={recentPackage.package.id}
                    to={`/packages/${recentPackage.package.name}`}
                    className="flex items-start space-x-3 rounded-lg p-2 transition-colors hover:bg-muted/50"
                  >
                    <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-primary/10">
                      <Package className="h-4 w-4" />
                    </div>
                    <div className="min-w-0 flex-1 space-y-1">
                      <p className="truncate font-medium text-sm leading-none">{recentPackage.package.name}</p>
                      <p className="text-muted-foreground text-xs">
                        {recentPackage.package.description
                          ? recentPackage.package.description.length > 60
                            ? `${recentPackage.package.description.substring(0, 60)}...`
                            : recentPackage.package.description
                          : "No description"}
                      </p>
                    </div>
                  </Link>
                ))}
              </div>
            ) : (
              <div className="flex h-32 items-center justify-center">
                <p className="text-muted-foreground text-sm">No recent packages</p>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
