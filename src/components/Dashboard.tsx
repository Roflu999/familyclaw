import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Play,
  Square,
  RotateCcw,
  Activity,
  FileText,
  Stethoscope,
  Globe,
  Loader2,
  CheckCircle2,
  AlertTriangle,
  XCircle,
  Zap,
  Lock,
} from "lucide-react";
import { cn } from "../lib/utils";

interface GatewayStatus {
  running: boolean;
  pid?: number;
  port?: number;
  health?: string;
  managed_by_shell: boolean;
}

interface DoctorIssue {
  severity: string;
  message: string;
  fix_action: string;
  auto_fixable: boolean;
}

interface DoctorResult {
  raw_output: string;
  issues: DoctorIssue[];
  healthy: boolean;
}

import { useNotification } from "../hooks/useNotification";

export default function Dashboard() {
  const [status, setStatus] = useState<GatewayStatus | null>(null);
  const [logs, setLogs] = useState("");
  const [loading, setLoading] = useState(false);
  const [doctor, setDoctor] = useState<DoctorResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const { notify } = useNotification();

  async function fetchStatus() {
    try {
      const s = await invoke<GatewayStatus>("gateway_status");
      setStatus(s);
    } catch {
      // Silently ignore polling errors — UI already shows error on user actions
    }
  }

  async function fetchLogs() {
    try {
      const l = await invoke<string>("get_logs", { lines: 100 });
      setLogs(l);
    } catch (e) {
      setLogs(`Failed to load logs: ${e}`);
    }
  }

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 3000);
    return () => clearInterval(interval);
  }, []);

  async function start() {
    setLoading(true);
    setError(null);
    try {
      await invoke("start_gateway");
      await notify("OpenClaw Shell", "Gateway started successfully");
    } catch (e) {
      setError(String(e));
      await notify("OpenClaw Shell", `Gateway failed to start: ${e}`);
    }
    await fetchStatus();
    setLoading(false);
  }

  async function stop() {
    setLoading(true);
    setError(null);
    try {
      await invoke("stop_gateway");
      await notify("OpenClaw Shell", "Gateway stopped");
    } catch (e) {
      setError(String(e));
      await notify("OpenClaw Shell", `Gateway failed to stop: ${e}`);
    }
    await fetchStatus();
    setLoading(false);
  }

  async function restart() {
    setLoading(true);
    setError(null);
    try {
      await invoke("restart_gateway");
      await notify("OpenClaw Shell", "Gateway restarted");
    } catch (e) {
      setError(String(e));
      await notify("OpenClaw Shell", `Gateway failed to restart: ${e}`);
    }
    await fetchStatus();
    setLoading(false);
  }

  async function runDoctor() {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<DoctorResult>("run_doctor");
      setDoctor(result);
    } catch (e) {
      setError(`Doctor failed: ${e}`);
    }
    setLoading(false);
  }

  async function openDashboard() {
    await invoke("launch_dashboard");
  }

  const externallyManaged = status?.running && !status?.managed_by_shell;

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">Dashboard</h2>
        <div className="flex items-center gap-2">
          {status?.running ? (
            <span className="flex items-center gap-1.5 text-sm text-green-600 bg-green-50 px-3 py-1 rounded-full">
              <Activity className="h-3.5 w-3.5" />
              {status.port ? `Port ${status.port}` : "Running"}
            </span>
          ) : (
            <span className="flex items-center gap-1.5 text-sm text-amber-600 bg-amber-50 px-3 py-1 rounded-full">
              <Activity className="h-3.5 w-3.5" /> Stopped
            </span>
          )}
        </div>
      </div>

      {externallyManaged && (
        <div className="p-3 rounded-lg bg-blue-50 text-blue-700 text-sm flex items-center gap-2">
          <Lock className="h-4 w-4" />
          Gateway is running independently (started outside this shell). Stop it via CLI first if you want the shell to manage it.
        </div>
      )}

      {error && (
        <div className="p-3 rounded-lg bg-red-50 text-red-700 text-sm flex items-center gap-2">
          <AlertTriangle className="h-4 w-4" />
          {error}
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <button
          onClick={start}
          disabled={loading || status?.running}
          className={cn(
            "flex items-center justify-center gap-2 px-4 py-3 rounded-lg font-medium text-sm border",
            status?.running
              ? "bg-muted text-muted-foreground cursor-not-allowed"
              : "bg-green-600 text-white hover:bg-green-700"
          )}
        >
          <Play className="h-4 w-4" /> Start Gateway
        </button>
        <button
          onClick={stop}
          disabled={loading || !status?.running || externallyManaged}
          className={cn(
            "flex items-center justify-center gap-2 px-4 py-3 rounded-lg font-medium text-sm border",
            !status?.running || externallyManaged
              ? "bg-muted text-muted-foreground cursor-not-allowed"
              : "bg-red-600 text-white hover:bg-red-700"
          )}
        >
          <Square className="h-4 w-4" /> Stop Gateway
        </button>
        <button
          onClick={restart}
          disabled={loading || externallyManaged}
          className={cn(
            "flex items-center justify-center gap-2 px-4 py-3 rounded-lg font-medium text-sm border",
            externallyManaged
              ? "bg-muted text-muted-foreground cursor-not-allowed"
              : "bg-background hover:bg-muted"
          )}
        >
          <RotateCcw className={cn("h-4 w-4", loading && "animate-spin")} /> Restart
        </button>
        <button
          onClick={openDashboard}
          className="flex items-center justify-center gap-2 px-4 py-3 rounded-lg font-medium text-sm border bg-background hover:bg-muted"
        >
          <Globe className="h-4 w-4" /> Web Dashboard
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div className="border rounded-lg bg-card">
          <div className="flex items-center justify-between px-4 py-3 border-b">
            <h3 className="font-medium text-sm flex items-center gap-2">
              <FileText className="h-4 w-4" /> Gateway Logs
            </h3>
            <button onClick={fetchLogs} className="text-xs text-primary hover:underline">
              Refresh
            </button>
          </div>
          <pre className="p-4 text-xs text-muted-foreground h-64 overflow-auto font-mono whitespace-pre-wrap">
            {logs || "Click Refresh to load logs"}
          </pre>
        </div>

        <div className="border rounded-lg bg-card">
          <div className="flex items-center justify-between px-4 py-3 border-b">
            <h3 className="font-medium text-sm flex items-center gap-2">
              <Stethoscope className="h-4 w-4" /> Doctor
            </h3>
            <button onClick={runDoctor} disabled={loading} className="text-xs text-primary hover:underline">
              {loading ? <Loader2 className="h-3 w-3 animate-spin inline" /> : "Run"}
            </button>
          </div>
          <div className="p-4 space-y-2 h-64 overflow-auto">
            {doctor ? (
              doctor.issues.map((issue, i) => (
                <div
                  key={i}
                  className={cn(
                    "p-3 rounded-lg text-sm",
                    issue.severity === "error" && "bg-red-50 text-red-700",
                    issue.severity === "warning" && "bg-amber-50 text-amber-700",
                    issue.severity === "info" && "bg-blue-50 text-blue-700",
                    issue.severity === "success" && "bg-green-50 text-green-700"
                  )}
                >
                  <div className="flex items-center gap-2 font-medium">
                    {issue.severity === "error" && <XCircle className="h-4 w-4" />}
                    {issue.severity === "warning" && <AlertTriangle className="h-4 w-4" />}
                    {issue.severity === "info" && <Zap className="h-4 w-4" />}
                    {issue.severity === "success" && <CheckCircle2 className="h-4 w-4" />}
                    {issue.message}
                  </div>
                  <p className="text-xs mt-1 opacity-80">{issue.fix_action}</p>
                  {issue.auto_fixable && (
                    <span className="text-xs bg-white/60 px-1.5 py-0.5 rounded mt-1 inline-block">
                      Auto-fixable
                    </span>
                  )}
                </div>
              ))
            ) : (
              <p className="text-xs text-muted-foreground">Click Run to diagnose your setup</p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
