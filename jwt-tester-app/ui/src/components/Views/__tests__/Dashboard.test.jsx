import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { Dashboard } from "../Dashboard.jsx";

// Mock api
vi.mock("../../../api.js", () => ({
  api: vi.fn(),
  formatTags: (tags) => tags ? tags.map(t => `#${t}`).join(" ") : "",
}));

describe("Dashboard Component", () => {
  const mockProjects = [
    { id: "p1", name: "Project Alpha", description: "Test Project", tags: ["test"], created_at: 1735862400 },
    { id: "p2", name: "Project Beta", created_at: 1735776000 },
  ];
  const mockKeys = [{ id: "k1" }];
  const mockTokens = [{ id: "t1" }, { id: "t2" }];

  it("renders welcome message and global stats", () => {
    render(
      <Dashboard
        projects={mockProjects}
        keys={[]}
        tokens={[]}
        selectedProjectId={null}
        onSelectProject={() => {}}
        onCreateProject={() => {}}
        onNavigate={() => {}}
        setStatus={() => {}}
      />
    );

    expect(screen.getByText("Dashboard")).toBeInTheDocument();
    expect(screen.getByText("Total Projects")).toBeInTheDocument();
    expect(screen.getByText("2")).toBeInTheDocument(); 
  });

  it("shows 'Create Your First Project' when no project selected", () => {
    const onCreateProject = vi.fn();
    render(
      <Dashboard
        projects={mockProjects}
        keys={[]}
        tokens={[]}
        selectedProjectId={null}
        onSelectProject={() => {}}
        onCreateProject={onCreateProject}
        onNavigate={() => {}}
        setStatus={() => {}}
      />
    );

    const btn = screen.getByText("Create Your First Project");
    expect(btn).toBeInTheDocument();
    fireEvent.click(btn);
    expect(onCreateProject).toHaveBeenCalled();
  });

  it("shows project details, default key status, and quick actions when project selected", () => {
    const onSelectProject = vi.fn();
    render(
      <Dashboard
        projects={mockProjects}
        keys={mockKeys}
        tokens={mockTokens}
        selectedProjectId="p1"
        onSelectProject={onSelectProject}
        onCreateProject={() => {}}
        onNavigate={() => {}}
        setStatus={() => {}}
        defaultKeyLabel="my-key (k1...)"
        defaultKeyId="k1"
      />
    );

    const projectNames = screen.getAllByText("Project Alpha");
    expect(projectNames.length).toBeGreaterThan(0);
    expect(screen.getByText("Test Project")).toBeInTheDocument();
    expect(screen.getByText("#test")).toBeInTheDocument();
    
    // Default Key Check
    expect(screen.getByText("Default Signing Key:")).toBeInTheDocument();
    expect(screen.getByText("my-key (k1...)")).toBeInTheDocument();
    
    // Stats for active project
    expect(screen.getByText("Active Keys")).toBeInTheDocument();
    expect(screen.getByText("1")).toBeInTheDocument();
    expect(screen.getByText("Active Tokens")).toBeInTheDocument();
    const tokenCounts = screen.getAllByText("2");
    expect(tokenCounts.length).toBeGreaterThan(0);
  });

  it("calls onNavigate when Quick Action buttons are clicked", () => {
    const onNavigate = vi.fn();
    render(
      <Dashboard
        projects={mockProjects}
        keys={mockKeys}
        tokens={mockTokens}
        selectedProjectId="p1"
        onSelectProject={() => {}}
        onCreateProject={() => {}}
        onNavigate={onNavigate}
        setStatus={() => {}}
      />
    );

    fireEvent.click(screen.getByText(/Manage Keys & Tokens/i));
    expect(onNavigate).toHaveBeenCalledWith("vault");

    fireEvent.click(screen.getByText(/Open Token Builder/i));
    expect(onNavigate).toHaveBeenCalledWith("builder");
  });

  it("toggles default startup project state", () => {
    const onSetDefaultProject = vi.fn();
    const setStatus = vi.fn();
    
    render(
      <Dashboard
        projects={mockProjects}
        keys={[]}
        tokens={[]}
        selectedProjectId="p1"
        onSelectProject={() => {}}
        defaultProjectId={null}
        onSetDefaultProject={onSetDefaultProject}
        setStatus={setStatus}
      />
    );
    
    fireEvent.click(screen.getByText(/Set as Startup/i));
    expect(onSetDefaultProject).toHaveBeenCalledWith("p1");
    expect(setStatus).toHaveBeenCalledWith("Project set as startup default.");
  });

  it("clears default startup project when already default", () => {
    const onSetDefaultProject = vi.fn();
    const setStatus = vi.fn();

    render(
      <Dashboard
        projects={mockProjects}
        keys={[]}
        tokens={[]}
        selectedProjectId="p1"
        onSelectProject={() => {}}
        defaultProjectId="p1"
        onSetDefaultProject={onSetDefaultProject}
        setStatus={setStatus}
      />
    );
    
    expect(screen.getByText("Startup Default")).toBeInTheDocument();
    fireEvent.click(screen.getByText(/Unset Startup/i));
    expect(onSetDefaultProject).toHaveBeenCalledWith(null);
    expect(setStatus).toHaveBeenCalledWith("Startup default cleared.");
  });
});