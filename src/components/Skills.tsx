import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Puzzle, AlertTriangle, Shield, Loader2 } from "lucide-react";
import { cn } from "../lib/utils";

interface SkillReview {
  name: string;
  description: string;
  permissions: string[];
  risk_level: string;
  safety_notes: string[];
  enabled: boolean;
}

export default function Skills() {
  const [skills, setSkills] = useState<SkillReview[]>([]);
  const [loading, setLoading] = useState(false);

  async function load() {
    setLoading(true);
    const list = await invoke<SkillReview[]>("list_skills");
    setSkills(list);
    setLoading(false);
  }

  useEffect(() => {
    load();
  }, []);

  async function toggle(name: string, enabled: boolean) {
    setLoading(true);
    await invoke("set_skill_enabled", { name, enabled });
    await load();
    setLoading(false);
  }

  return (
    <div className="p-6 space-y-6 max-w-3xl">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold flex items-center gap-2">
          <Puzzle className="h-6 w-6 text-primary" /> Skills
        </h2>
        <button
          onClick={load}
          className="text-xs text-primary hover:underline"
        >
          {loading ? <Loader2 className="h-3 w-3 animate-spin inline" /> : "Refresh"}
        </button>
      </div>

      <p className="text-sm text-muted-foreground">
        Skills extend what your AI can do. Each shows its permissions before you enable it.
      </p>

      <div className="space-y-3">
        {skills.map((skill) => (
          <div
            key={skill.name}
            className={cn(
              "border rounded-lg p-4 bg-card",
              skill.risk_level === "high" && "border-red-200",
              skill.risk_level === "medium" && "border-amber-200"
            )}
          >
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <div className="flex items-center gap-2 mb-1">
                  <h3 className="font-medium">{skill.name}</h3>
                  {skill.risk_level === "high" && (
                    <span className="text-xs bg-red-100 text-red-700 px-2 py-0.5 rounded-full flex items-center gap-1">
                      <AlertTriangle className="h-3 w-3" /> High Risk
                    </span>
                  )}
                  {skill.risk_level === "medium" && (
                    <span className="text-xs bg-amber-100 text-amber-700 px-2 py-0.5 rounded-full">
                      Medium Risk
                    </span>
                  )}
                  {skill.risk_level === "low" && (
                    <span className="text-xs bg-green-100 text-green-700 px-2 py-0.5 rounded-full flex items-center gap-1">
                      <Shield className="h-3 w-3" /> Low Risk
                    </span>
                  )}
                </div>
                <p className="text-xs text-muted-foreground mb-2">{skill.description}</p>

                <div className="space-y-1">
                  <p className="text-xs font-medium">Permissions:</p>
                  <div className="flex flex-wrap gap-1">
                    {skill.permissions.map((perm) => (
                      <span
                        key={perm}
                        className="text-xs bg-muted px-2 py-0.5 rounded"
                      >
                        {perm}
                      </span>
                    ))}
                  </div>
                </div>

                {skill.safety_notes.length > 0 && (
                  <div className="mt-2 space-y-1">
                    {skill.safety_notes.map((note, i) => (
                      <p key={i} className="text-xs text-amber-700 flex items-center gap-1">
                        <AlertTriangle className="h-3 w-3" /> {note}
                      </p>
                    ))}
                  </div>
                )}
              </div>

              <button
                onClick={() => toggle(skill.name, !skill.enabled)}
                disabled={loading}
                className={cn(
                  "ml-4 w-12 h-6 rounded-full transition-colors relative shrink-0",
                  skill.enabled ? "bg-primary" : "bg-muted"
                )}
              >
                <span
                  className={cn(
                    "absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform",
                    skill.enabled && "translate-x-6"
                  )}
                />
              </button>
            </div>
          </div>
        ))}

        {skills.length === 0 && !loading && (
          <div className="text-center py-12 text-muted-foreground text-sm">
            No skills installed yet. Skills will appear here when you add them to OpenClaw.
          </div>
        )}
      </div>
    </div>
  );
}
