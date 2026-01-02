import React, { useEffect, useState } from "react";
import { api, downloadText, formatTags, parseCsv } from "../../api.js";
import { Modal } from "../Shared/Modal.jsx";

// Icons
const KeyIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4" /></svg>
);
const TokenIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10" /><line x1="12" y1="8" x2="12" y2="12" /><line x1="12" y1="16" x2="12.01" y2="16" /></svg> 
);
const TokenFileIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>
);
const TrashIcon = () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
);
const DownloadIcon = () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
);
const CopyIcon = () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
);
const EyeIcon = () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg>
);
const EyeOffIcon = () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path><line x1="1" y1="1" x2="23" y2="23"></line></svg>
);
const StarIcon = ({ filled }) => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill={filled ? "currentColor" : "none"} stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon></svg>
);
const PlusIcon = () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
);

function AddKeyModal({ isOpen, onClose, projectId, onRefresh, setStatus, setGenerated }) {
    const [mode, setMode] = useState("manual");
    const [name, setName] = useState("");
    const [kind, setKind] = useState("hmac");
    const [kid, setKid] = useState("");
    const [description, setDescription] = useState("");
    const [tags, setTags] = useState("");
    const [secret, setSecret] = useState("");
    const [hmacBytes, setHmacBytes] = useState("32");
    const [rsaBits, setRsaBits] = useState("2048");
    const [ecCurve, setEcCurve] = useState("P-256");

    useEffect(() => {
        if (!isOpen) {
            // Reset fields on close
            setMode("manual");
            setName("");
            setKind("hmac");
            setKid("");
            setDescription("");
            setTags("");
            setSecret("");
        }
    }, [isOpen]);

    const parsePositiveInt = (value) => {
        const parsed = Number.parseInt(value, 10);
        return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
    };

    const handleAdd = async () => {
        if (!projectId) return;
        if (!secret.trim()) {
            setStatus("Secret/key material is required.");
            return;
        }
        await api("/api/vault/keys", {
            method: "POST",
            body: JSON.stringify({
                project_id: projectId,
                name: name.trim(),
                kind,
                secret: secret.trim(),
                kid: kid.trim() || null,
                description: description.trim() || null,
                tags: parseCsv(tags),
            }),
        });
        setStatus("Key saved.");
        await onRefresh();
        onClose();
    };

    const handleGenerate = async () => {
        if (!projectId) return;
        if (kind === "jwks") {
            setStatus("JWKS generation is not supported. Paste JWKS JSON.");
            return;
        }

        const payload = {
            project_id: projectId,
            name: name.trim(),
            kind,
            kid: kid.trim() || null,
            description: description.trim() || null,
            tags: parseCsv(tags),
        };

        if (kind === "hmac") {
            const bytes = parsePositiveInt(hmacBytes);
            if (!bytes) {
                setStatus("HMAC secret length must be a positive number.");
                return;
            }
            payload.hmac_bytes = bytes;
        }
        if (kind === "rsa") {
            const bits = parsePositiveInt(rsaBits);
            if (!bits) {
                setStatus("RSA key size is required.");
                return;
            }
            payload.rsa_bits = bits;
        }
        if (kind === "ec") {
            payload.ec_curve = ecCurve;
        }

        const res = await api("/api/vault/keys/generate", {
            method: "POST",
            body: JSON.stringify(payload),
        });

        setGenerated(
            res?.data?.material
                ? {
                      key: res.data.key,
                      material: res.data.material,
                      format: res.data.format || "pem",
                  }
                : null
        );
        setStatus("Key generated.");
        await onRefresh();
        onClose();
    };

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title="Add New Key"
            footer={
                <div style={{ display: 'flex', gap: '1rem', width: '100%', justifyContent: 'flex-end' }}>
                     <button className="button ghost" onClick={onClose}>Cancel</button>
                     <button className="button primary" onClick={mode === "manual" ? handleAdd : handleGenerate}>
                         {mode === "manual" ? "Save Key" : "Generate Key"}
                     </button>
                </div>
            }
        >
            <div className="row" style={{ gap: "0.5rem", marginBottom: "1.5rem" }}>
                <button
                    type="button"
                    className={`pill ${mode === "manual" ? "active" : ""}`}
                    onClick={() => setMode("manual")}
                >
                    Manual Entry
                </button>
                <button
                    type="button"
                    className={`pill ${mode === "generate" ? "active" : ""}`}
                    onClick={() => setMode("generate")}
                >
                    Generate New
                </button>
            </div>

            <div className="row">
                <label className="field" style={{ flex: 1 }}>
                    <span>Name</span>
                    <input
                        value={name}
                        onChange={(e) => setName(e.target.value)}
                        placeholder="e.g. production-signing-key"
                    />
                </label>
                <label className="field" style={{ width: '120px' }}>
                    <span>Kind</span>
                    <select value={kind} onChange={(e) => setKind(e.target.value)}>
                        <option value="hmac">HMAC</option>
                        <option value="rsa">RSA</option>
                        <option value="ec">EC</option>
                        <option value="eddsa">EdDSA</option>
                        <option value="jwks">JWKS</option>
                    </select>
                </label>
            </div>

            <div className="row">
                <label className="field" style={{ flex: 1 }}>
                    <span>KID (Optional)</span>
                    <input
                        value={kid}
                        onChange={(e) => setKid(e.target.value)}
                        placeholder="e.g. kid-2024-v1"
                    />
                </label>
            </div>
            
            <label className="field">
                <span>Description</span>
                <input
                    value={description}
                    onChange={(e) => setDescription(e.target.value)}
                    placeholder="e.g. Used for session signing"
                />
            </label>

            <label className="field">
                <span>Tags</span>
                <input
                    value={tags}
                    onChange={(e) => setTags(e.target.value)}
                    placeholder="e.g. signing, prod"
                />
            </label>

            {mode === "manual" ? (
                <label className="field">
                    <span>Secret / Key Material</span>
                    <textarea
                        value={secret}
                        onChange={(e) => setSecret(e.target.value)}
                        rows={5}
                        placeholder="Paste secret, PEM, or JWKS JSON"
                        style={{ fontFamily: 'monospace', fontSize: '0.85rem' }}
                    />
                </label>
            ) : (
                <div className="card" style={{ background: 'var(--bg-input)', border: '1px dashed var(--border)' }}>
                     {kind === "hmac" && (
                        <label className="field">
                            <span>Secret Length (bytes)</span>
                            <input
                                type="number"
                                min="16"
                                max="128"
                                value={hmacBytes}
                                onChange={(e) => setHmacBytes(e.target.value)}
                            />
                        </label>
                    )}
                    {kind === "rsa" && (
                        <label className="field">
                            <span>RSA Key Size</span>
                            <select value={rsaBits} onChange={(e) => setRsaBits(e.target.value)}>
                                <option value="2048">2048-bit</option>
                                <option value="3072">3072-bit</option>
                                <option value="4096">4096-bit</option>
                            </select>
                        </label>
                    )}
                    {kind === "ec" && (
                        <label className="field">
                            <span>Curve</span>
                            <select value={ecCurve} onChange={(e) => setEcCurve(e.target.value)}>
                                <option value="P-256">P-256</option>
                                <option value="P-384">P-384</option>
                            </select>
                        </label>
                    )}
                    {kind === "eddsa" && (
                        <div style={{ padding: '0.5rem 0', color: 'var(--text-muted)' }}>
                            Algorithm: Ed25519
                        </div>
                    )}
                    {kind === "jwks" && (
                        <div style={{ padding: '0.5rem 0', color: 'var(--error)' }}>
                             Generation not supported. Use Manual mode.
                        </div>
                    )}
                    <p style={{ fontSize: '0.8rem', color: 'var(--text-muted)', marginTop: '0.5rem' }}>
                         Key will be generated securely and stored in the vault.
                    </p>
                </div>
            )}
        </Modal>
    );
}

function AddTokenModal({ isOpen, onClose, projectId, onRefresh, setStatus, onNavigate }) {
    const [name, setName] = useState("");
    const [token, setToken] = useState("");

    useEffect(() => {
        if (!isOpen) {
            setName("");
            setToken("");
        }
    }, [isOpen]);

    const handleAdd = async () => {
        if (!projectId) return;
        if (!token.trim()) {
            setStatus("Token is required.");
            return;
        }
        await api("/api/vault/tokens", {
            method: "POST",
            body: JSON.stringify({ project_id: projectId, name: name.trim(), token: token.trim() }),
        });
        setStatus("Token saved.");
        await onRefresh();
        onClose();
    };

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title="Save Token"
            footer={
                <div style={{ display: 'flex', gap: '1rem', width: '100%', justifyContent: 'space-between' }}>
                     <div>
                        {onNavigate && (
                            <button className="button ghost" onClick={() => { onClose(); onNavigate("builder"); }}>
                                Open Token Builder
                            </button>
                        )}
                     </div>
                     <div style={{ display: 'flex', gap: '1rem' }}>
                         <button className="button ghost" onClick={onClose}>Cancel</button>
                         <button className="button primary" onClick={handleAdd}>Save Token</button>
                     </div>
                </div>
            }
        >
             <label className="field">
                <span>Name</span>
                <input
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder="e.g. admin-user-token"
                />
            </label>
            <label className="field">
                <span>Token (JWT)</span>
                <textarea
                    value={token}
                    onChange={(e) => setToken(e.target.value)}
                    rows={5}
                    placeholder="eyJ..."
                    style={{ fontFamily: 'monospace', fontSize: '0.85rem' }}
                />
            </label>
            <div style={{ color: 'var(--text-muted)', fontSize: '0.85rem', marginTop: '0.5rem' }}>
                Need to create a token first? Use the Token Builder.
            </div>
        </Modal>
    );
}

function KeysSection({ projectId, keys, defaultKeyId, onRefresh, setStatus, setGenerated }) {
    const [filter, setFilter] = useState("");
    const [isAddModalOpen, setIsAddModalOpen] = useState(false);

    const filteredKeys = keys.filter(k => 
        k.name?.toLowerCase().includes(filter.toLowerCase()) || 
        k.id.includes(filter) ||
        k.kid?.toLowerCase().includes(filter.toLowerCase()) ||
        k.tags?.some(t => t.toLowerCase().includes(filter.toLowerCase()))
    );

    const handleDelete = async (id) => {
        if (!window.confirm("Delete this key?")) return;
        await api(`/api/vault/keys/${id}`, { method: "DELETE" });
        setStatus("Key deleted.");
        await onRefresh();
    };

    const handleDefault = async (id, isDefault) => {
        if (!projectId) return;
        await api(`/api/vault/projects/${projectId}/default-key`, {
            method: "POST",
            body: JSON.stringify({ key_id: isDefault ? null : id }),
        });
        setStatus(isDefault ? "Default key cleared." : "Default key set.");
        await onRefresh();
    };

    return (
        <section className="card">
            <div className="card-header">
                <div>
                    <h2>Keys</h2>
                    <p>Cryptographic keys for signing and verification.</p>
                </div>
                <button className="button primary" onClick={() => setIsAddModalOpen(true)}>
                    <PlusIcon /> <span style={{ marginLeft: '0.5rem' }}>Add Key</span>
                </button>
            </div>

            <div style={{ marginBottom: '1rem' }}>
                <input 
                    placeholder="Filter keys..." 
                    value={filter} 
                    onChange={e => setFilter(e.target.value)}
                    style={{ maxWidth: '300px' }}
                />
            </div>

            <div className="list">
                {filteredKeys.length ? (
                    filteredKeys.map((key) => {
                        const isDefault = key.id === defaultKeyId;
                        return (
                            <div className="list-item" key={key.id}>
                                <div style={{ display: 'flex', gap: '1rem', alignItems: 'center', flex: 1 }}>
                                    <div style={{ 
                                        width: '40px', height: '40px', 
                                        borderRadius: '8px', background: 'rgba(14, 165, 233, 0.1)', 
                                        display: 'flex', alignItems: 'center', justifyContent: 'center',
                                        color: 'var(--primary)'
                                    }}>
                                        <KeyIcon />
                                    </div>
                                    <div>
                                        <div className="list-title" style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                            {key.name || "Unnamed Key"}
                                            {isDefault && <span className="badge">Default</span>}
                                        </div>
                                        <div className="list-sub" style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                                            <span style={{ textTransform: 'uppercase', fontWeight: 600, fontSize: '0.7rem', border: '1px solid var(--border)', padding: '0 4px', borderRadius: '4px' }}>
                                                {key.kind}
                                            </span>
                                            <span className="mono">{key.id.slice(0, 8)}</span>
                                            {key.kid && <span>• kid: {key.kid}</span>}
                                        </div>
                                        {key.tags?.length > 0 && (
                                             <div className="list-sub" style={{ marginTop: '0.25rem' }}>
                                                {formatTags(key.tags)}
                                             </div>
                                        )}
                                    </div>
                                </div>
                                <div className="row" style={{ gap: '0.5rem' }}>
                                    <button
                                        className="button ghost"
                                        title={isDefault ? "Unset Default" : "Set Default"}
                                        onClick={() => handleDefault(key.id, isDefault)}
                                        style={{ color: isDefault ? 'var(--warning)' : 'var(--text-secondary)' }}
                                    >
                                        <StarIcon filled={isDefault} />
                                    </button>
                                    <button
                                        className="button ghost"
                                        title="Delete"
                                        style={{ color: 'var(--error)' }}
                                        onClick={() => handleDelete(key.id)}
                                    >
                                        <TrashIcon />
                                    </button>
                                </div>
                            </div>
                        );
                    })
                ) : (
                    <div className="empty" style={{ padding: '2rem', textAlign: 'center', color: 'var(--text-muted)' }}>
                        {filter ? "No keys match filter." : "No keys found. Add one above."}
                    </div>
                )}
            </div>

            <AddKeyModal 
                isOpen={isAddModalOpen} 
                onClose={() => setIsAddModalOpen(false)} 
                projectId={projectId}
                onRefresh={onRefresh}
                setStatus={setStatus}
                setGenerated={setGenerated}
            />
        </section>
    );
}

function TokensSection({ projectId, tokens, onRefresh, setStatus, onNavigate }) {
    const [filter, setFilter] = useState("");
    const [isAddModalOpen, setIsAddModalOpen] = useState(false);
    const [revealed, setRevealed] = useState({});
    const [tokenMaterial, setTokenMaterial] = useState({});

    // Cleanup logic similar to original
    useEffect(() => {
        const ids = new Set(tokens.map((entry) => entry.id));
        setRevealed((prev) =>
            Object.fromEntries(
                Object.entries(prev).filter(([id, value]) => ids.has(id) && value)
            )
        );
        setTokenMaterial((prev) =>
            Object.fromEntries(Object.entries(prev).filter(([id]) => ids.has(id)))
        );
    }, [tokens]);

    const filteredTokens = tokens.filter(t => 
        t.name?.toLowerCase().includes(filter.toLowerCase()) || 
        t.id.includes(filter)
    );

    const resolveTokenFilename = (entry) => {
        if (!entry) return "jwt-tester-token.jwt";
        const base = entry.name?.trim() || `token-${entry.id?.slice(0, 8) || "stored"}`;
        return `${base}.jwt`;
    };

    const handleReveal = async (entry) => {
        if (!entry?.id) return;
        if (revealed[entry.id]) {
            setRevealed((prev) => ({ ...prev, [entry.id]: false }));
            setTokenMaterial((prev) => {
                const next = { ...prev };
                delete next[entry.id];
                return next;
            });
            setStatus("Token hidden.");
            return;
        }
        try {
            const res = await api(`/api/vault/tokens/${entry.id}/material`, {
                method: "POST",
            });
            const material = res?.data?.token;
            if (!material) {
                setStatus("Token material missing.");
                return;
            }
            setTokenMaterial((prev) => ({ ...prev, [entry.id]: material }));
            setRevealed((prev) => ({ ...prev, [entry.id]: true }));
            setStatus("Token revealed.");
        } catch (err) {
            setStatus(err?.message || "Failed to reveal token.");
        }
    };

    const handleDelete = async (id) => {
        if (!window.confirm("Delete this saved token?")) return;
        await api(`/api/vault/tokens/${id}`, { method: "DELETE" });
        setStatus("Token deleted.");
        await onRefresh();
    };

    const handleCopy = async (entry) => {
        const material = tokenMaterial[entry.id];
        if (!material) {
            setStatus("Reveal the token first.");
            return;
        }
        try {
            await navigator.clipboard.writeText(material);
            setStatus("Token copied to clipboard.");
        } catch (err) {
            setStatus("Failed to copy token.");
        }
    };

    const handleDownload = (entry) => {
        const material = tokenMaterial[entry.id];
        if (!material) {
            setStatus("Reveal the token first.");
            return;
        }
        const filename = resolveTokenFilename(entry);
        downloadText(filename, material, "text/plain");
        setStatus(`Downloaded ${filename}.`);
    };

    return (
        <section className="card">
             <div className="card-header">
                <div>
                    <h2>Saved Tokens</h2>
                    <p>JWTs stored for quick access.</p>
                </div>
                <button className="button primary" onClick={() => setIsAddModalOpen(true)}>
                    <PlusIcon /> <span style={{ marginLeft: '0.5rem' }}>Save Token</span>
                </button>
            </div>

            <div style={{ marginBottom: '1rem' }}>
                <input 
                    placeholder="Filter tokens..." 
                    value={filter} 
                    onChange={e => setFilter(e.target.value)}
                    style={{ maxWidth: '300px' }}
                />
            </div>

            <div className="list">
                {filteredTokens.length ? (
                    filteredTokens.map((entry) => {
                        const isRevealed = revealed[entry.id];
                        const material = tokenMaterial[entry.id];

                        return (
                            <div className="list-item" key={entry.id} style={{ display: 'block' }}>
                                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                                    <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
                                        <div style={{ 
                                            width: '40px', height: '40px', 
                                            borderRadius: '8px', background: 'rgba(139, 92, 246, 0.1)', 
                                            display: 'flex', alignItems: 'center', justifyContent: 'center',
                                            color: 'var(--secondary)'
                                        }}>
                                            <TokenFileIcon />
                                        </div>
                                        <div>
                                            <div className="list-title">
                                                {entry.name || "Unnamed Token"}
                                            </div>
                                            <div className="list-sub mono">
                                                {entry.id.slice(0, 8)}
                                            </div>
                                        </div>
                                    </div>
                                    
                                    <div className="row" style={{ gap: '0.5rem' }}>
                                         <button
                                            className="button ghost"
                                            title={isRevealed ? "Hide" : "Reveal"}
                                            onClick={() => handleReveal(entry)}
                                        >
                                            {isRevealed ? <EyeOffIcon /> : <EyeIcon />}
                                        </button>
                                         <button
                                            className="button ghost"
                                            title="Copy"
                                            disabled={!material}
                                            onClick={() => handleCopy(entry)}
                                        >
                                            <CopyIcon />
                                        </button>
                                         <button
                                            className="button ghost"
                                            title="Download"
                                            disabled={!material}
                                            onClick={() => handleDownload(entry)}
                                        >
                                            <DownloadIcon />
                                        </button>
                                        <button
                                            className="button ghost"
                                            title="Delete"
                                            style={{ color: 'var(--error)' }}
                                            onClick={() => handleDelete(entry.id)}
                                        >
                                            <TrashIcon />
                                        </button>
                                    </div>
                                </div>

                                {isRevealed && material && (
                                    <div style={{ marginTop: '1rem', position: 'relative' }}>
                                        <textarea
                                            readOnly
                                            value={material}
                                            rows={3}
                                            style={{
                                                fontFamily: "monospace",
                                                fontSize: "0.85rem",
                                                background: "var(--bg-input)",
                                                width: '100%',
                                                resize: 'vertical'
                                            }}
                                        />
                                    </div>
                                )}
                            </div>
                        );
                    })
                ) : (
                    <div className="empty" style={{ padding: '2rem', textAlign: 'center', color: 'var(--text-muted)' }}>
                        {filter ? "No tokens match filter." : "No saved tokens."}
                    </div>
                )}
            </div>

            <AddTokenModal 
                isOpen={isAddModalOpen}
                onClose={() => setIsAddModalOpen(false)}
                projectId={projectId}
                onRefresh={onRefresh}
                setStatus={setStatus}
                onNavigate={onNavigate}
            />
        </section>
    );
}

export function VaultManager({
    projectId,
    keys,
    tokens,
    defaultKeyId,
    onRefresh,
    setStatus,
    onNavigate
}) {
    // Shared state for generated key modal/alert
    const [generated, setGenerated] = useState(null);
    const [showMaterial, setShowMaterial] = useState(false);

    const resolveGeneratedFilename = (entry) => {
        if (!entry) return "jwt-tester-key.txt";
        const base = entry.key?.name?.trim() || `key-${entry.key?.id?.slice(0, 8) || "generated"}`;
        const ext = entry.format === "pem" ? "pem" : "txt";
        return `${base}.${ext}`;
    };

    const handleCopyGenerated = async () => {
        if (!generated?.material) return;
        try {
            await navigator.clipboard.writeText(generated.material);
            setStatus("Key material copied to clipboard.");
        } catch (err) {
            setStatus("Failed to copy key material.");
        }
    };

    const handleDownloadGenerated = () => {
        if (!generated?.material) return;
        const filename = resolveGeneratedFilename(generated);
        const contentType =
            generated.format === "pem" ? "application/x-pem-file" : "text/plain";
        downloadText(filename, generated.material, contentType);
        setStatus(`Downloaded ${filename}.`);
    };

    if (!projectId) {
        return (
            <div className="view-container">
                <section className="card" style={{ textAlign: "center", padding: "4rem", display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '1rem' }}>
                    <div style={{ width: '64px', height: '64px', borderRadius: '50%', background: 'var(--bg-panel-hover)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                        <span style={{ fontSize: '2rem' }}>🔐</span>
                    </div>
                    <div>
                        <h2>No Project Selected</h2>
                        <p>Please select a project in the sidebar to manage keys and tokens.</p>
                    </div>
                </section>
            </div>
        );
    }

    return (
        <div className="view-container">
             {generated && (
                <div className="card" style={{ 
                    border: '1px solid var(--success)', 
                    background: 'rgba(16, 185, 129, 0.05)',
                    marginBottom: '2rem',
                    animation: 'slideUp 0.3s ease-out'
                }}>
                    <div className="card-header" style={{ borderBottom: '1px solid rgba(16, 185, 129, 0.2)' }}>
                        <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center' }}>
                            <div style={{ color: 'var(--success)' }}><TokenIcon /></div> {/* Reusing icon */}
                            <div>
                                <h4 style={{ margin: 0, color: 'var(--success)' }}>Key Generated Successfully</h4>
                                <p style={{ fontSize: '0.9rem', opacity: 0.9 }}>
                                    This is the only time you will see the key material. Save it now.
                                </p>
                            </div>
                        </div>
                         <div className="row" style={{ gap: "0.5rem" }}>
                             <button className="button ghost" onClick={handleCopyGenerated}><CopyIcon /> Copy</button>
                             <button className="button ghost" onClick={handleDownloadGenerated}><DownloadIcon /> Download</button>
                             <button className="button ghost" onClick={() => { setGenerated(null); setShowMaterial(false); }}>Close</button>
                        </div>
                    </div>
                    
                    <div style={{ padding: '0 1.5rem 1.5rem 1.5rem' }}>
                         <div style={{ display: 'flex', justifyContent: 'flex-end', marginBottom: '0.5rem' }}>
                            <button
                                className="button ghost"
                                style={{ fontSize: '0.8rem', padding: '0.25rem 0.5rem' }}
                                onClick={() => setShowMaterial((prev) => !prev)}
                            >
                                {showMaterial ? "Hide Secret" : "Show Secret"}
                            </button>
                        </div>
                        {showMaterial ? (
                            <textarea
                                readOnly
                                value={generated.material}
                                rows={6}
                                style={{ 
                                    fontFamily: "monospace", 
                                    fontSize: "0.85rem", 
                                    width: '100%',
                                    background: 'var(--bg-main)',
                                    borderColor: 'var(--success)'
                                }}
                            />
                        ) : (
                            <div style={{ 
                                background: 'var(--bg-main)', 
                                padding: '1.5rem', 
                                borderRadius: '8px', 
                                textAlign: 'center', 
                                color: 'var(--text-muted)',
                                border: '1px dashed var(--success)'
                            }}>
                                ••••••••••••••••••••••••••••••
                            </div>
                        )}
                    </div>
                </div>
            )}

            <div className="two-column-layout">
                <div style={{ flex: 1 }}>
                     <KeysSection
                        projectId={projectId}
                        keys={keys}
                        defaultKeyId={defaultKeyId}
                        onRefresh={onRefresh}
                        setStatus={setStatus}
                        setGenerated={setGenerated}
                    />
                </div>
                <div style={{ flex: 1 }}>
                    <TokensSection
                        projectId={projectId}
                        tokens={tokens}
                        onRefresh={onRefresh}
                        setStatus={setStatus}
                        onNavigate={onNavigate}
                    />
                </div>
            </div>
        </div>
    );
}
