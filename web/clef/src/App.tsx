import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Navigate, Route, Routes } from "react-router";
import { composeProviders } from "./lib/compose-providers";
import { Dashboard } from "./pages/dashboard/dasboard";
import { Layout } from "./pages/layout/layout";
import { Package } from "./pages/package/package";
import { Packages } from "./pages/packages/packages";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 60 * 1000,
    },
  },
});

const Providers = composeProviders([[QueryClientProvider, { client: queryClient }], [BrowserRouter]]);

function App() {
  return (
    <Providers>
      <AppRoutes />
    </Providers>
  );
}

function AppRoutes() {
  return (
    <Routes>
      <Route path="/" element={<Layout />}>
        <Route index element={<Navigate to={"/dashboard"} />} />
        <Route path="/dashboard" element={<Dashboard />} />
        <Route path="/packages" element={<Packages />} />
        <Route path="/packages/*" element={<Package />} />
      </Route>
    </Routes>
  );
}

export default App;
