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

export interface PackageFile {
  id: number;
  version_id: number;
  filename: string;
  size_bytes: number;
  created_at: string;
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
  readme: string | null;
  created_at: string;
  updated_at: string;
}

export interface PackageVersionWithFiles {
  version: PackageVersion;
  files: PackageFile[];
}

export interface PackageWithVersions {
  package: Package;
  versions: PackageVersionWithFiles[];
}

export interface PaginationMetadata {
  page: number;
  limit: number;
  total_pages: number;
  has_next: boolean;
  has_prev: boolean;
}

export interface PackageListResponse {
  packages: PackageWithVersions[];
  total_count: number;
  total_size_bytes: number;
  total_size_mb: number;
  pagination: PaginationMetadata;
}

export interface PackageListParams {
  page?: number;
  limit?: number;
  search?: string;
  sort?: string;
  order?: "asc" | "desc";
}

// Single package response type
export interface PackageResponse {
  package: Package;
  versions: PackageVersionWithFiles[];
  total_size_bytes?: number;
  total_size_mb?: number;
}
