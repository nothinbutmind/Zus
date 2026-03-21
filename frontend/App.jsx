import { useState, useEffect, useRef } from "react";

/* ── Typewriter subtitle ── */
const SUBTITLE = "Zus is a token-gated rewards protocol built on Avalanche — where eligibility is verified, identity stays hidden, and balances remain confidential.";

function TypewriterSub() {
  const [text, setText] = useState("");
  const [done, setDone] = useState(false);
  useEffect(() => {
    let i = 0, alive = true;
    const t = setTimeout(function tick() {
      if (!alive) return;
      if (i <= SUBTITLE.length) {
        setText(SUBTITLE.slice(0, i));
        i++;
        setTimeout(tick, 28);
      } else { setDone(true); }
    }, 1200);
    return () => { alive = false; clearTimeout(t); };
  }, []);
  // highlight key phrases
  const highlight = (str) => {
    const parts = str.split(/(built on Avalanche|remain confidential)/g);
    return parts.map((p, i) =>
      (p === "built on Avalanche" || p === "remain confidential")
        ? <span key={i} style={{ color: "#00ddb0" }}>{p}</span>
        : p
    );
  };
  return (
    <p style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: 11, color: "#3a6660", maxWidth: 400, margin: "24px auto 36px", lineHeight: 1.9, animation: "fadeUp .9s .6s both", minHeight: 80 }}>
      {highlight(text)}
      {!done && <span style={{ animation: "cur 0.7s steps(1) infinite", color: "#00ffc8" }}>▋</span>}
    </p>
  );
}

/* ── Pixel cat ── */
function PixelCat() {
  const [hov, setHov] = useState(false);
  // 10x8 pixel grid, cyan on dark
  const pixels = [
    "0011011100",
    "0111111110",
    "1111111111",
    "1010110101",  // eyes + nose row
    "1111111111",
    "0111111110",
    "0101000101",  // legs
    "0101000101",
  ];
  return (
    <div
      onMouseEnter={() => setHov(true)}
      onMouseLeave={() => setHov(false)}
      style={{ cursor: "pointer", display: "inline-block", transition: "transform .2s", transform: hov ? "scale(1.3) translateY(-2px)" : "scale(1)" }}
      title="=^._.^="
    >
      {pixels.map((row, ri) => (
        <div key={ri} style={{ display: "flex" }}>
          {row.split("").map((px, ci) => (
            <div key={ci} style={{
              width: 3, height: 3,
              background: px === "1" ? (hov ? "#00ffc8" : "#00c49a") : "transparent",
              boxShadow: px === "1" && hov ? "0 0 4px rgba(0,255,200,.6)" : "none",
              transition: "background .2s, box-shadow .2s",
            }} />
          ))}
        </div>
      ))}
    </div>
  );
}

/* ── Scroll-reveal hook ── */
function useReveal(threshold = 0.15) {
  const ref = useRef(null);
  const [visible, setVisible] = useState(false);
  useEffect(() => {
    const obs = new IntersectionObserver(([e]) => { if (e.isIntersecting) { setVisible(true); obs.disconnect(); } }, { threshold });
    if (ref.current) obs.observe(ref.current);
    return () => obs.disconnect();
  }, []);
  return [ref, visible];
}

/* ── Particle canvas ── */
function Particles() {
  const ref = useRef(null);
  useEffect(() => {
    const c = ref.current; if (!c) return;
    const ctx = c.getContext("2d");
    let W = c.width = c.offsetWidth, H = c.height = c.offsetHeight;
    const pts = Array.from({ length: 55 }, () => ({
      x: Math.random() * W, y: Math.random() * H,
      vx: (Math.random() - .5) * .25, vy: (Math.random() - .5) * .25,
      r: Math.random() * 1.2 + .4, o: Math.random() * .4 + .1,
    }));
    let raf;
    function draw() {
      ctx.clearRect(0, 0, W, H);
      pts.forEach(p => {
        p.x += p.vx; p.y += p.vy;
        if (p.x < 0) p.x = W; if (p.x > W) p.x = 0;
        if (p.y < 0) p.y = H; if (p.y > H) p.y = 0;
        ctx.beginPath(); ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(0,255,200,${p.o})`; ctx.fill();
      });
      pts.forEach((a, i) => pts.slice(i + 1).forEach(b => {
        const d = Math.hypot(a.x - b.x, a.y - b.y);
        if (d < 90) {
          ctx.beginPath(); ctx.moveTo(a.x, a.y); ctx.lineTo(b.x, b.y);
          ctx.strokeStyle = `rgba(0,255,200,${(1 - d / 90) * .1})`; ctx.lineWidth = .5; ctx.stroke();
        }
      }));
      raf = requestAnimationFrame(draw);
    }
    draw();
    const onR = () => { W = c.width = c.offsetWidth; H = c.height = c.offsetHeight; };
    window.addEventListener("resize", onR);
    return () => { cancelAnimationFrame(raf); window.removeEventListener("resize", onR); };
  }, []);
  return <canvas ref={ref} style={{ position: "absolute", inset: 0, width: "100%", height: "100%", pointerEvents: "none" }} />;
}

/* ── Glitch text ── */
function Glitch({ children, color }) {
  const [on, setOn] = useState(false);
  useEffect(() => {
    const iv = setInterval(() => { setOn(true); setTimeout(() => setOn(false), 110); }, 5000 + Math.random() * 3000);
    return () => clearInterval(iv);
  }, []);
  return (
    <span style={{ color: color || "inherit", position: "relative", display: "inline-block", animation: on ? "glitch .12s steps(2) both" : "none" }}>
      {children}
    </span>
  );
}

/* ── Neon button ── */
function Btn({ children, outline }) {
  const [hov, setHov] = useState(false);
  const base = {
    fontFamily: "'Share Tech Mono', monospace",
    fontSize: 10,
    letterSpacing: 2,
    textTransform: "uppercase",
    padding: "10px 22px",
    cursor: "pointer",
    border: "1px solid",
    transition: "all .25s",
    position: "relative",
    display: "inline-block",
  };
  if (outline) return (
    <button onMouseEnter={() => setHov(true)} onMouseLeave={() => setHov(false)} style={{
      ...base,
      background: "transparent",
      color: hov ? "#00ffc8" : "#4a7a72",
      borderColor: hov ? "#00ffc8" : "#1a4040",
      boxShadow: hov ? "0 0 18px rgba(0,255,200,.35), inset 0 0 18px rgba(0,255,200,.04)" : "none",
    }}>{children}</button>
  );
  return (
    <button onMouseEnter={() => setHov(true)} onMouseLeave={() => setHov(false)} style={{
      ...base,
      background: hov ? "#00ffc8" : "#00ddb0",
      color: "#020d0f",
      borderColor: hov ? "#00ffc8" : "#00ddb0",
      boxShadow: hov ? "0 0 24px rgba(0,255,200,.7), 0 0 48px rgba(0,255,200,.3)" : "0 0 10px rgba(0,255,200,.2)",
      fontWeight: 700,
    }}>{children}</button>
  );
}

/* ── Feature card ── */
function FeatCard({ tag, title, desc, extra, delay }) {
  const [ref, vis] = useReveal();
  const [hov, setHov] = useState(false);
  return (
    <div ref={ref} onMouseEnter={() => setHov(true)} onMouseLeave={() => setHov(false)} style={{
      border: `1px solid ${hov ? "rgba(0,255,200,.25)" : "rgba(0,255,200,.08)"}`,
      background: hov ? "rgba(0,255,200,.03)" : "transparent",
      padding: "20px",
      position: "relative",
      opacity: vis ? 1 : 0,
      transform: vis ? "translateY(0)" : "translateY(20px)",
      transition: `opacity .6s ${delay}ms, transform .6s ${delay}ms, border-color .3s, background .3s`,
      overflow: "hidden",
    }}>
      <div style={{
        position: "absolute", top: 0, left: 0, right: 0, height: 1,
        background: "linear-gradient(90deg,transparent,#00ffc8,transparent)",
        opacity: hov ? 1 : 0, transition: "opacity .3s",
      }} />
      <div style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: 9, color: "#2a5550", letterSpacing: 2, marginBottom: 12 }}>{tag}</div>
      <h3 style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: 11, color: "#e0f0ed", letterSpacing: 1, marginBottom: 10, lineHeight: 1.5 }}>{title}</h3>
      <p style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: 9, color: "#3a6660", lineHeight: 1.8, marginBottom: 14 }}>{desc}</p>
      {extra && <div style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: 9, color: "#2a5550" }}>{extra}</div>}
      <div style={{ marginTop: 16, height: 1, background: "rgba(0,255,200,.06)" }}>
        <div style={{ height: "100%", width: "55%", background: "linear-gradient(90deg,#00ffc8,transparent)" }} />
      </div>
    </div>
  );
}

/* ── How step ── */
function HowStep({ dot, title, desc, delay }) {
  const [ref, vis] = useReveal();
  return (
    <div ref={ref} style={{
      display: "flex", gap: 16, alignItems: "flex-start",
      padding: "20px 24px",
      border: "1px solid rgba(0,255,200,.07)",
      opacity: vis ? 1 : 0,
      transform: vis ? "translateX(0)" : "translateX(-20px)",
      transition: `opacity .6s ${delay}ms, transform .6s ${delay}ms`,
    }}>
      <div style={{
        width: 32, height: 32, flexShrink: 0,
        background: "rgba(0,255,200,.08)",
        border: "1px solid rgba(0,255,200,.2)",
        display: "flex", alignItems: "center", justifyContent: "center",
        fontFamily: "'Share Tech Mono',monospace", fontSize: 11, color: "#00ddb0",
      }}>{dot}</div>
      <div>
        <div style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: 11, color: "#cce8e4", letterSpacing: 1, marginBottom: 6 }}>{title}</div>
        <div style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: 9, color: "#3a6660", lineHeight: 1.8 }}>{desc}</div>
      </div>
    </div>
  );
}

/* ── Use case card ── */
function UseCard({ title, desc, delay }) {
  const [ref, vis] = useReveal(0.1);
  const [hov, setHov] = useState(false);
  return (
    <div ref={ref} onMouseEnter={() => setHov(true)} onMouseLeave={() => setHov(false)} style={{
      border: `1px solid ${hov ? "rgba(0,255,200,.2)" : "rgba(0,255,200,.07)"}`,
      background: hov ? "rgba(0,255,200,.02)" : "transparent",
      padding: "20px",
      opacity: vis ? 1 : 0,
      transform: vis ? "translateY(0)" : "translateY(16px)",
      transition: `opacity .5s ${delay}ms, transform .5s ${delay}ms, border-color .3s`,
    }}>
      <div style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: 10, color: "#00ddb0", letterSpacing: 1.5, marginBottom: 8 }}>{title}</div>
      <div style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: 9, color: "#3a6660", lineHeight: 1.8 }}>{desc}</div>
    </div>
  );
}

export default function App() {
  const [scrolled, setScrolled] = useState(false);
  useEffect(() => {
    const fn = () => setScrolled(window.scrollY > 30);
    window.addEventListener("scroll", fn);
    return () => window.removeEventListener("scroll", fn);
  }, []);

  return (
    <>
      <style>{`
        @import url('https://fonts.googleapis.com/css2?family=Share+Tech+Mono&display=swap');
        *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
        ::-webkit-scrollbar { width: 3px; }
        ::-webkit-scrollbar-track { background: #020d0f; }
        ::-webkit-scrollbar-thumb { background: #00806a; }
        body { background: #020d0f; margin: 0; overflow-x: hidden; }
        @keyframes cur { 0%,100%{opacity:1} 50%{opacity:0} }
        @keyframes glitch {
          0%   { clip-path:inset(20% 0 60% 0); transform:translate(-2px,0); }
          33%  { clip-path:inset(60% 0 10% 0); transform:translate(2px,0); filter:hue-rotate(20deg); }
          66%  { clip-path:inset(40% 0 40% 0); transform:translate(-1px,0); }
          100% { clip-path:none; transform:translate(0); }
        }
        @keyframes fadeUp { from{opacity:0;transform:translateY(18px)} to{opacity:1;transform:translateY(0)} }
        @keyframes glowPulse {
          0%,100% { text-shadow: 0 0 28px rgba(0,255,200,.5),0 0 56px rgba(0,255,200,.2); }
          50%     { text-shadow: 0 0 48px rgba(0,255,200,.85),0 0 90px rgba(0,255,200,.4),0 0 130px rgba(0,255,200,.1); }
        }
        button { outline: none; }
      `}</style>

      {/* NAV */}
      <nav style={{
        position: "fixed", top: 0, left: 0, right: 0, zIndex: 100,
        display: "flex", alignItems: "center", justifyContent: "space-between",
        padding: "14px 40px",
        background: scrolled ? "rgba(2,13,15,.92)" : "transparent",
        backdropFilter: scrolled ? "blur(12px)" : "none",
        borderBottom: scrolled ? "1px solid rgba(0,255,200,.08)" : "1px solid transparent",
        transition: "all .4s",
      }}>
        <span style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: 12, color: "#00ffc8", letterSpacing: 3, textShadow: "0 0 14px rgba(0,255,200,.5)" }}>
          ZUS_PROTOCOL
        </span>
        <div style={{ display: "flex", gap: 32, alignItems: "center" }}>
          {["PRINCIPLES","FEATURES"].map(t => (
            <a key={t} href="#" style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: 9, color: "#3a6660", letterSpacing: 2, textDecoration: "none", transition: "color .2s" }}
              onMouseEnter={e => e.target.style.color = "#00ffc8"}
              onMouseLeave={e => e.target.style.color = "#3a6660"}>{t}</a>
          ))}
          <Btn>LAUNCH APP</Btn>
        </div>
      </nav>

      {/* HERO */}
      <section style={{ position: "relative", minHeight: "100vh", display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", textAlign: "center", padding: "120px 24px 80px", overflow: "hidden" }}>
        <div style={{ position: "absolute", inset: 0, zIndex: 0,
          background: "radial-gradient(ellipse 65% 45% at 50% 35%, rgba(0,255,200,.055) 0%, transparent 70%), radial-gradient(ellipse 35% 25% at 15% 85%, rgba(0,180,140,.03) 0%, transparent 60%), radial-gradient(ellipse 35% 25% at 85% 70%, rgba(0,100,80,.03) 0%, transparent 60%)" }} />
        <div style={{ position: "absolute", inset: 0, zIndex: 0,
          backgroundImage: "linear-gradient(rgba(0,255,200,.02) 1px,transparent 1px),linear-gradient(90deg,rgba(0,255,200,.02) 1px,transparent 1px)",
          backgroundSize: "55px 55px",
          maskImage: "radial-gradient(ellipse at center, black 25%, transparent 75%)" }} />
        <Particles />

        <div style={{ position: "relative", zIndex: 2 }}>
          <div style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: 9, letterSpacing: 3, color: "#00ddb0", border: "1px solid rgba(0,255,200,.22)", display: "inline-block", padding: "4px 14px", marginBottom: 28, animation: "fadeUp .8s .2s both" }}>
            PRIVACY-PRESERVING PROTOCOL
          </div>

          <h1 style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: "clamp(46px,8vw,104px)", fontWeight: 400, lineHeight: .96, letterSpacing: 2, animation: "fadeUp .9s .4s both" }}>
            <Glitch color="#ffffff">REWARDS_</Glitch><br />
            <span style={{ color: "#ffffff" }}>WITHOUT</span><br />
            <span style={{ color: "#00ffc8", display: "inline-block", animation: "glowPulse 3s 1.4s ease-in-out infinite" }}>
              <Glitch color="#00ffc8">EXPOSURE_</Glitch>
            </span>
          </h1>

          <TypewriterSub />

          <div style={{ display: "flex", gap: 14, justifyContent: "center", flexWrap: "wrap", animation: "fadeUp .9s .8s both" }}>
            <Btn>BOOK DEMO</Btn>
            <Btn outline>LAUNCH CAMPAIGN</Btn>
          </div>
        </div>
      </section>

      {/* ENCRYPTED BY DESIGN */}
      <section style={{ padding: "90px 48px", maxWidth: 1100, margin: "0 auto" }}>
        <div style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: 9, color: "#2a5550", letterSpacing: 2, marginBottom: 10 }}>ZRC_LAYER_LINE: 001</div>
        <h2 style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: "clamp(20px,3.5vw,40px)", color: "#cce8e4", letterSpacing: 3, marginBottom: 8 }}>ENCRYPTED BY DESIGN.</h2>
        <p style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: 9, color: "#3a6660", marginBottom: 40, lineHeight: 1.8 }}>
          Powered by ZK Proofs and Private Transactions, providing trust without public transparency.
        </p>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit,minmax(260px,1fr))", gap: 16 }}>
          <FeatCard tag="// CONFIDENTIAL_AIRDROP" title="DROP TO THE RIGHT WALLETS. TELL NO ONE ELSE." desc="Distribute tokens to verified holders without exposing the recipient list or individual balances." extra="+ MORE     ONLINE REWARDS >" delay={0} />
          <FeatCard tag="// NATURAL_LANGUAGE_REWARDS" title="SET RULES IN PLAIN PROMPTS." desc="Define eligibility criteria using natural language — no smart contract expertise or coding required for deployment." delay={120} />
          <FeatCard tag="// STEALTH_ADDRESSES" title="ONE-TIME ADDRESSES. ZERO TRACEABILITY." desc="Generate ephemeral stealth addresses using ZK tech so recipients can't be linked across transactions." extra="MAX SEND: 2D     AVAILABLE" delay={240} />
        </div>
      </section>

      {/* HOW ZUS WORKS */}
      <section style={{ padding: "90px 48px", maxWidth: 1100, margin: "0 auto" }}>
        <div style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: 9, color: "#2a5550", letterSpacing: 2, marginBottom: 10 }}>ZRC_LAYER_LINE: 002</div>
        <h2 style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: "clamp(20px,3.5vw,40px)", color: "#cce8e4", letterSpacing: 3, marginBottom: 4 }}>
          HOW <span style={{ color: "#00ffc8", textShadow: "0 0 16px rgba(0,255,200,.4)" }}>ZUS</span> WORKS
        </h2>
        <div style={{ width: 36, height: 2, background: "#00ffc8", margin: "10px 0 40px", boxShadow: "0 0 8px #00ffc8" }} />
        <div style={{ display: "flex", flexDirection: "column", gap: 16, maxWidth: 640 }}>
          <HowStep dot="S" title="PRIVATE BY DEFAULT" desc="Prove eligibility without revealing balances or activity. Our zero-knowledge infrastructure ensures your data footprint is non-existent." delay={0} />
          <HowStep dot="P" title="WORKS WITH PAYMENTS" desc="Trigger rewards from real merchant checkouts. Bridge the gap between on-chain incentives and off-chain commerce seamlessly." delay={150} />
          <HowStep dot="C" title="SMART CAMPAIGN LOGIC" desc="Define reward rules with flexible onchain intelligence. Programmatic loyalty that adapts to user behavior while maintaining total anonymity." delay={300} />
        </div>
      </section>

      {/* OPERATIONAL USE CASES */}
      <section style={{ padding: "90px 48px", textAlign: "center", position: "relative", overflow: "hidden" }}>
        <div style={{ position: "absolute", inset: 0, background: "radial-gradient(ellipse 70% 60% at 50% 50%, rgba(0,255,200,.02) 0%, transparent 70%)", pointerEvents: "none" }} />
        <div style={{ maxWidth: 1100, margin: "0 auto", position: "relative", zIndex: 1 }}>
          <div style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: 9, color: "#2a5550", letterSpacing: 2, marginBottom: 10 }}>ZRC_LAYER_LINE: 003</div>
          <h2 style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: "clamp(20px,3.5vw,40px)", color: "#cce8e4", letterSpacing: 4, marginBottom: 48 }}>
            OPERATIONAL USE CASES
          </h2>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, maxWidth: 700, margin: "0 auto" }}>
            <UseCard title="VIP DISCOUNTS" desc="Offer exclusive price reductions to your top-tier customers without revealing their transaction history." delay={0} />
            <UseCard title="CASHBACK CAMPAIGNS" desc="Automate reward rebates for verified purchases while maintaining absolute wallet confidentiality." delay={100} />
            <UseCard title="TOKEN-HOLDER PERKS" desc="Unlock ecosystem benefits for asset holders without exposing their total portfolio balances." delay={200} />
            <UseCard title="PRIVATE LOYALTY TIERS" desc="Classify users into status groups and trigger tailored rewards through anonymized proofs of activity." delay={300} />
          </div>
        </div>
      </section>

      {/* LEDGER OF SHADOWS */}
      <section style={{ padding: "120px 24px", textAlign: "center", position: "relative", overflow: "hidden" }}>
        <div style={{ position: "absolute", inset: 0, background: "radial-gradient(ellipse 55% 40% at 50% 50%, rgba(0,255,200,.035) 0%, transparent 70%)", pointerEvents: "none" }} />
        <div style={{ maxWidth: 600, margin: "0 auto", position: "relative", zIndex: 1 }}>
          <div style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: 9, color: "#2a5550", letterSpacing: 2, marginBottom: 16 }}>ZRC_LAYER_LINE: 004</div>
          <h2 style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: "clamp(30px,6vw,68px)", color: "#cce8e4", letterSpacing: 3, lineHeight: 1.05, marginBottom: 20 }}>
            THE LEDGER OF<br />SHADOWS.
          </h2>
          <p style={{ fontFamily: "'Share Tech Mono',monospace", fontSize: 9, color: "#3a6660", lineHeight: 2, marginBottom: 36 }}>
            Start distributing rewards that respect the right to<br />
            privacy. Join the next generation of protocol-level<br />
            confidentiality.
          </p>
          <Btn outline>CREATE CAMPAIGN</Btn>
        </div>
      </section>

      {/* FOOTER */}
      <footer style={{
        borderTop: "1px solid rgba(0,255,200,.06)",
        padding: "20px 40px",
        display: "flex", justifyContent: "space-between", alignItems: "center",
        fontFamily: "'Share Tech Mono',monospace", fontSize: 9, color: "#2a5550", letterSpacing: 1,
      }}>
        <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
          <span style={{ color: "#00ffc8" }}>ZUS_PROTOCOL_CORE</span>
          <PixelCat />
        </div>
        <div style={{ textAlign: "center", lineHeight: 2 }}>
          <div>© 2025 ZUS PROTOCOL. ALL RIGHTS RESERVED.</div>
          <div style={{ color: "#1a5550", fontSize: 8, letterSpacing: 2 }}>i like cats</div>
        </div>
        <span>PRIVACY · TERMS · DOCS</span>
      </footer>
    </>
  );
}
