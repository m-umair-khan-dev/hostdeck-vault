import { Outlet, Link, useLocation } from "react-router-dom";
import { Server, Shield, Key, GitBranch, Settings, LayoutDashboard, FileText } from "lucide-react";
import { SidebarProvider, Sidebar, SidebarContent, SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarFooter } from "@/components/ui/sidebar";

const navItems = [
  { icon: LayoutDashboard, label: "Dashboard", href: "/" },
  { icon: Server, label: "Servers", href: "/servers" },
  { icon: Shield, label: "Jump Hosts", href: "/jump-hosts" },
  { icon: Key, label: "SSH Keys", href: "/keys" },
  { icon: GitBranch, label: "Git Sync", href: "/git" },
  { icon: FileText, label: "Raw Config", href: "/config" },
  { icon: Settings, label: "Settings", href: "/settings" },
];

export default function DashboardLayout() {
  const location = useLocation();

  return (
    <SidebarProvider>
      <div className="flex h-screen w-full overflow-hidden bg-background">
        <Sidebar className="border-r h-full">
          <SidebarHeader className="p-4 border-b flex flex-row items-center space-x-2">
            <Server className="w-6 h-6 text-primary" />
            <h1 className="text-lg font-bold">HostDeck Vault</h1>
          </SidebarHeader>
          <SidebarContent className="p-2">
            <SidebarMenu>
              {navItems.map((item) => (
                <SidebarMenuItem key={item.href}>
                  <SidebarMenuButton
                    asChild
                    isActive={location.pathname === item.href}
                    className="w-full flex items-center p-2 rounded-md transition-colors"
                  >
                    <Link to={item.href}>
                      <item.icon className="w-5 h-5 mr-3" />
                      <span>{item.label}</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarContent>
          <SidebarFooter className="p-4 border-t text-xs text-muted-foreground text-center">
            v0.1.5
          </SidebarFooter>
        </Sidebar>

        <main className="flex-1 flex flex-col h-full overflow-y-auto">
          <div className="p-8 max-w-5xl mx-auto w-full">
            <Outlet />
          </div>
        </main>
      </div>
    </SidebarProvider>
  );
}
