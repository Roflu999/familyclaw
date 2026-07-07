import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Shield, AlertTriangle, CheckCircle2, Loader2 } from "lucide-react";
import { cn } from "../lib/utils";

interface SafetySettings {
  approval_mode: string;
  exec_timeout_seconds: number;
  disable_dangerous_tools: boolean;
  redact_secrets_in_logs: boolean;
  auto_backup_before_changes: boolean;
  family_mode: boolean;
}

export default function Safety() {
  const [settings, setSettings] = useState<SafetySettings | null>(null);
  const [saved, setSaved] = useState(false);
  const [loading, setLoading] = useState(false);

  async function load() {
    const s = await invoke<SafetySettings>("get_safety_settings");
    setSettings(s);
  }

  useEffect(() => {
    load();
  }, []);

  async function save() {
    if (!settings) return;
    setLoading(true);
    await invoke("set_safety_settings", { settings });
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
    setLoading(false);
  }

  if (!settings) {
    return (
      <div className="p-6 flex items-center justify-center h-full">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6 max-w-2xl">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold flex items-center gap-2">
          <Shield className="h-6 w-6 text-primary" /> Safety
        </h2>
        <button
          onClick={save}
          disabled={loading}
          className={cn(
            "px-4 py-2 rounded-lg text-sm font-medium",
            saved
              ? "bg-green-600 text-white"
              : "bg-primary text-primary-foreground hover:bg-primary/90"
          )}
        >
          {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : saved ? "Saved!" : "Save Changes"}
        </button>
      </div>

      <div className="space-y-4">
        {/* Family Mode */}
        <div className="border rounded-lg p-4 bg-card">
          <div className="flex items-center justify-between">
            <div>
              <p className="font-medium">Family Mode</p>
              <p className="text-sm text-muted-foreground">
                Enables all safety features at once. Recommended for shared computers.
              </p>
            </div>
            <button
              onClick={() =>
                setSettings({ ...settings, family_mode: !settings.family_mode })
              }
              className={cn(
                "w-12 h-6 rounded-full transition-colors relative",
                settings.family_mode ? "bg-primary" : "bg-muted"
              )}
            >
              <span
                className={cn(
                  "absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform",
                  settings.family_mode && "translate-x-6"
                )}
              />
            </button>
          </div>
        </div>

        {/* Approval Mode */}
        <div className="border rounded-lg p-4 bg-card">
          <p className="font-medium mb-2">Command Approval</p>
          <div className="grid grid-cols-3 gap-2">
            {["auto", "smart", "manual"].map((mode) => (
              <button
                key={mode}
                onClick={() => setSettings({ ...settings, approval_mode: mode })}
                className={cn(
                  "px-3 py-2 rounded text-sm font-medium border capitalize",
                  settings.approval_mode === mode
                    ? "bg-primary text-primary-foreground border-primary"
                    : "bg-background hover:bg-muted"
                )}
              >
                {mode}
              </button>
            ))}
          </div>
          <p className="text-xs text-muted-foreground mt-2">
            {settings.approval_mode === "auto" && "All commands run automatically. Least safe."}
            {settings.approval_mode === "smart" && "Potentially dangerous commands ask for approval."}
            {settings.approval_mode === "manual" && "Every command requires explicit approval. Safest."}
          </p>
        </div>

        {/* Timeout */}
        <div className="border rounded-lg p-4 bg-card">
          <div className="flex items-center justify-between mb-2">
            <p className="font-medium">Command Timeout</p>
            <span className="text-sm font-mono bg-muted px-2 py-0.5 rounded">
              {settings.exec_timeout_seconds}s
            </span>
          </div>
          <input
            type="range"
            min={10}
            max={300}
            step={10}
            value={settings.exec_timeout_seconds}
            onChange={(e) =>
              setSettings({
                ...settings,
                exec_timeout_seconds: Number(e.target.value),
              })
            }
            className="w-full"
          />
          <p className="text-xs text-muted-foreground mt-1">
            Commands that run longer than this are automatically killed.
          </p>
        </div>

        {/* Toggles */}
        <div className="border rounded-lg p-4 bg-card space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <AlertTriangle className="h-4 w-4 text-amber-500" />
              <span className="text-sm">Disable risky tools by default</span>
            </div>
            <button
              onClick={() =>
                setSettings({
                  ...settings,
                  disable_dangerous_tools: !settings.disable_dangerous_tools,
                })
              }
              className={cn(
                "w-10 h-5 rounded-full transition-colors relative",
                settings.disable_dangerous_tools ? "bg-primary" : "bg-muted"
              )}
            >
              <span
                className={cn(
                  "absolute top-0.5 left-0.5 w-4 h-4 bg-white rounded-full transition-transform",
                  settings.disable_dangerous_tools && "translate-x-5"
                )}
              />
            </button>
          </div>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <CheckCircle2 className="h-4 w-4 text-green-500" />
              <span className="text-sm">Auto-backup before config changes</span>
            </div>
            <button
              onClick={() =>
                setSettings({
                  ...settings,
                  auto_backup_before_changes: !settings.auto_backup_before_changes,
                })
              }
              className={cn(
                "w-10 h-5 rounded-full transition-colors relative",
                settings.auto_backup_before_changes ? "bg-primary" : "bg-muted"
              )}
            >
              <span
                className={cn(
                  "absolute top-0.5 left-0.5 w-4 h-4 bg-white rounded-full transition-transform",
                  settings.auto_backup_before_changes && "translate-x-5"
                )}
              />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
