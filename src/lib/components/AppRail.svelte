<script lang="ts">
  import { categoryLabels, orderedCategories } from "../core/command-sections";
  import {
    activeCategoryFilter,
    appState,
    recentCommands,
    selectCommand,
    setCategoryFilter
  } from "../stores/app-shell";
  import type { CommandAction, CommandCategory } from "../types";

  export let activeView: "workbench" | "history" | "settings" = "workbench";
  export let onNavigate: (view: "workbench" | "history" | "settings") => void;

  function openRecent(command: CommandAction) {
    onNavigate("workbench");
    selectCommand(command);
  }

  function chooseCategory(category: CommandCategory | "all") {
    onNavigate("workbench");
    setCategoryFilter(category);
  }
</script>

<aside class="flex h-full w-[160px] shrink-0 flex-col rounded-xl border border-chrome-800 bg-chrome-900">
  <div class="border-b border-chrome-800 px-3 py-3">
    <div class="text-xs font-semibold text-chrome-100">DevForge</div>
    <div class="mt-1 text-[11px] text-chrome-500">{$appState.health.profile.name}</div>
  </div>

  <div class="border-b border-chrome-800 p-2">
    <button
      class="mb-1 w-full rounded-lg px-3 py-2 text-left text-sm transition {activeView === 'workbench' ? 'bg-chrome-800 text-chrome-100' : 'text-chrome-300 hover:bg-chrome-800'}"
      on:click={() => onNavigate("workbench")}
    >
      Tools
    </button>
    <button
      class="mb-1 w-full rounded-lg px-3 py-2 text-left text-sm transition {activeView === 'history' ? 'bg-chrome-800 text-chrome-100' : 'text-chrome-300 hover:bg-chrome-800'}"
      on:click={() => onNavigate("history")}
    >
      History
    </button>
    <button
      class="w-full rounded-lg px-3 py-2 text-left text-sm transition {activeView === 'settings' ? 'bg-chrome-800 text-chrome-100' : 'text-chrome-300 hover:bg-chrome-800'}"
      on:click={() => onNavigate("settings")}
    >
      Settings
    </button>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto p-2">
    <div class="mb-2 px-2 text-[10px] uppercase tracking-[0.16em] text-chrome-500">Scope</div>
    <button
      class="mb-1 w-full rounded-lg px-3 py-2 text-left text-sm transition {$activeCategoryFilter === 'all' ? 'bg-chrome-800 text-chrome-100' : 'text-chrome-300 hover:bg-chrome-800'}"
      on:click={() => chooseCategory("all")}
    >
      All tools
    </button>
    {#each orderedCategories as category}
      <button
        class="mb-1 w-full rounded-lg px-3 py-2 text-left text-sm transition {$activeCategoryFilter === category ? 'bg-chrome-800 text-chrome-100' : 'text-chrome-300 hover:bg-chrome-800'}"
        on:click={() => chooseCategory(category)}
      >
        {categoryLabels[category]}
      </button>
    {/each}

    {#if $recentCommands.length > 0}
      <div class="mt-4 border-t border-chrome-800 pt-3">
        <div class="mb-2 px-2 text-[10px] uppercase tracking-[0.16em] text-chrome-500">Recent</div>
        {#each $recentCommands.slice(0, 4) as command}
          <button
            class="mb-1 w-full rounded-lg px-3 py-2 text-left text-sm text-chrome-300 transition hover:bg-chrome-800 hover:text-chrome-100"
            on:click={() => openRecent(command)}
          >
            <div class="truncate">{command.title}</div>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</aside>
