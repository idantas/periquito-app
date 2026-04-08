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

export interface AppSettings {
  notification_sound: string;
  is_muted: boolean;
  font_size: string;
  is_usage_enabled: boolean;
}

export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export async function updateSettings(settings: AppSettings): Promise<void> {
  return invoke<void>("update_settings", { settings });
}

export async function previewSound(name: string): Promise<void> {
  return invoke<void>("preview_sound", { name });
}

export async function getAvailableSounds(): Promise<string[]> {
  return invoke<string[]>("get_available_sounds");
}

export interface QuizItem {
  id: string;
  incorrectSentence: string;
  correctSentence: string;
  explanation: string;
  category: string;
  box: number;
  nextReviewDate: string;
  totalReviews: number;
  correctCount: number;
}

export interface QuizQuestion {
  item: QuizItem;
  options: string[];
}

export interface QuizResult {
  correct: boolean;
  correctAnswer: string;
  explanation: string;
  leitnerBox: number;
  correctCount: number;
}

export interface ReviewStats {
  totalItems: number;
  dueCount: number;
  masteredCount: number;
}

export async function getNextQuiz(): Promise<QuizQuestion | null> {
  return invoke<QuizQuestion | null>("get_next_quiz");
}

export async function submitQuizAnswer(
  itemId: string,
  answer: string
): Promise<QuizResult | null> {
  return invoke<QuizResult | null>("submit_quiz_answer", {
    itemId,
    answer,
  });
}

export async function getReviewStats(): Promise<ReviewStats> {
  return invoke<ReviewStats>("get_review_stats");
}
