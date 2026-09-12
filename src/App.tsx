import { HashRouter, Routes, Route } from "react-router-dom";
import DashboardLayout from "@/layouts/DashboardLayout";
import DashboardView from "@/views/DashboardView";
import ServersView from "@/views/ServersView";
import JumpHostsView from "@/views/JumpHostsView";
import KeysView from "@/views/KeysView";
import GitView from "@/views/GitView";
import ConfigView from "@/views/ConfigView";
import { SettingsView } from "@/views/SettingsView";
import { Toaster } from "@/components/ui/sonner";

function App() {
  return (
    <>
      <HashRouter>
        <Routes>
          <Route path="/" element={<DashboardLayout />}>
            <Route index element={<DashboardView />} />
            <Route path="servers" element={<ServersView />} />
            <Route path="jump-hosts" element={<JumpHostsView />} />
            <Route path="keys" element={<KeysView />} />
            <Route path="git" element={<GitView />} />
            <Route path="config" element={<ConfigView />} />
            <Route path="settings" element={<SettingsView />} />
          </Route>
        </Routes>
      </HashRouter>
      <Toaster />
    </>
  );
}

export default App;
