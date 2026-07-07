import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Key, Trash2, Save, AlertTriangle, CheckCircle2 } from "lucide-react";
import { cn } from "../lib/utils";

interface ApiKeyEntry {
  provider: string;
  name: string;
  has_key: boolean;
  key_preview?: string;
  valid_format: boolean;
}

const PROVIDER_HELP: Record<string, string> = {
  openai: "https://platform.openai.com/api-keys",
  anthropic: "https://console.anthropic.com/settings/keys",
  openrouter: "https://openrouter.ai/keys",
  deepseek: "https://platform.deepseek.com/api_keys",
  gemini: "https://aistudio.google.com/app/apikey",
  moonshot: "https://platform.moonshot.cn/console/api-keys",
  minimax: "https://www.minimaxi.com/platform",
  elevenlabs: "https://elevenlabs.io/app/settings/api-keys",
};

export default function ApiKeys({ compact, onDone }: { compact?: boolean; onDone?: () => void }) {
  const [keys, setKeys] = useState<ApiKeyEntry[]>([]);
  const [editing, setEditing] = useState<string | null>(null);
  const [value, setValue] = useState("");
  const [loading, setLoading] = useState(false);
  const [validationError, setValidationError] = useState("");

  async function load() {
    const list = await invoke<ApiKeyEntry[]>("list_api_keys");
    setKeys(list);
  }

  useEffect(() => {
    load();
  }, []);

  async function save(provider: string) {
    if (!value.trim()) return;
    setValidationError("");
    setLoading(true);
    const valid = await invoke<boolean>("set_api_key", { provider, key: value.trim() });
    if (!valid) {
      setValidationError("Key format looks invalid. Please check and try again.");
      setLoading(false);
      return;
    }
    setEditing(null);
    setValue("");
    await load();
    setLoading(false);
  }

  async function remove(provider: string) {
    setLoading(true);
    await invoke("delete_api_key", { provider });
    await load();
    setLoading(false);
  }

  return (
    <div className={cn("space-y-4", !compact && "p-6")}>
      {!compact && <h2 className="text-2xl font-bold">API Keys</h2>}
      <p className="text-sm text-muted-foreground">
        Add keys for the LLM providers you want to use. We validate the format before saving.
      </p>

      {validationError && (
        <div className="p-3 rounded-lg bg-red-50 text-red-700 text-sm flex items-center gap-2">
          <AlertTriangle className="h-4 w-4" /> {validationError}
        </div>
      )}

      <div className="space-y-2">
        {keys.map((k) => (
          <div key={k.provider} className="flex items-center gap-3 p-3 border rounded-lg bg-card">
            <Key className="h-4 w-4 text-muted-foreground" />
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2">
                <p className="text-sm font-medium">{k.name}</p>
                {k.has_key && k.valid_format && (
                  <CheckCircle2 className="h-3.5 w-3.5 text-green-500" />
                )}
                {k.has_key && !k.valid_format && (
                  <AlertTriangle className="h-3.5 w-3.5 text-amber-500" />
                )}
              </div>
              {k.has_key && !editing && (
                <p className="text-xs text-muted-foreground">{k.key_preview}</p>
              )}
            </div>

            {editing === k.provider ? (
              <div className="flex items-center gap-2">
                <input
                  type="password"
                  value={value}
                  onChange={(e) => setValue(e.target.value)}
                  placeholder="Paste API key"
                  className="px-2 py-1 text-sm border rounded w-48"
                />
                <button
                  onClick={() => save(k.provider)}
                  disabled={loading}
                  className="p-1.5 rounded bg-primary text-primary-foreground hover:bg-primary/90"
                >
                  <Save className="h-3.5 w-3.5" />
                </button>
                <button
                  onClick={() => { setEditing(null); setValue(""); setValidationError(""); }}
                  className="p-1.5 rounded border hover:bg-muted"
                >
                  <Trash2 className="h-3.5 w-3.5" />
                </button>
              </div>
            ) : (
              <div className="flex items-center gap-2">
                {k.has_key && (
                  <button
                    onClick={() => remove(k.provider)}
                    className="p-1.5 rounded border hover:bg-red-50 text-red-600"
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </button>
                )}
                <button
                  onClick={() => setEditing(k.provider)}
                  className="px-3 py-1.5 text-xs font-medium rounded border hover:bg-muted"
                >
                  {k.has_key ? "Update" : "Add"}
                </button>
                <a
                  href={PROVIDER_HELP[k.provider] || "#"}
                  target="_blank"
                  rel="noreferrer"
                  className="text-xs text-primary hover:underline"
                >
                  Get key →
                </a>
              </div>
            )}
          </div>
        ))}
      </div>

      {compact && onDone && (
        <button
          onClick={onDone}
          className="w-full py-2.5 rounded-lg font-medium text-sm bg-primary text-primary-foreground hover:bg-primary/90"
        >
          Finish Setup
        </button>
      )}
    </div>
  );
}
