import type { Config } from "../types";

export function configsEqual(left: Config, right: Config): boolean {
  return JSON.stringify(left, Object.keys(left).sort()) ===
         JSON.stringify(right, Object.keys(right).sort());
}

export function parseFilterWords(value: string): string[] {
  return value
    .split(",")
    .map((entry) => entry.trim())
    .filter(Boolean);
}
