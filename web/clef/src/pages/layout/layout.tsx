import { Outlet } from "react-router";

export function Layout() {
  return (
    <div className="relative flex min-h-svh flex-col bg-container">
      <main className="container mx-auto flex w-full max-w-7xl flex-1 flex-col gap-4 p-4 pb-16">
        <Outlet />
      </main>
    </div>
  );
}
