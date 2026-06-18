<script lang="ts">
  import {
    appState,
    historyLoading,
    historyQuery,
    historyResults,
    rerunHistoryEntry,
    searchHistory
  } from "../stores/app-shell";

  let localSearch = "";

  $: if (!$historyLoading && localSearch !== $historyQuery) {
    localSearch = $historyQuery;
  }
</script>

<section class="flex min-h-0 flex-1 flex-col rounded-[28px] border border-chrome-700/40 bg-[linear-gradient(180deg,rgba(14,18,25,0.98),rgba(10,13,19,0.96))]">
  <div class="border-b border-chrome-800/80 px-6 py-5">
    <div class="flex flex-wrap items-end justify-between gap-4">
      <div>
        <div class="text-[11px] uppercase tracking-[0.22em] text-accent-400">Navigation</div>
        <h2 class="mt-2 text-2xl font-semibold text-chrome-100">Action history</h2>
        <p class="mt-1 text-sm text-chrome-400">Search across previous runs and jump back into any tool with its original input.</p>
      </div>
      <div class="min-w-[280px] flex-1 max-w-[420px]">
        <input
          class="w-full rounded-2xl border border-chrome-700 bg-chrome-950/80 px-4 py-3 text-sm text-chrome-100 outline-none transition focus:border-accent-500/50 focus:ring-2 focus:ring-accent-500/15"
          bind:value={localSearch}
          placeholder="Search command id, query, or input"
          on:input={() => searchHistory(localSearch)}
        />
      </div>
    </div>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto p-6">
    <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
      {#if $historyLoading}
        <div class="rounded-[24px] border border-chrome-800 bg-chrome-900/65 p-5 text-sm text-chrome-400">Searching history...</div>
      {:else if $historyResults.length === 0}
        <div class="rounded-[24px] border border-chrome-800 bg-chrome-900/65 p-5 text-sm text-chrome-400">No matching executions.</div>
      {:else}
        {#each $historyResults as item}
          <article class="flex min-h-[220px] flex-col rounded-[24px] border border-chrome-800 bg-chrome-900/65 p-5 shadow-[0_10px_30px_rgba(0,0,0,0.16)]">
            <div class="flex items-start justify-between gap-3">
              <div class="min-w-0">
                <div class="truncate text-sm font-semibold text-chrome-100">{item.commandId}</div>
                <div class="mt-1 text-[11px] uppercase tracking-[0.14em] text-chrome-500">{item.executedAt}</div>
              </div>
              <span class="rounded-full border border-chrome-700 bg-chrome-950/80 px-2.5 py-1 text-[10px] text-chrome-400">
                {$appState.commands.find((command) => command.id === item.commandId)?.title ?? "Tool"}
              </span>
            </div>
            <div class="mt-4 rounded-2xl bg-chrome-950/80 p-4">
              <div class="text-[10px] uppercase tracking-[0.16em] text-chrome-500">Query</div>
              <div class="mt-2 text-sm text-chrome-200">{item.queryText || "No query text recorded."}</div>
            </div>
            <div class="mt-3 flex-1 overflow-hidden rounded-2xl border border-chrome-800 bg-chrome-950/60 p-4">
              <div class="text-[10px] uppercase tracking-[0.16em] text-chrome-500">Input snapshot</div>
              <pre class="mt-2 h-full overflow-auto whitespace-pre-wrap text-xs leading-5 text-chrome-300">{item.inputText || "No input payload."}</pre>
            </div>
            <button
              class="mt-4 rounded-2xl bg-accent-500/15 px-4 py-3 text-sm text-accent-400 transition hover:bg-accent-500/25"
              on:click={() => rerunHistoryEntry(item)}
            >
              Re-open tool
            </button>
          </article>
        {/each}
      {/if}
    </div>
  </div>
</section>
