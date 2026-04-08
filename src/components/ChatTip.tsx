import { parseCorrection } from "../lib/tip-parser";

interface EnglishTip {
  id: string;
  timestamp: string;
  prompt: string;
  tip_type: string;
  tip: string | null;
  category: string | null;
}

interface ChatTipProps {
  tip: EnglishTip;
}

export default function ChatTip({ tip }: ChatTipProps) {
  const isGood = tip.tip_type === "good";
  const corrections = tip.tip ? parseCorrection(tip.tip) : [];
  const hasCorrections = corrections.length > 0;

  return (
    <div
      style={{
        background: isGood
          ? "rgba(74, 222, 128, 0.1)"
          : "rgba(251, 191, 36, 0.1)",
        borderRadius: 10,
        padding: "10px 12px",
        fontSize: 12,
        lineHeight: 1.5,
      }}
    >
      {/* Header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 6,
          marginBottom: hasCorrections || tip.tip ? 6 : 0,
        }}
      >
        <span style={{ fontSize: 13 }}>{isGood ? "✓" : "✗"}</span>
        <span
          style={{
            fontWeight: 600,
            color: isGood ? "#4ade80" : "#fbbf24",
            fontSize: 11,
          }}
        >
          {isGood ? "Solid English" : "Correction"}
        </span>
        {tip.category && <CategoryBadge category={tip.category} />}
      </div>

      {/* Correction details */}
      {hasCorrections
        ? corrections.map((c, i) => (
            <div key={i} style={{ marginBottom: i < corrections.length - 1 ? 6 : 0 }}>
              <div>
                <span
                  style={{
                    textDecoration: "line-through",
                    color: "rgba(255,255,255,0.4)",
                  }}
                >
                  {c.wrong}
                </span>
                <span style={{ margin: "0 6px", opacity: 0.4 }}>→</span>
                <span style={{ color: "#4ade80", fontWeight: 500 }}>
                  {c.right}
                </span>
              </div>
              <div style={{ opacity: 0.5, fontSize: 11, marginTop: 2 }}>
                {c.explanation}
              </div>
            </div>
          ))
        : tip.tip && (
            <div style={{ opacity: 0.8 }}>
              {tip.tip}
            </div>
          )}
    </div>
  );
}

function CategoryBadge({ category }: { category: string }) {
  return (
    <span
      style={{
        background: "rgba(255,255,255,0.08)",
        borderRadius: 4,
        padding: "1px 7px",
        fontSize: 10,
        color: "rgba(255,255,255,0.5)",
        marginLeft: "auto",
      }}
    >
      {category}
    </span>
  );
}
