import { useEffect, useState } from "react";
import {
  createPublicClient,
  createWalletClient,
  custom,
  encodeFunctionData,
  parseEther,
  http,
  isAddress,
  stringToHex,
  toHex,
} from "viem";
import { appConfig, getCreateCampaignConfigErrors, resolveApiUrl } from "./config.js";
import { zusProtocolAbi } from "./zusProtocolAbi.js";

const HOME_HASH = "#/";
const CAMPAIGNS_HASH = "#/campaigns";
const EMPTY_RECIPIENT = { leaf_address: "", amount: "1" };

function getCurrentRoute() {
  if (typeof window === "undefined") {
    return "home";
  }

  return window.location.hash.startsWith(CAMPAIGNS_HASH) ? "campaigns" : "home";
}

function shortAddress(value) {
  if (!value) {
    return "Not connected";
  }

  return `${value.slice(0, 6)}...${value.slice(-4)}`;
}

function shortHash(value) {
  if (!value) {
    return "";
  }

  return `${value.slice(0, 10)}...${value.slice(-8)}`;
}

function parseErrorMessage(error) {
  if (!error) {
    return "Something went wrong.";
  }

  if (typeof error === "string") {
    return error;
  }

  const candidate =
    error?.shortMessage ||
    error?.details ||
    error?.message ||
    error?.cause?.message;

  return typeof candidate === "string" ? candidate : "Something went wrong.";
}

function parseNativeAmount(label, rawValue, { allowZero = false } = {}) {
  const value = rawValue.trim();
  if (!value) {
    throw new Error(`${label} is required.`);
  }

  try {
    const parsed = parseEther(value);
    if (!allowZero && parsed <= 0n) {
      throw new Error(`${label} must be greater than 0.`);
    }

    return {
      displayValue: value,
      wei: parsed.toString(),
    };
  } catch (error) {
    if (error instanceof Error && error.message === `${label} must be greater than 0.`) {
      throw error;
    }

    throw new Error(`${label} must be a valid AVAX amount.`);
  }
}

async function readJson(response) {
  const text = await response.text();
  let payload = null;

  if (text) {
    try {
      payload = JSON.parse(text);
    } catch {
      payload = null;
    }
  }

  if (!response.ok) {
    const message =
      payload?.message ||
      payload?.error ||
      payload?.details ||
      text ||
      `Request failed with status ${response.status}`;

    throw new Error(message);
  }

  return payload;
}

function makeExplorerUrl(hash) {
  if (!hash || !appConfig.explorerBaseUrl) {
    return "";
  }

  const base = appConfig.explorerBaseUrl.endsWith("/")
    ? appConfig.explorerBaseUrl
    : `${appConfig.explorerBaseUrl}/`;

  return `${base}${hash}`;
}

function Button({
  children,
  onClick,
  variant = "solid",
  type = "button",
  disabled = false,
  className = "",
}) {
  const classes = [
    "btn",
    variant === "ghost" ? "btn-ghost" : "btn-solid",
    disabled ? "is-disabled" : "",
    className,
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <button className={classes} onClick={onClick} type={type} disabled={disabled}>
      {children}
    </button>
  );
}

function MetricCard({ label, value, detail }) {
  return (
    <article className="metric-card">
      <span className="metric-label">{label}</span>
      <strong className="metric-value">{value}</strong>
      <span className="metric-detail">{detail}</span>
    </article>
  );
}

function WalletButton({ wallet, onConnect }) {
  const label = wallet.account ? shortAddress(wallet.account) : "Connect Wallet";

  return (
    <div className="wallet-cluster">
      <Button
        variant={wallet.account ? "ghost" : "solid"}
        onClick={() => {
          void onConnect();
        }}
        disabled={wallet.connecting}
      >
        {wallet.connecting ? "Connecting..." : label}
      </Button>
      <span className="wallet-meta">
        {wallet.account
          ? wallet.chainId
            ? `Chain ${wallet.chainId}`
            : "Wallet ready"
          : "Browser wallet required"}
      </span>
    </div>
  );
}

function Navigation({ route, onNavigate, wallet, onConnect }) {
  return (
    <header className="topbar">
      <div
        className="brand-block"
        onClick={() => onNavigate("home")}
        onKeyDown={(event) => {
          if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            onNavigate("home");
          }
        }}
        role="button"
        tabIndex={0}
      >
        <span className="brand-kicker">Privacy-first rewards</span>
        <span className="brand-title">ZUS_PROTOCOL</span>
      </div>

      <nav className="topnav">
        <button
          className={`nav-link ${route === "home" ? "is-active" : ""}`}
          onClick={() => onNavigate("home")}
          type="button"
        >
          Home
        </button>
        <button
          className={`nav-link ${route === "campaigns" ? "is-active" : ""}`}
          onClick={() => onNavigate("campaigns")}
          type="button"
        >
          Campaigns
        </button>
      </nav>

      <WalletButton wallet={wallet} onConnect={onConnect} />
    </header>
  );
}

function SectionHeader({ eyebrow, title, body }) {
  return (
    <div className="section-header">
      <span className="section-eyebrow">{eyebrow}</span>
      <h2>{title}</h2>
      <p>{body}</p>
    </div>
  );
}

function StatusPanel({
  wallet,
  campaignCount,
  creatorCount,
  totalRecipients,
  pendingDeployment,
  createState,
  onNavigate,
}) {
  const explorerUrl = makeExplorerUrl(createState.txHash);

  return (
    <aside className="panel status-panel">
      <SectionHeader
        eyebrow="App status"
        title="Ready for the Rust API and the contract"
        body="Get Started jumps into the live campaign feed, and the create flow stores the Merkle campaign offchain before asking your wallet to deploy the matching onchain campaign."
      />

      <div className="status-grid">
        <MetricCard
          label="Campaigns"
          value={campaignCount.toLocaleString()}
          detail="Loaded from GET /campaigns"
        />
        <MetricCard
          label="Creators"
          value={creatorCount.toLocaleString()}
          detail="Unique wallet creators in the API"
        />
        <MetricCard
          label="Recipients"
          value={totalRecipients.toLocaleString()}
          detail="Summed from Rust API leaf counts"
        />
      </div>

      <div className="callout">
        <span className="callout-label">Wallet</span>
        <strong>{wallet.account ? shortAddress(wallet.account) : "Connect before creating"}</strong>
        <p>
          {wallet.account
            ? "The connected wallet becomes the API campaign creator and signs the contract deployment."
            : "Reading campaigns does not need a wallet, but create campaign does."}
        </p>
      </div>

      <div className="callout">
        <span className="callout-label">Environment</span>
        <strong>{appConfig.apiBaseUrl}</strong>
        <p>
          RPC: {appConfig.rpcUrl || "missing"} | Protocol:{" "}
          {appConfig.protocolAddress || "missing"}
        </p>
      </div>

      {pendingDeployment ? (
        <div className="callout warning">
          <span className="callout-label">Pending onchain step</span>
          <strong>{pendingDeployment.apiCampaign.name}</strong>
          <p>
            Rust API campaign <code>{pendingDeployment.apiCampaign.campaign_id}</code> already
            exists. Finish the wallet transaction to avoid creating a duplicate API campaign.
          </p>
          <Button variant="ghost" onClick={() => onNavigate("campaigns")}>
            Finish Deployment
          </Button>
        </div>
      ) : null}

      {createState.success ? (
        <div className="flash success">
          <strong>{createState.success}</strong>
          {createState.apiCampaign ? (
            <p>
              API campaign: <code>{createState.apiCampaign.campaign_id}</code>
            </p>
          ) : null}
          {createState.txHash ? (
            explorerUrl ? (
              <a href={explorerUrl} target="_blank" rel="noreferrer">
                View transaction {shortHash(createState.txHash)}
              </a>
            ) : (
              <p>Transaction {shortHash(createState.txHash)}</p>
            )
          ) : null}
        </div>
      ) : null}

      {createState.error ? (
        <div className="flash error">
          <strong>Create flow needs attention</strong>
          <p>{createState.error}</p>
        </div>
      ) : null}
    </aside>
  );
}

function CampaignCard({ campaign }) {
  return (
    <article className="campaign-card">
      <div className="campaign-card-top">
        <span className="campaign-tag">Campaign</span>
        <span className="campaign-id">{campaign.campaign_id}</span>
      </div>
      <h3>{campaign.name}</h3>
      <dl>
        <div>
          <dt>Creator</dt>
          <dd>{campaign.campaign_creator_address}</dd>
        </div>
        <div>
          <dt>Recipients</dt>
          <dd>{campaign.leaf_count.toLocaleString()}</dd>
        </div>
        <div>
          <dt>Onchain ID</dt>
          <dd>{campaign.onchain_campaign_id}</dd>
        </div>
        <div>
          <dt>Merkle root</dt>
          <dd>{campaign.merkle_root}</dd>
        </div>
      </dl>
    </article>
  );
}

function HomePage({ campaigns, wallet, onConnect, onNavigate }) {
  return (
    <main className="page-shell">
      <section className="hero">
        <div className="hero-copy">
          <span className="hero-kicker">Zero-knowledge reward operations</span>
          <h1>
            Rewards without public recipient leakage.
          </h1>
          <p>
            The frontend now points at the Rust campaign API by environment variable, shows the
            live campaign feed, and asks the connected wallet to create the matching onchain
            campaign through the Zus protocol contract.
          </p>
          <div className="hero-actions">
            <Button onClick={() => onNavigate("campaigns")}>Get Started</Button>
            <Button
              variant="ghost"
              onClick={() => {
                void onConnect();
              }}
            >
              {wallet.account ? "Wallet Connected" : "Connect Wallet"}
            </Button>
          </div>
          <div className="hero-note">
            API: <code>{appConfig.apiBaseUrl}</code> | RPC:{" "}
            <code>{appConfig.rpcUrl || "set VITE_RPC_URL"}</code>
          </div>
        </div>

        <div className="hero-panel panel">
          <SectionHeader
            eyebrow="Live preview"
            title="Latest campaigns from the Rust service"
            body="These are loaded from the same API endpoint the dashboard uses, so the Get Started flow takes you into a real dataset instead of a hard-coded mock."
          />
          <div className="hero-campaigns">
            {campaigns.slice(0, 3).map((campaign) => (
              <div key={campaign.campaign_id} className="mini-campaign">
                <strong>{campaign.name}</strong>
                <span>{campaign.leaf_count} recipients</span>
                <code>{shortAddress(campaign.campaign_creator_address)}</code>
              </div>
            ))}
            {campaigns.length === 0 ? (
              <div className="mini-campaign empty">
                <strong>No campaigns yet</strong>
                <span>Once the Rust API has data, it will appear here and in the dashboard.</span>
              </div>
            ) : null}
          </div>
        </div>
      </section>

      <section className="feature-band">
        <article className="feature-card">
          <span>Step 1</span>
          <h3>Create in Rust API</h3>
          <p>
            The browser sends the campaign name, creator address, and recipient matrix to the Axum
            service at <code>/campaigns</code>.
          </p>
        </article>
        <article className="feature-card">
          <span>Step 2</span>
          <h3>Deploy onchain</h3>
          <p>
            Using the wallet plus env-configured RPC, the app calls
            <code> createCampaign(bytes32,address,bytes32,bytes8,uint256)</code>.
          </p>
        </article>
        <article className="feature-card">
          <span>Step 3</span>
          <h3>Read everything back</h3>
          <p>
            The campaigns page refreshes from the Rust API so operators can verify the offchain and
            onchain identifiers line up.
          </p>
        </article>
      </section>
    </main>
  );
}

function CampaignsPage({
  campaigns,
  campaignsLoading,
  campaignsError,
  form,
  wallet,
  createState,
  pendingDeployment,
  onFormChange,
  onRecipientChange,
  onAddRecipient,
  onRemoveRecipient,
  onSubmitCreate,
  onRefresh,
}) {
  return (
    <main className="page-shell campaigns-page">
      <section className="dashboard-header">
        <div>
          <span className="section-eyebrow">Get Started</span>
          <h1>Campaign operator dashboard</h1>
          <p>
            All campaigns below come from the Rust API configured by <code>VITE_API_BASE_URL</code>
            . Creating a campaign first persists the Merkle data in Rust, then deploys the matching
            onchain campaign through the connected wallet.
          </p>
        </div>
        <div className="dashboard-actions">
          <Button variant="ghost" onClick={onRefresh} disabled={campaignsLoading}>
            {campaignsLoading ? "Refreshing..." : "Refresh Campaigns"}
          </Button>
        </div>
      </section>

      <section className="dashboard-grid">
        <article className="panel form-panel">
          <SectionHeader
            eyebrow="Create campaign"
            title="API first, contract second"
            body="Use human-readable AVAX amounts for payout and funding. Recipient rows map directly to the Rust API payload."
          />

          <div className="form-grid">
            <label className="field">
              <span>Campaign name</span>
              <input
                value={form.name}
                onChange={(event) => onFormChange("name", event.target.value)}
                placeholder="Fuji community drop"
              />
            </label>

            <label className="field">
              <span>Creator wallet</span>
              <input value={wallet.account || ""} placeholder="Connect wallet" readOnly />
            </label>

            <label className="field">
              <span>Payout amount (AVAX)</span>
              <input
                value={form.payoutAmount}
                onChange={(event) => onFormChange("payoutAmount", event.target.value)}
                inputMode="decimal"
                placeholder="0.1"
              />
            </label>

            <label className="field">
              <span>Funding amount (AVAX)</span>
              <input
                value={form.fundingAmount}
                onChange={(event) => onFormChange("fundingAmount", event.target.value)}
                inputMode="decimal"
                placeholder="1.0"
              />
            </label>
          </div>

          <div className="recipient-header">
            <div>
              <span className="section-eyebrow">Recipients</span>
              <p>At least one address and integer amount are required.</p>
            </div>
            <Button variant="ghost" onClick={onAddRecipient}>
              Add row
            </Button>
          </div>

          <div className="recipient-list">
            {form.recipients.map((recipient, index) => (
              <div className="recipient-row" key={`recipient-${index}`}>
                <label className="field">
                  <span>Wallet {index + 1}</span>
                  <input
                    value={recipient.leaf_address}
                    onChange={(event) =>
                      onRecipientChange(index, "leaf_address", event.target.value)
                    }
                    placeholder="0x..."
                  />
                </label>
                <label className="field">
                  <span>Amount</span>
                  <input
                    value={recipient.amount}
                    onChange={(event) => onRecipientChange(index, "amount", event.target.value)}
                    inputMode="numeric"
                    placeholder="1"
                  />
                </label>
                <Button
                  variant="ghost"
                  className="remove-row"
                  onClick={() => onRemoveRecipient(index)}
                  disabled={form.recipients.length === 1}
                >
                  Remove
                </Button>
              </div>
            ))}
          </div>

          <div className="form-note">
            <strong>Configured contract message:</strong> <code>{appConfig.campaignMessage}</code>
            <br />
            <strong>Protocol contract:</strong> <code>{appConfig.protocolAddress || "missing"}</code>
            <br />
            <strong>Verifier:</strong> <code>{appConfig.verifierAddress || "missing"}</code>
          </div>

          <div className="form-actions">
            <Button
              onClick={() => {
                void onSubmitCreate();
              }}
              disabled={createState.loading}
            >
              {createState.loading
                ? "Working..."
                : pendingDeployment
                  ? "Complete Onchain Deployment"
                  : "Create Campaign"}
            </Button>
            <span className="muted-copy">
              {pendingDeployment
                ? "The Rust API step already succeeded. This resumes the wallet transaction only."
                : "This will POST to the Rust API and then open the wallet transaction."}
            </span>
          </div>
        </article>

        <article className="panel campaign-feed">
          <SectionHeader
            eyebrow="All campaigns"
            title="Live Rust API feed"
            body="This list is driven by GET /campaigns so the frontend stays in sync with the backend campaign catalog."
          />

          {campaignsError ? (
            <div className="flash error">
              <strong>Could not load campaigns</strong>
              <p>{campaignsError}</p>
            </div>
          ) : null}

          {campaignsLoading ? <div className="loading-state">Loading campaigns...</div> : null}

          {!campaignsLoading && campaigns.length === 0 ? (
            <div className="empty-state">
              <strong>No campaigns returned yet.</strong>
              <p>Create one above and it will appear here after the Rust API responds.</p>
            </div>
          ) : null}

          <div className="campaign-list">
            {campaigns.map((campaign) => (
              <CampaignCard campaign={campaign} key={campaign.campaign_id} />
            ))}
          </div>
        </article>
      </section>
    </main>
  );
}

export default function App() {
  const [route, setRoute] = useState(getCurrentRoute);
  const [wallet, setWallet] = useState({
    account: "",
    chainId: "",
    connecting: false,
    error: "",
  });
  const [campaigns, setCampaigns] = useState([]);
  const [campaignsLoading, setCampaignsLoading] = useState(true);
  const [campaignsError, setCampaignsError] = useState("");
  const [refreshNonce, setRefreshNonce] = useState(0);
  const [form, setForm] = useState({
    name: "",
    payoutAmount: appConfig.defaultPayoutAmount,
    fundingAmount: appConfig.defaultFundingAmount,
    recipients: [{ ...EMPTY_RECIPIENT }],
  });
  const [pendingDeployment, setPendingDeployment] = useState(null);
  const [createState, setCreateState] = useState({
    loading: false,
    error: "",
    success: "",
    txHash: "",
    apiCampaign: null,
  });

  useEffect(() => {
    if (typeof window === "undefined") {
      return undefined;
    }

    const handleHashChange = () => setRoute(getCurrentRoute());
    window.addEventListener("hashchange", handleHashChange);

    return () => window.removeEventListener("hashchange", handleHashChange);
  }, []);

  useEffect(() => {
    if (!window.ethereum?.request) {
      return undefined;
    }

    let cancelled = false;

    const syncWallet = async () => {
      try {
        const accounts = await window.ethereum.request({ method: "eth_accounts" });
        const chainIdHex = await window.ethereum.request({ method: "eth_chainId" });
        if (cancelled) {
          return;
        }

        setWallet((current) => ({
          ...current,
          account: accounts?.[0] || "",
          chainId: chainIdHex ? Number.parseInt(chainIdHex, 16).toString() : "",
          error: "",
        }));
      } catch (error) {
        if (cancelled) {
          return;
        }

        setWallet((current) => ({
          ...current,
          error: parseErrorMessage(error),
        }));
      }
    };

    syncWallet();

    const handleAccountsChanged = (accounts) => {
      setWallet((current) => ({
        ...current,
        account: accounts?.[0] || "",
        error: "",
      }));
    };

    const handleChainChanged = (chainIdHex) => {
      setWallet((current) => ({
        ...current,
        chainId: chainIdHex ? Number.parseInt(chainIdHex, 16).toString() : "",
      }));
    };

    window.ethereum.on?.("accountsChanged", handleAccountsChanged);
    window.ethereum.on?.("chainChanged", handleChainChanged);

    return () => {
      cancelled = true;
      window.ethereum.removeListener?.("accountsChanged", handleAccountsChanged);
      window.ethereum.removeListener?.("chainChanged", handleChainChanged);
    };
  }, []);

  useEffect(() => {
    let cancelled = false;

    const loadCampaigns = async () => {
      setCampaignsLoading(true);
      setCampaignsError("");

      try {
        const data = await readJson(await fetch(resolveApiUrl("/campaigns")));
        if (cancelled) {
          return;
        }

        setCampaigns(Array.isArray(data) ? data : []);
      } catch (error) {
        if (cancelled) {
          return;
        }

        setCampaignsError(parseErrorMessage(error));
      } finally {
        if (!cancelled) {
          setCampaignsLoading(false);
        }
      }
    };

    loadCampaigns();

    return () => {
      cancelled = true;
    };
  }, [refreshNonce]);

  const campaignCount = campaigns.length;
  const creatorCount = new Set(campaigns.map((campaign) => campaign.campaign_creator_address)).size;
  const totalRecipients = campaigns.reduce(
    (sum, campaign) => sum + Number(campaign.leaf_count || 0),
    0,
  );

  const navigate = (nextRoute) => {
    if (typeof window === "undefined") {
      setRoute(nextRoute);
      return;
    }

    window.location.hash = nextRoute === "campaigns" ? CAMPAIGNS_HASH : HOME_HASH;
  };

  const connectWallet = async () => {
    if (!window.ethereum?.request) {
      setWallet((current) => ({
        ...current,
        error: "No injected wallet found. Install MetaMask or another EVM wallet.",
      }));
      throw new Error("No injected wallet found.");
    }

    setWallet((current) => ({ ...current, connecting: true, error: "" }));

    try {
      const accounts = await window.ethereum.request({ method: "eth_requestAccounts" });
      const chainIdHex = await window.ethereum.request({ method: "eth_chainId" });
      const account = accounts?.[0] || "";
      const chainId = chainIdHex ? Number.parseInt(chainIdHex, 16).toString() : "";

      setWallet((current) => ({
        ...current,
        account,
        chainId,
        connecting: false,
        error: "",
      }));

      return account;
    } catch (error) {
      const message = parseErrorMessage(error);
      setWallet((current) => ({
        ...current,
        connecting: false,
        error: message,
      }));
      throw error;
    }
  };

  const updateForm = (field, value) => {
    setForm((current) => ({
      ...current,
      [field]: value,
    }));
  };

  const updateRecipient = (index, field, value) => {
    setForm((current) => ({
      ...current,
      recipients: current.recipients.map((recipient, recipientIndex) =>
        recipientIndex === index ? { ...recipient, [field]: value } : recipient,
      ),
    }));
  };

  const addRecipient = () => {
    setForm((current) => ({
      ...current,
      recipients: [...current.recipients, { ...EMPTY_RECIPIENT }],
    }));
  };

  const removeRecipient = (index) => {
    setForm((current) => ({
      ...current,
      recipients:
        current.recipients.length === 1
          ? current.recipients
          : current.recipients.filter((_, recipientIndex) => recipientIndex !== index),
    }));
  };

  const resetForm = () => {
    setForm({
      name: "",
      payoutAmount: appConfig.defaultPayoutAmount,
      fundingAmount: appConfig.defaultFundingAmount,
      recipients: [{ ...EMPTY_RECIPIENT }],
    });
  };

  const validateForm = async () => {
    const configErrors = getCreateCampaignConfigErrors();
    if (configErrors.length > 0) {
      throw new Error(`Missing config: ${configErrors.join(", ")}`);
    }

    const name = form.name.trim();
    if (!name) {
      throw new Error("Campaign name is required.");
    }

    const payout = parseNativeAmount("Payout amount", form.payoutAmount);
    const funding = parseNativeAmount("Funding amount", form.fundingAmount, {
      allowZero: true,
    });

    const recipients = [];
    for (const [index, entry] of form.recipients.entries()) {
      const leafAddress = entry.leaf_address.trim();
      const amount = entry.amount.trim();

      if (!leafAddress && !amount) {
        continue;
      }

      if (!leafAddress || !amount) {
        throw new Error(`Recipient row ${index + 1} needs both wallet and amount.`);
      }

      if (!isAddress(leafAddress)) {
        throw new Error(`Recipient row ${index + 1} has an invalid EVM address.`);
      }

      if (!/^[0-9]+$/.test(amount)) {
        throw new Error(`Recipient row ${index + 1} amount must be an integer string.`);
      }

      recipients.push({
        leaf_address: leafAddress,
        amount,
      });
    }

    if (recipients.length === 0) {
      throw new Error("Add at least one recipient.");
    }

    const creatorAddress = wallet.account || (await connectWallet());
    if (!creatorAddress || !isAddress(creatorAddress)) {
      throw new Error("Connect a valid wallet before creating campaigns.");
    }

    return {
      name,
      payoutAmount: payout.displayValue,
      payoutWei: payout.wei,
      fundingAmount: funding.displayValue,
      fundingWei: funding.wei,
      recipients,
      creatorAddress,
    };
  };

  const deployOnchainCampaign = async (deployment) => {
    if (!window.ethereum?.request) {
      throw new Error("No injected wallet found for the contract transaction.");
    }

    const account = wallet.account || (await connectWallet());
    if (!account) {
      throw new Error("Connect a wallet before deploying the onchain campaign.");
    }

    setCreateState({
      loading: true,
      error: "",
      success: "Rust API campaign ready. Waiting for wallet signature...",
      txHash: "",
      apiCampaign: deployment.apiCampaign,
    });

    const walletClient = createWalletClient({
      transport: custom(window.ethereum),
    });

    const transactionHash = await walletClient.sendTransaction({
      account,
      to: appConfig.protocolAddress,
      value: BigInt(deployment.fundingWei),
      data: encodeFunctionData({
        abi: zusProtocolAbi,
        functionName: "createCampaign",
        args: [
          deployment.apiCampaign.onchain_campaign_id,
          appConfig.verifierAddress,
          toHex(BigInt(deployment.apiCampaign.merkle_root), { size: 32 }),
          stringToHex(appConfig.campaignMessage, { size: 8 }),
          BigInt(deployment.payoutWei),
        ],
      }),
    });

    setCreateState({
      loading: true,
      error: "",
      success: "Transaction submitted. Waiting for RPC confirmation...",
      txHash: transactionHash,
      apiCampaign: deployment.apiCampaign,
    });

    const publicClient = createPublicClient({
      transport: http(appConfig.rpcUrl),
    });

    await publicClient.waitForTransactionReceipt({ hash: transactionHash });

    setPendingDeployment(null);
    setCreateState({
      loading: false,
      error: "",
      success: "Campaign created in the Rust API and confirmed onchain.",
      txHash: transactionHash,
      apiCampaign: deployment.apiCampaign,
    });
    resetForm();
    setRefreshNonce((current) => current + 1);
  };

  const submitCreate = async () => {
    setCreateState((current) => ({
      ...current,
      error: "",
      success: "",
      txHash: current.txHash,
    }));

    if (pendingDeployment) {
      try {
        await deployOnchainCampaign(pendingDeployment);
      } catch (error) {
        setCreateState({
          loading: false,
          error: `${parseErrorMessage(error)} The Rust API campaign still exists, so use this button again to finish the onchain step without creating a duplicate API campaign.`,
          success: "",
          txHash: "",
          apiCampaign: pendingDeployment.apiCampaign,
        });
      }

      return;
    }

    let deployment = null;

    try {
      const validated = await validateForm();

      setCreateState({
        loading: true,
        error: "",
        success: "Creating campaign in the Rust API...",
        txHash: "",
        apiCampaign: null,
      });

      const apiCampaign = await readJson(
        await fetch(resolveApiUrl("/campaigns"), {
          method: "POST",
          headers: {
            "content-type": "application/json",
          },
          body: JSON.stringify({
            name: validated.name,
            campaign_creator_address: validated.creatorAddress,
            recipients: validated.recipients,
          }),
        }),
      );

      deployment = {
        apiCampaign,
        payoutAmount: validated.payoutAmount,
        payoutWei: validated.payoutWei,
        fundingAmount: validated.fundingAmount,
        fundingWei: validated.fundingWei,
      };

      setPendingDeployment(deployment);
      await deployOnchainCampaign(deployment);
    } catch (error) {
      const detail = parseErrorMessage(error);
      setCreateState({
        loading: false,
        error: deployment
          ? `${detail} Rust API campaign ${deployment.apiCampaign.campaign_id} was created, but the contract transaction did not finish. Resume with "Complete Onchain Deployment".`
          : detail,
        success: "",
        txHash: "",
        apiCampaign: deployment?.apiCampaign || null,
      });
    }
  };

  return (
    <div className="app-frame">
      <Navigation route={route} onNavigate={navigate} wallet={wallet} onConnect={connectWallet} />

      {wallet.error ? (
        <div className="global-banner error">
          <strong>Wallet</strong>
          <span>{wallet.error}</span>
        </div>
      ) : null}

      <StatusPanel
        wallet={wallet}
        campaignCount={campaignCount}
        creatorCount={creatorCount}
        totalRecipients={totalRecipients}
        pendingDeployment={pendingDeployment}
        createState={createState}
        onNavigate={navigate}
      />

      {route === "campaigns" ? (
        <CampaignsPage
          campaigns={campaigns}
          campaignsLoading={campaignsLoading}
          campaignsError={campaignsError}
          form={form}
          wallet={wallet}
          createState={createState}
          pendingDeployment={pendingDeployment}
          onFormChange={updateForm}
          onRecipientChange={updateRecipient}
          onAddRecipient={addRecipient}
          onRemoveRecipient={removeRecipient}
          onSubmitCreate={submitCreate}
          onRefresh={() => setRefreshNonce((current) => current + 1)}
        />
      ) : (
        <HomePage
          campaigns={campaigns}
          wallet={wallet}
          onConnect={connectWallet}
          onNavigate={navigate}
        />
      )}
    </div>
  );
}
