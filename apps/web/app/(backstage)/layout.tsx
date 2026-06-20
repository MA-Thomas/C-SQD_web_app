import { AppSidebar } from "../components/app-sidebar";

/// Backstage shell: dense operational sidebar, role-aware sections.
export default function BackstageLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <main className="app-shell">
      <AppSidebar />
      <section className="workspace">{children}</section>
    </main>
  );
}
