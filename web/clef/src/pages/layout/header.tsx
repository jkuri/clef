import { NavLink } from "react-router";
import { Logo } from "@/components/logo";

// import { UserMenu } from "./user-menu";

const items = [
  { title: "Dashboard", href: "/dashboard" },
  { title: "Packages", href: "/packages" },
];

export function Header() {
  return (
    <header className="sticky top-0 z-50 w-full border-border border-b bg-background-header backdrop-blur supports-[backdrop-filter]:bg-background-header/60">
      <div className="mx-auto w-full max-w-7xl">
        <div className="mx-4 flex h-14 items-center justify-between gap-2 md:gap-4">
          <div className="flex items-center gap-2">
            <div className="mr-4">
              <NavLink to="/dashboard">
                <Logo className="h-6" />
              </NavLink>
            </div>
            <nav className="hidden items-center gap-4 text-sm md:flex xl:gap-6">
              {items.map((item) => (
                <NavLink
                  key={item.title}
                  to={item.href}
                  className={({ isActive }) =>
                    `transition-colors hover:text-foreground ${isActive ? "text-foreground" : "text-muted-foreground"}`
                  }
                >
                  {item.title}
                </NavLink>
              ))}
            </nav>
          </div>
          <div className="ml-auto flex items-center justify-end gap-6">
            {/* {user && <UserMenu username={user.username} />} */}
          </div>
        </div>
      </div>
    </header>
  );
}
