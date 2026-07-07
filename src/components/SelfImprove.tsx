import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Brain,
  Lightbulb,
  AlertTriangle,
  CheckCircle2,
  XCircle,
  Loader2,
  RefreshCw,
  TrendingUp,
  BookOpen,
  Zap,
} from "lucide-react";
import { cn } from "../lib/utils";

interface Lesson {
  id: string;
  timestamp: string;
  category: string;
  trigger: string;
  observation: string;
  correction: string;
  source: string;
  applied_count: number;
  success_count: number;
  auto_solved: boolean;
}

interface Insight {
  kind: string;
  message: string;
  confidence: number;
  action?: string;
}

interface UserPreference {
  key: string;
  value: string;
  confidence: number;
  category: string;
}

export default function SelfImprove() {
  const [lessons, setLessons] = useState<Lesson[]>([]);
  const [insights, setInsights] = useState<Insight[]>([]);
  const [prefs, setPrefs] = useState<UserPreference[]>([]);
  const [loading, setLoading] = useState(false);
  const [syncing, setSyncing] = useState(false);
  const [syncResult, setSyncResult] = useState("");
  const [activeTab, setActiveTab] = useState<"insights" | "lessons" | "prefs">("insights");

  async function load() {
    setLoading(true);
    const [l, i, p] = await Promise.all([
      invoke<Lesson[]>("list_self_improve_lessons", { category: null, limit: 50 }),
      invoke<Insight[]>("get_self_improve_insights"),
      invoke<UserPreference[]>("get_user_preferences", { category: null }),
    ]);
    setLessons(l);
    setInsights(i);
    setPrefs(p);
    setLoading(false);
  }

  useEffect(() => {
    load();
  }, []);

  async function syncOpenClaw() {
    setSyncing(true);
    try {
      const count = await invoke<number>("sync_openclaw_learnings");
      setSyncResult(`Imported ${count} learnings from OpenClaw`);
      await load();
    } catch (e) {
      setSyncResult(`Sync failed: ${e}`);
    } finally {
      setSyncing(false);
    }
  }

  async function markSuccess(id: string) {
    await invoke("mark_lesson_success", { lessonId: id });
    await load();
  }

  async function markFailure(id: string) {
    await invoke("mark_lesson_failure", { lessonId: id });
    await load();
  }

  const categoryColors: Record<string, string> = {
    install: "bg-blue-100 text-blue-700",
    gateway: "bg-purple-100 text-purple-700",
    config: "bg-amber-100 text-amber-700",
    api: "bg-green-100 text-green-700",
    safety: "bg-red-100 text-red-700",
    network: "bg-cyan-100 text-cyan-700",
    system: "bg-gray-100 text-gray-700",
    general: "bg-muted text-muted-foreground",
    openclaw: "bg-indigo-100 text-indigo-700",
  };

  return (
    <div className="p-6 space-y-6 max-w-3xl">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold flex items-center gap-2">
          <Brain className="h-6 w-6 text-primary" /> Self-Improvement
        </h2>
        <div className="flex items-center gap-2">
          <button
            onClick={syncOpenClaw}
            disabled={syncing}
            className="flex items-center gap-2 px-3 py-2 rounded-lg text-sm border hover:bg-muted"
          >
            {syncing ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <RefreshCw className="h-3.5 w-3.5" />}
            Sync OpenClaw
          </button>
          <button
            onClick={load}
            className="flex items-center gap-2 px-3 py-2 rounded-lg text-sm border hover:bg-muted"
          >
            <RefreshCw className={cn("h-3.5 w-3.5", loading && "animate-spin")} />
            Refresh
          </button>
        </div>
      </div>

      {syncResult && (
        <div className="p-2 rounded-lg bg-muted text-sm text-center">{syncResult}</div>
      )}

      <div className="flex gap-2 border-b">
        {[
          { id: "insights" as const, label: "Insights", icon: Lightbulb },
          { id: "lessons" as const, label: "Lessons", icon: BookOpen },
          { id: "prefs" as const, label: "Preferences", icon: TrendingUp },
        ].map((t) => (
          <button
            key={t.id}
            onClick={() => setActiveTab(t.id)}
            className={cn(
              "flex items-center gap-1.5 px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors",
              activeTab === t.id
                ? "border-primary text-primary"
                : "border-transparent text-muted-foreground hover:text-foreground"
            )}
          >
            <t.icon className="h-3.5 w-3.5" />
            {t.label}
          </button>
        ))}
      </div>

      {activeTab === "insights" && (
        <div className="space-y-3">
          {insights.length === 0 && !loading && (
            <div className="text-center py-12 text-muted-foreground text-sm">
              No insights yet. Use the shell more — it learns from every action.
            </div>
          )}
          {insights.map((insight, i) => (
            <div
              key={i}
              className={cn(
                "border rounded-lg p-4 bg-card",
                insight.kind === "recurring_error" && "border-red-200",
                insight.kind === "stability" && "border-amber-200",
                insight.kind === "preference" && "border-green-200"
              )}
            >
              <div className="flex items-start gap-3">
                {insight.kind === "recurring_error" && <AlertTriangle className="h-5 w-5 text-red-500 mt-0.5" />}
                {insight.kind === "stability" && <Zap className="h-5 w-5 text-amber-500 mt-0.5" />}
                {insight.kind === "preference" && <TrendingUp className="h-5 w-5 text-green-500 mt-0.5" />}
                {insight.kind === "api_health" && <AlertTriangle className="h-5 w-5 text-amber-500 mt-0.5" />}
                {insight.kind === "safety" && <AlertTriangle className="h-5 w-5 text-blue-500 mt-0.5" />}
                <div className="flex-1">
                  <p className="text-sm font-medium">{insight.message}</p>
                  {insight.action && (
                    <p className="text-xs text-muted-foreground mt-1">Suggested: {insight.action}</p>
                  )}
                  <div className="mt-2 flex items-center gap-2">
                    <div className="h-1.5 w-24 bg-muted rounded-full overflow-hidden">
                      <div
                        className="h-full bg-primary rounded-full"
                        style={{ width: `${insight.confidence * 100}%` }}
                      />
                    </div>
                    <span className="text-xs text-muted-foreground">
                      {Math.round(insight.confidence * 100)}% confidence
                    </span>
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {activeTab === "lessons" && (
        <div className="space-y-3">
          {lessons.length === 0 && !loading && (
            <div className="text-center py-12 text-muted-foreground text-sm">
              No lessons captured yet. They appear automatically when things go wrong.
            </div>
          )}
          {lessons.map((lesson) => (
            <div
              key={lesson.id}
              className={cn(
                "border rounded-lg p-4 bg-card",
                lesson.auto_solved && "border-green-200"
              )}
            >
              <div className="flex items-center gap-2 mb-2">
                <span
                  className={cn(
                    "text-xs px-2 py-0.5 rounded-full font-medium",
                    categoryColors[lesson.category] || categoryColors.general
                  )}
                >
                  {lesson.category}
                </span>
                <span className="text-xs text-muted-foreground">
                  {lesson.timestamp.slice(0, 16).replace("T", " ")}
                </span>
                {lesson.auto_solved && (
                  <span className="text-xs bg-green-100 text-green-700 px-2 py-0.5 rounded-full flex items-center gap-1">
                    <CheckCircle2 className="h-3 w-3" /> Auto-solved
                  </span>
                )}
              </div>
              <p className="text-sm font-medium">{lesson.trigger}</p>
              <p className="text-xs text-muted-foreground mt-1">{lesson.observation}</p>
              <p className="text-xs text-green-700 mt-1">Fix: {lesson.correction}</p>
              <div className="flex items-center gap-2 mt-3">
                <span className="text-xs text-muted-foreground">
                  Success rate: {lesson.applied_count > 0 ? Math.round((lesson.success_count / lesson.applied_count) * 100) : 0}%
                </span>
                <div className="flex-1" />
                <button
                  onClick={() => markSuccess(lesson.id)}
                  className="p-1 rounded hover:bg-green-50 text-green-600"
                  title="This fix worked"
                >
                  <CheckCircle2 className="h-4 w-4" />
                </button>
                <button
                  onClick={() => markFailure(lesson.id)}
                  className="p-1 rounded hover:bg-red-50 text-red-600"
                  title="This fix did not work"
                >
                  <XCircle className="h-4 w-4" />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {activeTab === "prefs" && (
        <div className="space-y-3">
          {prefs.length === 0 && !loading && (
            <div className="text-center py-12 text-muted-foreground text-sm">
              No preferences learned yet. The shell picks up on your habits over time.
            </div>
          )}
          {prefs.map((pref) => (
            <div key={pref.key} className="border rounded-lg p-4 bg-card flex items-center gap-3">
              <TrendingUp className="h-4 w-4 text-muted-foreground" />
              <div className="flex-1">
                <p className="text-sm font-medium">{pref.key}</p>
                <p className="text-xs text-muted-foreground">{pref.value}</p>
              </div>
              <div className="flex items-center gap-2">
                <div className="h-1.5 w-16 bg-muted rounded-full overflow-hidden">
                  <div
                    className="h-full bg-primary rounded-full"
                    style={{ width: `${pref.confidence * 100}%` }}
                  />
                </div>
                <span className="text-xs text-muted-foreground w-10 text-right">
                  {Math.round(pref.confidence * 100)}%
                </span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
