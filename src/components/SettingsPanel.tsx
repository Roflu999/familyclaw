import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
import { check } from "@tauri-apps/plugin-updater";
import {
  Settings,
  Download,
  Upload,
  RotateCcw,
  Loader2,
  AlertCircle,
  ShieldAlert,
  Sun,
  Moon,
  Monitor,
  Power,
  RefreshCw,
  CheckCircle2,
} from "lucide-react";
import { useTheme } from "../hooks/useTheme";
import { cn } from "../lib/utils";

export default function SettingsPanel() {
  const { theme, setTheme, resolved } = useTheme();
  const [backingUp, setBackingUp] = useState(false);
  const [backupPath, setBackupPath] = useState("");
  const [restoring, setRestoring] = useState(false);
  const [restorePath, setRestorePath] = useState("");
  const [message, setMessage] = useState("");

  // Auto-start
  const [autoStart, setAutoStart] = useState(false);
  const [autoStartLoading, setAutoStartLoading] = useState(false);

  // Updater
  const [checkingUpdate, setCheckingUpdate] = useState(false);
  const [updateResult, setUpdateResult] = useState("");

  useEffect(() => {
    isEnabled().then(setAutoStart).catch(() => {});
  }, []);

  async function toggleAutoStart() {
    setAutoStartLoading(true);
    try {
      if (autoStart) {
        await disable();
        setAutoStart(false);
        setMessage("Auto-start disabled");
      } else {
        await enable();
        setAutoStart(true);
        setMessage("Auto-start enabled — app will launch on login");
      }
    } catch (e) {
      setMessage(`Auto-start failed: ${e}`);
    } finally {
      setAutoStartLoading(false);
    }
  }

  async function checkForUpdates() {
    setCheckingUpdate(true);
    setUpdateResult("");
    try {
      const update = await check();
      if (update) {
        setUpdateResult(`Update available: ${update.version}. Installing...`);
        await update.downloadAndInstall();
        setUpdateResult("Update installed. Restarting...");
        await invoke("tauri", { __tauriModule: "Process", message: { cmd: "relaunch" } });
      } else {
        setUpdateResult("You are on the latest version.");
      }
    } catch (e: any) {
      if (String(e).includes("unsupported")) {
        setUpdateResult("Auto-updater requires a signed release. Check GitHub manually for now.");
      } else {
        setUpdateResult(`Update check failed: ${e}`);
      }
    } finally {
      setCheckingUpdate(false);
    }
  }

  async function createBackup() {
    setBackingUp(true);
    setMessage("");
    try {
      const path = await invoke<string>("create_backup");
      setBackupPath(path);
      setMessage(`Backup saved to: ${path}`);
    } catch (e) {
      setMessage(`Backup failed: ${e}`);
    } finally {
      setBackingUp(false);
    }
  }

  async function pickRestoreFile() {
    setMessage("");
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "Zip", extensions: ["zip"] }],
    });
    if (selected && typeof selected === "string") {
      setRestorePath(selected);
    }
  }

  async function restoreBackup() {
    if (!restorePath.trim()) return;
    setRestoring(true);
    setMessage("");
    try {
      await invoke("restore_backup", { path: restorePath.trim() });
      setMessage("Restore complete! Restart the app to apply changes.");
    } catch (e) {
      setMessage(`Restore failed: ${e}`);
    } finally {
      setRestoring(false);
    }
  }

  function resetSetup() {
    localStorage.removeItem("openclaw-shell-setup");
    setMessage("Setup flag cleared. Restart the app to run setup again.");
  }

  return (
    <div className="p-6 space-y-6 max-w-2xl">
      <h2 className="text-2xl font-bold flex items-center gap-2">
        <Settings className="h-6 w-6 text-primary" /> Settings
      </h2>

      {/* Appearance */}
      <div className="border rounded-lg p-4 bg-card space-y-3">
        <h3 className="font-medium flex items-center gap-2">
          <Sun className="h-4 w-4" /> Appearance
        </h3>
        <p className="text-sm text-muted-foreground">
          Choose how OpenClaw Shell looks. Auto matches your Windows theme.
        </p>
        <div className="flex gap-2">
          {[
            { id: "light" as const, label: "Light", icon: Sun },
            { id: "dark" as const, label: "Dark", icon: Moon },
            { id: "system" as const, label: "Auto", icon: Monitor },
          ].map((t) => (
            <button
              key={t.id}
              onClick={() => setTheme(t.id)}
              className={cn(
                "flex items-center gap-2 px-3 py-2 rounded-lg text-sm border capitalize transition-colors",
                theme === t.id
                  ? "bg-primary text-primary-foreground border-primary"
                  : "bg-background hover:bg-muted"
              )}
            >
              <t.icon className="h-4 w-4" />
              {t.label}
            </button>
          ))}
        </div>
        <p className="text-xs text-muted-foreground">
          Currently active: {resolved} mode
        </p>
      </div>

      {/* Auto-start */}
      <div className="border rounded-lg p-4 bg-card space-y-3">
        <h3 className="font-medium flex items-center gap-2">
          <Power className="h-4 w-4" /> Startup
        </h3>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm">Start on boot</p>
            <p className="text-xs text-muted-foreground">
              Launch OpenClaw Shell automatically when you log in
            </p>
          </div>
          <button
            onClick={toggleAutoStart}
            disabled={autoStartLoading}
            className={cn(
              "w-12 h-6 rounded-full transition-colors relative",
              autoStart ? "bg-primary" : "bg-muted"
            )}
          >
            <span
              className={cn(
                "absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform",
                autoStart && "translate-x-6"
              )}
            />
          </button>
        </div>
      </div>

      {/* Updater */}
      <div className="border rounded-lg p-4 bg-card space-y-3">
        <h3 className="font-medium flex items-center gap-2">
          <RefreshCw className="h-4 w-4" /> Updates
        </h3>
        <div className="flex items-center gap-2">
          <button
            onClick={checkForUpdates}
            disabled={checkingUpdate}
            className="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90"
          >
            {checkingUpdate ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <RefreshCw className="h-4 w-4" />
            )}
            Check for Updates
          </button>
          {updateResult && (
            <span className="text-sm text-muted-foreground flex items-center gap-1">
              {updateResult.includes("latest") ? (
                <CheckCircle2 className="h-4 w-4 text-green-500" />
              ) : null}
              {updateResult}
            </span>
          )}
        </div>
      </div>

      {/* Backup */}
      <div className="border rounded-lg p-4 bg-card space-y-3">
        <h3 className="font-medium">Backup & Restore</h3>
        <p className="text-sm text-muted-foreground">
          Create a zip backup of your entire OpenClaw configuration.
        </p>

        <div className="p-3 rounded-lg bg-amber-50 text-amber-800 text-sm flex items-start gap-2">
          <ShieldAlert className="h-4 w-4 mt-0.5 shrink-0" />
          <div>
            <p className="font-medium">Backup contains secrets</p>
            <p className="opacity-80">
              The zip includes your API keys and tokens in plaintext. Store it securely and do not share it.
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={createBackup}
            disabled={backingUp}
            className="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90"
          >
            {backingUp ? <Loader2 className="h-4 w-4 animate-spin" /> : <Download className="h-4 w-4" />}
            Create Backup
          </button>
        </div>
        {backupPath && (
          <p className="text-xs text-green-600 bg-green-50 p-2 rounded">{backupPath}</p>
        )}

        <div className="border-t pt-3 mt-3">
          <p className="text-sm text-muted-foreground mb-2">
            Restore from a backup zip file:
          </p>
          <div className="flex items-center gap-2">
            <button
              onClick={pickRestoreFile}
              className="flex items-center gap-2 px-3 py-2 rounded-lg text-sm border hover:bg-muted"
            >
              <Upload className="h-4 w-4" /> Choose File
            </button>
            {restorePath && (
              <span className="text-xs text-muted-foreground truncate max-w-[200px]">
                {restorePath}
              </span>
            )}
          </div>
          <div className="flex items-center gap-2 mt-2">
            <button
              onClick={restoreBackup}
              disabled={restoring || !restorePath.trim()}
              className="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium border hover:bg-muted disabled:opacity-50"
            >
              {restoring ? <Loader2 className="h-4 w-4 animate-spin" /> : <RotateCcw className="h-4 w-4" />}
              Restore
            </button>
          </div>
        </div>
      </div>

      {/* Reset */}
      <div className="border rounded-lg p-4 bg-card space-y-3">
        <h3 className="font-medium flex items-center gap-2">
          <AlertCircle className="h-4 w-4 text-amber-500" /> Reset
        </h3>
        <p className="text-sm text-muted-foreground">
          Clear the setup flag to re-run the first-time wizard on next launch.
        </p>
        <button
          onClick={resetSetup}
          className="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium border hover:bg-red-50 text-red-600"
        >
          <RotateCcw className="h-4 w-4" /> Reset Setup Wizard
        </button>
      </div>

      {message && (
        <div className="text-sm p-3 rounded-lg bg-muted">{message}</div>
      )}
    </div>
  );
}
