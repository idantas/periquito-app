import { useEffect, useState } from "react";
import {
  getHistoryStats,
  getLevelInfo,
  type HistoryStats,
  type LevelInfo,
} from "../lib/ipc";

export default function StatsView() {
  const [stats, setStats] = useState<HistoryStats | null>(null);
  const [level, setLevel] = useState<LevelInfo | null>(null);

  const loadData = () => {
    getHistoryStats().then(setStats);
    getLevelInfo().then(setLevel);
  };

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 5000);
    return () => clearInterval(interval);
  }, []);

  if (!stats || !level) {
    return (
      <div style={{ textAlign: "center", opacity: 0.3, padding: "20px 0" }}>
        Loading stats...
      </div>
    );
  }

  const accuracy = stats.accuracy ?? 0;
  const accuracyColor =
    accuracy >= 70 ? "#4ade80" : accuracy >= 40 ? "#fbbf24" : "#f87171";

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      {/* Level card */}
      <div
        style={{
          background: "rgba(255,255,255,0.05)",
          borderRadius: 12,
          padding: "12px 14px",
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            marginBottom: 8,
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <span style={{ fontSize: 20 }}>{level.emoji}</span>
            <div>
              <div style={{ fontWeight: 600, fontSize: 13 }}>
                {level.levelName}
              </div>
              <div style={{ fontSize: 10, opacity: 0.5 }}>
                {level.xp} XP
                {level.nextLevelXp != null && (
                  <span> / {level.nextLevelXp} XP</span>
                )}
              </div>
            </div>
          </div>
        </div>

        {/* XP progress bar */}
        {level.nextLevelXp != null && (
          <div
            style={{
              height: 4,
              background: "rgba(255,255,255,0.1)",
              borderRadius: 2,
              overflow: "hidden",
            }}
          >
            <div
              style={{
                height: "100%",
                width: `${(level.xpProgress * 100).toFixed(1)}%`,
                background: "linear-gradient(90deg, #818cf8, #a78bfa)",
                borderRadius: 2,
                transition: "width 0.5s ease",
              }}
            />
          </div>
        )}
      </div>

      {/* Accuracy + counters */}
      <div style={{ display: "flex", gap: 8 }}>
        {/* Accuracy ring */}
        <div
          style={{
            flex: 1,
            background: "rgba(255,255,255,0.05)",
            borderRadius: 12,
            padding: "12px 14px",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
          }}
        >
          <div
            style={{
              fontSize: 28,
              fontWeight: 700,
              color: accuracyColor,
              lineHeight: 1,
            }}
          >
            {accuracy}%
          </div>
          <div style={{ fontSize: 10, opacity: 0.5, marginTop: 4 }}>
            accuracy
          </div>
          {stats.rolling_accuracy != null && (
            <div style={{ fontSize: 10, opacity: 0.4, marginTop: 2 }}>
              last 50: {stats.rolling_accuracy}%
            </div>
          )}
        </div>

        {/* Counters */}
        <div style={{ flex: 1, display: "flex", flexDirection: "column", gap: 8 }}>
          <CounterCard
            label="evaluated"
            value={stats.total_evaluated}
            color="rgba(255,255,255,0.7)"
          />
          <CounterCard
            label="good"
            value={stats.total_good}
            color="#4ade80"
          />
          <CounterCard
            label="corrections"
            value={stats.total_corrections}
            color="#fbbf24"
          />
        </div>
      </div>
    </div>
  );
}

function CounterCard({
  label,
  value,
  color,
}: {
  label: string;
  value: number;
  color: string;
}) {
  return (
    <div
      style={{
        background: "rgba(255,255,255,0.05)",
        borderRadius: 8,
        padding: "6px 10px",
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
      }}
    >
      <span style={{ fontSize: 10, opacity: 0.5 }}>{label}</span>
      <span style={{ fontSize: 14, fontWeight: 600, color }}>{value}</span>
    </div>
  );
}
