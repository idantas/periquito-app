import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import NotchLayout from "./components/NotchLayout";
import ChatTip from "./components/ChatTip";
import StatsView from "./components/StatsView";
import SettingsView from "./components/SettingsView";
import QuizView from "./components/QuizView";

interface PeriquitoState {
  task: string;
  emotion: string;
}

interface StatePayload {
  unified_state: PeriquitoState;
  effective_session_id: string | null;
  active_session_count: number;
  is_any_analyzing: boolean;
}

interface EnglishTip {
  id: string;
  timestamp: string;
  prompt: string;
  tip_type: string;
  tip: string | null;
  category: string | null;
}

interface TipsPayload {
  all_tips: EnglishTip[];
}

type Tab = "tips" | "stats" | "quiz" | "settings";

const DEFAULT_STATE: PeriquitoState = { task: "idle", emotion: "neutral" };

function App() {
  const [state, setState] = useState<PeriquitoState>(DEFAULT_STATE);
  const [tips, setTips] = useState<EnglishTip[]>([]);
  const [sessionCount, setSessionCount] = useState(0);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [hooksInstalled, setHooksInstalled] = useState<boolean | null>(null);
  const [activeTab, setActiveTab] = useState<Tab>("tips");

  useEffect(() => {
    invoke<boolean>("is_hooks_installed").then(setHooksInstalled);

    const unlistenState = listen<StatePayload>("state-update", (event) => {
      setState(event.payload.unified_state);
      setSessionCount(event.payload.active_session_count);
      setIsAnalyzing(event.payload.is_any_analyzing);
    });

    const unlistenTips = listen<TipsPayload>("tips-update", (event) => {
      setTips(event.payload.all_tips);
    });

    return () => {
      unlistenState.then((f) => f());
      unlistenTips.then((f) => f());
    };
  }, []);

  const handleInstallHooks = async () => {
    try {
      await invoke("install_hooks");
      setHooksInstalled(true);
    } catch (e) {
      console.error("Failed to install hooks:", e);
    }
  };

  return (
    <NotchLayout state={state}>
      <div style={{ color: "#e0e0e0", fontFamily: "SF Mono, monospace", fontSize: 12 }}>
        {/* Hooks status */}
        {hooksInstalled === false && (
          <div style={{ marginBottom: 12, textAlign: "center" }}>
            <button
              onClick={handleInstallHooks}
              style={{
                background: "#3b82f6",
                color: "white",
                border: "none",
                borderRadius: 8,
                padding: "8px 20px",
                cursor: "pointer",
                fontSize: 13,
                fontWeight: 600,
              }}
            >
              Install Hooks
            </button>
          </div>
        )}

        {/* Tab bar */}
        <div
          style={{
            display: "flex",
            gap: 0,
            borderBottom: "1px solid rgba(255,255,255,0.1)",
            marginBottom: 10,
          }}
        >
          <TabButton
            label="Tips"
            active={activeTab === "tips"}
            onClick={() => setActiveTab("tips")}
          />
          <TabButton
            label="Stats"
            active={activeTab === "stats"}
            onClick={() => setActiveTab("stats")}
          />
          <TabButton
            label="Quiz"
            active={activeTab === "quiz"}
            onClick={() => setActiveTab("quiz")}
          />
          <TabButton
            label="Settings"
            active={activeTab === "settings"}
            onClick={() => setActiveTab("settings")}
          />
          <div style={{ flex: 1 }} />
          <span
            style={{
              fontSize: 10,
              opacity: 0.4,
              alignSelf: "center",
              paddingRight: 4,
            }}
          >
            {sessionCount > 0
              ? `${sessionCount} session${sessionCount > 1 ? "s" : ""}`
              : "idle"}
          </span>
        </div>

        {/* Analyzing indicator */}
        {isAnalyzing && (
          <div style={{ color: "#facc15", marginBottom: 8, fontSize: 11 }}>
            Analyzing English...
          </div>
        )}

        {/* Tab content */}
        {activeTab === "tips" ? (
          tips.length > 0 ? (
            <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
              {tips.slice(-6).map((tip) => (
                <ChatTip key={tip.id} tip={tip} />
              ))}
            </div>
          ) : (
            <div style={{ textAlign: "center", opacity: 0.3, padding: "20px 0" }}>
              {sessionCount > 0
                ? "Write something in Claude Code..."
                : "Waiting for Claude Code sessions..."}
            </div>
          )
        ) : activeTab === "stats" ? (
          <StatsView />
        ) : activeTab === "quiz" ? (
          <QuizView />
        ) : (
          <SettingsView />
        )}
      </div>
    </NotchLayout>
  );
}

function TabButton({
  label,
  active,
  onClick,
}: {
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      style={{
        background: "none",
        border: "none",
        borderBottom: active ? "2px solid #a78bfa" : "2px solid transparent",
        color: active ? "#e0e0e0" : "rgba(255,255,255,0.4)",
        fontFamily: "SF Mono, monospace",
        fontSize: 12,
        fontWeight: active ? 600 : 400,
        padding: "6px 14px",
        cursor: "pointer",
        transition: "all 0.2s",
      }}
    >
      {label}
    </button>
  );
}

export default App;
