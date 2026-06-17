import "./app.css";
import { mount } from "svelte";
import App from "./App.svelte";

function ensureAppTarget() {
  let target = document.getElementById("app");
  if (target) {
    return target;
  }

  target = document.createElement("div");
  target.id = "app";
  document.body.appendChild(target);
  return target;
}

function renderBootstrapError(title: string, detail: string) {
  document.body.innerHTML = `
    <div style="
      min-height: 100vh;
      margin: 0;
      padding: 24px;
      background: rgb(13 17 23);
      color: rgb(229 237 249);
      font-family: 'Fira Code', Consolas, monospace;
      box-sizing: border-box;
    ">
      <div style="
        max-width: 920px;
        margin: 0 auto;
        border: 1px solid rgb(35 48 68);
        background: rgb(19 26 34);
        border-radius: 8px;
        padding: 20px;
      ">
        <div style="font-size: 11px; text-transform: uppercase; letter-spacing: 0.18em; color: rgb(149 167 191);">
          DevForge Runtime Error
        </div>
        <h1 style="font-size: 20px; margin: 8px 0 0;">${escapeHtml(title)}</h1>
        <pre style="
          margin-top: 16px;
          white-space: pre-wrap;
          word-break: break-word;
          overflow-wrap: anywhere;
          border: 1px solid rgb(35 48 68);
          background: rgb(13 17 23);
          border-radius: 6px;
          padding: 12px;
          color: rgb(251 113 133);
        ">${escapeHtml(detail)}</pre>
      </div>
    </div>
  `;
}

function escapeHtml(value: string) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

window.addEventListener("error", (event) => {
  const error = event.error instanceof Error ? event.error.stack ?? event.error.message : String(event.message);
  renderBootstrapError("Unhandled window error", error);
});

window.addEventListener("unhandledrejection", (event) => {
  const reason = event.reason instanceof Error ? event.reason.stack ?? event.reason.message : String(event.reason);
  renderBootstrapError("Unhandled promise rejection", reason);
});

let app;

try {
  app = mount(App, {
    target: ensureAppTarget()
  });
} catch (error) {
  const detail = error instanceof Error ? error.stack ?? error.message : String(error);
  renderBootstrapError("Application bootstrap failed", detail);
  throw error;
}

export default app;
