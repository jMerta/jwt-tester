import React, { useState } from "react";
import { api, downloadText } from "../../api.js";

export function Settings({ onRefresh, setStatus }) {
    const [exportPass, setExportPass] = useState("");
    const [importPass, setImportPass] = useState("");
    const [importReplace, setImportReplace] = useState(false);
    const [importFile, setImportFile] = useState(null);

    const handleExport = async () => {
        if (!exportPass.trim()) {
            setStatus("Export passphrase is required.");
            return;
        }
        const res = await api("/api/vault/export", {
            method: "POST",
            body: JSON.stringify({ passphrase: exportPass.trim() }),
        });
        downloadText("jwt-tester-vault.json", res.data.bundle || "", "application/json");
        setStatus("Vault exported.");
    };

    const handleImport = async () => {
        if (!importFile) {
            setStatus("Select a bundle file to import.");
            return;
        }
        if (!importPass.trim()) {
            setStatus("Import passphrase is required.");
            return;
        }
        const bundle = await importFile.text();
        await api("/api/vault/import", {
            method: "POST",
            body: JSON.stringify({
                bundle,
                passphrase: importPass.trim(),
                replace: importReplace,
            }),
        });
        setStatus("Vault imported.");
        await onRefresh();
    };

    return (
        <div className="view-container">
            <header className="card-header" style={{ borderBottom: "none" }}>
                <div>
                    <h1>Settings</h1>
                    <p>Vault backup and restoration.</p>
                </div>
            </header>

            <section className="card">
                <div className="card-header">
                    <h2>Export Vault</h2>
                    <p>Create an encrypted backup of your projects, keys, and tokens.</p>
                </div>
                <div className="row">
                    <label className="field" style={{ flex: 1 }}>
                        <span>Encryption Passphrase</span>
                        <input
                            type="password"
                            value={exportPass}
                            onChange={(event) => setExportPass(event.target.value)}
                            placeholder="Strong passphrase"
                        />
                        <small>This passphrase will be required to import the bundle.</small>
                    </label>
                </div>
                <div className="row">
                    <button className="button primary" onClick={handleExport}>
                        Download Encrypted Bundle
                    </button>
                </div>
            </section>

            <section className="card">
                <div className="card-header">
                    <h2>Import Vault</h2>
                    <p>Restore from a backup file.</p>
                </div>

                <div className="row">
                    <label className="field">
                        <span>Select Bundle File (.json)</span>
                        <input type="file" accept=".json" onChange={(event) => setImportFile(event.target.files[0])} />
                    </label>
                    <label className="field">
                        <span>Decryption Passphrase</span>
                        <input
                            type="password"
                            value={importPass}
                            onChange={(event) => setImportPass(event.target.value)}
                            placeholder="Passphrase used during export"
                        />
                    </label>
                </div>

                <div className="row" style={{ marginTop: '1rem', alignItems: 'center' }}>
                    <div className="checkbox-wrap">
                        <input
                            type="checkbox"
                            checked={importReplace}
                            onChange={(event) => setImportReplace(event.target.checked)}
                            id="replaceVault"
                        />
                        <label htmlFor="replaceVault" style={{ color: 'var(--text-primary)' }}>
                            Replace entire existing vault (Generic warning: this cannot be undone)
                        </label>
                    </div>
                </div>

                <div className="row" style={{ marginTop: "1rem" }}>
                    <button className="button primary" onClick={handleImport}>
                        Import Bundle
                    </button>
                </div>
            </section>
        </div>
    );
}
