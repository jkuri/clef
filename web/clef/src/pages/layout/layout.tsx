import { Outlet } from "react-router";
import { Header } from "./header";

export function Layout() {
  return (
    <div className="relative flex min-h-svh flex-col bg-container">
      <Header />
      <main className="container mx-auto flex w-full max-w-7xl flex-1 flex-col gap-4 p-4 pb-16">
        <Outlet />
      </main>
    </div>
  );
}
