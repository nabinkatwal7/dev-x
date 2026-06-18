<script lang="ts">
  import { categoryLabels } from "../core/command-sections";
  import { getToolSpec } from "../core/tool-meta";
  import {
    applyCommandTemplate,
    commandError,
    commandInput,
    commandResult,
    commandRunning,
    executeSelectedCommand,
    previewRunning,
    query,
    togglePinnedCommand,
    updateCommandInput
  } from "../stores/app-shell";
  import type { CommandAction, CommandExecutionResult, PinnedModule } from "../types";

  export let command: CommandAction | null;
  export let pinnedModules: PinnedModule[] = [];

  $: spec = getToolSpec(command);
  $: parsed = parseResult($commandResult, command?.id ?? "");

  async function runCommand() {
    await executeSelectedCommand(command, $query);
  }

  function handleInput(event: Event) {
    updateCommandInput((event.currentTarget as HTMLTextAreaElement).value);
  }

  function handleSingleLineInput(event: Event) {
    updateCommandInput((event.currentTarget as HTMLInputElement).value);
  }

  function fill(value: string) {
    applyCommandTemplate(value);
  }

  function parseResult(result: CommandExecutionResult | null, commandId: string) {
    if (!result) {
      return {
        cards: [] as Array<{ label: string; value: string }>,
        lines: [] as string[],
        sections: [] as Array<{ title: string; body: string[] }>,
        blocks: [] as string[],
        colorHex: null as string | null,
        code: null as string | null
      };
    }

    const output = result.output ?? "";
    const lines = output.split("\n").filter(Boolean);
    const cards = lines
      .filter((line) => line.includes(":") && !line.startsWith("http"))
      .slice(0, 6)
      .map((line) => {
        const [label, ...rest] = line.split(":");
        return { label: label.trim(), value: rest.join(":").trim() };
      })
      .filter((entry) => entry.label && entry.value);

    const sections = output
      .split("\n\n")
      .map((block) => block.trim())
      .filter(Boolean)
      .map((block) => {
        const sectionLines = block.split("\n");
        const [title, ...body] = sectionLines;
        return { title: title.trim(), body: body.length > 0 ? body : [title] };
      });

    const colorMatch = output.match(/#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{6})\b/);
    const fenceMatch = output.match(/```[\w-]*\n([\s\S]*?)```/);
    const code = fenceMatch?.[1] ?? (commandId.includes("json") || commandId.includes("yaml") || commandId.includes("sql") ? output : null);

    return {
      cards,
      lines,
      sections,
      blocks: output.split("\n\n").filter(Boolean),
      colorHex: colorMatch?.[0] ?? null,
      code
    };
  }
</script>

<section class="flex min-h-0 flex-1 flex-col rounded-xl border border-chrome-800 bg-chrome-900">
  {#if command}
    <div class="border-b border-chrome-800 px-4 py-3">
      <div class="flex flex-wrap items-start justify-between gap-4">
        <div class="min-w-0">
          <div class="text-[11px] text-chrome-500">{command.id}</div>
          <h2 class="mt-1 text-lg font-semibold text-chrome-100">{command.title}</h2>
          <p class="mt-1 max-w-3xl text-sm text-chrome-500">{command.subtitle}</p>
        </div>

        <div class="flex flex-wrap items-center gap-2">
          <button
            class="rounded-lg border border-chrome-700 bg-chrome-950 px-3 py-2 text-sm text-chrome-300 transition hover:text-chrome-100"
            on:click={() => togglePinnedCommand(command)}
          >
            {pinnedModules.some((item) => item.commandId === command.id) ? "Unpin" : "Pin"}
          </button>
          <button
            class="rounded-lg bg-accent-500 px-3 py-2 text-sm font-medium text-chrome-950 transition hover:bg-accent-400 disabled:cursor-not-allowed disabled:opacity-50"
            disabled={$commandRunning}
            on:click={runCommand}
          >
            {$commandRunning ? "Running..." : "Run tool"}
          </button>
        </div>
      </div>

      <div class="mt-3 flex flex-wrap gap-2">
        {#if spec.sample}
          <button class="rounded-lg bg-chrome-950 px-3 py-1.5 text-[11px] text-chrome-300 transition hover:text-chrome-100" on:click={() => fill(spec.sample ?? "")}>
            Load sample
          </button>
        {/if}
        {#each spec.quickActions as action}
          <button class="rounded-lg bg-chrome-950 px-3 py-1.5 text-[11px] text-chrome-300 transition hover:text-chrome-100" on:click={() => fill(action.value)}>
            {action.label}
          </button>
        {/each}
      </div>
    </div>

    <div class="min-h-0 flex-1 overflow-hidden p-3">
      <div class="grid h-full min-h-0 gap-2 {spec.workspace === 'service' ? 'xl:grid-cols-[280px_minmax(0,1fr)]' : 'xl:grid-cols-[minmax(0,0.95fr)_minmax(0,1.05fr)]'}">
        {#if spec.inputMode !== "none"}
          <section class="flex min-h-0 flex-col rounded-lg border border-chrome-800 bg-chrome-950 p-3">
            <div class="flex items-center justify-between gap-2">
              <div>
                <div class="text-[10px] uppercase tracking-[0.18em] text-chrome-500">{spec.inputLabel}</div>
                <div class="mt-1 text-[11px] text-chrome-500">{spec.helper}</div>
              </div>
              <span class="text-[10px] uppercase tracking-[0.14em] text-chrome-600">
                {spec.livePreview ? ($previewRunning ? "live" : "ready") : "manual"}
              </span>
            </div>

            <div class="mt-3 min-h-0 flex-1">
              {#if spec.inputMode === "singleline"}
                <input
                  class="w-full rounded-lg border border-chrome-700 bg-chrome-900 px-3 py-2.5 text-sm text-chrome-100 outline-none transition placeholder:text-chrome-500 focus:border-accent-500/50 focus:ring-1 focus:ring-accent-500/20"
                  bind:value={$commandInput}
                  placeholder={spec.placeholder}
                  on:input={handleSingleLineInput}
                />
              {:else}
                <textarea
                  class="h-full min-h-[260px] w-full resize-none rounded-lg border border-chrome-700 bg-chrome-900 px-3 py-3 font-mono text-[13px] leading-6 text-chrome-100 outline-none transition placeholder:text-chrome-500 focus:border-accent-500/50 focus:ring-1 focus:ring-accent-500/20"
                  bind:value={$commandInput}
                  placeholder={spec.placeholder}
                  on:input={handleInput}
                ></textarea>
              {/if}
            </div>
          </section>
        {/if}

        {#if spec.workspace === "service"}
          <section class="flex min-h-0 flex-col rounded-lg border border-chrome-800 bg-chrome-950 p-3">
            <div class="text-[10px] uppercase tracking-[0.18em] text-chrome-500">Actions</div>
            <div class="mt-3 grid gap-2">
              {#if spec.sample}
                <button class="rounded-lg bg-chrome-800 px-3 py-2.5 text-left text-sm text-chrome-300 transition hover:text-chrome-100" on:click={() => fill(spec.sample ?? "")}>
                  Recommended action
                </button>
              {/if}
              {#each spec.quickActions as action}
                <button class="rounded-lg border border-chrome-700 bg-chrome-900 px-3 py-2.5 text-left text-sm text-chrome-300 transition hover:text-chrome-100" on:click={() => fill(action.value)}>
                  {action.label}
                </button>
              {/each}
            </div>

            <div class="mt-3 rounded-lg border border-chrome-800 bg-chrome-900 p-3 text-[12px] leading-6 text-chrome-400">
              Stateful tools run only when you explicitly press `Run tool`.
            </div>
          </section>
        {/if}

        <section class="flex min-h-0 flex-col rounded-lg border border-chrome-800 bg-chrome-950 p-3">
          <div class="flex items-center justify-between gap-2">
            <div>
              <div class="text-[10px] uppercase tracking-[0.18em] text-chrome-500">{spec.outputLabel}</div>
              <div class="mt-1 text-[11px] text-chrome-500">Result</div>
            </div>
            {#if parsed.colorHex}
              <div class="flex items-center gap-2 rounded-lg border border-chrome-700 bg-chrome-900 px-3 py-2 text-[11px] text-chrome-300">
                <span class="inline-flex h-4 w-4 rounded-full border border-white/10" style={`background:${parsed.colorHex}`}></span>
                {parsed.colorHex}
              </div>
            {/if}
          </div>

          {#if $commandError}
            <div class="mt-3 flex-1 overflow-auto rounded-lg border border-signal-danger/35 bg-[rgba(60,10,18,0.20)] p-4 text-sm leading-6 text-signal-danger">
              {$commandError}
            </div>
          {:else if $commandResult}
            <div class="mt-3 grid min-h-0 flex-1 gap-3 overflow-hidden {spec.workspace === 'library' ? 'lg:grid-cols-[minmax(0,0.75fr)_minmax(0,1.25fr)]' : ''}">
              <div class="min-h-0 overflow-auto">
                {#if parsed.cards.length > 0}
                  <div class="grid gap-2 md:grid-cols-2">
                    {#each parsed.cards as card}
                      <div class="rounded-lg border border-chrome-800 bg-chrome-900 p-3">
                        <div class="text-[10px] uppercase tracking-[0.16em] text-chrome-500">{card.label}</div>
                        <div class="mt-2 break-words text-sm text-chrome-100">{card.value}</div>
                      </div>
                    {/each}
                  </div>
                {:else if spec.workspace === 'library' || spec.workspace === 'service' || spec.workspace === 'inspector'}
                  <div class="space-y-2">
                    {#each parsed.sections as section}
                      <article class="rounded-lg border border-chrome-800 bg-chrome-900 p-3">
                        <div class="text-sm font-semibold text-chrome-100">{section.title}</div>
                        <div class="mt-2 space-y-1 text-sm text-chrome-300">
                          {#each section.body as line}
                            <div>{line}</div>
                          {/each}
                        </div>
                      </article>
                    {/each}
                  </div>
                {/if}
              </div>

              <div class="min-h-0 overflow-auto rounded-lg border border-chrome-800 bg-chrome-900">
                <div class="border-b border-chrome-800 px-3 py-2.5 text-[10px] uppercase tracking-[0.16em] text-chrome-500">
                  {parsed.code ? "Primary output" : "Full response"}
                </div>
                {#if parsed.code}
                  <pre class="h-full overflow-auto px-3 py-3 font-mono text-xs leading-6 text-chrome-200">{parsed.code}</pre>
                {:else if $commandResult.output}
                  <pre class="h-full overflow-auto whitespace-pre-wrap px-3 py-3 font-mono text-xs leading-6 text-chrome-200">{$commandResult.output}</pre>
                {:else}
                  <div class="px-3 py-3 text-sm text-chrome-400">{$commandResult.summary}</div>
                {/if}
              </div>
            </div>
          {:else}
            <div class="mt-3 flex flex-1 items-center justify-center rounded-lg border border-dashed border-chrome-700 bg-chrome-900 px-6 text-center text-sm text-chrome-500">
              {command.acceptsInput ? (spec.livePreview ? "Edit the input to inspect output, or run the tool." : "Prepare the request and run the tool.") : "Run the tool to load its output."}
            </div>
          {/if}
        </section>
      </div>
    </div>
  {:else}
    <div class="flex h-full items-center justify-center px-8 text-center text-sm text-chrome-500">
      Select a tool from the command rail to open its workspace.
    </div>
  {/if}
</section>
