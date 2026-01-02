import React, { useCallback, useEffect, useMemo, useState } from "react";
import { api } from "./api.js";
import { Sidebar } from "./components/Layout/Sidebar.jsx";
import { StatusBar } from "./components/Shared/StatusBar.jsx";
import { NewProjectModal } from "./components/Shared/NewProjectModal.jsx";
import { Dashboard } from "./components/Views/Dashboard.jsx";
import { VaultManager } from "./components/Views/VaultManager.jsx";
import { TokenBuilder } from "./components/Views/TokenBuilder.jsx";
import { TokenInspector } from "./components/Views/TokenInspector.jsx";
import { TokenVerifier } from "./components/Views/TokenVerifier.jsx";
import { Settings } from "./components/Views/Settings.jsx";
import { Help } from "./components/Views/Help.jsx";
import {
  DEFAULT_PROJECT_KEY,
  LAST_PROJECT_KEY,
  normalizeProjectId,
  pickProjectId,
} from "./utils/projectSelection.js";
import "./styles.css";

const readStoredValue = (key) => {
  try {
    return window.localStorage.getItem(key);
  } catch (err) {
    return null;
  }
};

const writeStoredValue = (key, value) => {
  try {
    if (!value) {
      window.localStorage.removeItem(key);
      return;
    }
    window.localStorage.setItem(key, value);
  } catch (err) {
    // Ignore storage errors (e.g. disabled storage)
  }
};

export default function App() {
  const [projects, setProjects] = useState([]);
  const [keys, setKeys] = useState([]);
  const [tokens, setTokens] = useState([]);
  const [selectedProjectId, setSelectedProjectId] = useState(null);
  const [defaultProjectId, setDefaultProjectId] = useState(() =>
    readStoredValue(DEFAULT_PROJECT_KEY)
  );
  const [lastProjectId, setLastProjectId] = useState(() =>
    readStoredValue(LAST_PROJECT_KEY)
  );
  const [status, setStatus] = useState("");
  const [loading, setLoading] = useState(true);
  const [showNewProjectModal, setShowNewProjectModal] = useState(false);

  // View State
  const [currentView, setCurrentView] = useState("dashboard");

  const persistDefaultProject = useCallback((projectId) => {
    const nextId = projectId || null;
    writeStoredValue(DEFAULT_PROJECT_KEY, nextId);
    setDefaultProjectId(nextId);
  }, []);

  const persistLastProject = useCallback((projectId) => {
    const nextId = projectId || null;
    writeStoredValue(LAST_PROJECT_KEY, nextId);
    setLastProjectId(nextId);
  }, []);

  const loadVault = useCallback(
    async (preferredId = null) => {
      // Don't set global loading true on refresh if we want smoother UX, 
      // but for safety we follow original patterns mostly, adjusted for less flicker
      // setLoading(true); 

      try {
        const projectRes = await api("/api/vault/projects");
        const nextProjects = projectRes.data || [];
        const nextDefault = normalizeProjectId(defaultProjectId, nextProjects);
        if (nextDefault !== defaultProjectId) {
          persistDefaultProject(nextDefault);
        }
        const nextLast = normalizeProjectId(lastProjectId, nextProjects);
        if (nextLast !== lastProjectId) {
          persistLastProject(nextLast);
        }

        const nextSelected = pickProjectId({
          preferredId,
          selectedId: selectedProjectId,
          defaultId: nextDefault,
          lastId: nextLast,
          projects: nextProjects,
        });

        setProjects(nextProjects);
        setSelectedProjectId(nextSelected || null);
        if (nextSelected) {
          persistLastProject(nextSelected);
        }

        if (nextSelected) {
          const [keyRes, tokenRes] = await Promise.all([
            api(`/api/vault/keys?project_id=${encodeURIComponent(nextSelected)}`),
            api(`/api/vault/tokens?project_id=${encodeURIComponent(nextSelected)}`),
          ]);
          setKeys(keyRes.data || []);
          setTokens(tokenRes.data || []);
        } else {
          setKeys([]);
          setTokens([]);
        }
      } catch (err) {
        setStatus(err.message || "Failed to load vault.");
      } finally {
        setLoading(false);
      }
    },
    [
      selectedProjectId,
      defaultProjectId,
      lastProjectId,
      persistDefaultProject,
      persistLastProject,
    ]
  );

  useEffect(() => {
    loadVault();
  }, []); // Initial load

  const selectedProject = useMemo(
    () => projects.find((p) => p.id === selectedProjectId) || null,
    [projects, selectedProjectId]
  );

  const defaultKeyId = selectedProject?.default_key_id || null;
  const defaultKey = defaultKeyId ? keys.find((k) => k.id === defaultKeyId) : null;
  const defaultKeyLabel = defaultKey
    ? `${defaultKey.name} (${defaultKey.id.slice(0, 8)})`
    : "none";

  const renderView = () => {
    const commonProps = {
      onRefresh: loadVault,
      setStatus,
      projects,
      keys,
      tokens,
      selectedProjectId,
      defaultKeyLabel,
      defaultKeyId,
      projectName: selectedProject?.name,
      defaultProjectId,
      onSetDefaultProject: persistDefaultProject,
    };

    switch (currentView) {
      case "dashboard":
        return (
          <Dashboard
            {...commonProps}
            onSelectProject={(id) => loadVault(id)}
            onCreateProject={() => setShowNewProjectModal(true)}
            onNavigate={setCurrentView}
          />
        );
      case "vault":
        return (
          <VaultManager
            {...commonProps}
            projectId={selectedProjectId}
            onNavigate={setCurrentView}
          />
        );
      case "builder":
        return <TokenBuilder {...commonProps} />;
      case "inspector":
        return <TokenInspector setStatus={setStatus} />;
      case "verifier":
        return <TokenVerifier {...commonProps} />;
      case "settings":
        return <Settings {...commonProps} />;
      case "help":
        return <Help />;
      default:
        return <Dashboard {...commonProps} />;
    }
  };

  if (loading) {
    return (
      <div
        className="app-container"
        style={{
          alignItems: "center",
          justifyContent: "center",
          background: "var(--bg-main)",
          color: "var(--text-primary)",
        }}
      >
        <div style={{ fontSize: "1.5rem", fontWeight: "bold" }}>
          <span style={{ color: 'var(--secondary)' }}>⚡</span> Loading Vault...
        </div>
      </div>
    );
  }

  return (
    <div className="app-container">
      <Sidebar
        currentView={currentView}
        setView={setCurrentView}
        projects={projects}
        selectedProjectId={selectedProjectId}
        onSelectProject={(id) => loadVault(id)}
        onNewProject={() => setShowNewProjectModal(true)}
      />
      <main className="main-content">
        {renderView()}
      </main>
      <StatusBar status={status} setStatus={setStatus} />
      <NewProjectModal
        isOpen={showNewProjectModal}
        onClose={() => setShowNewProjectModal(false)}
        onProjectCreated={async () => {
          await loadVault();
          setStatus("Project created successfully");
        }}
      />
    </div>
  );
}
