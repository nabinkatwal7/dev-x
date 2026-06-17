<script lang="ts">
  import { categoryLabels } from "../core/command-sections";
  import {
    executeSelectedCommand,
    filteredCommands,
    query,
    selectCommand,
    selectedCommand,
    selectedCommandId
  } from "../stores/app-shell";
  import type { CommandAction } from "../types";

  function choose(command: CommandAction) {
    selectCommand(command);
  }

  async function runSelected() {
    await executeSelectedCommand($selectedCommand, $query);
  }
</script>

<section class="flex min-h-0 flex-1 flex-col bg-chrome-800 rounded-xl">
  <div class="px-4 py-3">
    <input
      id="command-search"
      class="w-full rounded-lg bg-chrome-900 px-3 py-2.5 text-sm text-chrome-100 outline-none transition placeholder:text-chrome-400 focus:ring-1 focus:ring-accent-500/50"
      bind:value={$query}
      placeholder="Search commands, tools, snippets..."
    />
  </div>

  <div class="flex-1 overflow-auto px-2 pb-2">
    {#if $filteredCommands.length === 0}
      <div class="px-3 py-8 text-center text-sm text-chrome-400">
        No commands match the current query.
      </div>
    {:else}
      <ul class="space-y-0.5">
        {#each $filteredCommands as command}
          <li>
            <button
              class:selected={$selectedCommandId === command.id}
              class="flex w-full items-center justify-between rounded-lg px-3 py-2.5 text-left transition hover:bg-chrome-700/60"
              on:click={() => choose(command)}
            >
              <div class="min-w-0 pr-3">
                <div class="text-sm font-medium text-chrome-100">{command.title}</div>
                <div class="mt-0.5 flex items-center gap-2">
                  {#if command.subtitle}
                    <span class="truncate text-xs text-chrome-400">{command.subtitle}</span>
                  {/if}
                  <span class="shrink-0 rounded bg-chrome-700/50 px-1.5 py-0.5 text-[10px] uppercase tracking-wide text-chrome-400">{categoryLabels[command.category]}</span>
                </div>
              </div>
              {#if command.shortcut}
                <span class="shrink-0 rounded-md bg-chrome-900 px-2 py-1 font-mono text-[11px] text-chrome-400">
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
    background: rgb(30 41 59);
  }
</style>