import React, { useState } from "react";
import { api, parseCsv, ALGORITHMS } from "../../api.js";
import { Tooltip } from "../Shared/Tooltip.jsx";

export function TokenBuilder({ projectName, keys, onRefresh, setStatus }) {
    const [alg, setAlg] = useState("hs256");
    const [keyId, setKeyId] = useState("");
    const [kid, setKid] = useState("");
    const [typ, setTyp] = useState("");
    const [iss, setIss] = useState("");
    const [sub, setSub] = useState("");
    const [aud, setAud] = useState("");
    const [jti, setJti] = useState("");
    const [iat, setIat] = useState("");
    const [nbf, setNbf] = useState("");
    const [exp, setExp] = useState("");
    const [claims, setClaims] = useState("{\n  \"sub\": \"123\"\n}");
    const [output, setOutput] = useState("");

    const handleEncode = async () => {
        if (!projectName) {
            setStatus("Select a project before encoding.");
            return;
        }

        let parsedClaims = {};
        try {
            parsedClaims = JSON.parse(claims || "{}");
        } catch (e) {
            setStatus("Invalid JSON in Custom Claims.");
            return;
        }

        const trimOrNull = (val) => {
            const trimmed = val.trim();
            return trimmed.length ? trimmed : null;
        };

        try {
            const res = await api("/api/jwt/encode", {
                method: "POST",
                body: JSON.stringify({
                    project: projectName,
                    key_id: keyId || null,
                    key_name: null,
                    alg,
                    claims: claims.trim() || null,
                    kid: kid.trim() || null,
                    typ: typ.trim() || null,
                    iss: iss.trim() || null,
                    sub: sub.trim() || null,
                    aud: parseCsv(aud),
                    jti: jti.trim() || null,
                    iat: trimOrNull(iat),
                    nbf: trimOrNull(nbf),
                    exp: trimOrNull(exp),
                }),
            });
            setOutput(res.data.token || "");
            setStatus("Token encoded.");
            await onRefresh();
        } catch (err) {
            setOutput("");
            setStatus(err?.message || "Token encoding failed.");
        }
    };

    if (!projectName) {
        return (
            <div className="view-container">
                <section className="card" style={{ textAlign: "center", padding: "4rem" }}>
                    <h2>No Project Selected</h2>
                    <p>Please select a project in the Dashboard to generate tokens.</p>
                </section>
            </div>
        );
    }

    return (
        <div className="view-container">
            <header className="card-header" style={{ borderBottom: "none", paddingBottom: 0 }}>
                <div>
                    <h1>Token Builder</h1>
                    <p>Create and sign JWTs using your vault keys.</p>
                </div>
            </header>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '2rem', alignItems: 'start' }}>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
                    <section className="card">
                        <div className="card-header">
                            <h2>Configuration</h2>
                        </div>

                        <div className="row">
                            <label className="field" style={{ flex: 1 }} htmlFor="builder-algorithm">
                                <span>Algorithm <Tooltip text="The signing algorithm to use (e.g. HS256, RS256)">ℹ️</Tooltip></span>
                                <select id="builder-algorithm" value={alg} onChange={(event) => setAlg(event.target.value)}>
                                    {ALGORITHMS.map((algo) => (
                                        <option key={algo} value={algo}>
                                            {algo.toUpperCase()}
                                        </option>
                                    ))}
                                </select>
                            </label>
                            <label className="field" style={{ flex: 1 }} htmlFor="builder-key">
                                <span>Signing Key <Tooltip text="The key used to sign the token">ℹ️</Tooltip></span>
                                <select id="builder-key" value={keyId} onChange={(event) => setKeyId(event.target.value)}>
                                    <option value="">(Use Default Key)</option>
                                    {keys.map((key) => (
                                        <option key={key.id} value={key.id}>
                                            {key.name || "Unnamed"} ({key.id.slice(0, 8)})
                                        </option>
                                    ))}
                                </select>
                            </label>
                        </div>
                    </section>

                    <section className="card">
                        <div className="card-header">
                            <h2>Payload Claims</h2>
                        </div>

                        <h3 style={{ fontSize: '0.9rem', color: 'var(--text-muted)', marginBottom: '1rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Standard Claims</h3>
                        <div className="row">
                            <label className="field" style={{ flex: 1 }} htmlFor="builder-iss">
                                <span>iss (Issuer) <Tooltip text="Principal that issued this token">ℹ️</Tooltip></span>
                                <input id="builder-iss" value={iss} onChange={(event) => setIss(event.target.value)} placeholder="e.g. auth.service.com" />
                            </label>
                            <label className="field" style={{ flex: 1 }} htmlFor="builder-sub">
                                <span>sub (Subject) <Tooltip text="Principal that is the subject of the token">ℹ️</Tooltip></span>
                                <input id="builder-sub" value={sub} onChange={(event) => setSub(event.target.value)} placeholder="e.g. user_123" />
                            </label>
                        </div>
                        <div className="row">
                            <label className="field" style={{ flex: 1 }} htmlFor="builder-aud">
                                <span>aud (Audience) <Tooltip text="Recipients that the JWT is intended for (comma separated)">ℹ️</Tooltip></span>
                                <input id="builder-aud" value={aud} onChange={(event) => setAud(event.target.value)} placeholder="e.g. api, app" />
                            </label>
                            <label className="field" style={{ flex: 1 }} htmlFor="builder-jti">
                                <span>jti (JWT ID) <Tooltip text="Unique identifier for the JWT">ℹ️</Tooltip></span>
                                <input id="builder-jti" value={jti} onChange={(event) => setJti(event.target.value)} placeholder="Unique ID" />
                            </label>
                        </div>

                        <div className="divider" />

                        <h3 style={{ fontSize: '0.9rem', color: 'var(--text-muted)', marginBottom: '1rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Time Claims</h3>
                        <div className="row">
                            <label className="field" style={{ flex: 1 }} htmlFor="builder-iat">
                                <span>iat (Issued At) <Tooltip text="Time at which the JWT was issued (numeric timestamp)">ℹ️</Tooltip></span>
                                <input id="builder-iat" value={iat} onChange={(event) => setIat(event.target.value)} placeholder="Example: 1735770000" />
                            </label>
                            <label className="field" style={{ flex: 1 }} htmlFor="builder-nbf">
                                <span>nbf (Not Before) <Tooltip text="Time before which the JWT must not be accepted">ℹ️</Tooltip></span>
                                <input id="builder-nbf" value={nbf} onChange={(event) => setNbf(event.target.value)} placeholder="Example: 1735770000" />
                            </label>
                            <label className="field" style={{ flex: 1 }} htmlFor="builder-exp">
                                <span>exp (Expiration) <Tooltip text="Expiration time on or after which the JWT must not be accepted">ℹ️</Tooltip></span>
                                <input id="builder-exp" value={exp} onChange={(event) => setExp(event.target.value)} placeholder="Example: 1735773600" />
                            </label>
                        </div>
                    </section>

                    <section className="card">
                        <div className="card-header">
                            <h2>Custom Data</h2>
                        </div>
                        <label className="field" htmlFor="builder-claims">
                            <span>JSON Payload <Tooltip text="Additional data to include in the payload (must be valid JSON)">ℹ️</Tooltip></span>
                            <textarea
                                id="builder-claims"
                                value={claims}
                                onChange={(event) => setClaims(event.target.value)}
                                rows={8}
                                style={{ fontFamily: 'monospace', fontSize: '0.9rem', lineHeight: '1.4' }}
                            />
                        </label>
                        <div className="row" style={{ marginTop: "1rem", justifyContent: 'flex-end' }}>
                            <button className="button primary" onClick={handleEncode} disabled={!projectName} style={{ width: '100%' }}>
                                Generate Token
                            </button>
                        </div>
                    </section>
                </div>

                <div style={{ position: 'sticky', top: '1rem', display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
                    <section className="card" style={{ borderColor: 'var(--secondary)' }}>
                        <div className="card-header">
                            <h2 style={{ color: 'var(--secondary)' }}>Live Preview</h2>
                        </div>
                        <PreviewJSON
                            alg={alg}
                            typ={typ}
                            kid={kid}
                            iss={iss}
                            sub={sub}
                            aud={aud}
                            jti={jti}
                            iat={iat}
                            nbf={nbf}
                            exp={exp}
                            claims={claims}
                        />
                    </section>

                    {output && (
                        <section className="card" style={{ borderColor: 'var(--success)', animation: 'slideUp 0.3s ease-out' }}>
                            <div className="card-header">
                                <h2 style={{ color: 'var(--success)' }}>Generated Token</h2>
                            </div>
                            <label className="field">
                                <textarea
                                    value={output}
                                    readOnly
                                    rows={6}
                                    style={{
                                        fontFamily: 'monospace',
                                        color: 'var(--token-string, #a5d6ff)',
                                        borderColor: 'var(--success)',
                                        background: 'rgba(16, 185, 129, 0.05)',
                                        fontSize: '0.85rem'
                                    }}
                                />
                            </label>
                            <button
                                className="button ghost"
                                style={{ marginTop: '0.5rem', width: '100%', fontSize: '0.8rem' }}
                                onClick={() => navigator.clipboard.writeText(output)}
                            >
                                Copy to Clipboard
                            </button>
                        </section>
                    )}
                </div>
            </div>
        </div>
    );
}

function PreviewJSON({ alg, typ, kid, iss, sub, aud, jti, iat, nbf, exp, claims }) {
    let customClaims = {};
    try {
        customClaims = JSON.parse(claims || "{}");
    } catch (e) {
        // Ignore parse errors for preview
    }

    const header = {
        alg: alg.toUpperCase(),
        typ: typ || "JWT",
        ...(kid && { kid }),
    };

    const payload = {
        ...(iss && { iss }),
        ...(sub && { sub }),
        ...(aud && { aud: aud.includes(',') ? aud.split(',').map(s => s.trim()) : aud }),
        ...(jti && { jti }),
        ...(iat && { iat }),
        ...(nbf && { nbf }),
        ...(exp && { exp }),
        ...customClaims
    };

    return (
        <div style={{ fontFamily: 'monospace', fontSize: '0.9rem' }}>
            <div style={{ marginBottom: '1.5rem' }}>
                <span style={{ color: 'var(--text-muted)', display: 'block', marginBottom: '0.5rem', fontSize: '0.75rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Header</span>
                <div style={{ background: 'var(--bg-input)', padding: '1rem', borderRadius: '8px', border: '1px solid var(--border)' }}>
                    <pre style={{ margin: 0, color: 'var(--secondary)', overflowX: 'auto' }}>
                        {JSON.stringify(header, null, 2)}
                    </pre>
                </div>
            </div>

            <div>
                <span style={{ color: 'var(--text-muted)', display: 'block', marginBottom: '0.5rem', fontSize: '0.75rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Payload</span>
                <div style={{ background: 'var(--bg-input)', padding: '1rem', borderRadius: '8px', border: '1px solid var(--border)' }}>
                    <pre style={{ margin: 0, color: 'var(--primary)', overflowX: 'auto' }}>
                        {JSON.stringify(payload, null, 2)}
                    </pre>
                </div>
            </div>
        </div>
    );
}
