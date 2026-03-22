import { formatEther, isAddress } from "viem";

const DEFAULT_API_BASE_URL = "http://127.0.0.1:3000";
const DEFAULT_RPC_URL = "https://avalanche-fuji.drpc.org";
const DEFAULT_EXPLORER_BASE_URL = "https://testnet.snowtrace.io/tx/";
const DEFAULT_CHAIN_ID = 43113;
const DEFAULT_CHAIN_HEX_ID = "0xa869";
const DEFAULT_NETWORK_NAME = "Avalanche Fuji";
const DEFAULT_EXPLORER_SITE_URL = "https://testnet.snowtrace.io/";

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
  rpcUrl: cleanValue(import.meta.env.VITE_RPC_URL, DEFAULT_RPC_URL),
  protocolAddress: cleanValue(import.meta.env.VITE_ZUS_PROTOCOL_ADDRESS),
  verifierAddress: cleanValue(import.meta.env.VITE_ZUS_VERIFIER_ADDRESS),
  campaignMessage: cleanValue(import.meta.env.VITE_ZUS_CAMPAIGN_MESSAGE, "ZUSMVP01"),
  defaultPayoutAvax: defaultReadableAmount(
    import.meta.env.VITE_ZUS_DEFAULT_PAYOUT_AVAX,
    import.meta.env.VITE_ZUS_DEFAULT_PAYOUT_WEI,
    "0.0001",
  ),
  defaultFundingAvax: defaultReadableAmount(
    import.meta.env.VITE_ZUS_DEFAULT_FUNDING_AVAX,
    import.meta.env.VITE_ZUS_DEFAULT_FUNDING_WEI,
    "0.0001",
  ),
  defaultPayoutWei: cleanValue(import.meta.env.VITE_ZUS_DEFAULT_PAYOUT_WEI, "100000000000000"),
  defaultFundingWei: cleanValue(import.meta.env.VITE_ZUS_DEFAULT_FUNDING_WEI, "100000000000000"),
  explorerBaseUrl: cleanValue(import.meta.env.VITE_EXPLORER_BASE_URL, DEFAULT_EXPLORER_BASE_URL),
  explorerSiteUrl: cleanValue(import.meta.env.VITE_EXPLORER_SITE_URL, DEFAULT_EXPLORER_SITE_URL),
  chainId: Number.parseInt(cleanValue(import.meta.env.VITE_CHAIN_ID, `${DEFAULT_CHAIN_ID}`), 10),
  chainHexId: cleanValue(import.meta.env.VITE_CHAIN_HEX_ID, DEFAULT_CHAIN_HEX_ID),
  networkName: cleanValue(import.meta.env.VITE_NETWORK_NAME, DEFAULT_NETWORK_NAME),
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
