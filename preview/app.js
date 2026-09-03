const state = {
  payload: null,
  layerId: "base",
  hostId: "none",
  selectedId: "P00",
};

const elements = {
  title: document.querySelector("#title"),
  revision: document.querySelector("#revision"),
  layerDescription: document.querySelector("#layer-description"),
  layerTabs: document.querySelector("#layer-tabs"),
  hostControls: document.querySelector("#host-controls"),
  hostButtons: document.querySelector("#host-buttons"),
  keys: document.querySelector("#keys"),
  sequences: document.querySelector("#sequences"),
  questions: document.querySelector("#questions"),
};

const svgNamespace = "http://www.w3.org/2000/svg";

function activeLayer() {
  return state.payload.layers.find((layer) => layer.id === state.layerId);
}

function activeHost() {
  return state.payload.hostLegends.find((host) => host.id === state.hostId);
}

function labelFor(keyId, action) {
  return state.layerId === "base" ? (activeHost()?.keys[keyId] ?? action.primary) : action.primary;
}

function showDetail(keyId) {
  const layer = activeLayer();
  const action = layer.keys[keyId];
  const geometry = state.payload.geometry.find((key) => key.id === keyId);
  const hostLabel = state.layerId === "base" ? activeHost()?.keys[keyId] : null;
  state.selectedId = keyId;
  document.querySelector("#detail-primary").textContent = hostLabel ?? action.primary;
  document.querySelector("#detail-position").textContent = `${keyId} · ${geometry.hand} ${geometry.region}`;
  document.querySelector("#detail-tap").textContent = hostLabel
    ? `${hostLabel} · host interpretation of ANSI ${action.primary}`
    : action.tap;
  document.querySelector("#detail-hold").textContent = action.hold ?? "—";
  document.querySelector("#detail-secondary").textContent = action.secondary ?? "—";
  document.querySelector("#detail-note").textContent = action.note ?? "No additional note";
  elements.keys.querySelectorAll(".key-node").forEach((node) => {
    node.classList.toggle("selected", node.dataset.keyId === keyId);
  });
}

function renderKeys() {
  elements.keys.replaceChildren();
  const layer = activeLayer();
  for (const geometry of state.payload.geometry) {
    const action = layer.keys[geometry.id];
    const label = labelFor(geometry.id, action);
    const group = document.createElementNS(svgNamespace, "g");
    group.classList.add("key-node", `category-${action.category}`);
    group.dataset.keyId = geometry.id;
    if (geometry.rot) {
      group.setAttribute("transform", `rotate(${geometry.rot} ${geometry.rx} ${geometry.ry})`);
    }
    const foreignObject = document.createElementNS(svgNamespace, "foreignObject");
    foreignObject.setAttribute("x", geometry.x + 2);
    foreignObject.setAttribute("y", geometry.y + 2);
    foreignObject.setAttribute("width", geometry.w - 4);
    foreignObject.setAttribute("height", geometry.h - 4);
    const button = document.createElement("button");
    button.type = "button";
    button.className = "key";
    button.setAttribute(
      "aria-label",
      `${geometry.id}: ${label}. Tap: ${action.tap}. Hold: ${action.hold ?? "none"}.`,
    );
    const primary = document.createElement("span");
    primary.className = "key-primary";
    primary.textContent = label;
    const hold = document.createElement("span");
    hold.className = "key-hold";
    hold.textContent = action.holdFace ?? "";
    button.append(primary, hold);
    for (const event of ["mouseenter", "focus", "click"]) {
      button.addEventListener(event, () => showDetail(geometry.id));
    }
    foreignObject.append(button);
    group.append(foreignObject);
    elements.keys.append(group);
  }
  showDetail(state.selectedId);
}

function renderLayers() {
  elements.layerTabs.replaceChildren();
  for (const layer of state.payload.layers) {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = layer.name;
    button.setAttribute("aria-pressed", String(layer.id === state.layerId));
    button.addEventListener("click", () => {
      state.layerId = layer.id;
      render();
    });
    elements.layerTabs.append(button);
  }
}

function renderHosts() {
  elements.hostControls.hidden = state.layerId !== "base";
  elements.hostButtons.replaceChildren();
  for (const host of [{ id: "none", name: "Raw ANSI", keys: {} }, ...state.payload.hostLegends]) {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = host.name;
    button.setAttribute("aria-pressed", String(host.id === state.hostId));
    button.addEventListener("click", () => {
      state.hostId = host.id;
      renderHosts();
      renderKeys();
    });
    elements.hostButtons.append(button);
  }
}

function renderSupportingInfo() {
  elements.sequences.replaceChildren();
  for (const sequence of state.payload.sequences) {
    const card = document.createElement("div");
    card.className = "card";
    const title = document.createElement("strong");
    title.textContent = sequence.name;
    const description = document.createElement("p");
    description.textContent = sequence.description;
    const key = document.createElement("code");
    key.textContent = sequence.steps.map((step) => `${step.label} · ${step.key}`).join(" → ");
    card.append(title, description, key);
    elements.sequences.append(card);
  }
  elements.questions.replaceChildren();
  for (const question of state.payload.openQuestions) {
    const item = document.createElement("li");
    item.textContent = question;
    elements.questions.append(item);
  }
}

function render() {
  const layer = activeLayer();
  elements.title.textContent = state.payload.title;
  elements.revision.textContent = state.payload.revision;
  elements.layerDescription.textContent = layer.description;
  renderLayers();
  renderHosts();
  renderKeys();
  renderSupportingInfo();
}

async function load() {
  const response = await fetch("/layout.json", { cache: "no-store" });
  if (!response.ok) throw new Error(`layout request failed: ${response.status}`);
  state.payload = await response.json();
  render();
}

const reloadEvents = new EventSource("/events");
reloadEvents.addEventListener("reload", () => location.reload());
reloadEvents.addEventListener("generation-error", (event) => {
  console.error("Rust generation failed", event.data);
});

load().catch((error) => {
  document.body.dataset.error = "true";
  elements.title.textContent = "Preview failed to load";
  elements.layerDescription.textContent = error instanceof Error ? error.message : String(error);
});
