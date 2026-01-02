import React, { useState } from "react";
import { api, formatTags } from "../../api.js";
import { Stat } from "../Shared/Stat.jsx";

export function Dashboard({
    projects,
    keys,
    tokens,
    selectedProjectId,
    onSelectProject,
    onRefresh,
    onCreateProject,
    onNavigate,
    setStatus,
    defaultKeyLabel = "none",
    defaultKeyId = null,
    defaultProjectId = null,
    onSetDefaultProject = () => {},
}) {
    const selectedProject =
        projects.find((p) => p.id === selectedProjectId) || null;

    const isDefaultProject = selectedProjectId && selectedProjectId === defaultProjectId;

    const handleSetDefaultProject = () => {
        if (!selectedProjectId) return;
        const newId = isDefaultProject ? null : selectedProjectId;
        onSetDefaultProject(newId);
        setStatus(newId ? "Project set as startup default." : "Startup default cleared.");
    };

    const handleDelete = async () => {
        if (!selectedProjectId) return;
        if (
            !window.confirm(
                `Are you sure you want to delete project "${selectedProject?.name}"?\nThis action cannot be undone.`
            )
        ) {
            return;
        }
        try {
            await api(`/api/vault/projects/${selectedProjectId}`, { method: "DELETE" });
            setStatus("Project deleted.");
            await onRefresh();
        } catch (err) {
            setStatus(err.message || "Failed to delete project.");
        }
    };

    return (
        <div className="view-container">
            <header className="card-header" style={{ borderBottom: "none", paddingBottom: 0 }}>
                <div>
                    <h1>Dashboard</h1>
                    <p>Overview of your JWT testing workspace.</p>
                </div>
                <div style={{ textAlign: "right" }}>
                    <small style={{ display: "block", marginBottom: "0.25rem" }}>System Status</small>
                    <span className="badge">Online</span>
                </div>
            </header>

            <div className="stats-grid">
                <Stat label="Total Projects" value={projects.length} />
                <Stat label="Active Keys" value={selectedProjectId ? keys.length : "-"} />
                <Stat label="Active Tokens" value={selectedProjectId ? tokens.length : "-"} />
            </div>

            <div className="two-column-layout">
                {/* Active Workspace Card */}
                <section className="card" style={{ flex: 1.5 }}>
                    <div className="card-header">
                        <div>
                            <h2>Active Workspace</h2>
                            <p>Manage your current project context.</p>
                        </div>
                        <div style={{ display: 'flex', gap: '0.5rem' }}>
                             {isDefaultProject && (
                                <span className="pill active" style={{ borderColor: 'var(--success)', color: 'var(--success)', background: 'rgba(16, 185, 129, 0.1)' }}>
                                    Startup Default
                                </span>
                            )}
                            <span className={`pill ${selectedProjectId ? "active" : ""}`}>
                                {selectedProjectId ? "Active" : "None Selected"}
                            </span>
                        </div>
                    </div>

                    <div className="field">
                        <label>Select Project</label>
                        <div className="row" style={{ gap: "0.5rem" }}>
                            <select
                                style={{ flex: 1 }}
                                value={selectedProjectId || ""}
                                onChange={(event) => onSelectProject(event.target.value)}
                                disabled={!projects.length}
                            >
                                <option value="" disabled>
                                    -- Select a Project --
                                </option>
                                {projects.map((project) => (
                                    <option key={project.id} value={project.id}>
                                        {project.name}
                                    </option>
                                ))}
                            </select>
                        </div>
                    </div>

                    {selectedProject ? (
                        <div style={{ marginTop: "1.5rem" }}>
                            <div className="list-item" style={{ flexDirection: "column", alignItems: "flex-start", gap: "0.5rem" }}>
                                <div style={{ width: "100%", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                                    <span className="list-title">{selectedProject.name}</span>
                                    <small style={{ fontFamily: "monospace" }}>{selectedProject.id.slice(0, 8)}...</small>
                                </div>
                                {selectedProject.description && (
                                    <p style={{ margin: 0, fontSize: "0.9rem" }}>{selectedProject.description}</p>
                                )}
                                
                                <div style={{ 
                                    width: "100%", 
                                    padding: "0.5rem 0", 
                                    borderTop: "1px solid var(--border)",
                                    borderBottom: "1px solid var(--border)",
                                    display: "flex", 
                                    justifyContent: "space-between",
                                    alignItems: "center",
                                    margin: "0.5rem 0"
                                }}>
                                    <span style={{ fontSize: "0.85rem", color: "var(--text-muted)" }}>Default Signing Key:</span>
                                    {defaultKeyId ? (
                                        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                            <span style={{ color: "var(--primary)", fontWeight: "500", fontSize: "0.9rem" }}>
                                                {defaultKeyLabel}
                                            </span>
                                        </div>
                                    ) : (
                                        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                             <span style={{ color: "var(--warning)", fontSize: "0.85rem", fontStyle: "italic" }}>
                                                Not Set
                                            </span>
                                            <button 
                                                className="button ghost" 
                                                style={{ padding: '0.1rem 0.4rem', fontSize: '0.75rem', height: 'auto' }}
                                                onClick={() => onNavigate("vault")}
                                            >
                                                Configure
                                            </button>
                                        </div>
                                    )}
                                </div>

                                {selectedProject.tags && selectedProject.tags.length > 0 && (
                                    <small style={{ color: "var(--secondary)" }}>{formatTags(selectedProject.tags)}</small>
                                )}
                            </div>

                            <div style={{ marginTop: "1.5rem", display: "flex", justifyContent: "space-between", alignItems: "center", flexWrap: "wrap", gap: "1rem" }}>
                                <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
                                     <small>Created: {new Date(selectedProject.created_at * 1000).toLocaleDateString()}</small>
                                </div>
                                <div style={{ display: 'flex', gap: '0.5rem' }}>
                                    <button
                                        className="button ghost"
                                        onClick={handleSetDefaultProject}
                                        title="Automatically select this project when the app starts"
                                    >
                                        {isDefaultProject ? "Unset Startup" : "Set as Startup"}
                                    </button>
                                    <button
                                        className="button ghost"
                                        onClick={handleDelete}
                                        style={{ color: "var(--error)", borderColor: "rgba(239, 68, 68, 0.3)" }}
                                    >
                                        Delete
                                    </button>
                                </div>
                            </div>
                        </div>
                    ) : (
                        <div style={{ padding: "2rem", textAlign: "center", color: "var(--text-muted)", border: "1px dashed var(--border)", borderRadius: "var(--radius)" }}>
                            <p>No project selected.</p>
                            <button className="button primary" onClick={onCreateProject}>
                                Create Your First Project
                            </button>
                        </div>
                    )}
                </section>

                {/* Quick Actions Card */}
                <section className="card" style={{ flex: 1 }}>
                    <div className="card-header">
                        <h2>Quick Actions</h2>
                    </div>
                    <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
                        <button className="button primary" onClick={onCreateProject}>
                            <span style={{ marginRight: "0.5rem" }}>+</span> New Project
                        </button>
                        
                        <div className="divider" style={{ margin: "0.5rem 0" }} />
                        
                        <button 
                            className="button ghost" 
                            style={{ justifyContent: "flex-start" }}
                            onClick={() => onNavigate("vault")}
                            disabled={!selectedProjectId}
                        >
                            <span style={{ marginRight: "0.75rem" }}>🔑</span> Manage Keys & Tokens
                        </button>
                        
                        <button 
                            className="button ghost" 
                            style={{ justifyContent: "flex-start" }}
                            onClick={() => onNavigate("builder")}
                            disabled={!selectedProjectId}
                        >
                            <span style={{ marginRight: "0.75rem" }}>🔨</span> Open Token Builder
                        </button>
                        
                        <button 
                            className="button ghost" 
                            style={{ justifyContent: "flex-start" }}
                            onClick={() => onNavigate("inspector")}
                        >
                            <span style={{ marginRight: "0.75rem" }}>🔍</span> Inspect a Token
                        </button>

                         <button 
                            className="button ghost" 
                            style={{ justifyContent: "flex-start" }}
                            onClick={() => onNavigate("verifier")}
                            disabled={!selectedProjectId}
                        >
                            <span style={{ marginRight: "0.75rem" }}>🛡️</span> Verify Token
                        </button>
                    </div>
                </section>
            </div>
        </div>
    );
}