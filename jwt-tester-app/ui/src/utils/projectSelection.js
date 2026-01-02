export const DEFAULT_PROJECT_KEY = "jwt-tester-default-project";
export const LAST_PROJECT_KEY = "jwt-tester-last-project";

export function isProjectIdValid(id, projects) {
  if (!id) return false;
  return projects.some((project) => project.id === id);
}

export function normalizeProjectId(id, projects) {
  return isProjectIdValid(id, projects) ? id : null;
}

export function pickProjectId({
  preferredId,
  selectedId,
  defaultId,
  lastId,
  projects,
}) {
  const candidates = [preferredId, selectedId, defaultId, lastId];
  for (const candidate of candidates) {
    if (isProjectIdValid(candidate, projects)) {
      return candidate;
    }
  }
  return projects.length ? projects[0].id : null;
}
