<script lang="ts">
  import { onMount } from "svelte";
  import CommandPalette from "./lib/components/CommandPalette.svelte";
  import PreviewPane from "./lib/components/PreviewPane.svelte";
  import SettingsPanel from "./lib/components/SettingsPanel.svelte";
  import StatusBar from "./lib/components/StatusBar.svelte";
  import { bootstrapApp, hideOverlay } from "./lib/ipc/client";
  import {
    appState,
    loadBootstrap,
    refreshBootstrap,
    selectCommandById,
    selectedCommand
  } from "./lib/stores/app-shell";
  let loadError: string | null = null;
  let activeTab: "console" | "settings" = "console";

  function capitalize(s: string): string {
    return s.charAt(0).toUpperCase() + s.slice(1);
  }

  declare global {
    interface Window {
      __DEVFORGE_PINNED_COMMAND__?: string;
      __DEVFORGE_STATUSBAR__?: boolean;
    }
  }

  const pinnedCommandId =
    window.__DEVFORGE_PINNED_COMMAND__ ?? new URLSearchParams(window.location.search).get("pinned");
  const isStatusBar = window.__DEVFORGE_STATUSBAR__ === true;

  onMount(() => {
    let bootstrapPoll: ReturnType<typeof setInterval> | null = null;

    const focusSearch = () => {
      const search = document.getElementById("command-search") as HTMLInputElement | null;
      search?.focus();
      search?.select();
    };

    const handleKeydown = async (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        await hideOverlay();
      }
    };

    window.addEventListener("focus", focusSearch);
    window.addEventListener("keydown", handleKeydown);

    void (async () => {
      try {
        const payload = await bootstrapApp();
        loadBootstrap(payload);
        if (pinnedCommandId) {
          selectCommandById(pinnedCommandId);
        }

        bootstrapPoll = setInterval(async () => {
          try {
            refreshBootstrap(await bootstrapApp());
          } catch {
            // Ignore background sync errors and keep the current session state.
          }
        }, 3000);

        focusSearch();
      } catch (error) {
        loadError = error instanceof Error ? error.message : "Failed to load the application.";
      }
    })();

    return () => {
      if (bootstrapPoll) {
        clearInterval(bootstrapPoll);
      }
      window.removeEventListener("focus", focusSearch);
      window.removeEventListener("keydown", handleKeydown);
    };
  });
</script>

<svelte:head>
  <title>DevForge</title>
</svelte:head>

<main class="h-screen overflow-hidden text-chrome-100">
  {#if isStatusBar}
    <StatusBar />
  {:else if pinnedCommandId}
    <section class="flex h-full flex-col">
      <PreviewPane
        command={$selectedCommand}
        pinnedModules={$appState.pinnedModules}
      />
    </section>
  {:else}
    <div class="flex h-full flex-col px-4 py-3">
      <header class="flex shrink-0 items-center justify-between px-1 pb-3">
        <div class="flex items-center gap-3">
          <span class="text-xs font-semibold uppercase tracking-[0.24em] text-accent-400">DevForge</span>
          <span class="text-xs text-chrome-600">|</span>
          <span class="text-xs text-chrome-300">{capitalize($appState.health.profile.name)}</span>
        </div>
        <div class="flex items-center gap-1">
          <span class="mr-2 text-[11px] text-chrome-400">{activeTab === "console" ? $appState.health.commandCount + " commands" : ""}</span>
          <button
            class="rounded-full px-3 py-1.5 text-xs transition {activeTab === 'console' ? 'bg-chrome-700 text-chrome-100' : 'text-chrome-300 hover:bg-chrome-700 hover:text-chrome-100'}"
            on:click={() => activeTab = "console"}
          >
            Console
          </button>
          <button
            class="rounded-full px-3 py-1.5 text-xs transition {activeTab === 'settings' ? 'bg-chrome-700 text-chrome-100' : 'text-chrome-300 hover:bg-chrome-700 hover:text-chrome-100'}"
            on:click={() => activeTab = "settings"}
          >
            Settings
          </button>
        </div>
      </header>

      {#if loadError}
        <section class="flex flex-1 items-center justify-center rounded-lg border border-signal-danger/40 bg-chrome-800 px-6 py-8">
          <div class="max-w-2xl text-sm text-signal-danger">{loadError}</div>
        </section>
      {:else}
        {#if activeTab === "console"}
          <section class="flex min-h-0 flex-1 gap-3">
            <div class="flex w-[380px] shrink-0 flex-col">
              <CommandPalette />
            </div>
            <div class="flex min-h-0 flex-1 flex-col">
              <PreviewPane
                command={$selectedCommand}
                pinnedModules={$appState.pinnedModules}
              />
            </div>
          </section>
        {:else}
          <section class="flex min-h-0 flex-1 gap-3">
            <div class="flex w-[380px] shrink-0 flex-col">
              <CommandPalette />
            </div>
            <div class="flex min-h-0 flex-1 flex-col">
              <SettingsPanel
                health={$appState.health}
                settings={$appState.settings}
                profiles={$appState.profiles}
                commands={$appState.commands}
                recentHistory={$appState.recentHistory}
                extensions={$appState.extensions}
              />
            </div>
          </section>
        {/if}
      {/if}
    </div>
  {/if}
</main>
