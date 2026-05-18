import { invoke } from "@tauri-apps/api/core";

type Provider = "netease" | "bilibili";

type ProviderStatus = {
  provider: Provider;
  is_logged_in: boolean;
  session_path: string;
  summary: string;
};

const providerCopy: Record<
  Provider,
  {
    title: string;
    openButton: string;
    captureButton: string;
    loginUrl: string;
    intro: string;
    openSuccess: string;
    captureSuccess: string;
    clearSuccess: string;
  }
> = {
  netease: {
    title: "NetEase Cloud Music",
    openButton: "Open NetEase Login",
    captureButton: "Capture MUSIC_U",
    loginUrl: "music.163.com",
    intro: "Open the NetEase page in this window, log in, then capture the MUSIC_U cookie.",
    openSuccess: "Main webview navigated to music.163.com. Complete the login there, then capture cookies here.",
    captureSuccess: "Captured MUSIC_U from the Tauri webview and saved the NetEase session.",
    clearSuccess: "Cleared the saved NetEase session.",
  },
  bilibili: {
    title: "Bilibili",
    openButton: "Open Bilibili Login",
    captureButton: "Capture Bilibili Cookies",
    loginUrl: "www.bilibili.com",
    intro: "Open Bilibili in this window, log in there, then capture SESSDATA and the related session cookies.",
    openSuccess: "Main webview navigated to bilibili.com. Complete the login there, then capture cookies here.",
    captureSuccess: "Captured SESSDATA and related Bilibili cookies from the Tauri webview and saved the session.",
    clearSuccess: "Cleared the saved Bilibili session.",
  },
};

let currentProvider: Provider = "netease";
let titleEl: HTMLElement | null;
let providerHintEl: HTMLElement | null;
let statusValueEl: HTMLElement | null;
let sessionPathEl: HTMLElement | null;
let summaryEl: HTMLElement | null;
let messageEl: HTMLElement | null;
let openButtonEl: HTMLButtonElement | null;
let captureButtonEl: HTMLButtonElement | null;

function setMessage(message: string, kind: "info" | "error" = "info") {
  if (!messageEl) return;
  messageEl.textContent = message;
  messageEl.dataset.kind = kind;
}

function applyProviderCopy(provider: Provider) {
  const copy = providerCopy[provider];
  if (titleEl) titleEl.textContent = `Login to ${copy.title} in the embedded webview`;
  if (providerHintEl) providerHintEl.textContent = copy.intro;
  if (openButtonEl) openButtonEl.textContent = copy.openButton;
  if (captureButtonEl) captureButtonEl.textContent = copy.captureButton;

  document.querySelectorAll<HTMLButtonElement>("[data-provider]").forEach((button) => {
    button.dataset.active = button.dataset.provider === provider ? "true" : "false";
  });

  document.querySelectorAll<HTMLElement>("[data-provider-copy]").forEach((node) => {
    const key = node.dataset.providerCopy;
    if (key === "login-url") {
      node.textContent = copy.loginUrl;
    }
  });
}

function renderStatus(status: ProviderStatus) {
  if (statusValueEl) {
    statusValueEl.textContent = status.is_logged_in ? "Logged in" : "Not logged in";
  }
  if (sessionPathEl) {
    sessionPathEl.textContent = status.session_path;
  }
  if (summaryEl) {
    summaryEl.textContent = status.summary;
  }
}

async function refreshStatus(provider = currentProvider) {
  const status = await invoke<ProviderStatus>("login_status", { provider });
  if (provider === currentProvider) {
    renderStatus(status);
  }
  return status;
}

async function runAction<T>(action: () => Promise<T>, successMessage?: string) {
  try {
    const result = await action();
    if (successMessage) {
      setMessage(successMessage, "info");
    }
    return result;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    setMessage(message, "error");
    throw error;
  }
}

async function switchProvider(provider: Provider) {
  currentProvider = provider;
  applyProviderCopy(provider);
  try {
    const status = await refreshStatus(provider);
    renderStatus(status);
    setMessage(providerCopy[provider].intro, "info");
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    setMessage(message, "error");
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  titleEl = document.querySelector("#provider-title");
  providerHintEl = document.querySelector("#provider-hint");
  statusValueEl = document.querySelector("#status-value");
  sessionPathEl = document.querySelector("#session-path");
  summaryEl = document.querySelector("#summary");
  messageEl = document.querySelector("#message");
  openButtonEl = document.querySelector("#open-login");
  captureButtonEl = document.querySelector("#capture-login");

  document.querySelectorAll<HTMLButtonElement>("[data-provider]").forEach((button) => {
    button.addEventListener("click", async () => {
      await switchProvider(button.dataset.provider as Provider);
    });
  });

  openButtonEl?.addEventListener("click", async () => {
    const provider = currentProvider;
    await runAction(
      () => invoke("open_login", { provider }),
      providerCopy[provider].openSuccess,
    );
  });

  captureButtonEl?.addEventListener("click", async () => {
    const provider = currentProvider;
    const status = await runAction(
      () => invoke<ProviderStatus>("capture_login_cookie", { provider }),
      providerCopy[provider].captureSuccess,
    );
    renderStatus(status);
  });

  document.querySelector<HTMLButtonElement>("#refresh-status")?.addEventListener("click", async () => {
    const status = await runAction(() => refreshStatus(), "Session status refreshed.");
    renderStatus(status);
  });

  document.querySelector<HTMLButtonElement>("#logout")?.addEventListener("click", async () => {
    const provider = currentProvider;
    const status = await runAction(
      () => invoke<ProviderStatus>("logout", { provider }),
      providerCopy[provider].clearSuccess,
    );
    renderStatus(status);
  });

  await switchProvider("netease");
});
