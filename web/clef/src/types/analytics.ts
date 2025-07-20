export interface Package {
  id: number;
  name: string;
  description: string;
  author_id: number | null;
  homepage: string | null;
  repository_url: string | null;
  license: string | null;
  keywords: string | null;
  is_private: boolean;
  created_at: string;
  updated_at: string;
}

export interface PopularPackage {
  name: string;
  total_downloads: number;
  unique_versions: number;
  total_size_bytes: number;
}

export interface PackageVersion {
  id: number;
  package_id: number;
  version: string;
  description: string | null;
  main_file: string | null;
  scripts: string | null;
  dependencies: string | null;
  dev_dependencies: string | null;
  peer_dependencies: string | null;
  engines: string | null;
  shasum: string | null;
  created_at: string;
  updated_at: string;
}

export interface PackageFile {
  id: number;
  package_version_id: number;
  filename: string;
  size_bytes: number;
  content_type: string;
  etag: string;
  upstream_url: string;
  file_path: string;
  created_at: string;
  last_accessed: string;
  access_count: number;
}

export interface VersionWithFiles {
  version: PackageVersion;
  files: PackageFile[];
}

export interface RecentPackage {
  package: Package;
  versions: VersionWithFiles[];
}

export interface AnalyticsData {
  total_packages: number;
  total_size_bytes: number;
  total_size_mb: number;
  most_popular_packages: PopularPackage[];
  recent_packages: RecentPackage[];
  cache_hit_rate: number;
}

// The API returns the data directly, not wrapped in a response object
export type AnalyticsApiResponse = AnalyticsData;
