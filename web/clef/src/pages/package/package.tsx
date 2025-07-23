import { format } from "date-fns";
import {
  Calendar,
  Check,
  ChevronDown,
  ExternalLink,
  GitBranch,
  Globe,
  Package as PackageIcon,
  Search,
  Tag,
} from "lucide-react";
import { useMemo, useState } from "react";
import { useNavigate, useParams } from "react-router";
import semver from "semver";
import { Readme } from "@/components/readme";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
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

// Helper function to parse keywords safely
const parseKeywords = (keywordsString: string | null): string[] => {
  if (!keywordsString || typeof keywordsString !== "string") {
    return [];
  }

  // Handle JSON array format first
  if (keywordsString.trim().startsWith("[") && keywordsString.trim().endsWith("]")) {
    try {
      const parsed = JSON.parse(keywordsString);
      if (Array.isArray(parsed)) {
        return parsed
          .map((k) => String(k).trim())
          .filter((k) => k.length > 0 && k !== "null" && k !== "undefined")
          .filter((keyword, index, arr) => arr.findIndex((k) => k.toLowerCase() === keyword.toLowerCase()) === index);
      }
    } catch {
      // Fall through to string parsing
    }
  }

  // Handle different separators: comma, semicolon, space, tab, or newline
  const keywords = keywordsString
    .split(/[,;\s\t\n\r]+/)
    .map((keyword) => keyword.trim())
    .filter(
      (keyword) =>
        keyword.length > 0 &&
        keyword !== "null" &&
        keyword !== "undefined" &&
        keyword !== "[]" &&
        keyword !== "{}" &&
        !keyword.match(/^[[\]{}()]+$/), // Remove brackets/parentheses only strings
    );

  // Remove duplicates (case-insensitive) and sort alphabetically
  const uniqueKeywords = keywords
    .filter((keyword, index, arr) => arr.findIndex((k) => k.toLowerCase() === keyword.toLowerCase()) === index)
    .sort((a, b) => a.toLowerCase().localeCompare(b.toLowerCase()));

  return uniqueKeywords;
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

// Helper component for rendering version lists
const VersionList = ({
  versions,
  currentVersion,
  sortedVersions,
  latestStableVersion,
  onSelectVersion,
  maxDisplay = 5,
}: {
  versions: PackageVersionWithFiles[];
  currentVersion: string;
  sortedVersions: PackageVersionWithFiles[];
  latestStableVersion: PackageVersionWithFiles | undefined | null;
  onSelectVersion: (version: string) => void;
  maxDisplay?: number;
}) => (
  <div className="space-y-2">
    {versions.slice(0, maxDisplay).map((versionData) => (
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
                <span className="inline-flex items-center rounded-md bg-blue-50 px-2 py-1 font-medium text-blue-700 text-xs ring-1 ring-blue-600/20 ring-inset dark:bg-blue-950 dark:text-blue-300 dark:ring-blue-800">
                  Latest
                </span>
              )}
              {versionData === latestStableVersion && (
                <span className="inline-flex items-center rounded-md bg-green-50 px-2 py-1 font-medium text-green-700 text-xs ring-1 ring-green-600/20 ring-inset dark:bg-green-950 dark:text-green-300 dark:ring-green-800">
                  Stable
                </span>
              )}
              {!isStableVersion(versionData.version.version) && (
                <span className="inline-flex items-center rounded-md bg-yellow-50 px-2 py-1 font-medium text-xs text-yellow-700 ring-1 ring-yellow-600/20 ring-inset dark:bg-yellow-950 dark:text-yellow-300 dark:ring-yellow-800">
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
          onClick={() => onSelectVersion(versionData.version.version)}
        >
          {currentVersion === versionData.version.version ? "Selected" : "Select"}
        </Button>
      </div>
    ))}
    {versions.length > maxDisplay && (
      <p className="pt-2 text-center text-muted-foreground text-sm">
        ... and {versions.length - maxDisplay} more versions
      </p>
    )}
  </div>
);

export function Package() {
  const { "*": packagePath } = useParams<{ "*": string }>();
  const name = packagePath || "";
  const navigate = useNavigate();
  const { data, isPending: isLoading, error } = usePackage(name || "");
  const [selectedVersion, setSelectedVersion] = useState<string | null>(null);
  const [versionSearch, setVersionSearch] = useState("");
  const [copiedCommand, setCopiedCommand] = useState<string | null>(null);

  // Helper function to copy text and show feedback
  const copyToClipboard = async (text: string, commandType: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedCommand(commandType);
      // Clear the feedback after 2 seconds
      setTimeout(() => setCopiedCommand(null), 2000);
    } catch (err) {
      console.error("Failed to copy text: ", err);
    }
  };

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

  // Separate stable and prerelease versions
  const { stableVersions, prereleaseVersions } = useMemo(() => {
    const stable: typeof sortedVersions = [];
    const prerelease: typeof sortedVersions = [];

    sortedVersions.forEach((versionData) => {
      if (isStableVersion(versionData.version.version)) {
        stable.push(versionData);
      } else {
        prerelease.push(versionData);
      }
    });

    return { stableVersions: stable, prereleaseVersions: prerelease };
  }, [sortedVersions]);

  // Filter versions based on search
  const filteredStableVersions = useMemo(() => {
    if (!versionSearch.trim()) return stableVersions;
    return stableVersions.filter(
      (v) =>
        v.version.version.toLowerCase().includes(versionSearch.toLowerCase()) ||
        v.version.description?.toLowerCase().includes(versionSearch.toLowerCase()),
    );
  }, [stableVersions, versionSearch]);

  const filteredPrereleaseVersions = useMemo(() => {
    if (!versionSearch.trim()) return prereleaseVersions;
    return prereleaseVersions.filter(
      (v) =>
        v.version.version.toLowerCase().includes(versionSearch.toLowerCase()) ||
        v.version.description?.toLowerCase().includes(versionSearch.toLowerCase()),
    );
  }, [prereleaseVersions, versionSearch]);

  // Combined filtered versions for backward compatibility
  const filteredVersions = useMemo(() => {
    return [...filteredStableVersions, ...filteredPrereleaseVersions];
  }, [filteredStableVersions, filteredPrereleaseVersions]);

  // Sorted versions for dropdown (stable first, then prereleases)
  const dropdownSortedVersions = useMemo(() => {
    return [...stableVersions, ...prereleaseVersions];
  }, [stableVersions, prereleaseVersions]);

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
              <Button variant="outline" onClick={() => navigate(-1)}>
                ← Back
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
            <Button variant="outline" onClick={() => navigate(-1)}>
              ← Back
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
    <>
      {/* Header Section */}
      <div className="border-b">
        <div className="py-4">
          <div className="flex items-start justify-between">
            <div className="flex-1">
              <div className="mb-3 flex items-center gap-3">
                <PackageIcon className="h-8 w-8 text-primary" />
                <div>
                  <h1 className="font-bold text-4xl tracking-tight">{pkg.name}</h1>
                  <div className="mt-1 flex items-center gap-2">
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button variant="outline" className="font-mono text-lg">
                          {currentVersion}
                          <ChevronDown className="ml-2 h-4 w-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="start" className="max-h-64 overflow-y-auto">
                        {dropdownSortedVersions.map((versionData) => (
                          <DropdownMenuItem
                            key={versionData.version.id}
                            onClick={() => setSelectedVersion(versionData.version.version)}
                            className="font-mono"
                          >
                            {versionData.version.version}
                            {versionData.version.version === sortedVersions[0]?.version.version && (
                              <span className="ml-2 text-muted-foreground text-xs">(latest)</span>
                            )}
                          </DropdownMenuItem>
                        ))}
                      </DropdownMenuContent>
                    </DropdownMenu>
                    {pkg.is_private ? (
                      <span className="inline-flex items-center rounded-md bg-yellow-50 px-2 py-1 font-medium text-xs text-yellow-800 ring-1 ring-yellow-600/20 ring-inset dark:bg-yellow-950 dark:text-yellow-300 dark:ring-yellow-800">
                        Private
                      </span>
                    ) : (
                      <span className="inline-flex items-center rounded-md bg-green-50 px-2 py-1 font-medium text-green-700 text-xs ring-1 ring-green-600/20 ring-inset dark:bg-green-950 dark:text-green-300 dark:ring-green-800">
                        Public
                      </span>
                    )}
                    {latestStableVersion && currentVersion === latestStableVersion.version.version && (
                      <span className="inline-flex items-center rounded-md bg-blue-50 px-2 py-1 font-medium text-blue-700 text-xs ring-1 ring-blue-600/20 ring-inset dark:bg-blue-950 dark:text-blue-300 dark:ring-blue-800">
                        Latest
                      </span>
                    )}
                  </div>
                </div>
              </div>
              {pkg.description && <p className="text-muted-foreground">{pkg.description}</p>}
            </div>
            <Button variant="outline" size="sm" onClick={() => navigate(-1)}>
              ← Back
            </Button>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="py-4">
        <div className="grid gap-8 lg:grid-cols-5">
          {/* Main Content Area */}
          <div className="fade-in-50 slide-in-from-left-4 animate-in duration-500 lg:col-span-3">
            <Tabs defaultValue="readme" className="w-full">
              <TabsList className="grid w-full grid-cols-4">
                <TabsTrigger value="readme" className="flex items-center gap-2">
                  README
                </TabsTrigger>
                <TabsTrigger value="versions" className="flex items-center gap-2">
                  Versions
                  <span className="rounded-full bg-muted px-2 py-0.5 text-xs">{sortedVersions.length}</span>
                </TabsTrigger>
                <TabsTrigger value="dependencies" className="flex items-center gap-2">
                  Dependencies
                  {currentVersionData && (
                    <span className="rounded-full bg-muted px-2 py-0.5 text-xs">
                      {Object.keys(safeJsonParse(currentVersionData.version.dependencies) || {}).length}
                    </span>
                  )}
                </TabsTrigger>
                <TabsTrigger value="dev-dependencies" className="flex items-center gap-2">
                  Dev Dependencies
                  {currentVersionData && (
                    <span className="rounded-full bg-muted px-2 py-0.5 text-xs">
                      {Object.keys(safeJsonParse(currentVersionData.version.dev_dependencies) || {}).length}
                    </span>
                  )}
                </TabsTrigger>
              </TabsList>

              <TabsContent value="readme" className="mt-6">
                {currentVersionData && (
                  <Readme content={currentVersionData.version.readme} packageName={pkg.name} version={currentVersion} />
                )}
              </TabsContent>

              <TabsContent value="versions" className="mt-6">
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
                      <div className="space-y-6">
                        {/* Stable Versions */}
                        {filteredStableVersions.length > 0 && (
                          <div>
                            <div className="mb-3 flex items-center gap-2">
                              <h3 className="font-semibold text-sm">Stable Releases</h3>
                              <span className="inline-flex items-center rounded-md bg-green-50 px-2 py-1 font-medium text-green-700 text-xs ring-1 ring-green-600/20 ring-inset dark:bg-green-950 dark:text-green-300 dark:ring-green-800">
                                {filteredStableVersions.length}
                              </span>
                            </div>
                            <VersionList
                              versions={filteredStableVersions}
                              currentVersion={currentVersion}
                              sortedVersions={sortedVersions}
                              latestStableVersion={latestStableVersion}
                              onSelectVersion={setSelectedVersion}
                              maxDisplay={10}
                            />
                          </div>
                        )}

                        {/* Prerelease Versions */}
                        {filteredPrereleaseVersions.length > 0 && (
                          <div>
                            <div className="mb-3 flex items-center gap-2">
                              <h3 className="font-semibold text-sm">Prerelease Versions</h3>
                              <span className="inline-flex items-center rounded-md bg-yellow-50 px-2 py-1 font-medium text-xs text-yellow-700 ring-1 ring-yellow-600/20 ring-inset dark:bg-yellow-950 dark:text-yellow-300 dark:ring-yellow-800">
                                {filteredPrereleaseVersions.length}
                              </span>
                            </div>
                            <VersionList
                              versions={filteredPrereleaseVersions}
                              currentVersion={currentVersion}
                              sortedVersions={sortedVersions}
                              latestStableVersion={latestStableVersion}
                              onSelectVersion={setSelectedVersion}
                              maxDisplay={5}
                            />
                          </div>
                        )}
                      </div>
                    )}
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="dependencies" className="mt-6">
                {currentVersionData && (
                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center gap-2">
                        <PackageIcon className="h-5 w-5" />
                        Runtime Dependencies
                      </CardTitle>
                      <CardDescription>Runtime dependencies for v{currentVersion}</CardDescription>
                    </CardHeader>
                    <CardContent>
                      {currentVersionData.version.dependencies &&
                        (() => {
                          const dependencies = safeJsonParse(currentVersionData.version.dependencies);
                          return dependencies && Object.keys(dependencies).length > 0 ? (
                            <DependencyTable dependencies={dependencies} title="" />
                          ) : (
                            <p className="text-muted-foreground text-sm">
                              No runtime dependencies found for this version.
                            </p>
                          );
                        })()}
                      {!currentVersionData.version.dependencies && (
                        <p className="text-muted-foreground text-sm">No runtime dependencies found for this version.</p>
                      )}

                      {/* Peer Dependencies Section */}
                      {currentVersionData.version.peer_dependencies &&
                        (() => {
                          const peerDependencies = safeJsonParse(currentVersionData.version.peer_dependencies);
                          return peerDependencies && Object.keys(peerDependencies).length > 0 ? (
                            <div className="mt-6">
                              <h4 className="mb-3 font-semibold text-sm">Peer Dependencies</h4>
                              <DependencyTable dependencies={peerDependencies} title="" />
                            </div>
                          ) : null;
                        })()}
                    </CardContent>
                  </Card>
                )}
              </TabsContent>

              <TabsContent value="dev-dependencies" className="mt-6">
                {currentVersionData && (
                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center gap-2">
                        <PackageIcon className="h-5 w-5" />
                        Development Dependencies
                      </CardTitle>
                      <CardDescription>Development dependencies for v{currentVersion}</CardDescription>
                    </CardHeader>
                    <CardContent>
                      {currentVersionData.version.dev_dependencies &&
                        (() => {
                          const devDependencies = safeJsonParse(currentVersionData.version.dev_dependencies);
                          return devDependencies && Object.keys(devDependencies).length > 0 ? (
                            <DependencyTable dependencies={devDependencies} title="" />
                          ) : (
                            <p className="text-muted-foreground text-sm">
                              No development dependencies found for this version.
                            </p>
                          );
                        })()}
                      {!currentVersionData.version.dev_dependencies && (
                        <p className="text-muted-foreground text-sm">
                          No development dependencies found for this version.
                        </p>
                      )}
                    </CardContent>
                  </Card>
                )}
              </TabsContent>
            </Tabs>
          </div>

          {/* Sidebar */}
          <div className="slide-in-from-right-4 animate-in space-y-6 duration-500 lg:col-span-2">
            {/* Install Command */}
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Install</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div>
                    <p className="mb-2 text-muted-foreground text-sm">npm</p>
                    <div className="flex items-center gap-2 rounded-md bg-muted p-3">
                      <code className="flex-1 font-mono text-xs">npm install {pkg.name}</code>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => copyToClipboard(`npm install ${pkg.name}`, "npm")}
                          >
                            {copiedCommand === "npm" ? <Check className="h-4 w-4" /> : "Copy"}
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>{copiedCommand === "npm" ? "Copied!" : "Copy to clipboard"}</TooltipContent>
                      </Tooltip>
                    </div>
                  </div>
                  <div>
                    <p className="mb-2 text-muted-foreground text-sm">yarn</p>
                    <div className="flex items-center gap-2 rounded-md bg-muted p-3">
                      <code className="flex-1 font-mono text-xs">yarn add {pkg.name}</code>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => copyToClipboard(`yarn add ${pkg.name}`, "yarn")}
                          >
                            {copiedCommand === "yarn" ? <Check className="h-4 w-4" /> : "Copy"}
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>{copiedCommand === "yarn" ? "Copied!" : "Copy to clipboard"}</TooltipContent>
                      </Tooltip>
                    </div>
                  </div>
                  <div>
                    <p className="mb-2 text-muted-foreground text-sm">pnpm</p>
                    <div className="flex items-center gap-2 rounded-md bg-muted p-3">
                      <code className="flex-1 font-mono text-xs">pnpm add {pkg.name}</code>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => copyToClipboard(`pnpm add ${pkg.name}`, "pnpm")}
                          >
                            {copiedCommand === "pnpm" ? <Check className="h-4 w-4" /> : "Copy"}
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>{copiedCommand === "pnpm" ? "Copied!" : "Copy to clipboard"}</TooltipContent>
                      </Tooltip>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Package Info */}
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
                    <span className="text-muted-foreground text-sm">Version</span>
                    <span className="font-mono text-sm">{currentVersion}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground text-sm">License</span>
                    <span className="text-sm">{pkg.license || "N/A"}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground text-sm">Visibility</span>
                    {pkg.is_private ? (
                      <span className="inline-flex items-center rounded-md bg-yellow-50 px-2 py-1 font-medium text-xs text-yellow-800 ring-1 ring-yellow-600/20 ring-inset dark:bg-yellow-950 dark:text-yellow-300 dark:ring-yellow-800">
                        Private
                      </span>
                    ) : (
                      <span className="inline-flex items-center rounded-md bg-green-50 px-2 py-1 font-medium text-green-700 text-xs ring-1 ring-green-600/20 ring-inset dark:bg-green-950 dark:text-green-300 dark:ring-green-800">
                        Public
                      </span>
                    )}
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground text-sm">Created</span>
                    <span className="text-sm">{format(new Date(pkg.created_at), "MMM d, yyyy")}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground text-sm">Updated</span>
                    <span className="text-sm">{format(new Date(pkg.updated_at), "MMM d, yyyy")}</span>
                  </div>
                </div>

                {/* Links */}
                {(pkg.homepage || pkg.repository_url) && (
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
                )}
              </CardContent>
            </Card>

            {/* Package Statistics */}
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Statistics</CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground text-sm">Total Versions</span>
                  <span className="font-medium text-sm">{sortedVersions.length}</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground text-sm">Stable Releases</span>
                  <span className="font-medium text-sm">{stableVersions.length}</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground text-sm">Prerelease Versions</span>
                  <span className="font-medium text-sm">{prereleaseVersions.length}</span>
                </div>
                {currentVersionData && (
                  <>
                    <div className="flex items-center justify-between">
                      <span className="text-muted-foreground text-sm">Files in v{currentVersion}</span>
                      <span className="font-medium text-sm">{currentVersionData.files.length}</span>
                    </div>
                    <div className="flex items-center justify-between">
                      <span className="text-muted-foreground text-sm">Total Size</span>
                      <span className="font-medium text-sm">
                        {formatBytes(currentVersionData.files.reduce((sum, file) => sum + file.size_bytes, 0))}
                      </span>
                    </div>
                    {currentVersionData.version.dependencies && (
                      <div className="flex items-center justify-between">
                        <span className="text-muted-foreground text-sm">Dependencies</span>
                        <span className="font-medium text-sm">
                          {Object.keys(safeJsonParse(currentVersionData.version.dependencies) || {}).length}
                        </span>
                      </div>
                    )}
                  </>
                )}
              </CardContent>
            </Card>

            {/* Keywords */}
            {(() => {
              const keywords = parseKeywords(pkg.keywords);
              return keywords.length > 0 ? (
                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <Tag className="h-5 w-5" />
                      Keywords
                      <span className="ml-auto text-muted-foreground text-xs">
                        {keywords.length} {keywords.length === 1 ? "tag" : "tags"}
                      </span>
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="flex flex-wrap gap-2">
                      {keywords.map((keyword, index) => (
                        <span
                          key={`${keyword}-${index}`}
                          className="inline-flex items-center rounded-full bg-muted px-3 py-1.5 font-medium text-xs ring-1 ring-border ring-inset transition-colors hover:scale-105"
                          title={`Keyword: ${keyword}`}
                        >
                          {keyword}
                        </span>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              ) : null;
            })()}
          </div>
        </div>
      </div>
    </>
  );
}
