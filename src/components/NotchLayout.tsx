import { useEffect, useState, useRef, type ReactNode } from "react";
import GrassIsland from "./GrassIsland";
import SessionSprite from "./SessionSprite";
import { getNotchGeometry, getHistoryStats, type NotchGeometry } from "../lib/ipc";

interface PeriquitoState {
  task: string;
  emotion: string;
}

interface NotchLayoutProps {
  state: PeriquitoState;
  children?: ReactNode;
}

const PANEL_WIDTH = 450;
const PANEL_HEIGHT = 450;

export default function NotchLayout({ state, children }: NotchLayoutProps) {
  const [geometry, setGeometry] = useState<NotchGeometry | null>(null);
  const [isExpanded, setIsExpanded] = useState(false);
  const [isPinned, setIsPinned] = useState(false);
  const [streak, setStreak] = useState(0);
  const panelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    getNotchGeometry().then(setGeometry);
    getHistoryStats().then((s) => setStreak(s.currentStreak));
    const interval = setInterval(() => {
      getHistoryStats().then((s) => setStreak(s.currentStreak));
    }, 10000);
    return () => clearInterval(interval);
  }, []);

  // Click outside to collapse
  useEffect(() => {
    if (!isExpanded || isPinned) return;
    const handler = (e: MouseEvent) => {
      if (panelRef.current && !panelRef.current.contains(e.target as Node)) {
        setIsExpanded(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [isExpanded, isPinned]);

  // Escape to collapse
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isExpanded) {
        setIsExpanded(false);
      }
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [isExpanded]);

  if (!geometry) return null;

  const notchW = geometry.notch_width;
  const notchH = geometry.notch_height;
  const sideWidth = Math.max(0, notchH - 12) + 24;
  const totalNotchWidth = notchW + sideWidth;

  // Corner radii (matching Swift NotchContentView)
  const topR = isExpanded ? 19 : 6;
  const bottomR = isExpanded ? 24 : 14;

  const grassHeight = (PANEL_HEIGHT - notchH - 24) * 0.3 + notchH;
  const expandedContentH = PANEL_HEIGHT - notchH - 24;

  // Notch shape clip-path using SVG
  const panelW = isExpanded ? PANEL_WIDTH : totalNotchWidth;
  const panelH = isExpanded ? PANEL_HEIGHT : notchH + 6;

  return (
    <div
      style={{
        width: "100%",
        display: "flex",
        justifyContent: "center",
        paddingTop: 0,
      }}
    >
      <div
        ref={panelRef}
        onClick={() => {
          if (!isExpanded) setIsExpanded(true);
        }}
        style={{
          width: panelW,
          height: panelH,
          background: "black",
          borderRadius: `${topR}px ${topR}px ${bottomR}px ${bottomR}px`,
          overflow: "hidden",
          cursor: isExpanded ? "default" : "pointer",
          transition: "all 0.4s cubic-bezier(0.34, 1.56, 0.64, 1)",
          position: "relative",
          boxShadow: isExpanded ? "0 6px 20px rgba(0,0,0,0.7)" : "none",
        }}
      >
        {/* Header row - visible when collapsed */}
        <div
          style={{
            height: notchH,
            display: "flex",
            alignItems: "center",
            justifyContent: "flex-end",
            paddingRight: 8,
            opacity: isExpanded ? 0 : 1,
            transition: "opacity 0.2s",
            pointerEvents: isExpanded ? "none" : "auto",
          }}
        >
          {/* Streak badge (left side) */}
          {streak > 0 && (
            <div
              style={{
                fontSize: 9,
                fontWeight: 700,
                color: "#fbbf24",
                opacity: 0.8,
                paddingLeft: 6,
                whiteSpace: "nowrap",
              }}
            >
              🔥{streak}
            </div>
          )}
          {/* Leave space for the actual notch */}
          <div style={{ flex: 1 }} />
          <div style={{ width: sideWidth, display: "flex", justifyContent: "center" }}>
            <SessionSprite state={state} containerWidth={sideWidth} />
          </div>
        </div>

        {/* Expanded content */}
        {isExpanded && (
          <div style={{ position: "relative" }}>
            {/* Grass island background */}
            <GrassIsland
              state={state}
              width={PANEL_WIDTH}
              height={grassHeight}
            />

            {/* Header buttons */}
            <div
              style={{
                position: "absolute",
                top: 4,
                right: 8,
                display: "flex",
                gap: 8,
                zIndex: 10,
              }}
            >
              <HeaderButton
                icon={isPinned ? "📌" : "📍"}
                onClick={() => setIsPinned(!isPinned)}
              />
              <HeaderButton
                icon="✕"
                onClick={() => setIsExpanded(false)}
              />
            </div>

            {/* Content area */}
            <div
              style={{
                height: expandedContentH,
                padding: "0 16px 16px",
                overflowY: "auto",
              }}
            >
              {children}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function HeaderButton({ icon, onClick }: { icon: string; onClick: () => void }) {
  return (
    <button
      onClick={(e) => {
        e.stopPropagation();
        onClick();
      }}
      style={{
        background: "rgba(255,255,255,0.1)",
        border: "none",
        borderRadius: 6,
        width: 28,
        height: 28,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        cursor: "pointer",
        fontSize: 14,
        color: "rgba(255,255,255,0.7)",
      }}
    >
      {icon}
    </button>
  );
}
