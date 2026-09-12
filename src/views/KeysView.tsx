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
import { Key as KeyIcon, Plus, Trash2 } from "lucide-react";
import { toast } from "sonner";

interface SSHKey {
  name: string;
  private_key_path: string;
  public_key_path: string;
  key_type: string;
}

export default function KeysView() {
  const [keys, setKeys] = useState<SSHKey[]>([]);
  const [isDialogOpen, setIsDialogOpen] = useState(false);

  // Form State
  const [name, setName] = useState("");
  const [privateKeyPath, setPrivateKeyPath] = useState("");
  const [publicKeyPath, setPublicKeyPath] = useState("");
  const [keyType, setKeyType] = useState("rsa");

  const loadKeys = async () => {
    try {
      const data: SSHKey[] = await invoke("list_keys");
      setKeys(data);
    } catch (e) {
      toast.error("Failed to load keys");
    }
  };

  useEffect(() => {
    loadKeys();
  }, []);

  const handleOpenAdd = () => {
    setName("");
    setPrivateKeyPath("");
    setPublicKeyPath("");
    setKeyType("rsa");
    setIsDialogOpen(true);
  };

  const handleAddKey = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      const keyPayload = {
        name,
        private_key_path: privateKeyPath,
        public_key_path: publicKeyPath,
        key_type: keyType,
      };

      await invoke("add_key", { key: keyPayload });
      toast.success(`Key ${name} added successfully.`);
      setIsDialogOpen(false);
      loadKeys();
    } catch (e: any) {
      toast.error(e.toString());
    }
  };

  const handleDelete = async (keyName: string) => {
    try {
      await invoke("delete_key", { name: keyName });
      toast.success(`Key ${keyName} deleted.`);
      loadKeys();
    } catch (e: any) {
      toast.error(e.toString());
    }
  };

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">SSH Keys</h1>
          <p className="text-muted-foreground mt-2">Manage your SSH identity files here.</p>
        </div>
        <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
          <DialogTrigger asChild>
            <Button onClick={handleOpenAdd}>
              <Plus className="w-4 h-4 mr-2" />
              Add SSH Key
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Register SSH Key</DialogTitle>
              <DialogDescription>
                Register an existing SSH key pair to use with your servers.
              </DialogDescription>
            </DialogHeader>
            <form onSubmit={handleAddKey} className="space-y-4">
              <div className="space-y-4">
                <div className="space-y-2">
                  <Label>Key Name</Label>
                  <Input value={name} onChange={(e) => setName(e.target.value)} required placeholder="e.g. prod_key" />
                </div>
                <div className="space-y-2">
                  <Label>Private Key Path</Label>
                  <Input value={privateKeyPath} onChange={(e) => setPrivateKeyPath(e.target.value)} required placeholder="e.g. ~/.ssh/id_rsa" />
                </div>
                <div className="space-y-2">
                  <Label>Public Key Path (Optional)</Label>
                  <Input value={publicKeyPath} onChange={(e) => setPublicKeyPath(e.target.value)} placeholder="e.g. ~/.ssh/id_rsa.pub" />
                  <p className="text-xs text-muted-foreground">Required if you want to deploy this key to a remote server.</p>
                </div>
              </div>
              <Button type="submit" className="w-full">
                Save Key
              </Button>
            </form>
          </DialogContent>
        </Dialog>
      </div>

      <div className="border rounded-md">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Key Name</TableHead>
              <TableHead>Private Key</TableHead>
              <TableHead>Public Key</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {keys.length === 0 ? (
              <TableRow>
                <TableCell colSpan={4} className="text-center py-8 text-muted-foreground">
                  No SSH keys configured. Add one to get started.
                </TableCell>
              </TableRow>
            ) : (
              keys.map((k) => (
                <TableRow key={k.name}>
                  <TableCell className="font-medium flex items-center">
                    <KeyIcon className="w-4 h-4 mr-2 text-primary" />
                    {k.name}
                  </TableCell>
                  <TableCell className="text-xs text-muted-foreground font-mono">
                    {k.private_key_path}
                  </TableCell>
                  <TableCell className="text-xs text-muted-foreground font-mono">
                    {k.public_key_path || "N/A"}
                  </TableCell>
                  <TableCell className="text-right space-x-2">
                    <Button variant="destructive" size="sm" onClick={() => handleDelete(k.name)}>
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