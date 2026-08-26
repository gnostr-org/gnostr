#!/usr/bin/env node

const baseUrl = (process.argv[2] || process.env.GNOSTR_BASE_URL || "http://127.0.0.1:3030").replace(/\/+$/, "");
const decoder = new TextDecoder();

const routeChecks = [
  {
    path: "/",
    status: 200,
    expect: ["id=\"app-main\"", "id=\"global-header\"", "id=\"master-pane\"", "id=\"detail-pane\""],
  },
  {
    path: "/nip",
    status: 200,
    expect: [
      "id=\"app-main\"",
      "id=\"global-header\"",
      "id=\"master-pane\"",
      "id=\"detail-pane\"",
      "/js/ui/state.js?v=5",
      "/js/nip/34.js?v=4",
      "id=\"messages-banner\"",
      "id=\"notifications-banner\"",
      "id=\"dms\"",
      "id=\"timeline\"",
      "id=\"relays\"",
      "id=\"settings\"",
      "class=\"loading-events hide\"",
    ],
  },
  {
    path: "/home",
    status: 200,
    expect: ["id=\"app-main\"", "id=\"global-header\"", "data-view=\"friends\""],
  },
  {
    path: "/messages",
    status: 200,
    expect: ["id=\"app-main\"", "id=\"messages-banner\"", "id=\"notifications-banner\"", "id=\"dms\""],
  },
  {
    path: "/notifications",
    status: 200,
    expect: ["id=\"app-main\"", "id=\"notifications-banner\"", "data-view=\"notifications\""],
  },
  {
    path: "/relays",
    status: 200,
    expect: ["id=\"app-main\"", "id=\"relays\"", "data-view=\"relays\""],
  },
  {
    path: "/settings",
    status: 200,
    expect: ["id=\"app-main\"", "id=\"settings\"", "data-view=\"settings\""],
  },
  {
    path: "/nip/34",
    status: 200,
    expect: ["id=\"app-main\"", "/js/nip/34.js?v=4", "/js/ui/state.js?v=5"],
  },
  {
    path: "/nip/34/30617",
    status: 200,
    expect: ["id=\"app-main\"", "/js/nip/34.js?v=4", "/js/ui/state.js?v=5"],
  },
  {
    path: "/nip/34/99999",
    status: 404,
    expect: ["id=\"app-main\"", "/js/nip/34.js?v=4", "/js/ui/state.js?v=5"],
  },
  {
    path: "/nip/34/query",
    status: 200,
    expect: ["id=\"app-main\"", "/js/nip/34.js?v=4", "/js/ui/state.js?v=5"],
  },
  {
    path: "/nip/34/relays.yaml",
    status: 200,
    expect: ["id=\"app-main\"", "/js/nip/34.js?v=4", "/js/ui/state.js?v=5"],
  },
  {
    path: "/nip/34/relays.json",
    status: 200,
    expect: ["id=\"app-main\"", "/js/nip/34.js?v=4", "/js/ui/state.js?v=5"],
  },
  {
    path: "/nip/34/relays.txt",
    status: 200,
    expect: ["id=\"app-main\"", "/js/nip/34.js?v=4", "/js/ui/state.js?v=5"],
  },
  {
    path: "/repository-details/deadbeef",
    status: 200,
    expect: ["id=\"app-main\"", "id=\"global-header\"", "/js/ui/state.js?v=5"],
  },
];

const extraAssets = ["/js/nip89-app.json"];

const markerChecks = [
  {
    pathIncludes: "/js/ui/state.js",
    mustContain: [
      "messages-banner",
      "notifications-banner",
      "VM_NOTIFICATIONS",
      "VM_DM",
      "switch_view(view_name_to_mode(el.dataset.view))",
    ],
  },
  {
    pathIncludes: "/js/nip/34.js",
    mustContain: ["NIP EXPLORER RENDERED", "render_nip_explorer", "nip-explorer-list"],
  },
  {
    pathIncludes: "/js/ui/settings.js",
    mustContain: ["load_nip89_app_metadata", "render_nip89_app_metadata", "nip89-app-mount"],
  },
  {
    pathIncludes: "/js/relay.js",
    mustContain: ["SID_NOTIFICATIONS", "refresh_dm_subscriptions"],
  },
  {
    pathIncludes: "/js/bootstrap.js",
    mustContain: ["webapp_init", "parse_url_mode"],
  },
  {
    pathIncludes: "/css/styles.css",
    mustContain: ["#app-main + footer", "#master-pane", "#detail-pane", "#container-busy"],
  },
];

function fail(message) {
  console.error(`FAIL ${message}`);
  process.exitCode = 1;
}

function ok(message) {
  console.log(`OK ${message}`);
}

function isExternal(url) {
  return /^(?:[a-z][a-z0-9+.-]*:|\/\/)/i.test(url);
}

function stripFragment(url) {
  return url.split("#", 1)[0];
}

function normalizeShellUrl(url) {
  const cleaned = stripFragment(url.trim());
  if (!cleaned || cleaned.startsWith("javascript:") || cleaned.startsWith("mailto:") || cleaned.startsWith("tel:")) {
    return null;
  }
  if (isExternal(cleaned) && !cleaned.startsWith("/")) {
    return null;
  }
  return cleaned;
}

function assetType(path) {
  const clean = path.split("?", 1)[0];
  if (clean.endsWith(".css")) return "css";
  if (clean.endsWith(".js")) return "js";
  if (clean.endsWith(".json")) return "json";
  if (clean.endsWith(".svg")) return "svg";
  if (clean.endsWith(".png")) return "png";
  if (clean.endsWith(".jpg") || clean.endsWith(".jpeg")) return "jpg";
  if (clean.endsWith(".ico")) return "ico";
  if (clean.endsWith(".html")) return "html";
  return "bin";
}

function expectedContentType(path) {
  switch (assetType(path)) {
    case "css":
      return "text/css";
    case "js":
      return "application/javascript";
    case "json":
      return "application/json";
    case "svg":
      return "image/svg+xml";
    case "png":
      return "image/png";
    case "jpg":
      return "image/jpeg";
    case "ico":
      return "image/x-icon";
    case "html":
      return "text/html";
    default:
      return null;
  }
}

async function fetchResponse(path) {
  return fetch(`${baseUrl}${path}`, {
    headers: {
      Accept: "text/html,application/javascript,application/json,image/svg+xml,*/*;q=0.8",
    },
  });
}

async function readBody(res, path) {
  const bytes = new Uint8Array(await res.arrayBuffer());
  const textTypes = new Set(["css", "js", "json", "svg", "html"]);
  const text = textTypes.has(assetType(path)) ? decoder.decode(bytes) : "";
  return { bytes, text };
}

async function checkRoute({ path, status, expect, finalPath }) {
  const res = await fetchResponse(path);
  const text = await res.text();

  if (status != null && res.status !== status) {
    fail(`${path} returned ${res.status} not ${status}`);
    return;
  }

  if (finalPath && !res.url.endsWith(finalPath)) {
    fail(`${path} resolved to ${res.url}, expected ${finalPath}`);
    return;
  }

  const csp = res.headers.get("content-security-policy");
  if (!csp || !csp.includes("default-src *")) {
    fail(`${path} missing expected CSP header`);
    return;
  }

  const contentType = res.headers.get("content-type") || "";
  if (!contentType.includes("text/html")) {
    fail(`${path} missing text/html content type`);
    return;
  }

  for (const needle of expect || []) {
    if (!text.includes(needle)) {
      fail(`${path} missing ${needle}`);
      return;
    }
  }

  ok(`route ${path}`);
}

async function checkAsset(path) {
  const res = await fetchResponse(path);
  const { bytes, text } = await readBody(res, path);

  if (!res.ok) {
    fail(`${path} returned ${res.status}`);
    return;
  }

  if (bytes.length === 0) {
    fail(`${path} was empty`);
    return;
  }

  const expectedType = expectedContentType(path);
  const contentType = res.headers.get("content-type") || "";
  if (expectedType && !contentType.includes(expectedType)) {
    fail(`${path} content type ${contentType} did not include ${expectedType}`);
    return;
  }

  for (const check of markerChecks) {
    if (path.includes(check.pathIncludes)) {
      for (const marker of check.mustContain) {
        if (!text.includes(marker)) {
          fail(`${path} missing ${marker}`);
          return;
        }
      }
    }
  }

  if (path.endsWith("/js/nip89-app.json")) {
    try {
      const parsed = JSON.parse(text);
      if (parsed == null || (typeof parsed !== "object" && !Array.isArray(parsed))) {
        fail(`${path} did not parse to JSON`);
        return;
      }
    } catch (error) {
      fail(`${path} was not valid JSON: ${error.message}`);
      return;
    }
  }

  ok(`asset ${path}`);
}

function extractShellUrls(html) {
  const urls = new Set();
  const attrRe = /\b(?:src|href)=["']([^"']+)["']/g;
  let match;
  while ((match = attrRe.exec(html)) !== null) {
    const url = normalizeShellUrl(match[1]);
    if (!url) continue;
    urls.add(url);
  }
  return urls;
}

function isRoutePath(path) {
  const routes = new Set([
    "/",
    "/home",
    "/messages",
    "/notifications",
    "/relays",
    "/settings",
    "/nip",
    "/nip/34",
    "/nip/34/30617",
    "/nip/34/99999",
    "/nip/34/query",
    "/nip/34/relays.yaml",
    "/nip/34/relays.json",
    "/nip/34/relays.txt",
    "/repository-details/deadbeef",
  ]);
  if (routes.has(path)) return true;
  if (path.startsWith("/nip/34/")) return true;
  if (path.startsWith("/repository-details/")) return true;
  return false;
}

console.log(`Checking ${baseUrl}`);

for (const route of routeChecks) {
  await checkRoute(route);
}

const shellRes = await fetchResponse("/nip");
const shellText = await shellRes.text();
if (!shellRes.ok) {
  fail(`/nip returned ${shellRes.status}, cannot discover shell assets`);
} else {
  const shellUrls = [...extractShellUrls(shellText)].filter((url) => url.startsWith("/") && !isRoutePath(url.split("?", 1)[0]));
  const discoveredAssets = [...new Set([...shellUrls, ...extraAssets])];
  ok(`discovered ${discoveredAssets.length} shell assets`);

  for (const asset of discoveredAssets) {
    await checkAsset(asset);
  }
}

if (process.exitCode) {
  process.exit(process.exitCode);
}
