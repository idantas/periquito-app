export interface ParsedCorrection {
  wrong: string;
  right: string;
  explanation: string;
}

/**
 * Parse correction format: ❌ X → ✅ Y — Z
 * Also handles: ❌ X → ✅ Y - Z (with simple dash)
 */
export function parseCorrection(tip: string): ParsedCorrection[] {
  const results: ParsedCorrection[] = [];
  // Split by semicolons for multiple corrections
  const parts = tip.split(";").map((s) => s.trim());

  for (const part of parts) {
    // Match: ❌ wrong → ✅ right — explanation
    const match = part.match(
      /❌\s*(.+?)\s*→\s*✅\s*(.+?)\s*[—\-]\s*(.+)/
    );
    if (match) {
      results.push({
        wrong: match[1].trim(),
        right: match[2].trim(),
        explanation: match[3].trim(),
      });
    }
  }

  return results;
}
