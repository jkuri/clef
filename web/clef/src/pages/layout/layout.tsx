import { Outlet, useLocation } from "react-router";
import { AppSidebar } from "@/components/app-sidebar";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { Separator } from "@/components/ui/separator";
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";

// Define breadcrumb configuration with section and page info
const breadcrumbConfig: Record<string, { section: string; sectionHref: string; page: string }> = {
  "/dashboard": { section: "Dashboard", sectionHref: "/dashboard", page: "Overview" },
  "/packages": { section: "Packages", sectionHref: "/packages", page: "All Packages" },
  "/packages/popular": { section: "Packages", sectionHref: "/packages", page: "Popular" },
  "/docs": { section: "Documentation", sectionHref: "/docs", page: "Documentation" },
  "/docs/getting-started": { section: "Documentation", sectionHref: "/docs", page: "Getting Started" },
  "/docs/api": { section: "Documentation", sectionHref: "/docs", page: "API Reference" },
  "/docs/examples": { section: "Documentation", sectionHref: "/docs", page: "Examples" },
  "/docs/changelog": { section: "Documentation", sectionHref: "/docs", page: "Changelog" },
  "/settings": { section: "Settings", sectionHref: "/settings", page: "Settings" },
  "/settings/profile": { section: "Settings", sectionHref: "/settings", page: "Profile" },
  "/settings/account": { section: "Settings", sectionHref: "/settings", page: "Account" },
  "/settings/preferences": { section: "Settings", sectionHref: "/settings", page: "Preferences" },
  "/settings/billing": { section: "Settings", sectionHref: "/settings", page: "Billing" },
};

function generateBreadcrumbs(pathname: string) {
  const config = breadcrumbConfig[pathname];

  if (!config) {
    // Handle dynamic routes like /packages/:name
    if (pathname.startsWith("/packages/") && pathname !== "/packages") {
      const packageName = pathname.split("/packages/")[1];
      return {
        section: { title: "Packages", href: "/packages" },
        page: packageName,
      };
    }

    // Fallback for unknown routes
    return {
      section: { title: "Dashboard", href: "/dashboard" },
      page: "Overview",
    };
  }

  return {
    section: { title: config.section, href: config.sectionHref },
    page: config.page,
  };
}

export function Layout() {
  const location = useLocation();
  const { section, page } = generateBreadcrumbs(location.pathname);
  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <header className="flex h-16 shrink-0 items-center gap-2 transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-12">
          <div className="flex items-center gap-2 px-4">
            <SidebarTrigger className="-ml-1" />
            <Separator orientation="vertical" className="mr-2 data-[orientation=vertical]:h-4" />
            <Breadcrumb>
              <BreadcrumbList>
                <BreadcrumbItem className="hidden md:block">
                  <BreadcrumbLink href={section.href}>{section.title}</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator className="hidden md:block" />
                <BreadcrumbItem>
                  <BreadcrumbPage>{page}</BreadcrumbPage>
                </BreadcrumbItem>
              </BreadcrumbList>
            </Breadcrumb>
          </div>
        </header>
        <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
          <Outlet />
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}
