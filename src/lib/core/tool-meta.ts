import type { CommandAction } from "../types";

export type ToolWorkspaceKind =
  | "lab"
  | "generator"
  | "console"
  | "inspector"
  | "library"
  | "service";

export interface ToolPreset {
  label: string;
  value: string;
}

export interface ToolSpec {
  workspace: ToolWorkspaceKind;
  inputMode: "multiline" | "singleline" | "none";
  livePreview: boolean;
  inputLabel: string;
  outputLabel: string;
  placeholder: string;
  helper: string;
  sample?: string;
  quickActions: ToolPreset[];
}

const defaultSpec: ToolSpec = {
  workspace: "lab",
  inputMode: "multiline",
  livePreview: true,
  inputLabel: "Input",
  outputLabel: "Result",
  placeholder: "Paste input or type a command...",
  helper: "Run the tool to inspect the result.",
  quickActions: []
};

const byId: Record<string, Partial<ToolSpec>> = {
  "data.struct-diff": {
    workspace: "lab",
    placeholder: "{\n  \"before\": true\n}\n---\n{\n  \"before\": false\n}",
    helper: "Compare two payloads separated by a line containing only ---."
  },
  "clip.diff": {
    workspace: "lab",
    placeholder: "first version\n---\nsecond version",
    helper: "Paste two text blocks separated by --- to review line differences."
  },
  "shell.code-vault": {
    workspace: "console",
    sample: "add:deploy-script:bash\nnpm run build\nnpm run deploy",
    quickActions: [
      { label: "List snippets", value: "list" },
      { label: "Search deploy", value: "search:deploy" },
      { label: "Usage", value: "" }
    ]
  },
  "clip.snippets": {
    workspace: "console",
    sample: "add:review-template:Please address the requested changes before merge.",
    quickActions: [
      { label: "List", value: "list" },
      { label: "Clear", value: "clear" }
    ]
  },
  "clip.queue": {
    workspace: "console",
    sample: "push:first value",
    quickActions: [
      { label: "List queue", value: "list" },
      { label: "Pop", value: "pop" },
      { label: "Clear", value: "clear" }
    ]
  },
  "mock.http-server": {
    workspace: "service",
    inputMode: "singleline",
    sample: "start:7878",
    quickActions: [
      { label: "Start 7878", value: "start:7878" },
      { label: "Status", value: "status" },
      { label: "Stop", value: "stop" }
    ]
  },
  "mock.webhook": {
    workspace: "service",
    inputMode: "singleline",
    sample: "start:8787",
    quickActions: [
      { label: "Start 8787", value: "start:8787" },
      { label: "Logs", value: "logs" },
      { label: "Stop", value: "stop" }
    ]
  },
  "mock.rest-collection": {
    workspace: "console",
    sample: "add:healthcheck\nmethod:GET\nurl:http://localhost:3000/health",
    quickActions: [
      { label: "List", value: "list" },
      { label: "Clear", value: "clear" }
    ]
  },
  "fs.file-sentinel": {
    workspace: "service",
    inputMode: "singleline",
    sample: "watch:.",
    quickActions: [
      { label: "Snapshot workspace", value: "watch:." },
      { label: "Diff workspace", value: "diff:." },
      { label: "Clear snapshots", value: "clear" }
    ]
  },
  "fs.symlink-matrix": {
    workspace: "console",
    inputMode: "singleline",
    sample: "list",
    quickActions: [
      { label: "List links", value: "list" },
      { label: "Inspect", value: "inspect:." }
    ]
  },
  "shell.cheatsheet": {
    workspace: "library",
    inputMode: "singleline",
    sample: "tar extract",
    quickActions: [
      { label: "tar", value: "tar" },
      { label: "find", value: "find" },
      { label: "chmod", value: "chmod" }
    ]
  },
  "shell.git-wizard": {
    workspace: "library",
    inputMode: "singleline",
    sample: "interactive rebase",
    quickActions: [
      { label: "Rebase", value: "rebase" },
      { label: "Squash", value: "squash" },
      { label: "Reset", value: "reset" }
    ]
  },
  "shell.exit-code": {
    workspace: "library",
    inputMode: "singleline",
    sample: "137",
    quickActions: [
      { label: "127", value: "127" },
      { label: "130", value: "130" },
      { label: "9009", value: "9009" }
    ]
  },
  "mock.status-codes": {
    workspace: "library",
    inputMode: "singleline",
    sample: "404",
    quickActions: [
      { label: "200", value: "200" },
      { label: "404", value: "404" },
      { label: "500", value: "500" }
    ]
  },
  "design.color-swap": {
    workspace: "inspector",
    inputMode: "singleline",
    sample: "#0ea5e9",
    quickActions: [
      { label: "Sky", value: "#0ea5e9" },
      { label: "Graphite", value: "#1f2937" },
      { label: "Signal", value: "rgb(251,113,133)" }
    ]
  },
  "design.eyedropper": {
    workspace: "inspector",
    inputMode: "singleline",
    sample: "#2E6BFF"
  },
  "design.contrast": {
    workspace: "inspector",
    sample: "fg=#111827\nbg=#F9FAFB"
  },
  "design.shadow-gradient": {
    workspace: "generator",
    sample: "gradient=#0ea5e9,#1d4ed8\nangle=135",
    quickActions: [
      { label: "Gradient", value: "gradient=#0ea5e9,#1d4ed8\nangle=135" },
      { label: "Soft shadow", value: "shadow=soft\ncolor=#0f172a\nopacity=0.18" }
    ]
  },
  "mock.cookie-parser": {
    workspace: "inspector",
    inputMode: "singleline",
    sample: "session=abc123; HttpOnly; Secure; SameSite=Lax"
  }
};

export function getToolSpec(command: CommandAction | null): ToolSpec {
  if (!command) return defaultSpec;

  const base = deriveBaseSpec(command);
  const override = byId[command.id] ?? {};
  return { ...base, ...override, quickActions: override.quickActions ?? base.quickActions };
}

function deriveBaseSpec(command: CommandAction): ToolSpec {
  if (!command.acceptsInput) {
    return {
      ...defaultSpec,
      workspace: "service",
      inputMode: "none",
      livePreview: false,
      helper: "This tool reads local state immediately when you run it."
    };
  }

  if (command.id.startsWith("mock.") && /server|webhook|load|graphql|grpc|websocket|rest-collection/.test(command.id)) {
    return {
      ...defaultSpec,
      workspace: "service",
      inputMode: "singleline",
      livePreview: false,
      inputLabel: "Action",
      helper: "Use action commands or a compact request profile."
    };
  }

  if (command.id.startsWith("shell.") && /cheatsheet|git-wizard|exit-code/.test(command.id)) {
    return {
      ...defaultSpec,
      workspace: "library",
      inputMode: "singleline",
      livePreview: false,
      inputLabel: "Search",
      helper: "Search by tool, topic, code, or workflow."
    };
  }

  if (command.id.startsWith("design.") && /color|contrast|eyedropper/.test(command.id)) {
    return {
      ...defaultSpec,
      workspace: "inspector",
      inputMode: "singleline",
      livePreview: true,
      inputLabel: "Sample",
      helper: "Paste a value and inspect structured visual conversions."
    };
  }

  if (command.id.startsWith("fs.") && /file-sentinel|symlink/.test(command.id)) {
    return {
      ...defaultSpec,
      workspace: "console",
      inputMode: "singleline",
      livePreview: false,
      inputLabel: "Command",
      helper: "These tools behave like local control consoles with action verbs."
    };
  }

  if (
    command.id.startsWith("clip.") ||
    command.id.startsWith("shell.") ||
    command.id.startsWith("fs.")
  ) {
    return {
      ...defaultSpec,
      workspace: "console",
      livePreview: false,
      inputLabel: "Command",
      helper: "Use compact commands, quick actions, or paste raw input."
    };
  }

  if (
    command.id.startsWith("ai.") ||
    command.id.startsWith("design.") ||
    /gen|compose|docs|scaffold|layout|type-scale|aspect-ratio/.test(command.id)
  ) {
    return {
      ...defaultSpec,
      workspace: "generator",
      livePreview: true,
      helper: "Shape the request on the left and refine the generated output on the right."
    };
  }

  if (/diff|inspect|parse|classify|check|audit|discover|trace|monitor/.test(command.id)) {
    return {
      ...defaultSpec,
      workspace: "inspector",
      livePreview: true,
      helper: "This tool is best used as an inspection workspace with structured results."
    };
  }

  return defaultSpec;
}
