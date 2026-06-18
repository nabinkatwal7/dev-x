<script lang="ts">
  import { onMount } from "svelte";
  import AppRail from "./lib/components/AppRail.svelte";
  import CommandPalette from "./lib/components/CommandPalette.svelte";
  import HistoryPanel from "./lib/components/HistoryPanel.svelte";
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
  let activeView: "workbench" | "history" | "settings" = "workbench";

  function navigate(view: "workbench" | "history" | "settings") {
    activeView = view;
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
    <div class="h-full overflow-hidden bg-chrome-950 p-2">
      <div class="flex h-full min-h-0 gap-2">
        <AppRail activeView={activeView} onNavigate={navigate} />

        {#if loadError}
          <section class="flex flex-1 items-center justify-center rounded-2xl border border-signal-danger/40 bg-chrome-900 px-6 py-8">
            <div class="max-w-2xl text-sm text-signal-danger">{loadError}</div>
          </section>
        {:else if activeView === "workbench"}
          <section class="flex min-h-0 flex-1 gap-2">
            <div class="flex w-[300px] shrink-0 flex-col">
              <CommandPalette />
            </div>
            <div class="flex min-h-0 flex-1 flex-col">
              <PreviewPane
                command={$selectedCommand}
                pinnedModules={$appState.pinnedModules}
              />
            </div>
          </section>
        {:else if activeView === "history"}
          <HistoryPanel />
        {:else}
          <section class="flex min-h-0 flex-1 flex-col">
            <SettingsPanel
              health={$appState.health}
              settings={$appState.settings}
              profiles={$appState.profiles}
              commands={$appState.commands}
              recentHistory={$appState.recentHistory}
              extensions={$appState.extensions}
            />
          </section>
        {/if}
      </div>
    </div>
  {/if}
</main>
