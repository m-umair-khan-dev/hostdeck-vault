import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Server as ServerIcon, Plus, Trash2, Key } from "lucide-react";
import { toast } from "sonner";

interface Server {
  alias: string;
  hostname: string;
  username: string;
  port: number;
  key_name?: string;
  jump_host?: string;
  extra_options?: string;
}

interface SSHKey {
  name: string;
  private_key_path: string;
  public_key_path: string;
  key_type: string;
}

export default function ServersView() {
  const [servers, setServers] = useState<Server[]>([]);
  const [keys, setKeys] = useState<SSHKey[]>([]);
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [editingHost, setEditingHost] = useState<string | null>(null);

  // Form State
  const [host, setHost] = useState("");
  const [hostname, setHostname] = useState("");
  const [user, setUser] = useState("");
  const [port, setPort] = useState("22");
  const [identityMode, setIdentityMode] = useState("none");
  const [identityFile, setIdentityFile] = useState("");

  // Deploy Form State
  const [showDeployForm, setShowDeployForm] = useState(false);
  const [deployPassword, setDeployPassword] = useState("");
  const [deployKeyName, setDeployKeyName] = useState("");
  const [isDeploying, setIsDeploying] = useState(false);

  const loadData = async () => {
    try {
      const serverData: Server[] = await invoke("list_servers");
      setServers(serverData);
      
      const keysData: SSHKey[] = await invoke("list_keys");
      setKeys(keysData);
    } catch (e) {
      toast.error("Failed to load data");
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const handleOpenAdd = () => {
    setEditingHost(null);
    setHost("");
    setHostname("");
    setUser("");
    setPort("22");
    setIdentityMode("none");
    setIdentityFile("");
    setShowDeployForm(false);
    setDeployPassword("");
    setIsDialogOpen(true);
  };

  const handleOpenEdit = (srv: Server) => {
    setEditingHost(srv.alias);
    setHost(srv.alias);
    setHostname(srv.hostname);
    setUser(srv.username);
    setPort(srv.port.toString());
    setShowDeployForm(false);
    setDeployPassword("");
    
    if (srv.key_name) {
      // If the key is mapped to a known SSHKey, we select that
      const knownKey = keys.find(k => k.name === srv.key_name);
      if (knownKey) {
        setIdentityMode(knownKey.name);
        setIdentityFile("");
      } else {
        setIdentityMode("custom");
        setIdentityFile(srv.key_name);
      }
    } else {
      setIdentityMode("none");
      setIdentityFile("");
    }
    
    setIsDialogOpen(true);
  };

  const handleSaveServer = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      let finalKeyName = null;
      if (identityMode === "custom" && identityFile.trim() !== "") {
        finalKeyName = identityFile.trim();
      } else if (identityMode !== "none" && identityMode !== "custom") {
        finalKeyName = identityMode;
      }

      const serverPayload = {
        alias: host,
        hostname,
        username: user,
        port: parseInt(port),
        key_name: finalKeyName,
      };

      if (editingHost) {
        await invoke("update_server", { originalAlias: editingHost, server: serverPayload });
        toast.success(`Server ${host} updated successfully.`);
      } else {
        await invoke("add_server", { server: serverPayload });
        toast.success(`Server ${host} added successfully.`);
      }
      setIsDialogOpen(false);
      loadData();
    } catch (e: any) {
      toast.error(e.toString());
    }
  };

  const handleDelete = async (hostAlias: string) => {
    try {
      await invoke("delete_server", { host: hostAlias });
      toast.success(`Server ${hostAlias} deleted.`);
      loadData();
    } catch (e: any) {
      toast.error(e.toString());
    }
  };

  const handleDeployKey = async () => {
    if (!deployKeyName) {
      toast.error("Please select a key to deploy.");
      return;
    }
    if (!deployPassword) {
      toast.error("Please provide the SSH password to deploy the key.");
      return;
    }

    const selectedKey = keys.find(k => k.name === deployKeyName);
    if (!selectedKey) {
      toast.error("Key not found.");
      return;
    }

    setIsDeploying(true);
    toast.loading("Deploying key...", { id: "deploy" });

    try {
      await invoke("tauri_deploy_key", {
        hostname: hostname,
        username: user,
        password: deployPassword,
        publicKeyPath: selectedKey.public_key_path,
        port: parseInt(port)
      });
      
      toast.success(`Key successfully deployed to ${hostname}!`, { id: "deploy" });
      
      // Auto-update the identity file to this key
      setIdentityMode(deployKeyName);
      setIdentityFile("");
      setShowDeployForm(false);
      setDeployPassword("");
      
    } catch (e: any) {
      toast.error(e.toString(), { id: "deploy" });
    } finally {
      setIsDeploying(false);
    }
  };

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Servers</h1>
          <p className="text-muted-foreground mt-2">Manage your SSH Servers here.</p>
        </div>
        <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
          <DialogTrigger asChild>
            <Button onClick={handleOpenAdd}>
              <Plus className="w-4 h-4 mr-2" />
              Add Server
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>{editingHost ? "Edit Server" : "Add New Server"}</DialogTitle>
              <DialogDescription>
                Configure connection details for your SSH server.
              </DialogDescription>
            </DialogHeader>
            <form onSubmit={handleSaveServer} className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>Alias (Host)</Label>
                  <Input value={host} onChange={(e) => setHost(e.target.value)} required placeholder="web-prod-1" />
                </div>
                <div className="space-y-2">
                  <Label>Hostname / IP</Label>
                  <Input value={hostname} onChange={(e) => setHostname(e.target.value)} required placeholder="192.168.1.10" />
                </div>
                <div className="space-y-2">
                  <Label>User</Label>
                  <Input value={user} onChange={(e) => setUser(e.target.value)} required placeholder="root" />
                </div>
                <div className="space-y-2">
                  <Label>Port</Label>
                  <Input type="number" value={port} onChange={(e) => setPort(e.target.value)} required />
                </div>
                <div className="col-span-2 space-y-2">
                  <Label>Identity File / Key Name</Label>
                  <Select value={identityMode} onValueChange={setIdentityMode}>
                    <SelectTrigger>
                      <SelectValue placeholder="Select an SSH Key" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="none">No Key (Password Auth)</SelectItem>
                      {keys.map(k => (
                        <SelectItem key={k.name} value={k.name}>{k.name}</SelectItem>
                      ))}
                      <SelectItem value="custom">Custom Path...</SelectItem>
                    </SelectContent>
                  </Select>
                  
                  {identityMode === "custom" && (
                    <div className="pt-2">
                      <Input 
                        value={identityFile} 
                        onChange={(e) => setIdentityFile(e.target.value)} 
                        placeholder="e.g. ~/.ssh/id_rsa" 
                        required 
                      />
                    </div>
                  )}
                </div>
              </div>
              <Button type="submit" className="w-full">
                {editingHost ? "Save Changes" : "Save Server"}
              </Button>
            </form>
            {editingHost && (
              <div className="pt-4 mt-4 border-t border-border">
                <h4 className="text-sm font-medium mb-2">Advanced Actions</h4>
                
                {!showDeployForm ? (
                  <Button variant="outline" className="w-full text-blue-500" onClick={() => setShowDeployForm(true)}>
                    <Key className="w-4 h-4 mr-2" />
                    Deploy SSH Key to Server
                  </Button>
                ) : (
                  <div className="space-y-4 p-4 border border-dashed rounded-md bg-secondary/20">
                    <h5 className="text-sm font-medium">Deploy Public Key</h5>
                    <div className="space-y-2">
                      <Label>Select Key to Deploy</Label>
                      <Select value={deployKeyName} onValueChange={setDeployKeyName}>
                        <SelectTrigger>
                          <SelectValue placeholder="Select a key..." />
                        </SelectTrigger>
                        <SelectContent>
                          {keys.map(k => (
                            <SelectItem key={k.name} value={k.name}>{k.name}</SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="space-y-2">
                      <Label>Temporary SSH Password</Label>
                      <Input 
                        type="password" 
                        value={deployPassword} 
                        onChange={(e) => setDeployPassword(e.target.value)} 
                        placeholder="Required for initial deployment" 
                      />
                    </div>
                    <div className="flex space-x-2 pt-2">
                      <Button variant="outline" className="w-full" onClick={() => setShowDeployForm(false)}>Cancel</Button>
                      <Button className="w-full bg-blue-600 hover:bg-blue-700 text-white" onClick={handleDeployKey} disabled={isDeploying}>
                        {isDeploying ? "Deploying..." : "Deploy Now"}
                      </Button>
                    </div>
                  </div>
                )}
              </div>
            )}
          </DialogContent>
        </Dialog>
      </div>

      <div className="border rounded-md">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Host Alias</TableHead>
              <TableHead>Address</TableHead>
              <TableHead>Auth</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {servers.length === 0 ? (
              <TableRow>
                <TableCell colSpan={4} className="text-center py-8 text-muted-foreground">
                  No servers found. Add one to get started.
                </TableCell>
              </TableRow>
            ) : (
              servers.map((srv) => (
                <TableRow key={srv.alias}>
                  <TableCell className="font-medium flex items-center">
                    <ServerIcon className="w-4 h-4 mr-2 text-primary" />
                    {srv.alias}
                  </TableCell>
                  <TableCell>
                    {srv.username}@{srv.hostname}:{srv.port}
                  </TableCell>
                  <TableCell>
                    {srv.key_name ? (
                      <span className="inline-flex items-center text-xs bg-secondary px-2 py-1 rounded-md">
                        <Key className="w-3 h-3 mr-1" /> Key
                      </span>
                    ) : (
                      <span className="inline-flex items-center text-xs border px-2 py-1 rounded-md">
                        Password
                      </span>
                    )}
                  </TableCell>
                  <TableCell className="text-right space-x-2">
                    <Button variant="outline" size="sm" onClick={() => handleOpenEdit(srv)}>
                      Edit
                    </Button>
                    <Button variant="destructive" size="sm" onClick={() => handleDelete(srv.alias)}>
                      <Trash2 className="w-4 h-4" />
                    </Button>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}
