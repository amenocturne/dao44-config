import { watch } from "node:fs";
import { extname, join } from "node:path";

const root = join(import.meta.dir, "..");
const clients = new Set<ReadableStreamDefaultController<Uint8Array>>();
const encoder = new TextEncoder();
let generationRunning = false;
let generationQueued = false;
let debounce: ReturnType<typeof setTimeout> | undefined;

function broadcast(event: string, data = "changed") {
  const payload = encoder.encode(`event: ${event}\ndata: ${data.replaceAll("\n", " ")}\n\n`);
  for (const client of clients) {
    try {
      client.enqueue(payload);
    } catch {
      clients.delete(client);
    }
  }
}

setInterval(() => broadcast("keepalive"), 5_000);

async function generate() {
  if (generationRunning) {
    generationQueued = true;
    return;
  }
  generationRunning = true;
  const process = Bun.spawn(["cargo", "run", "--quiet", "--", "generate"], {
    cwd: root,
    stdout: "pipe",
    stderr: "pipe",
  });
  const [exitCode, stdout, stderr] = await Promise.all([
    process.exited,
    new Response(process.stdout).text(),
    new Response(process.stderr).text(),
  ]);
  if (exitCode === 0) {
    console.log(stdout.trim());
    broadcast("reload");
  } else {
    const message = stderr.trim() || stdout.trim() || `generator exited ${exitCode}`;
    console.error(message);
    broadcast("generation-error", message);
  }
  generationRunning = false;
  if (generationQueued) {
    generationQueued = false;
    await generate();
  }
}

function schedule(kind: "generate" | "reload") {
  clearTimeout(debounce);
  debounce = setTimeout(() => {
    if (kind === "generate") void generate();
    else broadcast("reload");
  }, 80);
}

for (const directory of [join(root, "src"), join(root, "preview")]) {
  watch(directory, { recursive: true }, (_event, filename) => {
    if (!filename || filename === "server.ts") return;
    schedule(directory.endsWith("src") ? "generate" : "reload");
  });
}
watch(join(root, "Cargo.toml"), () => schedule("generate"));

const files = new Map([
  ["/", "preview/index.html"],
  ["/app.js", "preview/app.js"],
  ["/styles.css", "preview/styles.css"],
  ["/layout.json", "generated/layout.json"],
]);
const contentTypes: Record<string, string> = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
};

await generate();

const server = Bun.serve({
  hostname: "127.0.0.1",
  idleTimeout: 30,
  port: 0,
  async fetch(request) {
    const { pathname } = new URL(request.url);
    if (pathname === "/events") {
      let controller: ReadableStreamDefaultController<Uint8Array>;
      const stream = new ReadableStream<Uint8Array>({
        start(value) {
          controller = value;
          clients.add(value);
          value.enqueue(encoder.encode("retry: 250\n\n"));
        },
        cancel() {
          clients.delete(controller);
        },
      });
      return new Response(stream, {
        headers: {
          "cache-control": "no-cache",
          connection: "keep-alive",
          "content-type": "text/event-stream",
        },
      });
    }

    const relative = files.get(pathname);
    if (!relative) return new Response("Not found", { status: 404 });
    const file = Bun.file(join(root, relative));
    if (!(await file.exists())) return new Response("Generated layout is missing", { status: 503 });
    return new Response(file, {
      headers: {
        "cache-control": "no-store",
        "content-type": contentTypes[extname(relative)] ?? "application/octet-stream",
      },
    });
  },
});

console.log(`Dao44 preview: ${server.url}`);
