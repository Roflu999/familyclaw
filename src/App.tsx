import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Brain,
  LayoutDashboard,
  Key,
  Shield,
  Bug,
  Terminal,
  Wrench,
  ChevronRight,
  CheckCircle2,
  AlertCircle,
  Loader2,
  MessageSquare,
  Puzzle,
} from "lucide-react";
import { cn } from "./lib/utils";
import Dashboard from "./components/Dashboard";
import ApiKeys from "./components/ApiKeys";
import Safety from "./components/Safety";
import DebugPanel from "./components/DebugPanel";
import SettingsPanel from "./components/SettingsPanel";
import Channels from "./components/Channels";
import Skills from "./components/Skills";
import SelfImprove from "./components/SelfImprove";

type Page = "dashboard" | "apikeys" | "channels" | "skills" | "safety" | "debug" | "settings" | "selfimprove";

interface PrereqStatus {
  node_installed: boolean;
  node_version?: string;
  npm_installed: boolean;
  openclaw_installed: boolean;
  openclaw_version?: string;
  managed_runtime: boolean;
}

function SidebarItem({
  icon: Icon,
  label,
  active,
  onClick,
}: {
  icon: React.ElementType;
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "flex items-center gap-3 w-full px-3 py-2 rounded-lg text-sm font-medium transition-colors",
        active
          ? "bg-primary/10 text-primary"
          : "text-muted-foreground hover:bg-muted hover:text-foreground"
      )}
    >
      <Icon className="h-4 w-4" />
      {label}
    </button>
  );
}

function SetupWizard({ onDone }: { onDone: () => void }) {
  const [step, setStep] = useState(0);
  const [prereqs, setPrereqs] = useState<PrereqStatus | null>(null);
  const [installing, setInstalling] = useState(false);
  const [installLog, setInstallLog] = useState("");

  useEffect(() => {
    checkPrereqs();
  }, []);

  async function checkPrereqs() {
    const status = await invoke<PrereqStatus>("check_prerequisites");
    setPrereqs(status);
  }

  async function doInstall() {
    setInstalling(true);
    setInstallLog("Checking prerequisites...");
    try {
      const status = await invoke<PrereqStatus>("check_prerequisites");
      if (!status.node_installed) {
        setInstallLog("Installing managed Node.js runtime... (this may take a few minutes)");
        await invoke("install_nodejs");
      }
      if (!status.openclaw_installed) {
        setInstallLog("Installing OpenClaw...");
        await invoke("install_openclaw");
      }
      setInstallLog("Done! Verifying...");
      await checkPrereqs();
      setStep(1);
    } catch (e) {
      setInstallLog(`Error: ${e}`);
    } finally {
      setInstalling(false);
    }
  }

  if (step === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-6 p-8">
        <div className="max-w-md w-full space-y-4">
          <h1 className="text-2xl font-bold text-center">Welcome to OpenClaw Shell</h1>
          <p className="text-muted-foreground text-center">
            Let's get everything set up for you. No manual installs needed.
          </p>

          <div className="bg-card border rounded-lg p-4 space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span>Node.js Runtime</span>
              {prereqs?.node_installed ? (
                <span className="text-green-600 flex items-center gap-1">
                  <CheckCircle2 className="h-3.5 w-3.5" /> {prereqs.managed_runtime ? "Managed" : prereqs.node_version}
                </span>
              ) : (
                <span className="text-amber-600 flex items-center gap-1">
                  <AlertCircle className="h-3.5 w-3.5" /> Will install
                </span>
              )}
            </div>
            <div className="flex items-center justify-between text-sm">
              <span>OpenClaw</span>
              {prereqs?.openclaw_installed ? (
                <span className="text-green-600 flex items-center gap-1">
                  <CheckCircle2 className="h-3.5 w-3.5" /> {prereqs.openclaw_version}
                </span>
              ) : (
                <span className="text-amber-600 flex items-center gap-1">
                  <AlertCircle className="h-3.5 w-3.5" /> Will install
                </span>
              )}
            </div>
          </div>

          <button
            onClick={doInstall}
            disabled={installing || (prereqs?.node_installed && prereqs?.openclaw_installed)}
            className={cn(
              "w-full py-2.5 rounded-lg font-medium text-sm flex items-center justify-center gap-2",
              prereqs?.node_installed && prereqs?.openclaw_installed
                ? "bg-green-600 text-white hover:bg-green-700"
                : "bg-primary text-primary-foreground hover:bg-primary/90",
              installing && "opacity-70 cursor-not-allowed"
            )}
          >
            {installing && <Loader2 className="h-4 w-4 animate-spin" />}
            {prereqs?.node_installed && prereqs?.openclaw_installed
              ? "All Set — Continue"
              : installing
              ? "Installing..."
              : "Install Automatically"}
          </button>

          {installLog && (
            <p className="text-xs text-muted-foreground text-center">{installLog}</p>
          )}
        </div>
      </div>
    );
  }

  if (step === 1) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-6 p-8">
        <div className="max-w-md w-full space-y-4">
          <h1 className="text-2xl font-bold text-center">Safety Settings</h1>
          <p className="text-muted-foreground text-center">
            Family mode is on by default. Commands need approval before running.
          </p>
          <div className="bg-card border rounded-lg p-4 space-y-3">
            <div className="flex items-center gap-3">
              <Shield className="h-5 w-5 text-primary" />
              <div className="flex-1">
                <p className="text-sm font-medium">Family Mode</p>
                <p className="text-xs text-muted-foreground">
                  Smart approvals, timeouts, and restricted tools
                </p>
              </div>
              <span className="text-xs bg-green-100 text-green-700 px-2 py-0.5 rounded-full">ON</span>
            </div>
            <div className="flex items-center gap-3">
              <Key className="h-5 w-5 text-primary" />
              <div className="flex-1">
                <p className="text-sm font-medium">Secrets Protection</p>
                <p className="text-xs text-muted-foreground">API keys are redacted in logs</p>
              </div>
              <span className="text-xs bg-green-100 text-green-700 px-2 py-0.5 rounded-full">ON</span>
            </div>
          </div>
          <button
            onClick={() => setStep(2)}
            className="w-full py-2.5 rounded-lg font-medium text-sm bg-primary text-primary-foreground hover:bg-primary/90 flex items-center justify-center gap-2"
          >
            Continue <ChevronRight className="h-4 w-4" />
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center justify-center h-full gap-6 p-8">
      <div className="max-w-md w-full space-y-4">
        <h1 className="text-2xl font-bold text-center">Add an API Key</h1>
        <p className="text-muted-foreground text-center">
          Choose a provider and paste your key. You can add more later.
        </p>
        <ApiKeys compact onDone={onDone} />
      </div>
    </div>
  );
}

export default function App() {
  const [page, setPage] = useState<Page>("dashboard");
  const [setupDone, setSetupDone] = useState(false);

  useEffect(() => {
    const done = localStorage.getItem("openclaw-shell-setup");
    if (done === "true") setSetupDone(true);
  }, []);

  function finishSetup() {
    localStorage.setItem("openclaw-shell-setup", "true");
    setSetupDone(true);
  }

  if (!setupDone) {
    return <SetupWizard onDone={finishSetup} />;
  }

  return (
    <div className="flex h-screen bg-background text-foreground">
      <aside className="w-56 border-r bg-muted/30 flex flex-col">
        <div className="p-4 border-b">
          <h1 className="text-lg font-bold flex items-center gap-2">
            <Terminal className="h-5 w-5 text-primary" />
            OpenClaw Shell
          </h1>
        </div>
        <nav className="flex-1 p-3 space-y-1 overflow-auto">
          <SidebarItem icon={LayoutDashboard} label="Dashboard" active={page === "dashboard"} onClick={() => setPage("dashboard")} />
          <SidebarItem icon={Key} label="API Keys" active={page === "apikeys"} onClick={() => setPage("apikeys")} />
          <SidebarItem icon={MessageSquare} label="Channels" active={page === "channels"} onClick={() => setPage("channels")} />
          <SidebarItem icon={Puzzle} label="Skills" active={page === "skills"} onClick={() => setPage("skills")} />
          <SidebarItem icon={Shield} label="Safety" active={page === "safety"} onClick={() => setPage("safety")} />
          <SidebarItem icon={Bug} label="Debug" active={page === "debug"} onClick={() => setPage("debug")} />
          <SidebarItem icon={Brain} label="Self-Improve" active={page === "selfimprove"} onClick={() => setPage("selfimprove")} />
          <SidebarItem icon={Wrench} label="Settings" active={page === "settings"} onClick={() => setPage("settings")} />
        </nav>
        <div className="p-3 border-t">
          <button
            onClick={() => invoke("launch_tui")}
            className="flex items-center gap-2 w-full px-3 py-2 rounded-lg text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90"
          >
            <Terminal className="h-4 w-4" />
            Open TUI
          </button>
        </div>
      </aside>

      <main className="flex-1 overflow-auto">
        {page === "dashboard" && <Dashboard />}
        {page === "apikeys" && <ApiKeys />}
        {page === "channels" && <Channels />}
        {page === "skills" && <Skills />}
        {page === "safety" && <Safety />}
        {page === "debug" && <DebugPanel />}
        {page === "settings" && <SettingsPanel />}
        {page === "selfimprove" && <SelfImprove />}
      </main>
    </div>
  );
}
