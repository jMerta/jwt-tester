let csrfPromise = null;
let cachedCsrf = null;

export async function getCsrfToken() {
  if (cachedCsrf) return cachedCsrf;
  const meta = document.querySelector('meta[name="csrf-token"]');
  if (meta && meta.content) {
    cachedCsrf = meta.content;
    return cachedCsrf;
  }
  if (!csrfPromise) {
    csrfPromise = fetch("/api/csrf")
      .then((res) => res.json())
      .then((body) => {
        if (body && body.csrf) {
          cachedCsrf = body.csrf;
        } else {
          cachedCsrf = "";
        }
        return cachedCsrf;
      })
      .catch(() => "");
  }
  return csrfPromise;
}

export async function api(path, options = {}) {
  const csrf = await getCsrfToken();
  const headers = {
    "Content-Type": "application/json",
    "X-CSRF-Token": csrf,
    ...(options.headers || {}),
  };
  const res = await fetch(path, { ...options, headers });
  const body = await res.json().catch(() => null);
  if (!res.ok) {
    const msg = body && body.error ? body.error : `HTTP ${res.status}`;
    throw new Error(msg);
  }
  return body;
}

export function parseCsv(raw) {
  return raw
    .split(",")
    .map((val) => val.trim())
    .filter(Boolean);
}

export function formatTags(tags) {
  if (!tags || !tags.length) return "";
  return tags.map((tag) => `#${tag}`).join(" ");
}

export function downloadText(filename, text, contentType = "text/plain") {
  const blob = new Blob([text], { type: contentType });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
}

export const ALGORITHMS = [
  "hs256",
  "hs384",
  "hs512",
  "rs256",
  "rs384",
  "rs512",
  "ps256",
  "ps384",
  "ps512",
  "es256",
  "es384",
  "eddsa",
];
