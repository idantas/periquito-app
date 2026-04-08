/**
 * Piecewise cubic easeInOut bob — port from BobAnimation.swift
 * Returns value in [-amplitude, +amplitude]
 */
export function bobOffset(
  timeMs: number,
  duration: number,
  amplitude: number
): number {
  if (amplitude <= 0) return 0;
  const t = timeMs / 1000;
  const phase = ((t / duration) % 1 + 1) % 1; // ensure positive modulo
  const inFirstHalf = phase < 0.5;
  const u = inFirstHalf ? phase * 2 : (phase - 0.5) * 2;
  const eased =
    u < 0.5 ? 4 * u * u * u : 1 - Math.pow(-2 * u + 2, 3) / 2;
  const wave = inFirstHalf ? 1 - 2 * eased : -1 + 2 * eased;
  return wave * amplitude;
}

/**
 * Rapid horizontal shake for distressed states (sob)
 */
const TREMBLE_FREQUENCY_HZ = 2;
export function trembleOffset(timeMs: number, amplitude: number): number {
  if (amplitude <= 0) return 0;
  const t = timeMs / 1000;
  return Math.sin(t * TREMBLE_FREQUENCY_HZ * Math.PI * 2) * amplitude;
}

/**
 * Sway rotation in degrees
 */
export function swayDegrees(
  timeMs: number,
  amplitude: number,
  duration: number = 2.0
): number {
  if (amplitude <= 0) return 0;
  const t = timeMs / 1000;
  const phase = ((t / duration) % 1 + 1) % 1;
  return Math.sin(phase * Math.PI * 2) * amplitude;
}
