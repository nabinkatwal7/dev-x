<script lang="ts">
  import { categoryLabels } from "../core/command-sections";
  import {
    executeSelectedCommand,
    filteredCommands,
    query,
    selectedCommand,
    selectedCommandId
  } from "../stores/app-shell";
  import type { CommandAction } from "../types";

  export let commandCount = 0;

  function choose(command: CommandAction) {
    selectedCommandId.set(command.id);
  }

  async function runSelected() {
    await executeSelectedCommand($selectedCommand, $query);
  }
</script>

<section class="flex h-full flex-col rounded-lg border border-chrome-700/70 bg-chrome-900 shadow-overlay">
  <div class="border-b border-chrome-700/70 px-4 py-3">
    <div class="mb-2 flex items-center justify-between text-[11px] uppercase tracking-[0.18em] text-chrome-300">
      <span>Command Palette</span>
      <span>{commandCount} indexed</span>
    </div>
    <input
      class="w-full rounded-md border border-chrome-600 bg-chrome-950 px-3 py-2 text-sm text-chrome-100 outline-none transition focus:border-accent-400"
      bind:value={$query}
      placeholder="Search commands, tools, snippets, actions..."
    />
    <div class="mt-2 flex justify-end">
      <button
        class="rounded-md border border-accent-400/70 bg-accent-500/15 px-3 py-1.5 text-xs font-semibold text-chrome-100 transition hover:bg-accent-500/25"
        on:click={runSelected}
      >
        Record Selection
      </button>
    </div>
  </div>

  <div class="flex-1 overflow-auto p-1.5">
    {#if $filteredCommands.length === 0}
      <div class="rounded-md border border-dashed border-chrome-700 p-4 text-sm text-chrome-300">
        No commands match the current query.
      </div>
    {:else}
      <ul class="space-y-1">
        {#each $filteredCommands as command}
          <li>
            <button
              class:selected={$selectedCommandId === command.id}
              class="flex w-full items-start justify-between rounded-md border border-transparent px-3 py-2.5 text-left transition hover:border-chrome-600 hover:bg-chrome-800/60"
              on:click={() => choose(command)}
            >
              <div class="pr-4">
                <div class="text-sm font-semibold text-chrome-100">{command.title}</div>
                <div class="mt-0.5 text-xs leading-5 text-chrome-300">{command.subtitle}</div>
                <div class="mt-1.5 text-[11px] uppercase tracking-[0.18em] text-chrome-300">
                  {categoryLabels[command.category]}
                </div>
              </div>
              {#if command.shortcut}
                <span class="rounded border border-chrome-600 px-2 py-1 font-mono text-[11px] text-chrome-200">
                  {command.shortcut}
                </span>
              {/if}
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</section>

<style>
  .selected {
    border-color: rgba(88, 196, 220, 0.6);
    background: rgba(35, 48, 68, 0.85);
  }
</style>
