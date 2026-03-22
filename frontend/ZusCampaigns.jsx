import { useState, useEffect, useRef } from "react";

/* ─── shared tokens ─── */
const CYAN = "#00ffc8";
const CYAN_DIM = "#00ddb0";
const CYAN_MID = "#00c49a";
const BG = "#020d0f";
const BG2 = "#041418";
const MUTED = "#3a6660";
const MUTED2 = "#2a5550";
const TEXT = "#cce8e4";
const MONO = "'Share Tech Mono', monospace";
const BORDER = "rgba(0,255,200,.08)";
const BORDER_HOV = "rgba(0,255,200,.25)";

/* ─── pixel cat (same as landing) ─── */
function PixelCat() {
  const [hov, setHov] = useState(false);
  const pixels = [
    "0011011100","0111111110","1111111111",
    "1010110101","1111111111","0111111110",
    "0101000101","0101000101",
  ];
  return (
    <div onMouseEnter={() => setHov(true)} onMouseLeave={() => setHov(false)}
      style={{ cursor:"pointer", display:"inline-block", transition:"transform .2s", transform: hov ? "scale(1.3) translateY(-2px)" : "scale(1)" }}
      title="=^._.^=">
      {pixels.map((row, ri) => (
        <div key={ri} style={{ display:"flex" }}>
          {row.split("").map((px, ci) => (
            <div key={ci} style={{
              width:3, height:3,
              background: px==="1" ? (hov ? CYAN : CYAN_MID) : "transparent",
              boxShadow: px==="1" && hov ? "0 0 4px rgba(0,255,200,.6)" : "none",
              transition:"background .2s, box-shadow .2s",
            }}/>
          ))}
        </div>
      ))}
    </div>
  );
}

/* ─── neon button ─── */
function Btn({ children, outline, small, onClick, disabled }) {
  const [hov, setHov] = useState(false);
  const base = {
    fontFamily: MONO, fontSize: small ? 9 : 10, letterSpacing: 2,
    textTransform:"uppercase", padding: small ? "7px 16px" : "11px 24px",
    cursor: disabled ? "not-allowed" : "pointer",
    border:"1px solid", transition:"all .25s",
    opacity: disabled ? 0.4 : 1,
  };
  if (outline) return (
    <button onClick={onClick} disabled={disabled}
      onMouseEnter={() => !disabled && setHov(true)} onMouseLeave={() => setHov(false)}
      style={{ ...base, background:"transparent", color: hov ? CYAN : MUTED,
        borderColor: hov ? CYAN : "#1a4040",
        boxShadow: hov ? "0 0 18px rgba(0,255,200,.35),inset 0 0 18px rgba(0,255,200,.04)" : "none" }}>
      {children}
    </button>
  );
  return (
    <button onClick={onClick} disabled={disabled}
      onMouseEnter={() => !disabled && setHov(true)} onMouseLeave={() => setHov(false)}
      style={{ ...base, background: hov ? CYAN : CYAN_DIM, color: BG,
        borderColor: hov ? CYAN : CYAN_DIM, fontWeight:700,
        boxShadow: hov ? "0 0 24px rgba(0,255,200,.7),0 0 48px rgba(0,255,200,.3)" : "0 0 10px rgba(0,255,200,.2)" }}>
      {children}
    </button>
  );
}

/* ─── toggle switch ─── */
function Toggle({ on, onChange }) {
  return (
    <div onClick={() => onChange(!on)} style={{
      width:36, height:18, borderRadius:9,
      background: on ? "rgba(0,255,200,.2)" : "rgba(0,255,200,.05)",
      border: `1px solid ${on ? CYAN_DIM : BORDER}`,
      position:"relative", cursor:"pointer",
      transition:"all .3s",
      boxShadow: on ? "0 0 10px rgba(0,255,200,.3)" : "none",
    }}>
      <div style={{
        width:12, height:12, borderRadius:"50%",
        background: on ? CYAN : MUTED2,
        position:"absolute", top:2,
        left: on ? 20 : 2,
        transition:"left .3s, background .3s",
        boxShadow: on ? "0 0 6px rgba(0,255,200,.8)" : "none",
      }}/>
    </div>
  );
}

/* ─── typewriter hook ─── */
function useTypewriter(text, speed=22, delay=0) {
  const [out, setOut] = useState("");
  const [done, setDone] = useState(false);
  useEffect(() => {
    let i = 0, alive = true;
    const t = setTimeout(function tick() {
      if (!alive) return;
      if (i <= text.length) { setOut(text.slice(0, i)); i++; setTimeout(tick, speed); }
      else setDone(true);
    }, delay);
    return () => { alive = false; clearTimeout(t); };
  }, [text]);
  return { out, done };
}

/* ─── JSON preview panel with typewriter ─── */
const JSON_TEXT = `{
  "campaign_id": "ZUS-7781-ALPHA",
  "distribution_matrix": [
    {
      "address": "0x71C765...d897",
      "weight": 0.45
    },
    {
      "address": "0x24F211...e112",
      "weight": 0.25
    }
  ]
}`;

function JsonPanel({ campaignName, instant, merkle }) {
  const dynamic = `{
  "campaign_id": "${campaignName || "ZUS-7781-ALPHA"}",
  "instant_release": ${instant},
  "merkle_verified": ${merkle},
  "distribution_matrix": [
    {
      "address": "0x71C76i5...d897",
      "weight": 0.45
    },
    {
      "address": "0x24F211...e112",
      "weight": 0.25
    }
  ]
}`;
  const { out, done } = useTypewriter(dynamic, 12, 400);
  const colorize = (str) => {
    return str.split("\n").map((line, i) => {
      const parts = line.split(/(".*?")/g);
      return (
        <div key={i}>
          {parts.map((p, j) => {
            if (p.startsWith('"') && p.endsWith('"')) {
              if (p.includes("ZUS") || p.includes("0x")) return <span key={j} style={{ color: CYAN }}>{p}</span>;
              return <span key={j} style={{ color: CYAN_DIM }}>{p}</span>;
            }
            if (/0\.\d+/.test(p)) return <span key={j} style={{ color: "#ffc87a" }}>{p}</span>;
            if (p === "true" || p === "false") return <span key={j} style={{ color: p === "true" ? CYAN : MUTED }}>{p}</span>;
            return <span key={j} style={{ color: MUTED }}>{p}</span>;
          })}
        </div>
      );
    });
  };

  return (
    <div style={{
      background: "rgba(0,255,200,.02)", border:`1px solid ${BORDER}`,
      padding:16, fontFamily:MONO, fontSize:9.5, lineHeight:1.9,
      position:"relative", height:"100%",
    }}>
      <div style={{ display:"flex", justifyContent:"space-between", alignItems:"center", marginBottom:12 }}>
        <span style={{ color: MUTED2, letterSpacing:2, fontSize:8 }}>RECIPIENT_PAYLOAD.JSON</span>
        <div style={{ display:"flex", gap:5 }}>
          {["#ff6060","#ffb740","#00ffc8"].map((c,i) => (
            <div key={i} style={{ width:8, height:8, borderRadius:"50%", background:c, opacity:.7 }}/>
          ))}
        </div>
      </div>
      <div style={{ whiteSpace:"pre-wrap", wordBreak:"break-all" }}>
        {colorize(out)}
        {!done && <span style={{ animation:"cur .7s steps(1) infinite", color: CYAN }}>▋</span>}
      </div>
    </div>
  );
}

/* ─── egress bar ─── */
function EgressBar({ pct, live }) {
  const [width, setWidth] = useState(0);
  useEffect(() => { const t = setTimeout(() => setWidth(pct), 300); return () => clearTimeout(t); }, [pct]);
  return (
    <div style={{ minWidth:120 }}>
      <div style={{ display:"flex", justifyContent:"space-between", marginBottom:4 }}>
        <span style={{ fontFamily:MONO, fontSize:8, color: MUTED2, letterSpacing:1 }}>EGRESS</span>
        <span style={{ fontFamily:MONO, fontSize:8, color: live ? CYAN : MUTED, letterSpacing:1 }}>{pct}%</span>
      </div>
      <div style={{ height:2, background:"rgba(0,255,200,.07)", borderRadius:1 }}>
        <div style={{
          height:"100%", width:`${width}%`,
          background: live ? `linear-gradient(90deg,${CYAN_DIM},${CYAN})` : `linear-gradient(90deg,${MUTED2},${MUTED})`,
          borderRadius:1, transition:"width 1.2s cubic-bezier(.4,0,.2,1)",
          boxShadow: live && width > 0 ? `0 0 6px rgba(0,255,200,.5)` : "none",
        }}/>
      </div>
    </div>
  );
}

/* ─── campaign row ─── */
function CampaignRow({ status, id, name, sub, recipients, egress, delay=0 }) {
  const [vis, setVis] = useState(false);
  const [hov, setHov] = useState(false);
  const ref = useRef(null);
  useEffect(() => {
    const obs = new IntersectionObserver(([e]) => { if (e.isIntersecting) { setVis(true); obs.disconnect(); } }, { threshold:.1 });
    if (ref.current) obs.observe(ref.current);
    return () => obs.disconnect();
  }, []);
  const live = status === "LIVE";
  const dotColor = live ? CYAN : MUTED2;
  return (
    <div ref={ref} onMouseEnter={() => setHov(true)} onMouseLeave={() => setHov(false)}
      style={{
        display:"flex", alignItems:"center", gap:16,
        border:`1px solid ${hov ? BORDER_HOV : BORDER}`,
        background: hov ? "rgba(0,255,200,.02)" : "transparent",
        padding:"18px 20px",
        opacity: vis ? 1 : 0,
        transform: vis ? "translateX(0)" : "translateX(20px)",
        transition:`opacity .5s ${delay}ms, transform .5s ${delay}ms, border-color .25s, background .25s`,
        cursor:"pointer",
        position:"relative", overflow:"hidden",
      }}>
      {/* hover top line */}
      <div style={{ position:"absolute", top:0, left:0, right:0, height:1,
        background:`linear-gradient(90deg,transparent,${CYAN},transparent)`,
        opacity: hov ? 1 : 0, transition:"opacity .3s" }}/>
      {/* dot */}
      <div style={{ width:10, height:10, borderRadius:"50%", background:dotColor, flexShrink:0,
        boxShadow: live ? `0 0 8px ${CYAN},0 0 16px rgba(0,255,200,.4)` : "none",
        animation: live ? "pulse 2s ease-in-out infinite" : "none" }}/>
      {/* badge */}
      <div style={{ fontFamily:MONO, fontSize:8, color: live ? CYAN : MUTED, border:`1px solid ${live ? "rgba(0,255,200,.3)" : BORDER}`,
        padding:"2px 8px", letterSpacing:1.5, flexShrink:0, minWidth:48, textAlign:"center" }}>
        {status}
      </div>
      <div style={{ fontFamily:MONO, fontSize:8, color: MUTED2, minWidth:36 }}>{id}</div>
      {/* name */}
      <div style={{ flex:1 }}>
        <div style={{ fontFamily:MONO, fontSize:13, color: TEXT, letterSpacing:2, marginBottom:3 }}>{name}</div>
        <div style={{ fontFamily:MONO, fontSize:8, color: MUTED2, letterSpacing:1 }}>{sub}</div>
      </div>
      {/* recipients */}
      <div style={{ minWidth:80, textAlign:"right" }}>
        <div style={{ fontFamily:MONO, fontSize:8, color: MUTED2, letterSpacing:1, marginBottom:2 }}>RECIPIENTS</div>
        <div style={{ fontFamily:MONO, fontSize:14, color: TEXT }}>{recipients.toLocaleString()}</div>
      </div>
      {/* egress */}
      <div style={{ minWidth:140 }}>
        <EgressBar pct={egress} live={live} />
      </div>
      {/* arrow */}
      <div style={{ color: hov ? CYAN : MUTED2, fontFamily:MONO, fontSize:14, transition:"color .25s, transform .25s",
        transform: hov ? "translateX(3px)" : "translateX(0)" }}>›</div>
    </div>
  );
}

/* ─── nav item ─── */
function NavItem({ icon, label, active, onClick }) {
  const [hov, setHov] = useState(false);
  const hi = active || hov;
  return (
    <div onClick={onClick} onMouseEnter={() => setHov(true)} onMouseLeave={() => setHov(false)}
      style={{
        display:"flex", alignItems:"center", gap:12, padding:"10px 16px",
        cursor:"pointer", position:"relative",
        background: active ? "rgba(0,255,200,.05)" : hov ? "rgba(0,255,200,.02)" : "transparent",
        borderRight: active ? `2px solid ${CYAN}` : "2px solid transparent",
        transition:"all .2s",
      }}>
      <span style={{ fontFamily:MONO, fontSize:11, color: hi ? CYAN : MUTED2,
        textShadow: active ? `0 0 8px rgba(0,255,200,.5)` : "none",
        transition:"color .2s" }}>{icon}</span>
      <span style={{ fontFamily:MONO, fontSize:9, color: hi ? TEXT : MUTED2, letterSpacing:2,
        transition:"color .2s" }}>{label}</span>
    </div>
  );
}

/* ─── main ─── */
export default function App() {
  const [active, setActive] = useState("CAMPAIGNS");
  const [campaignName, setCampaignName] = useState("");
  const [instant, setInstant] = useState(true);
  const [merkle, setMerkle] = useState(false);
  const [submitted, setSubmitted] = useState(false);
  const [submitAnim, setSubmitAnim] = useState(false);
  const [walletHov, setWalletHov] = useState(false);

  const handleCreate = () => {
    if (submitted) return;
    setSubmitAnim(true);
    setTimeout(() => { setSubmitted(true); setSubmitAnim(false); }, 1200);
  };

  const navItems = [
    { icon:"▦", label:"DASHBOARD" },
    { icon:"◎", label:"CAMPAIGNS" },
    { icon:"▣", label:"INVENTORY" },
    { icon:"▤", label:"SETTLEMENTS" },
    { icon:"▧", label:"SECURITY" },
  ];

  return (
    <>
      <style>{`
        @import url('https://fonts.googleapis.com/css2?family=Share+Tech+Mono&display=swap');
        *, *::before, *::after { box-sizing:border-box; margin:0; padding:0; }
        body { background:${BG}; margin:0; overflow-x:hidden; font-family:${MONO}; }
        ::-webkit-scrollbar { width:3px; }
        ::-webkit-scrollbar-track { background:${BG}; }
        ::-webkit-scrollbar-thumb { background:#00806a; }
        input { outline:none; }
        button { outline:none; }
        @keyframes cur { 0%,100%{opacity:1} 50%{opacity:0} }
        @keyframes fadeUp { from{opacity:0;transform:translateY(14px)} to{opacity:1;transform:translateY(0)} }
        @keyframes fadeIn { from{opacity:0} to{opacity:1} }
        @keyframes slideIn { from{opacity:0;transform:translateX(-14px)} to{opacity:1;transform:translateX(0)} }
        @keyframes pulse {
          0%,100% { box-shadow:0 0 6px ${CYAN},0 0 12px rgba(0,255,200,.3); }
          50%      { box-shadow:0 0 12px ${CYAN},0 0 28px rgba(0,255,200,.6); }
        }
        @keyframes scanline {
          0%   { top:-2px; }
          100% { top:100%; }
        }
        @keyframes spin { from{transform:rotate(0deg)} to{transform:rotate(360deg)} }
        @keyframes successPop {
          0%   { transform:scale(.9); opacity:0; }
          60%  { transform:scale(1.04); opacity:1; }
          100% { transform:scale(1); }
        }
      `}</style>

      <div style={{ display:"flex", minHeight:"100vh", background:BG }}>

        {/* ── SIDEBAR ── */}
        <aside style={{
          width:196, flexShrink:0,
          background:"rgba(4,20,24,.95)",
          borderRight:`1px solid ${BORDER}`,
          display:"flex", flexDirection:"column",
          animation:"slideIn .5s .1s both",
          position:"relative", overflow:"hidden",
        }}>
          {/* scanline effect */}
          <div style={{
            position:"absolute", left:0, right:0, height:2,
            background:"linear-gradient(180deg,transparent,rgba(0,255,200,.06),transparent)",
            animation:"scanline 6s linear infinite", pointerEvents:"none",
          }}/>

          {/* logo */}
          <div style={{ padding:"20px 16px 24px", borderBottom:`1px solid ${BORDER}` }}>
            <div style={{ fontFamily:MONO, fontSize:13, color:CYAN, letterSpacing:3,
              textShadow:`0 0 14px rgba(0,255,200,.5)` }}>ZUS_PROTOCOL</div>
          </div>

          {/* nav */}
          <div style={{ flex:1, paddingTop:12 }}>
            {navItems.map(n => (
              <NavItem key={n.label} icon={n.icon} label={n.label}
                active={active === n.label} onClick={() => setActive(n.label)} />
            ))}
          </div>

          {/* bottom */}
          <div style={{ borderTop:`1px solid ${BORDER}`, paddingTop:12, paddingBottom:16 }}>
            <NavItem icon="◈" label="SETTINGS" active={false} onClick={() => {}} />
            <NavItem icon="?" label="SUPPORT" active={false} onClick={() => {}} />
          </div>

          <div style={{ padding:"8px 16px 16px", fontFamily:MONO, fontSize:8, color:MUTED2, letterSpacing:1, lineHeight:1.8 }}>
            ZUS_PROTOCOL_CORE<br />
            <span style={{ color:"#1a4040" }}>© 2026 ZUS PROTOCOL. CATS ARE AMAZING!</span><br />
            <span style={{ color:"#1a4040" }}>BUILD_2984-X | NODE_STABLE | UPTIME_99.9%</span>
          </div>
        </aside>

        {/* ── MAIN ── */}
        <div style={{ flex:1, display:"flex", flexDirection:"column", minHeight:"100vh" }}>

          {/* ── TOP NAV ── */}
          <header style={{
            display:"flex", alignItems:"center", justifyContent:"space-between",
            padding:"0 32px",
            height:56, borderBottom:`1px solid ${BORDER}`,
            background:"rgba(2,13,15,.6)", backdropFilter:"blur(8px)",
            animation:"fadeUp .5s both",
            position:"sticky", top:0, zIndex:50,
          }}>
            <div style={{ display:"flex", gap:0 }}>
              {["Campaigns","Analytics","Vault","Protocols"].map((tab, i) => {
                const isActive = tab === "Campaigns";
                return (
                  <div key={tab} style={{
                    fontFamily:MONO, fontSize:10, letterSpacing:2,
                    color: isActive ? TEXT : MUTED,
                    padding:"0 20px", height:56, display:"flex", alignItems:"center",
                    borderBottom: isActive ? `2px solid ${CYAN}` : "2px solid transparent",
                    cursor:"pointer", transition:"color .2s",
                  }}
                    onMouseEnter={e => !isActive && (e.currentTarget.style.color = TEXT)}
                    onMouseLeave={e => !isActive && (e.currentTarget.style.color = MUTED)}
                  >{tab}</div>
                );
              })}
            </div>

            {/* wallet button */}
            <div onMouseEnter={() => setWalletHov(true)} onMouseLeave={() => setWalletHov(false)}
              style={{
                fontFamily:MONO, fontSize:9, letterSpacing:2,
                color: walletHov ? BG : CYAN,
                border:`1px solid ${CYAN}`,
                padding:"8px 16px", cursor:"pointer",
                background: walletHov ? CYAN : "transparent",
                boxShadow: walletHov ? `0 0 20px rgba(0,255,200,.5)` : `0 0 10px rgba(0,255,200,.15)`,
                transition:"all .25s",
              }}>
              WALLET_<br/>CONNECTED
            </div>
          </header>

          {/* ── CONTENT ── */}
          <main style={{ flex:1, padding:"36px 40px", overflowY:"auto" }}>

            {/* subtle bg glow */}
            <div style={{ position:"fixed", inset:0, pointerEvents:"none", zIndex:0,
              background:"radial-gradient(ellipse 50% 40% at 70% 30%, rgba(0,255,200,.03) 0%, transparent 70%)" }}/>

            <div style={{ position:"relative", zIndex:1 }}>

              {/* ── CREATE CAMPAIGN FORM ── */}
              <div style={{ animation:"fadeUp .6s .2s both" }}>
                <div style={{ fontFamily:MONO, fontSize:9, color:CYAN_DIM, letterSpacing:2, marginBottom:8 }}>
                  ENTRY: 0X00_INIT
                </div>
                <h1 style={{ fontFamily:MONO, fontSize:"clamp(22px,3vw,34px)", color:TEXT, letterSpacing:3, marginBottom:28 }}>
                  CREATE NEW CAMPAIGN_
                </h1>
              </div>

              <div style={{
                border:`1px solid ${BORDER}`, background:"rgba(4,20,24,.7)",
                padding:28, marginBottom:40,
                animation:"fadeUp .6s .35s both",
                position:"relative", overflow:"hidden",
              }}>
                {/* card scanline */}
                <div style={{
                  position:"absolute", left:0, right:0, height:1,
                  background:`linear-gradient(90deg,transparent,${CYAN},transparent)`,
                  top:0, opacity:.4,
                }}/>

                <div style={{ display:"grid", gridTemplateColumns:"1fr 1fr", gap:28 }}>

                  {/* LEFT: form */}
                  <div>
                    {/* name field */}
                    <div style={{ marginBottom:28 }}>
                      <label style={{ fontFamily:MONO, fontSize:8, color:MUTED2, letterSpacing:2, display:"block", marginBottom:10 }}>
                        NAME
                      </label>
                      <input
                        value={campaignName}
                        onChange={e => setCampaignName(e.target.value.toUpperCase())}
                        placeholder="ZUS_AIRDROP_PROXIMA"
                        maxLength={32}
                        style={{
                          width:"100%", fontFamily:MONO, fontSize:12,
                          background:"rgba(0,255,200,.03)",
                          border:`1px solid ${campaignName ? CYAN_DIM : BORDER}`,
                          color: TEXT, padding:"12px 14px",
                          letterSpacing:2,
                          boxShadow: campaignName ? `0 0 12px rgba(0,255,200,.1)` : "none",
                          transition:"border-color .3s, box-shadow .3s",
                        }}
                      />
                      <div style={{ fontFamily:MONO, fontSize:8, color:MUTED2, letterSpacing:1, marginTop:8, lineHeight:1.8 }}>
                        ENTER A UNIQUE IDENTIFIER FOR THIS CRYPTOGRAPHIC<br/>CAMPAIGN CLUSTER.
                      </div>
                    </div>

                    <div style={{ height:1, background:BORDER, marginBottom:24 }}/>

                    {/* emission settings */}
                    <div>
                      <label style={{ fontFamily:MONO, fontSize:8, color:MUTED2, letterSpacing:2, display:"block", marginBottom:16 }}>
                        EMISSION SETTINGS
                      </label>
                      <div style={{ display:"flex", flexDirection:"column", gap:14 }}>
                        {[
                          { label:"INSTANT RELEASE", val:instant, set:setInstant },
                          { label:"MERKLE VERIFIED", val:merkle, set:setMerkle },
                        ].map(({ label, val, set }) => (
                          <div key={label} style={{ display:"flex", justifyContent:"space-between", alignItems:"center" }}>
                            <span style={{ fontFamily:MONO, fontSize:10, color: val ? TEXT : MUTED, letterSpacing:1, transition:"color .3s" }}>
                              {label}
                            </span>
                            <Toggle on={val} onChange={set} />
                          </div>
                        ))}
                      </div>
                    </div>

                    <div style={{ height:1, background:BORDER, margin:"24px 0" }}/>

                    {/* status + button */}
                    <div style={{ display:"flex", alignItems:"center", justifyContent:"space-between" }}>
                      <div style={{ display:"flex", alignItems:"center", gap:8 }}>
                        <div style={{
                          width:7, height:7, borderRadius:"50%",
                          background: submitted ? CYAN : MUTED2,
                          boxShadow: submitted ? `0 0 8px ${CYAN}` : "none",
                          transition:"all .4s",
                        }}/>
                        <span style={{ fontFamily:MONO, fontSize:8, color: submitted ? CYAN_DIM : MUTED2, letterSpacing:1.5, transition:"color .4s" }}>
                          {submitted ? "CAMPAIGN DEPLOYED" : submitAnim ? "DEPLOYING..." : "SYSTEM READY: AWAITING CONFIG"}
                        </span>
                      </div>
                      <Btn onClick={handleCreate} disabled={submitted}>
                        {submitAnim
                          ? <span style={{ display:"flex", alignItems:"center", gap:8 }}>
                              <span style={{ display:"inline-block", animation:"spin .7s linear infinite" }}>◌</span> DEPLOYING
                            </span>
                          : submitted ? "DEPLOYED ✓" : "CREATE CAMPAIGN"
                        }
                      </Btn>
                    </div>

                    {/* success overlay */}
                    {submitted && (
                      <div style={{
                        marginTop:16, padding:"12px 16px",
                        border:`1px solid rgba(0,255,200,.25)`,
                        background:"rgba(0,255,200,.04)",
                        animation:"successPop .5s both",
                      }}>
                        <div style={{ fontFamily:MONO, fontSize:9, color:CYAN_DIM, letterSpacing:2, marginBottom:4 }}>
                          [CAMPAIGN_CLUSTER_INITIALIZED]
                        </div>
                        <div style={{ fontFamily:MONO, fontSize:8, color:MUTED, letterSpacing:1, lineHeight:1.8 }}>
                          ID: ZUS-{Math.floor(Math.random()*9000+1000)}-ALPHA &nbsp;·&nbsp;
                          STATUS: PROPAGATING &nbsp;·&nbsp;
                          ETA: 12s
                        </div>
                      </div>
                    )}
                  </div>

                  {/* RIGHT: json preview */}
                  <JsonPanel campaignName={campaignName} instant={instant} merkle={merkle} />
                </div>
              </div>

              {/* ── ACTIVE CAMPAIGNS ── */}
              <div style={{ animation:"fadeUp .6s .5s both" }}>
                <div style={{ fontFamily:MONO, fontSize:9, color:CYAN_DIM, letterSpacing:2, marginBottom:6 }}>
                  OPERATIONAL STREAM
                </div>
                <div style={{ display:"flex", alignItems:"center", justifyContent:"space-between", marginBottom:24 }}>
                  <h2 style={{ fontFamily:MONO, fontSize:"clamp(18px,2.5vw,28px)", color:TEXT, letterSpacing:3 }}>
                    ACTIVE CAMPAIGNS
                  </h2>
                  <div style={{ display:"flex", gap:10 }}>
                    <Btn outline small>FILTER</Btn>
                    <Btn outline small>EXPORT</Btn>
                  </div>
                </div>

                <div style={{ display:"flex", flexDirection:"column", gap:10 }}>
                  <CampaignRow status="LIVE" id="#0021" name="NEXUS Q1 REWARD" sub="MAINNET DISTRIBUTION PROTOCOL" recipients={1420} egress={70.2} delay={0} />
                  <CampaignRow status="DRAFT" id="#0022" name="INTERNAL TEST NET" sub="SANDBOX LOGIC CLUSTER" recipients={12} egress={0.0} delay={120} />
                  <CampaignRow status="LIVE" id="#0710" name="GOVERNANCE AIRDROP" sub="DAO PARTICIPATION INCENTIVE" recipients={5201} egress={99.1} delay={240} />
                </div>
              </div>

            </div>
          </main>

          {/* ── FOOTER ── */}
          <footer style={{
            borderTop:`1px solid ${BORDER}`,
            padding:"14px 40px",
            display:"flex", justifyContent:"space-between", alignItems:"center",
            fontFamily:MONO, fontSize:8, color:MUTED2, letterSpacing:1,
            background:"rgba(2,13,15,.6)",
          }}>
            <div style={{ display:"flex", alignItems:"center", gap:10 }}>
              <span style={{ color:CYAN }}>ZUS_PROTOCOL_CORE</span>
              <PixelCat />
            </div>
            <div style={{ textAlign:"center", lineHeight:2 }}>
              <div>© 2026 ZUS PROTOCOL. ALL RIGHTS RESERVED.</div>
              <div style={{ color:"#1a5550", fontSize:7, letterSpacing:2 }}>i like cats</div>
            </div>
            <span>X_FEED · DISCORD_SERVER · GITHUB_REPO · PRIVACY_POLICY · NEWSLETTER_SUB</span>
          </footer>
        </div>
      </div>
    </>
  );
}
