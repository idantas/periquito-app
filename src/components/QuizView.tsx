import { useEffect, useState } from "react";
import {
  getNextQuiz,
  submitQuizAnswer,
  getReviewStats,
  type QuizQuestion,
  type QuizResult,
  type ReviewStats,
} from "../lib/ipc";

type QuizState =
  | { type: "idle" }
  | { type: "asking"; quiz: QuizQuestion }
  | { type: "evaluating" }
  | { type: "result"; result: QuizResult; selected: string };

const CORRECT_PHRASES = [
  "Nailed it!",
  "Spot on!",
  "You got it!",
  "Perfect!",
  "Well done!",
];

export default function QuizView() {
  const [state, setState] = useState<QuizState>({ type: "idle" });
  const [stats, setStats] = useState<ReviewStats | null>(null);

  const loadStats = () => getReviewStats().then(setStats);

  useEffect(() => {
    loadStats();
  }, []);

  const startQuiz = async () => {
    const quiz = await getNextQuiz();
    if (quiz) {
      setState({ type: "asking", quiz });
    }
  };

  const handleAnswer = async (quiz: QuizQuestion, answer: string) => {
    setState({ type: "evaluating" });
    const result = await submitQuizAnswer(quiz.item.id, answer);
    if (result) {
      setState({ type: "result", result, selected: answer });
    } else {
      setState({ type: "idle" });
    }
    loadStats();
  };

  const dismiss = () => {
    setState({ type: "idle" });
    loadStats();
  };

  // Idle — show stats and start button
  if (state.type === "idle") {
    return (
      <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
        {/* Review queue stats */}
        {stats && (
          <div
            style={{
              background: "rgba(255,255,255,0.05)",
              borderRadius: 12,
              padding: "12px 14px",
              display: "flex",
              justifyContent: "space-between",
              fontSize: 12,
            }}
          >
            <StatItem label="In queue" value={stats.totalItems} />
            <StatItem label="Due now" value={stats.dueCount} color="#fbbf24" />
            <StatItem label="Mastered" value={stats.masteredCount} color="#4ade80" />
          </div>
        )}

        {/* Start quiz button */}
        {stats && stats.dueCount > 0 ? (
          <button
            onClick={startQuiz}
            style={{
              background: "rgba(167, 139, 250, 0.15)",
              color: "#a78bfa",
              border: "1px solid rgba(167, 139, 250, 0.2)",
              borderRadius: 10,
              padding: "10px 20px",
              fontSize: 13,
              fontWeight: 600,
              cursor: "pointer",
              fontFamily: "SF Mono, monospace",
            }}
          >
            Start Quiz ({stats.dueCount} due)
          </button>
        ) : (
          <div style={{ textAlign: "center", opacity: 0.3, padding: "16px 0", fontSize: 12 }}>
            {stats && stats.totalItems > 0
              ? "All caught up! Next review later."
              : "No corrections yet — keep writing in English!"}
          </div>
        )}
      </div>
    );
  }

  // Asking — multiple choice
  if (state.type === "asking") {
    const { quiz } = state;
    return (
      <div
        style={{
          background: "rgba(251, 191, 36, 0.06)",
          borderRadius: 12,
          padding: "14px",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 6, marginBottom: 10 }}>
          <span style={{ fontSize: 16 }}>🦜</span>
          <span style={{ fontSize: 12, fontWeight: 600 }}>Which is correct?</span>
        </div>

        {quiz.item.explanation && (
          <div style={{ fontSize: 11, opacity: 0.5, marginBottom: 10, lineHeight: 1.4 }}>
            {quiz.item.explanation}
          </div>
        )}

        <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          {quiz.options.map((option) => (
            <button
              key={option}
              onClick={() => handleAnswer(quiz, option)}
              style={{
                background: "rgba(255,255,255,0.08)",
                color: "#e0e0e0",
                border: "1px solid rgba(255,255,255,0.06)",
                borderRadius: 8,
                padding: "8px 12px",
                fontSize: 12,
                fontWeight: 500,
                cursor: "pointer",
                textAlign: "left",
                fontFamily: "SF Mono, monospace",
                transition: "background 0.15s",
              }}
              onMouseEnter={(e) =>
                (e.currentTarget.style.background = "rgba(255,255,255,0.14)")
              }
              onMouseLeave={(e) =>
                (e.currentTarget.style.background = "rgba(255,255,255,0.08)")
              }
            >
              {option}
            </button>
          ))}
        </div>

        <button
          onClick={dismiss}
          style={{
            background: "none",
            border: "none",
            color: "rgba(255,255,255,0.3)",
            fontSize: 10,
            cursor: "pointer",
            marginTop: 8,
            padding: "4px 0",
          }}
        >
          Skip
        </button>
      </div>
    );
  }

  // Evaluating — spinner
  if (state.type === "evaluating") {
    return (
      <div
        style={{
          background: "rgba(251, 191, 36, 0.06)",
          borderRadius: 12,
          padding: "20px 14px",
          textAlign: "center",
        }}
      >
        <span style={{ fontSize: 16 }}>🦜</span>
        <span style={{ fontSize: 12, opacity: 0.5, marginLeft: 8 }}>Checking...</span>
      </div>
    );
  }

  // Result — feedback
  if (state.type === "result") {
    const { result } = state;
    const phrase = CORRECT_PHRASES[Math.floor(Math.random() * CORRECT_PHRASES.length)];

    return (
      <div
        style={{
          background: result.correct
            ? "rgba(74, 222, 128, 0.06)"
            : "rgba(248, 113, 113, 0.06)",
          borderRadius: 12,
          padding: "14px",
        }}
      >
        {/* Header */}
        <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 10 }}>
          <div
            style={{
              width: 24,
              height: 24,
              borderRadius: 12,
              background: result.correct
                ? "rgba(74, 222, 128, 0.2)"
                : "rgba(248, 113, 113, 0.2)",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontSize: 13,
              color: result.correct ? "#4ade80" : "#f87171",
              fontWeight: 700,
            }}
          >
            {result.correct ? "✓" : "✗"}
          </div>
          <span style={{ fontSize: 13, fontWeight: 600 }}>
            {result.correct ? phrase : "Not quite..."}
          </span>
        </div>

        {/* Correct answer */}
        <div style={{ fontSize: 12, marginBottom: 6 }}>
          <span style={{ color: "#4ade80", fontWeight: 500 }}>
            {result.correctAnswer}
          </span>
        </div>

        {/* Explanation */}
        {result.explanation && (
          <div style={{ fontSize: 11, opacity: 0.5, marginBottom: 10, lineHeight: 1.4 }}>
            {result.explanation}
          </div>
        )}

        {/* Progress badge */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 10,
            marginBottom: 12,
          }}
        >
          {/* Streak */}
          {result.correctCount > 1 && (
            <span style={{ fontSize: 10, color: "#fbbf24" }}>
              🔥 {result.correctCount} in a row
            </span>
          )}

          {/* Leitner box dots */}
          <div style={{ display: "flex", gap: 3 }}>
            {[1, 2, 3, 4, 5].map((b) => (
              <div
                key={b}
                style={{
                  width: 6,
                  height: 6,
                  borderRadius: 3,
                  background:
                    b <= result.leitnerBox
                      ? "#a78bfa"
                      : "rgba(255,255,255,0.1)",
                }}
              />
            ))}
          </div>

          {result.leitnerBox >= 5 && (
            <span style={{ fontSize: 10, color: "#4ade80", fontWeight: 600 }}>
              Mastered!
            </span>
          )}
        </div>

        {/* Actions */}
        <div style={{ display: "flex", gap: 8 }}>
          <button
            onClick={dismiss}
            style={{
              background: "rgba(255,255,255,0.08)",
              color: "#e0e0e0",
              border: "none",
              borderRadius: 6,
              padding: "6px 14px",
              fontSize: 11,
              fontWeight: 600,
              cursor: "pointer",
            }}
          >
            Got it
          </button>
          <button
            onClick={startQuiz}
            style={{
              background: "none",
              border: "none",
              color: "rgba(255,255,255,0.4)",
              fontSize: 11,
              cursor: "pointer",
              padding: "6px 0",
            }}
          >
            Next quiz →
          </button>
        </div>
      </div>
    );
  }

  return null;
}

function StatItem({
  label,
  value,
  color,
}: {
  label: string;
  value: number;
  color?: string;
}) {
  return (
    <div style={{ textAlign: "center" }}>
      <div style={{ fontSize: 16, fontWeight: 700, color: color ?? "#e0e0e0" }}>
        {value}
      </div>
      <div style={{ fontSize: 9, opacity: 0.4, marginTop: 2 }}>{label}</div>
    </div>
  );
}
