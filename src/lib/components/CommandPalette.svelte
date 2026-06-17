<script lang="ts">
  import { categoryLabels } from "../core/command-sections";
  import { filteredCommands, query, selectedCommandId } from "../stores/app-shell";
  import type { CommandAction } from "../types";

  export let commandCount = 0;

  function choose(command: CommandAction) {
    selectedCommandId.set(command.id);
  }
</script>

<section class="flex h-full flex-col rounded-lg border border-chrome-700/70 bg-chrome-900/70 shadow-overlay backdrop-blur">
  <div class="border-b border-chrome-700/70 px-5 py-4">
    <div class="mb-3 flex items-center justify-between text-xs uppercase tracking-[0.18em] text-chrome-300">
      <span>Command Palette</span>
      <span>{commandCount} indexed</span>
    </div>
    <input
      class="w-full rounded-md border border-chrome-600 bg-chrome-950 px-4 py-3 text-base text-chrome-100 outline-none transition focus:border-accent-400"
      bind:value={$query}
      placeholder="Search commands, tools, snippets, actions..."
    />
  </div>

  <div class="flex-1 overflow-auto p-2">
    {#if $filteredCommands.length === 0}
      <div class="rounded-md border border-dashed border-chrome-700 p-5 text-sm text-chrome-300">
        No commands match the current query.
      </div>
    {:else}
      <ul class="space-y-1">
        {#each $filteredCommands as command}
          <li>
            <button
              class:selected={$selectedCommandId === command.id}
              class="flex w-full items-start justify-between rounded-md border border-transparent px-3 py-3 text-left transition hover:border-chrome-600 hover:bg-chrome-800/60"
              on:click={() => choose(command)}
            >
              <div class="pr-4">
                <div class="text-sm font-semibold text-chrome-100">{command.title}</div>
                <div class="mt-1 text-sm text-chrome-300">{command.subtitle}</div>
                <div class="mt-2 text-xs uppercase tracking-[0.18em] text-chrome-300">
                  {categoryLabels[command.category]}
                </div>
              </div>
              {#if command.shortcut}
                <span class="rounded border border-chrome-600 px-2 py-1 font-mono text-xs text-chrome-200">
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
