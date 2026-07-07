import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Settings, Download, Upload, RotateCcw, Loader2, AlertCircle, ShieldAlert } from "lucide-react";

export default function SettingsPanel() {
  const [backingUp, setBackingUp] = useState(false);
  const [backupPath, setBackupPath] = useState("");
  const [restoring, setRestoring] = useState(false);
  const [restorePath, setRestorePath] = useState("");
  const [message, setMessage] = useState("");

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
