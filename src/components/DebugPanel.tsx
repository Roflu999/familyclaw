import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Bug, FolderOpen, ClipboardCopy, Loader2 } from "lucide-react";
import { cn } from "../lib/utils";

interface DebugInfo {
  node_version?: string;
  npm_version?: string;
  openclaw_version?: string;
  openclaw_path?: string;
  config_path?: string;
  os: string;
  gateway_running: boolean;
}

export default function DebugPanel() {
  const [info, setInfo] = useState<DebugInfo | null>(null);
  const [loading, setLoading] = useState(false);

  async function load() {
    setLoading(true);
    const i = await invoke<DebugInfo>("get_debug_info");
    setInfo(i);
    setLoading(false);
  }

  useEffect(() => {
    load();
  }, []);

  async function copyDebug() {
    if (!info) return;
    const text = JSON.stringify(info, null, 2);
    await navigator.clipboard.writeText(text);
  }

  async function openFolder() {
    await invoke("open_openclaw_folder");
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold flex items-center gap-2">
          <Bug className="h-6 w-6 text-primary" /> Debug
        </h2>
        <div className="flex items-center gap-2">
          <button
            onClick={copyDebug}
            className="flex items-center gap-2 px-3 py-2 rounded-lg text-sm border hover:bg-muted"
          >
            <ClipboardCopy className="h-4 w-4" /> Copy Info
          </button>
          <button
            onClick={openFolder}
            className="flex items-center gap-2 px-3 py-2 rounded-lg text-sm border hover:bg-muted"
          >
            <FolderOpen className="h-4 w-4" /> Open Folder
          </button>
          <button
            onClick={load}
            className="flex items-center gap-2 px-3 py-2 rounded-lg text-sm border hover:bg-muted"
          >
            <Loader2 className={cn("h-4 w-4", loading && "animate-spin")} /> Refresh
          </button>
        </div>
      </div>

      {info && (
        <div className="border rounded-lg bg-card overflow-hidden">
          <table className="w-full text-sm">
            <tbody>
              {[
                ["OS", info.os],
                ["Node.js", info.node_version || "Not found"],
                ["npm", info.npm_version || "Not found"],
                ["OpenClaw", info.openclaw_version || "Not found"],
                ["OpenClaw Path", info.openclaw_path || "Not found"],
                ["Config Path", info.config_path || "Not found"],
                ["Gateway Running", info.gateway_running ? "Yes" : "No"],
              ].map(([label, value]) => (
                <tr key={label} className="border-b last:border-b-0">
                  <td className="px-4 py-3 font-medium text-muted-foreground w-40">{label}</td>
                  <td className="px-4 py-3 font-mono text-xs">{value}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
