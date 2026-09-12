import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardHeader, CardTitle, CardContent, CardDescription, CardFooter } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { GitBranch, Upload, Link as LinkIcon, Unlink } from "lucide-react";
import { toast } from "sonner";

export default function GitView() {
  const [isConnected, setIsConnected] = useState(false);
  const [repoUrl, setRepoUrl] = useState("");
  const [token, setToken] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  const checkStatus = async () => {
    try {
      const connected: boolean = await invoke("check_git_connected");
      setIsConnected(connected);
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    checkStatus();
  }, []);

  const handleConnect = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    try {
      await invoke("git_connect", { repoUrl, token });
      toast.success("Successfully connected to Git repository.");
      setToken(""); // clear token from UI for security
      checkStatus();
    } catch (e: any) {
      toast.error(e.toString());
    } finally {
      setIsLoading(false);
    }
  };

  const handleDisconnect = async () => {
    try {
      await invoke("git_disconnect");
      toast.success("Disconnected from Git repository.");
      checkStatus();
    } catch (e: any) {
      toast.error(e.toString());
    }
  };

  const handlePush = async () => {
    setIsLoading(true);
    try {
      await invoke("git_push", { commitMessage: "Auto-sync from HostDeck Vault" });
      toast.success("Changes pushed to Git successfully.");
    } catch (e: any) {
      toast.error(e.toString());
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Git Sync</h1>
        <p className="text-muted-foreground mt-2">Backup and synchronize your SSH configs securely using GitHub.</p>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center">
              <GitBranch className="w-5 h-5 mr-2" />
              Repository Connection
            </CardTitle>
            <CardDescription>
              Connect a private GitHub repository to store your configuration. 
              We use the OS native Keyring to store your Personal Access Token securely.
            </CardDescription>
          </CardHeader>
          <CardContent>
            {isConnected ? (
              <div className="flex flex-col items-center justify-center py-6 space-y-4 bg-secondary/30 rounded-md border border-dashed">
                <LinkIcon className="w-12 h-12 text-green-500 mb-2" />
                <h3 className="text-lg font-medium">Connected to Git</h3>
                <p className="text-sm text-muted-foreground text-center">
                  Your SSH configuration is linked to a remote repository and your token is secured in the OS keychain.
                </p>
              </div>
            ) : (
              <form onSubmit={handleConnect} className="space-y-4">
                <div className="space-y-2">
                  <Label>Repository URL</Label>
                  <Input 
                    placeholder="https://github.com/username/ssh-config.git" 
                    value={repoUrl}
                    onChange={(e) => setRepoUrl(e.target.value)}
                    required
                  />
                </div>
                <div className="space-y-2">
                  <Label>Personal Access Token (PAT)</Label>
                  <Input 
                    type="password" 
                    placeholder="ghp_xxxxxxxxxxxx" 
                    value={token}
                    onChange={(e) => setToken(e.target.value)}
                    required
                  />
                  <p className="text-xs text-muted-foreground">
                    Requires 'repo' scope. It will be stored in your system's native credential manager.
                  </p>
                </div>
                <Button type="submit" className="w-full" disabled={isLoading}>
                  {isLoading ? "Connecting..." : "Connect Repository"}
                </Button>
              </form>
            )}
          </CardContent>
          {isConnected && (
            <CardFooter>
              <Button variant="destructive" className="w-full" onClick={handleDisconnect}>
                <Unlink className="w-4 h-4 mr-2" />
                Disconnect
              </Button>
            </CardFooter>
          )}
        </Card>

        <Card className={isConnected ? "" : "opacity-50 pointer-events-none"}>
          <CardHeader>
            <CardTitle className="flex items-center">
              <GitBranch className="w-5 h-5 mr-2" />
              Sync Operations
            </CardTitle>
            <CardDescription>
              Push your local changes to the remote repository.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <Button className="w-full" onClick={handlePush} disabled={!isConnected || isLoading}>
              <Upload className="w-4 h-4 mr-2" />
              {isLoading ? "Pushing..." : "Push to Remote"}
            </Button>
            <p className="text-sm text-muted-foreground">
              This will commit the current state of your servers, keys, and jump hosts to the remote repository.
            </p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}