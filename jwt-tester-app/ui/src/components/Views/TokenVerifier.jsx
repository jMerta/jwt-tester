import React, { useState } from "react";
import { api, parseCsv, ALGORITHMS } from "../../api.js";

export function TokenVerifier({ projectName, keys, setStatus }) {
    const [token, setToken] = useState("");
    const [alg, setAlg] = useState("");
    const [keyId, setKeyId] = useState("");
    const [iss, setIss] = useState("");
    const [sub, setSub] = useState("");
    const [aud, setAud] = useState("");
    const [requireClaims, setRequireClaims] = useState("");
    const [leeway, setLeeway] = useState("30");
    const [tryAll, setTryAll] = useState(false);
    const [ignoreExp, setIgnoreExp] = useState(false);
    const [explain, setExplain] = useState(false);
    const [output, setOutput] = useState("");
    const [loading, setLoading] = useState(false);

    const handleVerify = async () => {
        if (!projectName) {
            setStatus("Select a project before verifying.");
            return;
        }
        if (!token.trim()) {
            setStatus("Token is required.");
            return;
        }
        const leewayValue = leeway.trim() ? Number(leeway) : null;
        if (leeway.trim() && Number.isNaN(leewayValue)) {
            setStatus("Leeway must be a number.");
            return;
        }

        setLoading(true);
        try {
            const res = await api("/api/jwt/verify", {
                method: "POST",
                body: JSON.stringify({
                    project: projectName,
                    key_id: keyId || null,
                    key_name: null,
                    alg: alg || null,
                    token,
                    try_all_keys: tryAll,
                    ignore_exp: ignoreExp,
                    leeway_secs: leewayValue,
                    iss: iss.trim() || null,
                    sub: sub.trim() || null,
                    aud: parseCsv(aud),
                    require: parseCsv(requireClaims),
                    explain,
                }),
            });
            setOutput(JSON.stringify(res.data, null, 2));
            setStatus("Verification complete.");
        } catch (err) {
            setOutput("");
            setStatus(err?.message || "Verification failed.");
        } finally {
            setLoading(false);
        }
    };

    if (!projectName) {
        return (
            <div className="view-container">
                <section className="card" style={{ textAlign: "center", padding: "4rem" }}>
                    <h2>No Project Selected</h2>
                    <p>Please select a project in the Dashboard to verify tokens.</p>
                </section>
            </div>
        );
    }

    const isValidResult = output && output.includes('"valid": true');

    return (
        <div className="view-container">
            <header className="card-header" style={{ borderBottom: "none" }}>
                <div>
                    <h1>Token Verifier</h1>
                    <p>Verify signatures and claims against your vault keys.</p>
                </div>
            </header>

            <div className="two-column-layout">
                <section className="card" style={{ height: 'fit-content' }}>
                    <label className="field" htmlFor="verify-token">
                        <span>Token to Verify</span>
                        <textarea
                            id="verify-token"
                            value={token}
                            onChange={(event) => setToken(event.target.value)}
                            rows={5}
                            className="mono-input"
                            style={{ fontFamily: 'monospace', fontSize: '0.85rem' }}
                            placeholder="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
                        />
                    </label>

                    <div className="row">
                        <label className="field" style={{ flex: 1 }} htmlFor="verify-algorithm">
                            <span>Algorithm</span>
                            <select
                                id="verify-algorithm"
                                value={alg}
                                onChange={(event) => setAlg(event.target.value)}
                            >
                                <option value="">Auto (Infer from header)</option>
                                {ALGORITHMS.map((algo) => (
                                    <option key={algo} value={algo}>
                                        {algo.toUpperCase()}
                                    </option>
                                ))}
                            </select>
                        </label>
                        <label className="field" style={{ flex: 1 }} htmlFor="verify-key">
                            <span>Verification Key</span>
                            <select
                                id="verify-key"
                                value={keyId}
                                onChange={(event) => setKeyId(event.target.value)}
                            >
                                <option value="">(Use Default Key)</option>
                                {keys.map((key) => (
                                    <option key={key.id} value={key.id}>
                                        {key.name || "Unnamed"} ({key.id.slice(0, 8)})
                                    </option>
                                ))}
                            </select>
                        </label>
                    </div>

                    <div className="divider" />
                    <h3>Expected Claims Validation</h3>

                    <div className="row">
                        <label className="field" style={{ flex: 1 }} htmlFor="verify-iss">
                            <span>iss (Issuer)</span>
                            <input
                                id="verify-iss"
                                value={iss}
                                onChange={(event) => setIss(event.target.value)}
                                placeholder="Exact match required"
                            />
                        </label>
                        <label className="field" style={{ flex: 1 }} htmlFor="verify-sub">
                            <span>sub (Subject)</span>
                            <input
                                id="verify-sub"
                                value={sub}
                                onChange={(event) => setSub(event.target.value)}
                                placeholder="Exact match required"
                            />
                        </label>
                        <label className="field" style={{ flex: 1 }} htmlFor="verify-aud">
                            <span>aud (Audience)</span>
                            <input
                                id="verify-aud"
                                value={aud}
                                onChange={(event) => setAud(event.target.value)}
                                placeholder="Must contain one"
                            />
                        </label>
                    </div>

                    <div className="row">
                        <label className="field" style={{ flex: 2 }} htmlFor="verify-required">
                            <span>Required Claims (csv)</span>
                            <input
                                id="verify-required"
                                value={requireClaims}
                                onChange={(event) => setRequireClaims(event.target.value)}
                                placeholder="e.g. exp, nbf, customized_claim"
                            />
                        </label>
                        <label className="field" style={{ flex: 1 }} htmlFor="verify-leeway">
                            <span>Leeway (sec)</span>
                            <input
                                id="verify-leeway"
                                value={leeway}
                                onChange={(event) => setLeeway(event.target.value)}
                                placeholder="30"
                            />
                        </label>
                    </div>

                    <div className="row" style={{ paddingTop: '1rem', gap: '1.5rem', flexWrap: 'wrap' }}>
                        <div className="checkbox-wrap">
                            <input type="checkbox" checked={tryAll} onChange={(event) => setTryAll(event.target.checked)} id="tryAll" />
                            <label htmlFor="tryAll">Try All Keys</label>
                        </div>
                        <div className="checkbox-wrap">
                            <input
                                type="checkbox"
                                checked={ignoreExp}
                                onChange={(event) => setIgnoreExp(event.target.checked)}
                                id="ignoreExp"
                            />
                            <label htmlFor="ignoreExp">Ignore Expiration</label>
                        </div>
                        <div className="checkbox-wrap">
                            <input
                                type="checkbox"
                                checked={explain}
                                onChange={(event) => setExplain(event.target.checked)}
                                id="explain"
                            />
                            <label htmlFor="explain">Provide Explanation</label>
                        </div>
                    </div>
                    
                    <div className="row" style={{ marginTop: "2rem" }}>
                        <button 
                            className="button primary" 
                            onClick={handleVerify}
                            disabled={loading}
                            style={{ width: '100%' }}
                        >
                            {loading ? "Verifying..." : "Verify Token"}
                        </button>
                    </div>
                </section>

                {output && (
                    <section 
                        className="card" 
                        style={{ 
                            animation: 'fadeIn 0.5s',
                            display: 'flex',
                            flexDirection: 'column',
                            height: 'fit-content',
                            borderColor: isValidResult ? 'var(--success)' : 'var(--error)'
                        }}
                    >
                        <div className="card-header" style={{ marginBottom: '1rem' }}>
                            <h2 style={{ color: isValidResult ? 'var(--success)' : 'var(--error)' }}>
                                {isValidResult ? "Valid Token" : "Invalid Token"}
                            </h2>
                            {isValidResult && <span className="badge">Signature Verified</span>}
                        </div>
                        <textarea
                            value={output}
                            readOnly
                            rows={20}
                            style={{
                                fontFamily: 'monospace',
                                fontSize: '0.85rem',
                                background: '#0d1117',
                                color: '#e6edf3',
                                border: '1px solid var(--border)',
                                width: '100%',
                                resize: 'vertical'
                            }}
                        />
                    </section>
                )}
            </div>
        </div>
    );
}