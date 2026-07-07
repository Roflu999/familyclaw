import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MessageSquare, ChevronRight, ExternalLink, Save, CheckCircle2, Loader2 } from "lucide-react";
import { cn } from "../lib/utils";

interface GuideStep {
  title: string;
  instruction: string;
  action_type: string;
  url?: string;
  field_label?: string;
  placeholder?: string;
}

interface ChannelGuide {
  id: string;
  name: string;
  description: string;
  difficulty: string;
  steps: GuideStep[];
}

export default function Channels() {
  const [guides, setGuides] = useState<ChannelGuide[]>([]);
  const [active, setActive] = useState<string | null>(null);
  const [stepIndex, setStepIndex] = useState(0);
  const [token, setToken] = useState("");
  const [saved, setSaved] = useState(false);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    invoke<ChannelGuide[]>("get_channel_guides").then(setGuides);
  }, []);

  function selectChannel(id: string) {
    setActive(id);
    setStepIndex(0);
    setToken("");
    setSaved(false);
  }

  async function saveToken() {
    if (!active || !token.trim()) return;
    setLoading(true);
    await invoke("save_channel_config", { channel: active, token: token.trim() });
    setSaved(true);
    setLoading(false);
  }

  const current = guides.find((g) => g.id === active);

  return (
    <div className="p-6 space-y-6 max-w-3xl">
      <h2 className="text-2xl font-bold flex items-center gap-2">
        <MessageSquare className="h-6 w-6 text-primary" /> Channels
      </h2>
      <p className="text-sm text-muted-foreground">
        Connect your AI to messaging platforms. Each guide walks you through the setup step by step.
      </p>

      {!active ? (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {guides.map((g) => (
            <button
              key={g.id}
              onClick={() => selectChannel(g.id)}
              className="p-4 border rounded-lg bg-card text-left hover:border-primary transition-colors"
            >
              <div className="flex items-center justify-between mb-2">
                <h3 className="font-medium">{g.name}</h3>
                <span
                  className={cn(
                    "text-xs px-2 py-0.5 rounded-full",
                    g.difficulty === "easy" && "bg-green-100 text-green-700",
                    g.difficulty === "medium" && "bg-amber-100 text-amber-700",
                    g.difficulty === "hard" && "bg-red-100 text-red-700"
                  )}
                >
                  {g.difficulty}
                </span>
              </div>
              <p className="text-xs text-muted-foreground">{g.description}</p>
              <div className="flex items-center gap-1 text-xs text-primary mt-3">
                Setup <ChevronRight className="h-3 w-3" />
              </div>
            </button>
          ))}
        </div>
      ) : (
        <div className="border rounded-lg bg-card">
          <div className="flex items-center justify-between px-4 py-3 border-b">
            <h3 className="font-medium">{current?.name} Setup</h3>
            <button
              onClick={() => setActive(null)}
              className="text-xs text-muted-foreground hover:text-foreground"
            >
              Back to channels
            </button>
          </div>

          <div className="p-4 space-y-4">
            {current?.steps.map((step, i) => (
              <div
                key={i}
                className={cn(
                  "p-3 rounded-lg border",
                  i === stepIndex ? "border-primary bg-primary/5" : "border-transparent bg-muted/30"
                )}
              >
                <div className="flex items-center gap-2 mb-1">
                  <span
                    className={cn(
                      "w-5 h-5 rounded-full flex items-center justify-center text-xs font-bold",
                      i < stepIndex
                        ? "bg-green-600 text-white"
                        : i === stepIndex
                        ? "bg-primary text-primary-foreground"
                        : "bg-muted text-muted-foreground"
                    )}
                  >
                    {i < stepIndex ? <CheckCircle2 className="h-3 w-3" /> : i + 1}
                  </span>
                  <p className="text-sm font-medium">{step.title}</p>
                </div>
                <p className="text-xs text-muted-foreground ml-7">{step.instruction}</p>

                {i === stepIndex && step.action_type === "open_url" && step.url && (
                  <a
                    href={step.url}
                    target="_blank"
                    rel="noreferrer"
                    className="ml-7 mt-2 inline-flex items-center gap-1 text-xs text-primary hover:underline"
                  >
                    <ExternalLink className="h-3 w-3" /> Open in browser
                  </a>
                )}

                {i === stepIndex && step.action_type === "paste_token" && (
                  <div className="ml-7 mt-2 flex items-center gap-2">
                    <input
                      type="password"
                      value={token}
                      onChange={(e) => setToken(e.target.value)}
                      placeholder={step.placeholder || "Paste token here"}
                      className="flex-1 px-2 py-1 text-sm border rounded"
                    />
                    <button
                      onClick={saveToken}
                      disabled={loading || !token.trim()}
                      className="px-3 py-1.5 text-xs font-medium rounded bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                    >
                      {loading ? <Loader2 className="h-3 w-3 animate-spin" /> : <Save className="h-3 w-3" />}
                    </button>
                  </div>
                )}

                {i === stepIndex && step.action_type === "none" && (
                  <button
                    onClick={() => setStepIndex(i + 1)}
                    className="ml-7 mt-2 text-xs text-primary hover:underline"
                  >
                    Done, next step →
                  </button>
                )}
              </div>
            ))}

            {saved && (
              <div className="p-3 rounded-lg bg-green-50 text-green-700 text-sm flex items-center gap-2">
                <CheckCircle2 className="h-4 w-4" /> Token saved! Restart the gateway to apply changes.
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
