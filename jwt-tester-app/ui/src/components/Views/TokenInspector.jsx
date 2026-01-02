import React, { useState } from "react";
import { api } from "../../api.js";

export function TokenInspector({ setStatus }) {
    const [token, setToken] = useState("");
    const [dateMode, setDateMode] = useState("");
    const [showSegments, setShowSegments] = useState(false);
    const [output, setOutput] = useState("");

    const handleInspect = async () => {
        if (!token.trim()) {
            setStatus("Please enter a token.");
            return;
        }
        const res = await api("/api/jwt/inspect", {
            method: "POST",
            body: JSON.stringify({
                token,
                date: dateMode || null,
                show_segments: showSegments,
            }),
        });
        setOutput(JSON.stringify(res.data, null, 2));
        setStatus("Token inspected.");
    };

    return (
        <div className="view-container">
            <header className="card-header" style={{ borderBottom: "none" }}>
                <div>
                    <h1>Token Inspector</h1>
                    <p>Decode and inspect JWT content without signature verification.</p>
                </div>
            </header>

            <section className="card">
                <label className="field">
                    <span>JWT String</span>
                    <textarea
                        value={token}
                        onChange={(event) => setToken(event.target.value)}
                        rows={5}
                        placeholder="Paste JWT here..."
                        style={{ fontFamily: 'monospace', fontSize: '0.85rem' }}
                    />
                </label>

                <div className="row">
                    <label className="field">
                        <span>Date Display Mode</span>
                        <select value={dateMode} onChange={(event) => setDateMode(event.target.value)}>
                            <option value="">Unix Timestamp (Default)</option>
                            <option value="utc">UTC String</option>
                            <option value="local">Local Time String</option>
                            {/* <option value="+02:00">Fixed Offset (+02:00)</option> */}
                        </select>
                    </label>
                    <div className="checkbox-wrap" style={{ marginTop: '1.5rem' }}>
                        <input
                            type="checkbox"
                            checked={showSegments}
                            onChange={(event) => setShowSegments(event.target.checked)}
                            id="showSeg"
                        />
                        <label htmlFor="showSeg" style={{ cursor: 'pointer', color: 'var(--text-primary)' }}>
                            Show Raw Segments
                        </label>
                    </div>
                </div>

                <div className="row" style={{ marginTop: "1rem" }}>
                    <button className="button primary" onClick={handleInspect}>
                        Inspect Token
                    </button>
                </div>
            </section>

            {output && (
                <section className="card" style={{ animation: 'fadeIn 0.5s' }}>
                    <div className="card-header">
                        <h2>Inspection Result</h2>
                    </div>
                    <textarea
                        value={output}
                        readOnly
                        rows={15}
                        style={{
                            fontFamily: 'monospace',
                            fontSize: '0.9rem',
                            background: '#0d1117',
                            color: '#e6edf3',
                            border: '1px solid var(--border)'
                        }}
                    />
                </section>
            )}
        </div>
    );
}
