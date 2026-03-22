import { formatEther, isAddress } from "viem";

const DEFAULT_API_BASE_URL = "http://127.0.0.1:3000";

function cleanValue(value, fallback = "") {
  return typeof value === "string" && value.trim() ? value.trim() : fallback;
}

function defaultReadableAmount(readableEnv, weiEnv, fallback) {
  const readable = cleanValue(readableEnv);
  if (readable) {
    return readable;
  }

  const wei = cleanValue(weiEnv);
  if (wei && /^[0-9]+$/.test(wei)) {
    try {
      return formatEther(BigInt(wei));
    } catch {
      return fallback;
    }
  }

  return fallback;
}

export const appConfig = {
  apiBaseUrl: cleanValue(import.meta.env.VITE_API_BASE_URL, DEFAULT_API_BASE_URL),
  rpcUrl: cleanValue(import.meta.env.VITE_RPC_URL),
  protocolAddress: cleanValue(import.meta.env.VITE_ZUS_PROTOCOL_ADDRESS),
  verifierAddress: cleanValue(import.meta.env.VITE_ZUS_VERIFIER_ADDRESS),
  campaignMessage: cleanValue(import.meta.env.VITE_ZUS_CAMPAIGN_MESSAGE, "ZUSMVP01"),
  defaultPayoutAmount: defaultReadableAmount(
    import.meta.env.VITE_ZUS_DEFAULT_PAYOUT_AVAX,
    import.meta.env.VITE_ZUS_DEFAULT_PAYOUT_WEI,
    "0.0001",
  ),
  defaultFundingAmount: defaultReadableAmount(
    import.meta.env.VITE_ZUS_DEFAULT_FUNDING_AVAX,
    import.meta.env.VITE_ZUS_DEFAULT_FUNDING_WEI,
    "0.0001",
  ),
  explorerBaseUrl: cleanValue(import.meta.env.VITE_EXPLORER_BASE_URL),
};

export function resolveApiUrl(path) {
  const base = appConfig.apiBaseUrl;

  if (base.startsWith("http://") || base.startsWith("https://")) {
    const normalizedBase = base.endsWith("/") ? base : `${base}/`;
    return new URL(path.replace(/^\//, ""), normalizedBase).toString();
  }

  const normalizedPath = path.startsWith("/") ? path : `/${path}`;
  return `${base.replace(/\/$/, "")}${normalizedPath}`;
}

export function getCreateCampaignConfigErrors() {
  const issues = [];

  if (!appConfig.apiBaseUrl) {
    issues.push("VITE_API_BASE_URL");
  }

  if (!appConfig.rpcUrl) {
    issues.push("VITE_RPC_URL");
  }

  if (!isAddress(appConfig.protocolAddress)) {
    issues.push("VITE_ZUS_PROTOCOL_ADDRESS");
  }

  if (!isAddress(appConfig.verifierAddress)) {
    issues.push("VITE_ZUS_VERIFIER_ADDRESS");
  }

  if (new TextEncoder().encode(appConfig.campaignMessage).length !== 8) {
    issues.push("VITE_ZUS_CAMPAIGN_MESSAGE(8 ASCII bytes)");
  }

  return issues;
}
