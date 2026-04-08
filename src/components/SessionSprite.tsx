import { useEffect, useRef, useState } from "react";
import SpriteSheet from "./SpriteSheet";
import { bobOffset, trembleOffset, swayDegrees } from "../lib/sprite-physics";

// Import all sprites
import idle_neutral from "../assets/sprites/idle_neutral.png";
import idle_happy from "../assets/sprites/idle_happy.png";
import idle_sad from "../assets/sprites/idle_sad.png";
import idle_sob from "../assets/sprites/idle_sob.png";
import working_neutral from "../assets/sprites/working_neutral.png";
import working_happy from "../assets/sprites/working_happy.png";
import working_sad from "../assets/sprites/working_sad.png";
import working_sob from "../assets/sprites/working_sob.png";
import waiting_neutral from "../assets/sprites/waiting_neutral.png";
import waiting_happy from "../assets/sprites/waiting_happy.png";
import waiting_sad from "../assets/sprites/waiting_sad.png";
import waiting_sob from "../assets/sprites/waiting_sob.png";
import sleeping_neutral from "../assets/sprites/sleeping_neutral.png";
import sleeping_happy from "../assets/sprites/sleeping_happy.png";
import compacting_neutral from "../assets/sprites/compacting_neutral.png";
import compacting_happy from "../assets/sprites/compacting_happy.png";
import walkSprite from "../assets/sprites/walk.png";

const SPRITE_MAP: Record<string, string> = {
  idle_neutral,
  idle_happy,
  idle_sad,
  idle_sob,
  working_neutral,
  working_happy,
  working_sad,
  working_sob,
  waiting_neutral,
  waiting_happy,
  waiting_sad,
  waiting_sob,
  sleeping_neutral,
  sleeping_happy,
  compacting_neutral,
  compacting_happy,
  walk: walkSprite,
};

interface PeriquitoState {
  task: string;
  emotion: string;
}

// Animation parameters per task x emotion (ported from PeriquitoState.swift)
function getAnimParams(task: string, emotion: string) {
  const fps = getFps(task, emotion);
  const frameCount = getFrameCount(task, emotion);
  const bobDur = getBobDuration(task);
  const bobAmp = getBobAmplitude(task, emotion);
  const swayAmp = getSwayAmplitude(task, emotion);
  const canWalk = emotion !== "sob" && (task === "idle" || task === "working");
  const walkFreq = getWalkFrequencyRange(task);

  return { fps, frameCount, bobDur, bobAmp, swayAmp, canWalk, walkFreq };
}

function getFps(task: string, emotion: string): number {
  const key = `${task}_${emotion}`;
  const map: Record<string, number> = {
    idle_happy: 10, idle_sad: 6, idle_sob: 4, idle_neutral: 8,
    working_happy: 12, working_sad: 8, working_sob: 6, working_neutral: 10,
    waiting_happy: 10, waiting_sad: 6, waiting_sob: 5, waiting_neutral: 8,
    compacting_happy: 14, compacting_neutral: 14, compacting_sad: 14, compacting_sob: 14,
    sleeping_happy: 6, sleeping_neutral: 6, sleeping_sad: 6, sleeping_sob: 6,
  };
  return map[key] ?? 8;
}

function getFrameCount(task: string, emotion: string): number {
  if (task === "idle" && emotion === "happy") return 16;
  if (task === "idle" && (emotion === "sad" || emotion === "sob")) return 16;
  if (task === "working" && (emotion === "sad" || emotion === "sob")) return 16;
  if (task === "waiting" && emotion === "sob") return 16;
  if (task === "sleeping") return 16;
  return 8;
}

function getBobDuration(task: string): number {
  const map: Record<string, number> = {
    sleeping: 4.0, idle: 1.5, waiting: 1.5, working: 1.0, compacting: 0.5,
  };
  return map[task] ?? 1.5;
}

function getBobAmplitude(task: string, emotion: string): number {
  if (emotion === "sob") return 0;
  const base: Record<string, number> = {
    sleeping: 0, compacting: 0, idle: 1.5, waiting: 0.5, working: 0.5,
  };
  const amp = base[task] ?? 0;
  return emotion === "sad" ? amp * 0.5 : (task === "working" ? 0.5 : 0.3);
}

function getSwayAmplitude(task: string, emotion: string): number {
  if (task === "sleeping" || task === "compacting") return 0;
  const map: Record<string, number> = {
    neutral: 0.5, happy: 1.0, sad: 0.25, sob: 0.15,
  };
  return map[emotion] ?? 0.5;
}

function getWalkFrequencyRange(task: string): [number, number] {
  const map: Record<string, [number, number]> = {
    sleeping: [30, 60], waiting: [30, 60],
    idle: [8, 15], working: [5, 12], compacting: [15, 25],
  };
  return map[task] ?? [8, 15];
}

function getSpriteSheetName(task: string, emotion: string): string {
  const name = `${task}_${emotion}`;
  if (SPRITE_MAP[name]) return name;
  if (emotion === "sob" && SPRITE_MAP[`${task}_sad`]) return `${task}_sad`;
  return `${task}_neutral`;
}

interface SessionSpriteProps {
  state: PeriquitoState;
  containerWidth: number;
}

const SPRITE_SIZE = 64;
const SOB_TREMBLE_AMP = 0.3;
const WALK_SPEED = 22; // px/s

export default function SessionSprite({ state, containerWidth }: SessionSpriteProps) {
  const [now, setNow] = useState(performance.now());
  const walkOffsetRef = useRef(0);
  const walkDirectionRef = useRef(1);
  const [isWalking, setIsWalking] = useState(false);
  const walkTargetRef = useRef(0);
  const walkStartTimeRef = useRef(0);
  const walkStartPosRef = useRef(0);
  const walkDurationRef = useRef(0);
  const walkTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const params = getAnimParams(state.task, state.emotion);
  const walkRange = containerWidth * 0.1;

  // Animation loop
  useEffect(() => {
    let id: number;
    const tick = (t: number) => {
      setNow(t);
      id = requestAnimationFrame(tick);
    };
    id = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(id);
  }, []);

  // Walk logic
  useEffect(() => {
    if (!params.canWalk) {
      setIsWalking(false);
      if (walkTimerRef.current) clearTimeout(walkTimerRef.current);
      return;
    }

    const scheduleWalk = () => {
      const [min, max] = params.walkFreq;
      const delay = (min + Math.random() * (max - min)) * 1000;
      walkTimerRef.current = setTimeout(() => {
        if (!params.canWalk) return;

        const target = (Math.random() * 2 - 1) * walkRange;
        const currentPos = walkOffsetRef.current;
        const distance = Math.abs(target - currentPos);
        const duration = Math.max(0.5, distance / WALK_SPEED);

        walkTargetRef.current = target;
        walkStartPosRef.current = currentPos;
        walkStartTimeRef.current = performance.now();
        walkDurationRef.current = duration;
        walkDirectionRef.current = target > currentPos ? 1 : -1;
        setIsWalking(true);

        setTimeout(() => {
          walkOffsetRef.current = target;
          setIsWalking(false);
          scheduleWalk();
        }, duration * 1000);
      }, delay);
    };

    scheduleWalk();
    return () => {
      if (walkTimerRef.current) clearTimeout(walkTimerRef.current);
    };
  }, [params.canWalk, params.walkFreq[0], params.walkFreq[1], walkRange]);

  // Calculate walk position with easing
  let walkX = walkOffsetRef.current;
  if (isWalking) {
    const elapsed = (now - walkStartTimeRef.current) / 1000;
    const progress = Math.min(1, elapsed / walkDurationRef.current);
    // easeInOut
    const eased = progress < 0.5
      ? 2 * progress * progress
      : 1 - Math.pow(-2 * progress + 2, 2) / 2;
    walkX = walkStartPosRef.current + (walkTargetRef.current - walkStartPosRef.current) * eased;
  }

  const bob = bobOffset(now, params.bobDur, params.bobAmp);
  const tremble = state.emotion === "sob" ? trembleOffset(now, SOB_TREMBLE_AMP) : 0;
  const sway = swayDegrees(now, params.swayAmp);

  const activeSpriteSheet = isWalking ? "walk" : getSpriteSheetName(state.task, state.emotion);
  const activeFrameCount = isWalking ? 16 : params.frameCount;
  const activeFps = isWalking ? 10 : params.fps;

  // Walk sprite faces LEFT by default; others face RIGHT
  const flipX = isWalking
    ? walkDirectionRef.current >= 0
    : walkDirectionRef.current < 0;

  return (
    <div
      style={{
        position: "relative",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        transform: `translate(${walkX + tremble}px, ${8 + bob}px) rotate(${sway}deg)`,
        transformOrigin: "bottom center",
      }}
    >
      <SpriteSheet
        src={SPRITE_MAP[activeSpriteSheet] || SPRITE_MAP["idle_neutral"]}
        frameCount={activeFrameCount}
        columns={4}
        fps={activeFps}
        width={SPRITE_SIZE}
        height={SPRITE_SIZE}
        flipX={flipX}
      />
      {/* Shadow */}
      <div
        style={{
          width: 24,
          height: 6,
          borderRadius: "50%",
          background: "rgba(0,0,0,0.28)",
          filter: "blur(1.5px)",
          marginTop: -13,
        }}
      />
    </div>
  );
}
