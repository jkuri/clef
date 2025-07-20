import {
  AudioWaveform,
  BookOpen,
  Frame,
  GalleryVerticalEnd,
  Map as MapIcon,
  Package2,
  PieChart,
  Settings2,
  SquareTerminal,
} from "lucide-react";
import type * as React from "react";

import { NavMain } from "@/components/nav-main";
import { NavProjects } from "@/components/nav-projects";
import { NavUser } from "@/components/nav-user";
import { TeamSwitcher } from "@/components/team-switcher";
import { Sidebar, SidebarContent, SidebarFooter, SidebarHeader, SidebarRail } from "@/components/ui/sidebar";

const data = {
  user: {
    name: "John Doe",
    email: "john@clef.app",
    avatar: "/avatars/user.jpg",
  },
  teams: [
    {
      name: "Clef",
      logo: GalleryVerticalEnd,
      plan: "Pro",
    },
    {
      name: "Personal",
      logo: AudioWaveform,
      plan: "Free",
    },
  ],
  navMain: [
    {
      title: "Dashboard",
      url: "/dashboard",
      icon: SquareTerminal,
      isActive: true,
      items: [
        {
          title: "Overview",
          url: "/dashboard",
        },
        {
          title: "Analytics",
          url: "/dashboard/analytics",
        },
        {
          title: "Reports",
          url: "/dashboard/reports",
        },
      ],
    },
    {
      title: "Packages",
      url: "/packages",
      icon: Package2,
      items: [
        {
          title: "All",
          url: "/packages",
        },
        {
          title: "Popular",
          url: "/packages/popular",
        },
      ],
    },
    {
      title: "Documentation",
      url: "/docs",
      icon: BookOpen,
      items: [
        {
          title: "Getting Started",
          url: "/docs/getting-started",
        },
        {
          title: "API Reference",
          url: "/docs/api",
        },
        {
          title: "Examples",
          url: "/docs/examples",
        },
        {
          title: "Changelog",
          url: "/docs/changelog",
        },
      ],
    },
    {
      title: "Settings",
      url: "/settings",
      icon: Settings2,
      items: [
        {
          title: "Profile",
          url: "/settings/profile",
        },
        {
          title: "Account",
          url: "/settings/account",
        },
        {
          title: "Preferences",
          url: "/settings/preferences",
        },
        {
          title: "Billing",
          url: "/settings/billing",
        },
      ],
    },
  ],
  projects: [
    {
      name: "Web Application",
      url: "#",
      icon: Frame,
    },
    {
      name: "Mobile App",
      url: "#",
      icon: PieChart,
    },
    {
      name: "API Service",
      url: "#",
      icon: MapIcon,
    },
  ],
};

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader>
        <TeamSwitcher teams={data.teams} />
      </SidebarHeader>
      <SidebarContent>
        <NavMain items={data.navMain} />
        <NavProjects projects={data.projects} />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={data.user} />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
