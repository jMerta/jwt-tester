import React, { useState } from 'react';
import { api, parseCsv } from '../../api';

export function NewProjectModal({ isOpen, onClose, onProjectCreated }) {
    const [name, setName] = useState('');
    const [description, setDescription] = useState('');
    const [tags, setTags] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState(null);

    if (!isOpen) return null;

    const handleSubmit = async (e) => {
        e.preventDefault();
        setError(null);

        if (!name.trim()) {
            setError("Project name is required.");
            return;
        }

        setIsLoading(true);

        try {
            const res = await api("/api/vault/projects", {
                method: "POST",
                body: JSON.stringify({
                    name: name.trim(),
                    description: description.trim() || null,
                    tags: parseCsv(tags),
                }),
            });

            // If successful, reset and close
            setName("");
            setDescription("");
            setTags("");
            if (onProjectCreated) {
                // Determine the new project ID if possible, or just refresh
                // The API might return the new project data? 
                // Based on dashboard logic, it just refreshes. 
                // We'll pass logic to parent to refresh.
                await onProjectCreated();
            }
            onClose();
        } catch (err) {
            setError(err.message || "Failed to create project");
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <div className="modal-overlay" onClick={onClose}>
            <div className="modal-content" onClick={e => e.stopPropagation()}>
                <div className="modal-header">
                    <h2>Create New Project</h2>
                    <button className="modal-close" onClick={onClose}>×</button>
                </div>

                {error && (
                    <div style={{ color: 'var(--error)', marginBottom: '1rem' }}>
                        {error}
                    </div>
                )}

                <form onSubmit={handleSubmit}>
                    <label className="field">
                        <span>Project Name</span>
                        <input
                            value={name}
                            onChange={(e) => setName(e.target.value)}
                            placeholder="e.g. Production Vault"
                            autoFocus
                            disabled={isLoading}
                        />
                    </label>

                    <label className="field">
                        <span>Description</span>
                        <input
                            value={description}
                            onChange={(e) => setDescription(e.target.value)}
                            placeholder="e.g. Main keys for prod"
                            disabled={isLoading}
                        />
                    </label>

                    <label className="field">
                        <span>Tags</span>
                        <input
                            value={tags}
                            onChange={(e) => setTags(e.target.value)}
                            placeholder="e.g. prod, secure"
                            disabled={isLoading}
                        />
                    </label>

                    <div className="modal-footer">
                        <button
                            type="button"
                            className="button ghost"
                            onClick={onClose}
                            disabled={isLoading}
                        >
                            Cancel
                        </button>
                        <button
                            type="submit"
                            className="button primary"
                            disabled={isLoading}
                        >
                            {isLoading ? 'Creating...' : 'Create Project'}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
}
