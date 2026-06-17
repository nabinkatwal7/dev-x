<script lang="ts">
  import { categoryLabels } from "../core/command-sections";
  import {
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
  import type { CommandAction, PinnedModule } from "../types";

  export let command: CommandAction | null;
  export let pinnedModules: PinnedModule[] = [];

  async function runCommand() {
    await executeSelectedCommand(command, $query);
  }

  function handleInput(event: Event) {
    updateCommandInput((event.currentTarget as HTMLTextAreaElement).value);
  }
</script>

<section class="flex min-h-0 flex-1 flex-col rounded-xl bg-chrome-800">
  {#if command}
    <div class="flex items-center justify-between gap-3 px-5 py-3">
      <div class="min-w-0">
        <h2 class="truncate text-sm font-semibold text-chrome-100">{command.title}</h2>
        <div class="mt-0.5 flex items-center gap-1.5 text-[11px] text-chrome-400">
          {#each command.tags as tag, i}
            <span>{tag}</span>
            {#if i < command.tags.length - 1}
              <span class="text-chrome-600">•</span>
            {/if}
          {/each}
        </div>
      </div>
      {#if command}
        <div class="flex shrink-0 items-center gap-2">
          {#if command.acceptsInput}
            <span class="text-[11px] text-chrome-400">
              {$previewRunning ? "Previewing" : "Live"}
            </span>
          {/if}
          <button
            class="rounded-md bg-chrome-900 px-2.5 py-1.5 text-[11px] text-chrome-300 transition hover:text-chrome-100"
            on:click={() => togglePinnedCommand(command)}
          >
            {pinnedModules.some((item) => item.commandId === command.id) ? "Unpin" : "Pin"}
          </button>
          <button
            class="rounded-md bg-accent-500/15 px-3 py-1.5 text-[11px] text-accent-400 transition hover:bg-accent-500/25 disabled:cursor-not-allowed disabled:opacity-40"
            disabled={$commandRunning}
            on:click={runCommand}
          >
            {$commandRunning ? "Running" : "Run"}
          </button>
        </div>
      {/if}
    </div>

    <div class="min-h-0 flex-1 px-5 pb-4">
      {#if command.acceptsInput}
        <div class="flex h-full flex-col gap-3">
          <div class="relative flex flex-1 flex-col overflow-hidden rounded-lg bg-chrome-950">
            <span class="pointer-events-none absolute left-3 top-2 z-10 text-[10px] font-medium uppercase tracking-[0.12em] text-chrome-400">Input</span>
            <textarea
              class="h-full resize-none bg-transparent px-3 pb-3 pt-7 text-sm leading-6 text-chrome-100 outline-none"
              bind:value={$commandInput}
              on:input={handleInput}
            ></textarea>
          </div>

          {#if $commandError}
            <div class="relative flex flex-1 flex-col overflow-hidden rounded-lg border border-signal-danger/30 bg-chrome-950">
              <span class="pointer-events-none absolute left-3 top-2 z-10 text-[10px] font-medium uppercase tracking-[0.12em] text-signal-danger">Error</span>
              <div class="h-full overflow-auto px-3 pb-3 pt-7 text-sm leading-6 text-signal-danger">{$commandError}</div>
            </div>
          {:else if $commandResult}
            <div class="relative flex flex-1 flex-col overflow-hidden rounded-lg bg-chrome-950">
              <span class="pointer-events-none absolute left-3 top-2 z-10 text-[10px] font-medium uppercase tracking-[0.12em] text-chrome-400">Result</span>
              {#if $commandResult.output}
                <pre class="h-full overflow-auto px-3 pb-3 pt-7 text-xs leading-5 text-chrome-200">{$commandResult.output}</pre>
              {:else}
                <div class="h-full overflow-auto px-3 pb-3 pt-7 text-sm text-chrome-400">{$commandResult.summary}</div>
              {/if}
            </div>
          {:else}
            <div class="relative flex flex-1 flex-col overflow-hidden rounded-lg bg-chrome-950">
              <span class="pointer-events-none absolute left-3 top-2 z-10 text-[10px] font-medium uppercase tracking-[0.12em] text-chrome-400">Result</span>
              <div class="flex h-full items-center justify-center px-3 pt-7 text-sm text-chrome-400">
                Provide input and run the command.
              </div>
            </div>
          {/if}
        </div>
      {:else}
        <div class="flex h-full flex-col">
          {#if $commandError}
            <div class="relative flex flex-1 flex-col overflow-hidden rounded-lg border border-signal-danger/30 bg-chrome-950">
              <span class="pointer-events-none absolute left-3 top-2 z-10 text-[10px] font-medium uppercase tracking-[0.12em] text-signal-danger">Error</span>
              <div class="h-full overflow-auto px-3 pb-3 pt-7 text-sm leading-6 text-signal-danger">{$commandError}</div>
            </div>
          {:else if $commandResult}
            <div class="relative flex flex-1 flex-col overflow-hidden rounded-lg bg-chrome-950">
              <span class="pointer-events-none absolute left-3 top-2 z-10 text-[10px] font-medium uppercase tracking-[0.12em] text-chrome-400">Result</span>
              {#if $commandResult.output}
                <pre class="h-full overflow-auto px-3 pb-3 pt-7 text-xs leading-5 text-chrome-200">{$commandResult.output}</pre>
              {:else}
                <div class="h-full overflow-auto px-3 pb-3 pt-7 text-sm text-chrome-400">{$commandResult.summary}</div>
              {/if}
            </div>
          {:else}
            <div class="flex h-full items-center justify-center">
              <p class="text-sm text-chrome-400">Run the command to see its result.</p>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  {:else}
    <div class="flex h-full items-center justify-center">
      <p class="text-sm text-chrome-400">Select a command from the palette to begin.</p>
    </div>
  {/if}
</section>