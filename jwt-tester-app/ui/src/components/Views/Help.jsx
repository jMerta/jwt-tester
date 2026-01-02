import React, { useState, useMemo } from "react";
import { Tooltip } from "../Shared/Tooltip.jsx";

// --- Components ---

function HelpItem({ title, badge, children }) {
  return (
    <div className="list-item" style={{ alignItems: "flex-start", gap: "1rem" }}>
      <div style={{ flex: 1 }}>
        <div className="list-title">{title}</div>
        <div className="list-sub">{children}</div>
      </div>
      {badge ? (
        <span className="pill" style={{ whiteSpace: "nowrap" }}>
          {badge}
        </span>
      ) : null}
    </div>
  );
}

function Section({ title, children }) {
  return (
    <section className="card" style={{ animation: "fadeIn 0.3s ease-out" }}>
      {title && <h2>{title}</h2>}
      {children}
    </section>
  );
}

function FAQItem({ question, answer }) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div
      className="list-item"
      style={{
        flexDirection: "column",
        alignItems: "stretch",
        gap: "0.5rem",
        cursor: "pointer",
        background: isOpen ? "var(--bg-panel-hover)" : "var(--bg-input)",
      }}
      onClick={() => setIsOpen(!isOpen)}
    >
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <div className="list-title" style={{ color: "var(--primary)" }}>
          {question}
        </div>
        <div style={{ transform: isOpen ? "rotate(180deg)" : "rotate(0deg)", transition: "transform 0.2s" }}>
          ▼
        </div>
      </div>
      {isOpen && (
        <div className="list-sub" style={{ marginTop: "0.5rem", color: "var(--text-primary)", lineHeight: "1.6" }}>
          {answer}
        </div>
      )}
    </div>
  );
}

// --- Content Data ---

const CATEGORIES = ["All", "Concepts", "Reference", "Guides", "FAQ"];

const HELP_CONTENT = [
  {
    id: "overview",
    category: "Concepts",
    title: "Overview",
    tags: ["intro", "jwt", "what is"],
    render: () => (
      <>
        <p>
          JWT Tester is a local workspace for building, inspecting, and verifying{" "}
          <Tooltip text="JSON Web Tokens (JWS): compact, URL-safe signed tokens that carry claims">
            JWTs
          </Tooltip>
          . It combines a guided UI with a scriptable CLI, all backed by a local vault. Tokens are{" "}
          <Tooltip text="Signed, not encrypted. Use JWE for confidentiality.">signed</Tooltip> (integrity) rather than
          encrypted (confidentiality).
        </p>
        <p style={{ marginBottom: 0 }}>
          Use the UI for interactive workflows and the CLI for automation, CI, and repeatable scripts.
        </p>
      </>
    ),
  },
  {
    id: "key-concepts",
    category: "Concepts",
    title: "Key Tools",
    tags: ["vault", "builder", "inspector", "verifier", "settings", "cli"],
    render: () => (
      <>
        <HelpItem title="Vault Manager" badge="UI">
          Organize <Tooltip text="A collection of keys and tokens">projects</Tooltip>, store{" "}
          <Tooltip text="Cryptographic keys used to sign or verify tokens">keys</Tooltip>, and keep{" "}
          <Tooltip text="Saved JWTs for later inspection or testing">tokens</Tooltip> in one place.
        </HelpItem>
        <HelpItem title="Token Builder" badge="UI">
          Create JWTs with standard claims like <Tooltip text="Issuer - who created the token">iss</Tooltip>,{" "}
          <Tooltip text="Subject - whom the token refers to">sub</Tooltip>, and{" "}
          <Tooltip text="Expiration time">exp</Tooltip>, plus any custom payload data.
        </HelpItem>
        <HelpItem title="Token Inspector" badge="UI">
          Decode header + payload without verifying the signature (useful for debugging structure).
        </HelpItem>
        <HelpItem title="Token Verifier" badge="UI">
          Validate signatures and claims against your vault keys with optional leeway and required claims.
        </HelpItem>
      </>
    ),
  },
  {
    id: "anatomy",
    category: "Concepts",
    title: "JWT Anatomy (JWS)",
    tags: ["header", "payload", "signature", "structure"],
    render: () => (
      <>
        <HelpItem title="Header" badge="JSON">
          Metadata like <code>alg</code> (algorithm), <code>typ</code> (type), and <code>kid</code> (key id). Treated as
          untrusted input.
        </HelpItem>
        <HelpItem title="Payload" badge="JSON">
          Claims about the subject and context (registered + custom).
        </HelpItem>
        <HelpItem title="Signature" badge="Bytes">
          Cryptographic signature over <code>base64url(header)</code> + <code>.</code> + <code>base64url(payload)</code>.
        </HelpItem>
        <p style={{ marginBottom: 0, color: "var(--text-muted)"}}>
          A JWT is compact and URL-safe, but anyone can decode it. Only <strong>verification</strong> establishes
          authenticity.
        </p>
      </>
    ),
  },
  {
    id: "validation",
    category: "Concepts",
    title: "Verification Process",
    tags: ["verify", "validate", "steps"],
    render: () => (
      <>
        <HelpItem title="1) Decode & select key" badge="Step">
          Parse the token, infer or pin the <code>alg</code>, and pick a vault key (by <code>kid</code>, key id, or
          default key).
        </HelpItem>
        <HelpItem title="2) Verify signature" badge="Step">
          Ensure the signature matches the chosen key and algorithm.
        </HelpItem>
        <HelpItem title="3) Validate claims" badge="Step">
          Check <code>exp</code>, <code>nbf</code>, <code>iss</code>, <code>sub</code>, and <code>aud</code> with
          optional leeway.
        </HelpItem>
        <HelpItem title="4) Require custom claims" badge="Step">
          Enforce presence of required fields (e.g., <code>role</code>, <code>tenant</code>).
        </HelpItem>
      </>
    ),
  },
  {
    id: "algorithms",
    category: "Reference",
    title: "Algorithms & Keys",
    tags: ["hs256", "rs256", "es256", "eddsa", "hmac", "rsa", "ecdsa"],
    render: () => (
      <>
        <HelpItem title="HS256 / HS384 / HS512" badge="HMAC">
          Symmetric signing with a shared secret. The same key signs and verifies.
        </HelpItem>
        <HelpItem title="RS256 / RS384 / RS512" badge="RSA">
          Asymmetric RSA signatures. Sign with a private key, verify with a public key.
        </HelpItem>
        <HelpItem title="PS256 / PS384 / PS512" badge="RSA-PSS">
          RSA-PSS with stronger padding. Private key signs; public key verifies.
        </HelpItem>
        <HelpItem title="ES256 / ES384" badge="ECDSA">
          Elliptic Curve signatures (P-256 / P-384). Smaller keys, fast verification.
        </HelpItem>
        <HelpItem title="EdDSA" badge="Ed25519">
          Modern elliptic curve signatures. Compact keys and deterministic signing.
        </HelpItem>
      </>
    ),
  },
  {
    id: "claims",
    category: "Reference",
    title: "Standard Claims",
    tags: ["iss", "sub", "aud", "exp", "nbf", "iat", "jti"],
    render: () => (
      <div className="two-column-layout" style={{ gap: "1rem" }}>
        <div>
          <HelpItem title={<code>iss</code>} badge="string">
            Issuer of the token (who created it).
          </HelpItem>
          <HelpItem title={<code>sub</code>} badge="string">
            Subject of the token (who it is about).
          </HelpItem>
          <HelpItem title={<code>aud</code>} badge="string[]">
            Intended audience. Single string or array.
          </HelpItem>
          <HelpItem title={<code>jti</code>} badge="string">
            Unique token id for replay protection.
          </HelpItem>
        </div>
        <div>
          <HelpItem title={<code>iat</code>} badge="NumericDate">
            Issued-at time (seconds since epoch).
          </HelpItem>
          <HelpItem title={<code>nbf</code>} badge="NumericDate">
            Not-before time. Invalid before this.
          </HelpItem>
          <HelpItem title={<code>exp</code>} badge="NumericDate">
            Expiration time. Invalid on/after this.
          </HelpItem>
        </div>
      </div>
    ),
  },
  {
    id: "ui-workflow",
    category: "Guides",
    title: "UI Workflow",
    tags: ["tutorial", "how to", "ui"],
    render: () => (
      <>
        <HelpItem title="1) Create a project" badge="Dashboard">
          Projects scope keys and tokens. Set a startup default for faster work.
        </HelpItem>
        <HelpItem title="2) Add or generate a key" badge="Vault Manager">
          Paste a secret/PEM/JWKS, or generate HMAC/RSA/EC/EdDSA keys. Assign a <code>kid</code>.
        </HelpItem>
        <HelpItem title="3) Build tokens" badge="Token Builder">
          Fill standard claims, add custom JSON, and sign with your chosen algorithm.
        </HelpItem>
        <HelpItem title="4) Verify and debug" badge="Token Verifier">
          Validate signatures and claims. Use "Try All Keys" if the <code>kid</code> is unknown.
        </HelpItem>
      </>
    ),
  },
  {
    id: "cli-quickstart",
    category: "Guides",
    title: "CLI Quickstart",
    tags: ["terminal", "command", "script"],
    render: () => (
      <>
        <p>Run <code>jwt-tester ui</code> to start the local app.</p>
        <HelpItem title="Encode a token" badge="CLI">
          <div style={{ fontFamily: "monospace", fontSize: "0.85rem" }}>
            jwt-tester encode --alg hs256 --secret env:JWT_SECRET --iss auth --exp +30m
          </div>
        </HelpItem>
        <HelpItem title="Verify a token" badge="CLI">
          <div style={{ fontFamily: "monospace", fontSize: "0.85rem" }}>
            jwt-tester verify --project MyProject --aud api --require exp &lt;TOKEN&gt;
          </div>
        </HelpItem>
        <HelpItem title="Inspect (No Verify)" badge="CLI">
          <div style={{ fontFamily: "monospace", fontSize: "0.85rem" }}>
            jwt-tester inspect --date utc &lt;TOKEN&gt;
          </div>
        </HelpItem>
      </>
    ),
  },
];

const FAQS = [
  {
    q: "Why do I get an 'Invalid signature' error?",
    a: "This happens if the token content has changed, the wrong key is used, or the algorithm doesn't match the key type. Ensure you are using the exact key that signed the token.",
  },
  {
    q: "My token is expired. How can I still test it?",
    a: "In the Verifier, check the 'Ignore Expiration' option. This disables the 'exp' and 'nbf' checks, allowing you to debug the signature and other claims even if the token is old.",
  },
  {
    q: "What is an 'Audience mismatch'?",
    a: "The 'aud' claim in the token must match one of the expected audiences provided during verification. If the token has 'aud': 'api-v1', you must verify expecting 'api-v1'.",
  },
  {
    q: "Is 'Inspecting' the same as 'Verifying'?",
    a: "No! Inspecting only Base64-decodes the token so you can read it. It does NOT check the signature. Anyone can inspect a JWT. Only Verification ensures it hasn't been tampered with.",
  },
  {
    q: "Where are my keys stored?",
    a: "Keys are stored in a local SQLite vault file in your user data directory. You can export this vault from the Settings page for backup or transfer.",
  },
  {
    q: "How do I use environment variables with the CLI?",
    a: "Use the 'env:VAR_NAME' prefix for sensitive inputs. For example, '--secret env:MY_SECRET' reads the value from the 'MY_SECRET' environment variable instead of the command line args.",
  },
];

export function Help() {
  const [activeCategory, setActiveCategory] = useState("All");
  const [searchQuery, setSearchQuery] = useState("");

  const filteredContent = useMemo(() => {
    const lowerQuery = searchQuery.toLowerCase();
    
    return HELP_CONTENT.filter((section) => {
      // 1. Filter by Category
      if (activeCategory !== "All" && section.category !== activeCategory) {
        return false;
      }

      // 2. Filter by Search Query
      if (!lowerQuery) return true;

      const titleMatch = section.title.toLowerCase().includes(lowerQuery);
      const tagMatch = section.tags.some((tag) => tag.includes(lowerQuery));
      // Simple content heuristic: checking if component renders specific text is hard without
      // mounting, so we rely on title/tags for effective search.
      
      return titleMatch || tagMatch;
    });
  }, [activeCategory, searchQuery]);

  const showFaq = activeCategory === "All" || activeCategory === "FAQ";
  const filteredFaqs = useMemo(() => {
     if (!searchQuery) return FAQS;
     const lower = searchQuery.toLowerCase();
     return FAQS.filter(f => f.q.toLowerCase().includes(lower) || f.a.toLowerCase().includes(lower));
  }, [searchQuery]);

  return (
    <div className="view-container">
      <header className="card-header" style={{ borderBottom: "none", flexDirection: "column", gap: "1.5rem" }}>
        <div style={{ width: "100%", display: "flex", justifyContent: "space-between", alignItems: "flex-end", flexWrap: "wrap", gap: "1rem" }}>
          <div>
            <h1>Help & Documentation</h1>
            <p>Master the JWT Tester tools and concepts.</p>
          </div>
          <div className="field" style={{ marginBottom: 0, width: "100%", maxWidth: "300px" }}>
             <input 
                type="text" 
                placeholder="Search help..." 
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                style={{ paddingLeft: "1rem" }}
             />
          </div>
        </div>

        {/* Navigation Tabs */}
        <div style={{ display: "flex", gap: "0.5rem", overflowX: "auto", paddingBottom: "0.5rem", width: "100%" }}>
          {CATEGORIES.map((cat) => (
            <button
              key={cat}
              className={`pill ${activeCategory === cat ? "active" : ""}`}
              onClick={() => setActiveCategory(cat)}
              style={{ 
                cursor: "pointer", 
                fontSize: "0.9rem", 
                padding: "0.5rem 1rem",
                background: activeCategory === cat ? "rgba(14, 165, 233, 0.15)" : "var(--bg-panel)",
                border: activeCategory === cat ? "1px solid var(--primary)" : "1px solid var(--border)",
                color: activeCategory === cat ? "var(--primary)" : "var(--text-secondary)"
              }}
            >
              {cat}
            </button>
          ))}
        </div>
      </header>

      {/* Main Content Grid */}
      <div style={{ display: "flex", flexDirection: "column", gap: "1.5rem" }}>
        
        {/* Render filtered help sections */}
        {filteredContent.map((section) => (
          <Section key={section.id} title={section.title}>
            {section.render()}
          </Section>
        ))}

        {/* FAQ Section */}
        {showFaq && filteredFaqs.length > 0 && (
          <Section title="Frequently Asked Questions">
             <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem" }}>
               {filteredFaqs.map((faq, i) => (
                 <FAQItem key={i} question={faq.q} answer={faq.a} />
               ))}
             </div>
          </Section>
        )}

        {filteredContent.length === 0 && (!showFaq || filteredFaqs.length === 0) && (
            <div style={{ textAlign: "center", padding: "3rem", color: "var(--text-muted)" }}>
                No results found for "{searchQuery}"
            </div>
        )}

      </div>
    </div>
  );
}
