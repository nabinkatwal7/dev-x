<script lang="ts">
  import { categoryLabels } from "../core/command-sections";
  import { getToolSpec } from "../core/tool-meta";
  import {
    activeCategoryFilter,
    applyCommandTemplate,
    commandRunning,
    executeSelectedCommand,
    filteredCommands,
    query,
    recentCommands,
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

  function navigate(offset: number) {
    if ($filteredCommands.length === 0) return;
    const currentIndex = $filteredCommands.findIndex((command) => command.id === $selectedCommandId);
    const nextIndex = currentIndex === -1
      ? 0
      : (currentIndex + offset + $filteredCommands.length) % $filteredCommands.length;
    selectCommand($filteredCommands[nextIndex]);
  }

  function handleSearchKeydown(event: KeyboardEvent) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      navigate(1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      navigate(-1);
    } else if (event.key === "Enter") {
      event.preventDefault();
      void runSelected();
    }
  }
</script>

<section class="flex min-h-0 flex-1 flex-col rounded-xl border border-chrome-800 bg-chrome-900">
  <div class="border-b border-chrome-800 px-3 py-3">
    <div class="flex items-center justify-between gap-3">
      <div class="text-sm text-chrome-300">
        {$activeCategoryFilter === "all" ? "All tools" : categoryLabels[$activeCategoryFilter]}
      </div>
      <button
        class="rounded-lg border border-chrome-700 bg-chrome-950 px-3 py-2 text-[11px] text-chrome-300 transition hover:text-chrome-100"
        disabled={$commandRunning}
        on:click={runSelected}
      >
        {$commandRunning ? "Running" : "Run selected"}
      </button>
    </div>

    <div class="mt-4">
      <input
        id="command-search"
        class="w-full rounded-lg border border-chrome-700 bg-chrome-950 px-3 py-2.5 text-sm text-chrome-100 outline-none transition placeholder:text-chrome-500 focus:border-accent-500/50 focus:ring-1 focus:ring-accent-500/20"
        bind:value={$query}
        placeholder="Search tools"
        on:keydown={handleSearchKeydown}
      />
    </div>

    {#if $selectedCommand}
      <div class="mt-3 rounded-lg border border-chrome-800 bg-chrome-950 p-3">
        <div class="flex items-start justify-between gap-3">
          <div class="min-w-0">
            <div class="truncate text-sm font-semibold text-chrome-100">{$selectedCommand.title}</div>
            <div class="mt-1 truncate text-[11px] text-chrome-500">{$selectedCommand.id}</div>
          </div>
        </div>
        {#if getToolSpec($selectedCommand).sample}
          <button
            class="mt-3 w-full rounded-lg bg-chrome-800 px-3 py-2 text-left text-[11px] text-chrome-300 transition hover:text-chrome-100"
            on:click={() => applyCommandTemplate(getToolSpec($selectedCommand).sample ?? "")}
          >
            Load sample input
          </button>
        {/if}
      </div>
    {/if}
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto p-2">
    {#if $recentCommands.length > 0 && !$query.trim()}
      <div class="mb-4">
        <div class="px-2 pb-2 text-[10px] uppercase tracking-[0.18em] text-chrome-500">Return to recent</div>
        <div class="space-y-1">
          {#each $recentCommands as command}
            <button
              class="flex w-full items-center gap-3 rounded-lg border border-chrome-800 bg-chrome-950 px-3 py-2.5 text-left transition hover:bg-chrome-800"
              on:click={() => choose(command)}
            >
              <div class="min-w-0">
                <div class="truncate text-sm text-chrome-100">{command.title}</div>
                <div class="truncate text-[11px] text-chrome-500">{command.id}</div>
              </div>
            </button>
          {/each}
        </div>
      </div>
    {/if}

    {#if $filteredCommands.length === 0}
      <div class="rounded-lg border border-chrome-800 bg-chrome-950 px-4 py-10 text-center text-sm text-chrome-500">
        No commands match the current search and category filter.
      </div>
    {:else}
      <ul class="space-y-1">
        {#each $filteredCommands as command}
          <li>
            <button
              class:selected={$selectedCommandId === command.id}
              class="group flex w-full items-start justify-between gap-3 rounded-lg border border-chrome-800 bg-chrome-950 px-3 py-2.5 text-left transition hover:bg-chrome-800"
              on:click={() => choose(command)}
            >
              <div class="min-w-0">
                <div class="text-sm font-semibold text-chrome-100">{command.title}</div>
                <div class="mt-1 line-clamp-1 text-[11px] text-chrome-500">{command.subtitle}</div>
              </div>
              <div class="shrink-0 text-[10px] text-chrome-600">
                {command.id}
              </div>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</section>

<style>
  .selected {
    border-color: rgba(56, 189, 248, 0.28);
    background: rgba(15, 23, 42, 0.96);
  }
</style>
