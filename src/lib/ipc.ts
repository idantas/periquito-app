import { invoke } from "@tauri-apps/api/core";

export interface NotchGeometry {
  notch_width: number;
  notch_height: number;
  screen_width: number;
  screen_height: number;
}

export async function getNotchGeometry(): Promise<NotchGeometry> {
  return invoke<NotchGeometry>("get_notch_geometry");
}

export async function installHooks(): Promise<boolean> {
  return invoke<boolean>("install_hooks");
}

export async function uninstallHooks(): Promise<void> {
  return invoke<void>("uninstall_hooks");
}

export async function isHooksInstalled(): Promise<boolean> {
  return invoke<boolean>("is_hooks_installed");
}

export interface HistoryStats {
  total_good: number;
  total_corrections: number;
  total_evaluated: number;
  accuracy: number | null;
  rolling_accuracy: number | null;
}

export async function getHistoryStats(): Promise<HistoryStats> {
  return invoke<HistoryStats>("get_history_stats");
}

export interface LevelInfo {
  level: string;
  levelName: string;
  emoji: string;
  xp: number;
  xpThreshold: number;
  nextLevelXp: number | null;
  xpProgress: number;
}

export async function getLevelInfo(): Promise<LevelInfo> {
  return invoke<LevelInfo>("get_level_info");
}
