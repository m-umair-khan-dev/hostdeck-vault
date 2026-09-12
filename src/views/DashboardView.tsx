import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Server, Shield, Key, GitBranch } from "lucide-react";

export default function DashboardView() {
  const [stats, setStats] = useState({
    servers: 0,
    jumpHosts: 0,
    keys: 0,
    gitConnected: false,
  });

  useEffect(() => {
    async function loadStats() {
      try {
        const servers: any[] = await invoke("list_servers");
        const isConnected: boolean = await invoke("check_git_connected");
        // We haven't implemented list_jump_hosts or list_keys yet but we can mock or add them later
        setStats({
          servers: servers.length,
          jumpHosts: 0,
          keys: 0,
          gitConnected: isConnected,
        });
      } catch (e) {
        console.error("Failed to load stats", e);
      }
    }
    loadStats();
  }, []);

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
        <p className="text-muted-foreground mt-2">
          Overview of your SSH configuration.
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Servers</CardTitle>
            <Server className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.servers}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Jump Hosts</CardTitle>
            <Shield className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.jumpHosts}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">SSH Keys</CardTitle>
            <Key className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.keys}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Git Sync</CardTitle>
            <GitBranch className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {stats.gitConnected ? (
                <span className="text-green-500">Connected</span>
              ) : (
                <span className="text-destructive">Disconnected</span>
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
