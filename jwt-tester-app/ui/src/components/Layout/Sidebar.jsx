import React, { useState } from 'react';

export function Sidebar({ currentView, setView, projects = [], selectedProjectId, onSelectProject, onNewProject }) {
    const [collapsed, setCollapsed] = useState(false);

    const navItems = [
        { id: 'dashboard', label: 'Dashboard', icon: '⚡' },
        { id: 'vault', label: 'Vault Manager', icon: '🔐' },
        { id: 'builder', label: 'Token Builder', icon: '🛠️' },
        { id: 'inspector', label: 'Token Inspector', icon: '🔍' },
        { id: 'verifier', label: 'Token Verifier', icon: '✅' },
        { id: 'settings', label: 'Settings', icon: '⚙️' },
        { id: 'help', label: 'Help', icon: '❓' },
    ];

    return (
        <aside className={`sidebar ${collapsed ? 'collapsed' : ''}`}>
            <button
                className="collapse-toggle"
                onClick={() => setCollapsed(!collapsed)}
                title={collapsed ? "Expand Sidebar" : "Collapse Sidebar"}
            >
                {collapsed ? '»' : '«'}
            </button>

            <div className="logo">
                <span style={{ fontSize: '24px' }}>⚡</span>
                {!collapsed && <span>jwt-tester</span>}
            </div>

            <div className="project-context">
                {!collapsed && <label>Active Project</label>}
                <div className="select-wrapper">
                    <select
                        value={selectedProjectId || ''}
                        onChange={(e) => {
                            if (e.target.value === '__NEW__') {
                                onNewProject();
                            } else {
                                onSelectProject(e.target.value);
                            }
                        }}
                        disabled={!projects.length}
                        title={collapsed ? projects.find(p => p.id === selectedProjectId)?.name || "Select Project" : ""}
                    >
                        {projects.length === 0 ? (
                            <option value="">{collapsed ? 'No Projects' : 'No Projects'}</option>
                        ) : (
                            <>
                                {projects.map(p => (
                                    <option key={p.id} value={p.id}>{p.name}</option>
                                ))}
                                <option disabled>──────────</option>
                                <option value="__NEW__">+ New Project</option>
                            </>
                        )}
                    </select>
                    {!collapsed && <span className="select-arrow">▼</span>}
                </div>
            </div>

            <nav>
                {navItems.map((item) => (
                    <div
                        key={item.id}
                        className={`nav-item ${currentView === item.id ? 'active' : ''}`}
                        onClick={() => setView(item.id)}
                        title={collapsed ? item.label : ""}
                    >
                        <span>{item.icon}</span>
                        {!collapsed && item.label}
                    </div>
                ))}
            </nav>
            <div style={{ marginTop: 'auto', padding: '1rem', fontSize: '0.8rem', color: 'var(--text-muted)', textAlign: collapsed ? 'center' : 'left' }}>
                {!collapsed ? 'Local UI v0.1.0' : 'v0.1'}
            </div>
        </aside>
    );
}
