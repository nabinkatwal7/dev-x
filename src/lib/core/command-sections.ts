import type { CommandCategory } from "../types";

export const categoryLabels: Record<CommandCategory, string> = {
  ai: "Local AI",
  clipboard: "Clipboard",
  core: "Launcher Core",
  crypto: "Crypto",
  data: "Data Tools",
  filesystem: "Files",
  network: "Network",
  system: "System"
};

export const categoryIcons: Record<CommandCategory, string> = {
  ai: "AI",
  clipboard: "CL",
  core: "HM",
  crypto: "CR",
  data: "DT",
  filesystem: "FS",
  network: "NW",
  system: "SY"
};

export const orderedCategories: CommandCategory[] = [
  "data",
  "clipboard",
  "crypto",
  "system",
  "filesystem",
  "network",
  "ai",
  "core"
];
