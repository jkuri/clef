import { format } from "date-fns";
import { Calendar, ExternalLink, GitBranch, Globe, Package as PackageIcon, Search, Tag } from "lucide-react";
import { useMemo, useState } from "react";
import { Link, useParams } from "react-router";
import semver from "semver";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { usePackage } from "@/hooks/use-package";
import { formatBytes } from "@/lib/utils";
import type { PackageVersionWithFiles } from "@/types/packages";

// Helper function to safely parse JSON
const safeJsonParse = (jsonString: string | null): Record<string, string> | null => {
  if (!jsonString) return null;
  try {
    return JSON.parse(jsonString);
  } catch {
    return null;
  }
};

// Helper function to check if a version is stable (not alpha, beta, canary, etc.)
const isStableVersion = (version: string): boolean => {
  const lowerVersion = version.toLowerCase();
  const prereleaseKeywords = ["alpha", "beta", "canary", "rc", "next", "dev", "snapshot", "pre"];
  return !prereleaseKeywords.some((keyword) => lowerVersion.includes(keyword));
};

// Helper function to find the latest stable version
const findLatestStableVersion = (versions: PackageVersionWithFiles[]): PackageVersionWithFiles | null => {
  const stableVersions = versions.filter((v) => isStableVersion(v.version.version));
  return stableVersions.length > 0 ? stableVersions[0] : versions[0] || null;
};

// Component to display dependencies in a nice table format
const DependencyTable = ({ dependencies, title }: { dependencies: Record<string, string>; title: string }) => (
  <div>
    <h4 className="mb-3 font-medium text-sm">{title}</h4>
    <div className="rounded border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead className="w-2/3">Package</TableHead>
            <TableHead className="w-1/3">Version</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {Object.entries(dependencies).map(([pkg, version]) => (
            <TableRow key={pkg}>
              <TableCell className="break-all font-medium">{pkg}</TableCell>
              <TableCell className="font-mono text-sm">{version}</TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  </div>
);

// Component to display engines in a nice format
const EnginesDisplay = ({ engines }: { engines: Record<string, string> }) => (
  <div>
    <h4 className="mb-3 font-medium text-sm">Engines</h4>
    <div className="grid gap-2">
      {Object.entries(engines).map(([engine, version]) => (
        <div key={engine} className="flex items-center justify-between rounded bg-muted p-3">
          <span className="font-medium capitalize">{engine}</span>
          <code className="rounded bg-background px-2 py-1 font-mono text-sm">{version}</code>
        </div>
      ))}
    </div>
  </div>
);

export function Package() {
  const { "*": packagePath } = useParams<{ "*": string }>();
  const name = packagePath || "";
  const { data, isPending: isLoading, error } = usePackage(name || "");
  const [selectedVersion, setSelectedVersion] = useState<string | null>(null);
  const [versionSearch, setVersionSearch] = useState("");

  // Sort versions by semver (latest first)
  const sortedVersions = useMemo(() => {
    if (!data?.versions) return [];
    return [...data.versions].sort((a, b) => {
      try {
        return semver.rcompare(a.version.version, b.version.version);
      } catch {
        // Fallback to string comparison if semver parsing fails
        return b.version.version.localeCompare(a.version.version);
      }
    });
  }, [data?.versions]);

  // Filter versions based on search
  const filteredVersions = useMemo(() => {
    if (!versionSearch.trim()) return sortedVersions;
    return sortedVersions.filter(
      (v) =>
        v.version.version.toLowerCase().includes(versionSearch.toLowerCase()) ||
        v.version.description?.toLowerCase().includes(versionSearch.toLowerCase()),
    );
  }, [sortedVersions, versionSearch]);

  if (!name) {
    return (
      <div className="container mx-auto py-10">
        <Card>
          <CardHeader>
            <CardTitle>Error</CardTitle>
            <CardDescription>Package name is required</CardDescription>
          </CardHeader>
        </Card>
      </div>
    );
  }

  if (error) {
    return (
      <div className="container mx-auto py-10">
        <Card>
          <CardHeader>
            <CardTitle>Error</CardTitle>
            <CardDescription>Failed to load package</CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-muted-foreground text-sm">
              {error instanceof Error ? error.message : "An unknown error occurred"}
            </p>
            <div className="mt-4">
              <Button asChild variant="outline">
                <Link to="/packages">← Back to Packages</Link>
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div>
          <Skeleton className="mb-2 h-8 w-64" />
          <Skeleton className="h-4 w-96" />
        </div>
        <div className="grid gap-6 md:grid-cols-3">
          <div className="space-y-6 md:col-span-2">
            <Card>
              <CardHeader>
                <Skeleton className="h-6 w-32" />
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  {Array.from({ length: 3 }).map((_, i) => (
                    <div key={i} className="flex items-center justify-between">
                      <Skeleton className="h-4 w-24" />
                      <Skeleton className="h-4 w-16" />
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </div>
          <div className="space-y-6">
            <Card>
              <CardHeader>
                <Skeleton className="h-6 w-24" />
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {Array.from({ length: 4 }).map((_, i) => (
                    <Skeleton key={i} className="h-4 w-full" />
                  ))}
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="container mx-auto py-10">
        <Card>
          <CardHeader>
            <CardTitle>Package Not Found</CardTitle>
            <CardDescription>The package "{name}" could not be found</CardDescription>
          </CardHeader>
          <CardContent>
            <Button asChild variant="outline">
              <Link to="/packages">← Back to Packages</Link>
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  const { package: pkg } = data;
  const latestStableVersion = findLatestStableVersion(sortedVersions);
  const latestVersion = latestStableVersion?.version || sortedVersions[0]?.version;
  const currentVersion = selectedVersion || latestVersion?.version || "";
  const currentVersionData = sortedVersions.find((v) => v.version.version === currentVersion);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div>
          <div className="mb-2 flex items-center gap-2">
            <PackageIcon className="h-6 w-6" />
            <h1 className="font-bold text-3xl tracking-tight">{pkg.name}</h1>
            {pkg.is_private && (
              <span className="inline-flex items-center rounded-md bg-yellow-50 px-2 py-1 font-medium text-xs text-yellow-800 ring-1 ring-yellow-600/20 ring-inset">
                Private
              </span>
            )}
          </div>
          <p className="text-muted-foreground">{pkg.description}</p>
        </div>
        <Button asChild variant="outline">
          <Link to="/packages">← Back to Packages</Link>
        </Button>
      </div>

      {/* Main Content */}
      <div className="grid gap-6 md:grid-cols-3">
        {/* Left Column - Versions and Files */}
        <div className="space-y-6 md:col-span-2">
          {/* Versions */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Tag className="h-5 w-5" />
                Versions ({sortedVersions.length})
              </CardTitle>
              <CardDescription>Available versions of this package</CardDescription>
            </CardHeader>
            <CardContent className="pb-4">
              <div className="relative">
                <Search className="-translate-y-1/2 absolute top-1/2 left-3 h-4 w-4 text-muted-foreground" />
                <Input
                  placeholder="Search versions..."
                  value={versionSearch}
                  onChange={(e) => setVersionSearch(e.target.value)}
                  className="pl-10"
                />
              </div>
            </CardContent>
            <CardContent>
              {filteredVersions.length === 0 ? (
                <p className="text-muted-foreground text-sm">
                  {versionSearch.trim() ? "No versions match your search" : "No versions available"}
                </p>
              ) : (
                <div className="space-y-2">
                  {filteredVersions.slice(0, 10).map((versionData) => (
                    <div
                      key={versionData.version.id}
                      className={`flex items-center justify-between rounded-lg border p-3 transition-colors hover:bg-muted/50 ${
                        currentVersion === versionData.version.version ? "bg-muted" : ""
                      }`}
                    >
                      <div className="flex items-center gap-3">
                        <GitBranch className="h-4 w-4 text-muted-foreground" />
                        <div>
                          <div className="flex items-center gap-2">
                            <span className="font-medium">{versionData.version.version}</span>
                            {(versionData === sortedVersions[0] || versionData === latestStableVersion) && (
                              <span className="inline-flex items-center rounded-md bg-blue-50 px-2 py-1 font-medium text-blue-700 text-xs ring-1 ring-blue-600/20 ring-inset">
                                Latest
                              </span>
                            )}
                            {versionData === latestStableVersion && (
                              <span className="inline-flex items-center rounded-md bg-green-50 px-2 py-1 font-medium text-green-700 text-xs ring-1 ring-green-600/20 ring-inset">
                                Stable
                              </span>
                            )}
                            {!isStableVersion(versionData.version.version) && (
                              <span className="inline-flex items-center rounded-md bg-yellow-50 px-2 py-1 font-medium text-xs text-yellow-700 ring-1 ring-yellow-600/20 ring-inset">
                                Prerelease
                              </span>
                            )}
                          </div>
                          <div className="space-y-1">
                            <div className="flex items-center gap-4 text-muted-foreground text-sm">
                              <span className="flex items-center gap-1">
                                <Calendar className="h-3 w-3" />
                                {format(new Date(versionData.version.created_at), "MMM d, yyyy 'at' h:mm a")}
                              </span>
                            </div>
                            {versionData.version.description && (
                              <p className="text-muted-foreground text-xs">{versionData.version.description}</p>
                            )}
                          </div>
                        </div>
                      </div>
                      <Button
                        variant={currentVersion === versionData.version.version ? "default" : "outline"}
                        size="sm"
                        onClick={() => setSelectedVersion(versionData.version.version)}
                      >
                        {currentVersion === versionData.version.version ? "Selected" : "Select"}
                      </Button>
                    </div>
                  ))}
                  {filteredVersions.length > 10 && (
                    <p className="pt-2 text-center text-muted-foreground text-sm">
                      ... and {filteredVersions.length - 10} more {versionSearch.trim() ? "matching " : ""}versions
                    </p>
                  )}
                </div>
              )}
            </CardContent>
          </Card>

          {/* Version Details */}
          {currentVersionData && (
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Tag className="h-5 w-5" />
                  Version Details - v{currentVersion}
                </CardTitle>
                <CardDescription>Detailed information about this version</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                {currentVersionData.version.description && (
                  <div>
                    <h4 className="mb-2 font-medium text-sm">Description</h4>
                    <p className="text-muted-foreground text-sm">{currentVersionData.version.description}</p>
                  </div>
                )}

                {currentVersionData.version.main_file && (
                  <div>
                    <h4 className="mb-2 font-medium text-sm">Main File</h4>
                    <code className="rounded bg-muted px-2 py-1 text-sm">{currentVersionData.version.main_file}</code>
                  </div>
                )}

                {currentVersionData.version.engines &&
                  (() => {
                    const engines = safeJsonParse(currentVersionData.version.engines);
                    return engines ? <EnginesDisplay engines={engines} /> : null;
                  })()}

                {currentVersionData.version.dependencies &&
                  (() => {
                    const dependencies = safeJsonParse(currentVersionData.version.dependencies);
                    return dependencies ? (
                      <div className="max-h-64 overflow-y-auto">
                        <DependencyTable dependencies={dependencies} title="Dependencies" />
                      </div>
                    ) : null;
                  })()}

                {currentVersionData.version.peer_dependencies &&
                  (() => {
                    const peerDependencies = safeJsonParse(currentVersionData.version.peer_dependencies);
                    return peerDependencies ? (
                      <div className="max-h-64 overflow-y-auto">
                        <DependencyTable dependencies={peerDependencies} title="Peer Dependencies" />
                      </div>
                    ) : null;
                  })()}

                {currentVersionData.version.dev_dependencies &&
                  (() => {
                    const devDependencies = safeJsonParse(currentVersionData.version.dev_dependencies);
                    return devDependencies ? (
                      <div className="max-h-64 overflow-y-auto">
                        <DependencyTable dependencies={devDependencies} title="Dev Dependencies" />
                      </div>
                    ) : null;
                  })()}

                {currentVersionData.version.shasum && (
                  <div>
                    <h4 className="mb-2 font-medium text-sm">SHA Sum</h4>
                    <code className="break-all rounded bg-muted px-2 py-1 text-sm">
                      {currentVersionData.version.shasum}
                    </code>
                  </div>
                )}
              </CardContent>
            </Card>
          )}
        </div>

        {/* Right Column - Package Info */}
        <div className="space-y-6">
          {/* Package Details */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <PackageIcon className="h-5 w-5" />
                Package Info
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground text-sm">Latest Stable</span>
                  <span className="font-medium">{latestStableVersion?.version.version || "N/A"}</span>
                </div>
                {sortedVersions[0] && sortedVersions[0] !== latestStableVersion && (
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground text-sm">Latest (All)</span>
                    <span className="font-medium">{sortedVersions[0].version.version}</span>
                  </div>
                )}
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground text-sm">Total Versions</span>
                  <span className="font-medium">{sortedVersions.length}</span>
                </div>
                {data.total_size_bytes && (
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground text-sm">Total Size</span>
                    <span className="font-medium">{formatBytes(data.total_size_bytes)}</span>
                  </div>
                )}
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground text-sm">Visibility</span>
                  <span className="font-medium">
                    {pkg.is_private ? (
                      <span className="inline-flex items-center rounded-md bg-yellow-50 px-2 py-1 font-medium text-xs text-yellow-800 ring-1 ring-yellow-600/20 ring-inset">
                        Private
                      </span>
                    ) : (
                      <span className="inline-flex items-center rounded-md bg-green-50 px-2 py-1 font-medium text-green-700 text-xs ring-1 ring-green-600/20 ring-inset">
                        Public
                      </span>
                    )}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground text-sm">Created</span>
                  <span className="font-medium">{format(new Date(pkg.created_at), "MMM d, yyyy 'at' h:mm a")}</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground text-sm">Updated</span>
                  <span className="font-medium">{format(new Date(pkg.updated_at), "MMM d, yyyy 'at' h:mm a")}</span>
                </div>
                {pkg.license && (
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground text-sm">License</span>
                    <span className="font-medium">{pkg.license}</span>
                  </div>
                )}
              </div>

              {/* Links */}
              <div className="space-y-2 border-t pt-4">
                {pkg.homepage && (
                  <Button variant="outline" size="sm" className="w-full justify-start" asChild>
                    <a href={pkg.homepage} target="_blank" rel="noopener noreferrer">
                      <Globe className="h-4 w-4" />
                      Homepage
                      <ExternalLink className="ml-auto h-3 w-3" />
                    </a>
                  </Button>
                )}
                {pkg.repository_url && (
                  <Button variant="outline" size="sm" className="w-full justify-start" asChild>
                    <a href={pkg.repository_url} target="_blank" rel="noopener noreferrer">
                      <GitBranch className="h-4 w-4" />
                      Repository
                      <ExternalLink className="ml-auto h-3 w-3" />
                    </a>
                  </Button>
                )}
              </div>
            </CardContent>
          </Card>

          {/* Keywords */}
          {pkg.keywords && (
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Tag className="h-5 w-5" />
                  Keywords
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="flex flex-wrap gap-2">
                  {pkg.keywords.split(",").map((keyword, index) => (
                    <span
                      key={index}
                      className="inline-flex items-center rounded-md bg-blue-50 px-2 py-1 font-medium text-blue-700 text-xs ring-1 ring-blue-600/20 ring-inset"
                    >
                      {keyword.trim()}
                    </span>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}

          {/* Installation */}
          <Card>
            <CardHeader>
              <CardTitle>Installation</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                <div>
                  <p className="mb-2 text-muted-foreground text-sm">npm</p>
                  <div className="rounded-md bg-muted p-3 font-mono text-sm">npm install {pkg.name}</div>
                </div>
                <div>
                  <p className="mb-2 text-muted-foreground text-sm">yarn</p>
                  <div className="rounded-md bg-muted p-3 font-mono text-sm">yarn add {pkg.name}</div>
                </div>
                <div>
                  <p className="mb-2 text-muted-foreground text-sm">pnpm</p>
                  <div className="rounded-md bg-muted p-3 font-mono text-sm">pnpm add {pkg.name}</div>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
