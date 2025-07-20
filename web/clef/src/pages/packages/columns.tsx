"use client";

import type { ColumnDef } from "@tanstack/react-table";
import { format, isValid, parseISO } from "date-fns";
import { ArrowDown, ArrowUp, ArrowUpDown, MoreHorizontal, Package } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { PackageWithVersions } from "@/types/packages";

interface ColumnProps {
  onSort: (field: string) => void;
  sortField: string | null;
  sortOrder: "asc" | "desc" | null;
}

export const createColumns = ({ onSort, sortField, sortOrder }: ColumnProps): ColumnDef<PackageWithVersions>[] => [
  {
    id: "select",
    header: ({ table }) => (
      <Checkbox
        checked={table.getIsAllPageRowsSelected() || (table.getIsSomePageRowsSelected() && "indeterminate")}
        onCheckedChange={(value) => table.toggleAllPageRowsSelected(!!value)}
        aria-label="Select all"
      />
    ),
    cell: ({ row }) => (
      <Checkbox
        checked={row.getIsSelected()}
        onCheckedChange={(value) => row.toggleSelected(!!value)}
        aria-label="Select row"
      />
    ),
    enableSorting: false,
    enableHiding: false,
  },
  {
    accessorKey: "name",
    header: () => {
      const isActive = sortField === "name";

      return (
        <Button variant="ghost" onClick={() => onSort("name")}>
          Package Name
          {isActive ? (
            sortOrder === "asc" ? (
              <ArrowUp className="ml-2 h-4 w-4" />
            ) : (
              <ArrowDown className="ml-2 h-4 w-4" />
            )
          ) : (
            <ArrowUpDown className="ml-2 h-4 w-4" />
          )}
        </Button>
      );
    },
    cell: ({ row }) => {
      const item = row.original;
      const pkg = item.package;
      return (
        <div className="flex items-center space-x-3">
          <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-primary/10">
            <Package className="h-4 w-4" />
          </div>
          <div className="min-w-0 flex-1">
            <p className="truncate font-medium text-sm leading-none">{pkg.name}</p>
            <p className="mt-1 text-muted-foreground text-xs">
              {pkg.description
                ? pkg.description.length > 60
                  ? `${pkg.description.substring(0, 60)}...`
                  : pkg.description
                : "No description"}
            </p>
          </div>
        </div>
      );
    },
  },
  {
    accessorKey: "versions",
    header: "Versions",
    cell: ({ row }) => {
      const item = row.original;
      const versions = item.versions;
      return (
        <div className="text-sm">
          <span className="font-medium">{versions.length}</span>
          <span className="ml-1 text-muted-foreground">version{versions.length !== 1 ? "s" : ""}</span>
        </div>
      );
    },
  },
  {
    id: "total_size",
    header: () => <div className="text-right">Total Size</div>,
    cell: ({ row }) => {
      const item = row.original;
      const totalSizeBytes = item.versions.flatMap((ver) => ver.files).reduce((sum, file) => sum + file.size_bytes, 0);

      const formatBytes = (bytes: number) => {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const sizes = ["B", "KB", "MB", "GB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return `${parseFloat((bytes / k ** i).toFixed(1))} ${sizes[i]}`;
      };

      return <div className="text-right font-medium">{formatBytes(totalSizeBytes)}</div>;
    },
  },
  {
    accessorKey: "license",
    header: "License",
    cell: ({ row }) => {
      const item = row.original;
      const license = item.package.license;
      return (
        <div className="text-sm">
          {license ? (
            <span className="inline-flex items-center rounded-md bg-gray-50 px-2 py-1 font-medium text-gray-600 text-xs ring-1 ring-gray-500/10 ring-inset">
              {license}
            </span>
          ) : (
            <span className="text-muted-foreground">-</span>
          )}
        </div>
      );
    },
  },
  {
    accessorKey: "created_at",
    header: () => {
      const isActive = sortField === "created_at";

      return (
        <Button variant="ghost" onClick={() => onSort("created_at")}>
          Created
          {isActive ? (
            sortOrder === "asc" ? (
              <ArrowUp className="ml-2 h-4 w-4" />
            ) : (
              <ArrowDown className="ml-2 h-4 w-4" />
            )
          ) : (
            <ArrowUpDown className="ml-2 h-4 w-4" />
          )}
        </Button>
      );
    },
    cell: ({ row }) => {
      const item = row.original;
      const dateString = item.package.created_at;

      try {
        const date = parseISO(dateString);
        if (isValid(date)) {
          return (
            <div className="text-sm">
              <div className="font-medium">{format(date, "MMM d, yyyy")}</div>
              <div className="text-muted-foreground text-xs">{format(date, "h:mm a")}</div>
            </div>
          );
        }
      } catch {
        // Fall back to basic parsing if parseISO fails
        const fallbackDate = new Date(dateString);
        if (!Number.isNaN(fallbackDate.getTime())) {
          return (
            <div className="text-sm">
              <div className="font-medium">{format(fallbackDate, "MMM d, yyyy")}</div>
              <div className="text-muted-foreground text-xs">{format(fallbackDate, "h:mm a")}</div>
            </div>
          );
        }
      }

      return <div className="text-muted-foreground text-sm">Invalid date</div>;
    },
  },
  {
    accessorKey: "updated_at",
    header: () => {
      const isActive = sortField === "updated_at";

      return (
        <Button variant="ghost" onClick={() => onSort("updated_at")}>
          Updated
          {isActive ? (
            sortOrder === "asc" ? (
              <ArrowUp className="ml-2 h-4 w-4" />
            ) : (
              <ArrowDown className="ml-2 h-4 w-4" />
            )
          ) : (
            <ArrowUpDown className="ml-2 h-4 w-4" />
          )}
        </Button>
      );
    },
    cell: ({ row }) => {
      const item = row.original;
      const dateString = item.package.updated_at;

      try {
        const date = parseISO(dateString);
        if (isValid(date)) {
          return (
            <div className="text-sm">
              <div className="font-medium">{format(date, "MMM d, yyyy")}</div>
              <div className="text-muted-foreground text-xs">{format(date, "h:mm a")}</div>
            </div>
          );
        }
      } catch {
        // Fall back to basic parsing if parseISO fails
        const fallbackDate = new Date(dateString);
        if (!Number.isNaN(fallbackDate.getTime())) {
          return (
            <div className="text-sm">
              <div className="font-medium">{format(fallbackDate, "MMM d, yyyy")}</div>
              <div className="text-muted-foreground text-xs">{format(fallbackDate, "h:mm a")}</div>
            </div>
          );
        }
      }

      return <div className="text-muted-foreground text-sm">Invalid date</div>;
    },
  },
  {
    id: "actions",
    enableHiding: false,
    cell: ({ row }) => {
      const item = row.original;
      const pkg = item.package;

      return (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" className="h-8 w-8 p-0">
              <span className="sr-only">Open menu</span>
              <MoreHorizontal className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuLabel>Actions</DropdownMenuLabel>
            <DropdownMenuItem onClick={() => navigator.clipboard.writeText(pkg.name)}>
              Copy package name
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem>View details</DropdownMenuItem>
            <DropdownMenuItem>View versions</DropdownMenuItem>
            {pkg.homepage && (
              <DropdownMenuItem onClick={() => window.open(pkg.homepage!, "_blank")}>Open homepage</DropdownMenuItem>
            )}
            {pkg.repository_url && (
              <DropdownMenuItem onClick={() => window.open(pkg.repository_url!, "_blank")}>
                Open repository
              </DropdownMenuItem>
            )}
          </DropdownMenuContent>
        </DropdownMenu>
      );
    },
  },
];
