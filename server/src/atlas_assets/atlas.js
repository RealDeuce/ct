"use strict";

(() => {
  const routeAlgorithms = globalThis.CepheusAtlasRoutes;
  const canvas = document.getElementById("starmap");
  const context = canvas.getContext("2d", { alpha: false });
  const status = document.getElementById("map-status");
  const scope = document.getElementById("scope");
  const searchForm = document.getElementById("search-form");
  const searchInput = document.getElementById("search");
  const nameList = document.getElementById("system-names");
  const jumpRange = document.getElementById("jump-range");
  const jumpRangeValue = document.getElementById("jump-range-value");
  const pointSize = document.getElementById("point-size");
  const pointSizeValue = document.getElementById("point-size-value");
  const routeForm = document.getElementById("route-form");
  const routeOriginInput = document.getElementById("route-origin");
  const routeDestinationInput = document.getElementById("route-destination");
  const routeStatus = document.getElementById("route-status");
  const routePath = document.getElementById("route-path");
  const pickRouteButton = document.getElementById("pick-route");
  const frontierButton = document.getElementById("mark-frontiers");

  const detail = {
    title: document.getElementById("details-title"),
    world: document.getElementById("detail-world"),
    position: document.getElementById("detail-position"),
    starport: document.getElementById("detail-starport"),
    population: document.getElementById("detail-population"),
    tech: document.getElementById("detail-tech"),
    polity: document.getElementById("detail-polity"),
    neighbors: document.getElementById("detail-neighbors"),
    visited: document.getElementById("detail-visited"),
    known: document.getElementById("detail-known")
  };

  let systems = [];
  let systemIndexById = new Map();
  let selected = null;
  let routeOrigin = null;
  let routeDestination = null;
  let plottedRoute = [];
  let routeSpatialIndex = null;
  let routePickStage = null;
  let routeRequest = 0;
  let routeTimer = null;
  let markFrontiers = false;
  let frontierCount = 0;
  let projected = [];
  let width = 1;
  let height = 1;
  let pixelRatio = 1;
  let drawPending = false;
  let drag = null;
  const camera = { yaw: -0.68, pitch: 0.42, distance: 25, panX: 0, panY: 0 };
  const initialCamera = { ...camera };

  function safeNumber(value, fallback = 0) {
    return Number.isFinite(value) ? value : fallback;
  }

  function validateSnapshot(data) {
    if (!data || data.schemaVersion !== 1 || !Array.isArray(data.systems)) {
      throw new Error("unsupported or malformed universe snapshot");
    }
    return data.systems.filter((system) =>
      system && Number.isSafeInteger(system.id) && typeof system.name === "string" &&
      Array.isArray(system.position) && system.position.length === 3 &&
      system.position.every(Number.isFinite)
    );
  }

  function formatGameTime(seconds) {
    const days = Math.floor(safeNumber(seconds) / 86400);
    const hours = Math.floor((safeNumber(seconds) % 86400) / 3600);
    return `day ${days}, ${String(hours).padStart(2, "0")}:00`;
  }

  function load() {
    fetch("universe.json", { cache: "no-store" })
      .then((response) => {
        if (!response.ok) throw new Error(`universe.json returned HTTP ${response.status}`);
        return response.json();
      })
      .then((snapshot) => {
        systems = validateSnapshot(snapshot);
        systemIndexById = new Map(systems.map((system, index) => [system.id, index]));
        frontierCount = systems.filter((system) => system.visited === false).length;
        updateFrontierButton();
        const visibility = snapshot.visibility === "omniscient" ? "OMNISCIENT" : "UNIVERSALLY KNOWN";
        scope.textContent = `${visibility} · ${systems.length.toLocaleString()} SYSTEMS · ${formatGameTime(snapshot.gameSecond)}`;
        populateSearch();
        frameAll();
        if (systems.length) selectSystem(systems[0]);
        status.classList.add("ready");
        requestDraw();
      })
      .catch((error) => {
        status.textContent = `Atlas unavailable: ${error.message}`;
        status.classList.add("error");
        scope.textContent = "SNAPSHOT LOAD FAILED";
      });
  }

  function populateSearch() {
    const fragment = document.createDocumentFragment();
    systems.slice(0, 10000).forEach((system) => {
      const option = document.createElement("option");
      option.value = system.name;
      if (system.world && system.world !== system.name) option.label = system.world;
      fragment.appendChild(option);
    });
    nameList.replaceChildren(fragment);
  }

  function frameAll() {
    if (!systems.length) return;
    let radius = 1;
    for (const system of systems) {
      radius = Math.max(radius, Math.hypot(...system.position));
    }
    camera.distance = Math.max(7, radius * 2.35);
    initialCamera.distance = camera.distance;
  }

  function portColor(port) {
    if (port === "A" || port === "B") return "#78f0e4";
    if (port === "C" || port === "D") return "#ffca70";
    if (port === "E" || port === "X") return "#ff806c";
    return "#8ca7a1";
  }

  function updateFrontierButton() {
    frontierButton.disabled = frontierCount === 0;
    frontierButton.setAttribute("aria-pressed", String(markFrontiers));
    frontierButton.textContent = frontierCount === 0
      ? "No frontier systems"
      : markFrontiers
        ? `Hide frontier markers (${frontierCount.toLocaleString()})`
        : `Mark frontier systems (${frontierCount.toLocaleString()})`;
  }

  function rotate(position) {
    const cy = Math.cos(camera.yaw);
    const sy = Math.sin(camera.yaw);
    const cp = Math.cos(camera.pitch);
    const sp = Math.sin(camera.pitch);
    const x1 = position[0] * cy - position[1] * sy;
    const y1 = position[0] * sy + position[1] * cy;
    return [x1, y1 * cp - position[2] * sp, y1 * sp + position[2] * cp];
  }

  function project(position) {
    const [x, y, z] = rotate(position);
    const depth = Math.max(0.25, camera.distance - z);
    const scale = Math.min(width, height) * 0.82 / depth;
    return {
      x: width / 2 + camera.panX + x * scale,
      y: height / 2 + camera.panY - y * scale,
      z,
      scale,
      visible: z < camera.distance - 0.3
    };
  }

  function drawAxis(origin, vector, color, label) {
    const a = project(origin);
    const b = project(vector);
    context.strokeStyle = color;
    context.fillStyle = color;
    context.globalAlpha = 0.72;
    context.beginPath();
    context.moveTo(a.x, a.y);
    context.lineTo(b.x, b.y);
    context.stroke();
    context.fillText(label, b.x + 5, b.y - 3);
  }

  function drawGrid() {
    const extent = Math.max(2, camera.distance * 0.36);
    const step = extent > 40 ? 10 : extent > 15 ? 5 : 2;
    context.lineWidth = 1;
    context.strokeStyle = "#1b3538";
    context.globalAlpha = 0.45;
    for (let i = -extent; i <= extent; i += step) {
      const a = project([-extent, i, 0]);
      const b = project([extent, i, 0]);
      const c = project([i, -extent, 0]);
      const d = project([i, extent, 0]);
      context.beginPath();
      context.moveTo(a.x, a.y); context.lineTo(b.x, b.y);
      context.moveTo(c.x, c.y); context.lineTo(d.x, d.y);
      context.stroke();
    }
    context.globalAlpha = 1;
    drawAxis([0, 0, 0], [extent, 0, 0], "#ff8b78", "+X");
    drawAxis([0, 0, 0], [0, extent, 0], "#75d1ff", "+Y");
    drawAxis([0, 0, 0], [0, 0, extent], "#8ee295", "+Z");
  }

  function drawLinks() {
    if (!selected) return;
    const source = project(selected.position);
    const range = Number(jumpRange.value);
    context.strokeStyle = "#62e4dc";
    context.lineWidth = 1;
    context.globalAlpha = 0.32;
    for (const system of systems) {
      if (system === selected || distance(selected, system) > range) continue;
      const target = project(system.position);
      if (!source.visible || !target.visible) continue;
      context.beginPath();
      context.moveTo(source.x, source.y);
      context.lineTo(target.x, target.y);
      context.stroke();
    }
    context.globalAlpha = 1;
  }

  function drawRoute() {
    if (!plottedRoute.length) return;
    const points = plottedRoute.map((system) => project(system.position));
    for (const [color, lineWidth] of [["#071012", 6], ["#ffca70", 2.5]]) {
      context.strokeStyle = color;
      context.lineWidth = lineWidth;
      context.globalAlpha = 0.95;
      context.beginPath();
      let drawing = false;
      for (const point of points) {
        if (!point.visible) {
          drawing = false;
          continue;
        }
        if (drawing) context.lineTo(point.x, point.y);
        else context.moveTo(point.x, point.y);
        drawing = true;
      }
      context.stroke();
    }
    context.globalAlpha = 1;
  }

  function drawRouteEndpoints() {
    for (const [system, label, color] of [
      [routeOrigin, "O", "#8ee295"],
      [routeDestination, "D", "#ffca70"]
    ]) {
      if (!system) continue;
      const point = project(system.position);
      if (!point.visible) continue;
      context.fillStyle = "#071012";
      context.strokeStyle = color;
      context.lineWidth = 2;
      context.beginPath();
      context.arc(point.x, point.y, 10, 0, Math.PI * 2);
      context.fill();
      context.stroke();
      context.fillStyle = color;
      context.textAlign = "center";
      context.textBaseline = "middle";
      context.fillText(label, point.x, point.y + 0.5);
    }
    context.textAlign = "start";
    context.textBaseline = "alphabetic";
  }

  function draw() {
    drawPending = false;
    context.setTransform(pixelRatio, 0, 0, pixelRatio, 0, 0);
    context.fillStyle = "#050b0d";
    context.fillRect(0, 0, width, height);
    context.font = "11px ui-monospace, monospace";
    drawGrid();
    drawLinks();
    drawRoute();

    projected = new Array(systems.length);
    const baseSize = Number(pointSize.value);
    for (let index = 0; index < systems.length; index += 1) {
      const system = systems[index];
      const point = project(system.position);
      projected[index] = point;
      if (!point.visible || point.x < -10 || point.x > width + 10 || point.y < -10 || point.y > height + 10) continue;
      const radius = Math.max(1, Math.min(5, baseSize * (0.6 + point.scale * 0.025)));
      context.fillStyle = portColor(system.starport);
      context.globalAlpha = Math.max(0.35, Math.min(1, 0.52 + point.z / camera.distance));
      context.beginPath();
      context.arc(point.x, point.y, radius, 0, Math.PI * 2);
      context.fill();
      if (markFrontiers && system.visited === false) {
        const markerRadius = Math.max(5, radius + 3.5);
        context.strokeStyle = "#d49cff";
        context.lineWidth = 1.5;
        context.globalAlpha = 0.95;
        context.beginPath();
        context.moveTo(point.x, point.y - markerRadius);
        context.lineTo(point.x + markerRadius, point.y);
        context.lineTo(point.x, point.y + markerRadius);
        context.lineTo(point.x - markerRadius, point.y);
        context.closePath();
        context.stroke();
      }
    }
    context.globalAlpha = 1;

    if (selected) {
      const point = project(selected.position);
      if (point.visible) {
        context.strokeStyle = "#ffffff";
        context.lineWidth = 1.5;
        context.beginPath();
        context.arc(point.x, point.y, 8, 0, Math.PI * 2);
        context.stroke();
        context.fillStyle = "#e8f5f0";
        context.fillText(selected.name, point.x + 12, point.y - 8);
      }
    }
    drawRouteEndpoints();
  }

  function requestDraw() {
    if (!drawPending) {
      drawPending = true;
      requestAnimationFrame(draw);
    }
  }

  function resize() {
    const rect = canvas.getBoundingClientRect();
    width = Math.max(1, rect.width);
    height = Math.max(1, rect.height);
    pixelRatio = Math.min(2, window.devicePixelRatio || 1);
    canvas.width = Math.round(width * pixelRatio);
    canvas.height = Math.round(height * pixelRatio);
    requestDraw();
  }

  function distance(a, b) {
    return routeAlgorithms.distance(a, b);
  }

  function neighborsOf(system) {
    const range = Number(jumpRange.value);
    let count = 0;
    for (const candidate of systems) {
      if (candidate !== system && distance(system, candidate) <= range) count += 1;
    }
    return count;
  }

  function showDetails(system) {
    detail.title.textContent = system ? system.name : "None";
    detail.world.textContent = system?.world ?? "Uncatalogued";
    detail.position.textContent = system ? system.position.map((value) => value.toFixed(3)).join(", ") + " pc" : "—";
    detail.starport.textContent = system?.starport ?? "Uncatalogued";
    detail.population.textContent = Number.isInteger(system?.population) ? `10^${system.population}` : "Uncatalogued";
    detail.tech.textContent = Number.isInteger(system?.techLevel) ? String(system.techLevel) : "Uncatalogued";
    detail.polity.textContent = system ? `#${system.polityId}` : "—";
    detail.neighbors.textContent = system ? `${neighborsOf(system)} within ${Number(jumpRange.value).toFixed(1)} pc` : "—";
    detail.visited.textContent = !system
      ? "—"
      : system.visited === false
        ? "No — frontier"
        : system.visited === true
          ? "Yes"
          : "Unknown";
    detail.known.textContent = system?.universallyKnownSecond == null ? "No" : formatGameTime(system.universallyKnownSecond);
  }

  function selectSystem(system) {
    selected = system;
    searchInput.value = system.name;
    showDetails(system);
    requestDraw();
  }

  function findSystem(query) {
    const normalized = query.trim().toLocaleLowerCase();
    if (!normalized) return null;
    const exact = systems.find((system) =>
      system.name.toLocaleLowerCase() === normalized ||
      system.world?.toLocaleLowerCase() === normalized
    );
    return exact || systems.find((system) =>
      system.name.toLocaleLowerCase().includes(normalized) ||
      system.world?.toLocaleLowerCase().includes(normalized)
    ) || null;
  }

  function locateByName(query) {
    const match = findSystem(query);
    if (match) selectSystem(match);
    else {
      searchInput.setCustomValidity("No matching system in this snapshot");
      searchInput.reportValidity();
    }
  }

  function setRouteStatus(message, kind = "") {
    routeStatus.textContent = message;
    routeStatus.classList.toggle("success", kind === "success");
    routeStatus.classList.toggle("error", kind === "error");
  }

  function clearRoutePath() {
    plottedRoute = [];
    routePath.replaceChildren();
    requestDraw();
  }

  function showRoutePath() {
    const fragment = document.createDocumentFragment();
    plottedRoute.forEach((system, index) => {
      const item = document.createElement("li");
      item.textContent = index === 0
        ? system.name
        : `${system.name} · ${distance(plottedRoute[index - 1], system).toFixed(2)} pc`;
      fragment.appendChild(item);
    });
    routePath.replaceChildren(fragment);
  }

  function calculateRoute() {
    routeRequest += 1;
    const request = routeRequest;
    clearTimeout(routeTimer);
    if (!routeOrigin || !routeDestination) {
      clearRoutePath();
      setRouteStatus("Choose both an origin and a destination.");
      return;
    }
    const range = Number(jumpRange.value);
    setRouteStatus(`Calculating a ${range.toFixed(1)} pc route…`);
    requestAnimationFrame(() => {
      if (request !== routeRequest) return;
      if (!routeSpatialIndex || routeSpatialIndex.jumpRange !== range) {
        routeSpatialIndex = routeAlgorithms.buildSpatialIndex(systems, range);
      }
      const indices = routeAlgorithms.shortestRoute(
        systems,
        systemIndexById.get(routeOrigin.id),
        systemIndexById.get(routeDestination.id),
        routeSpatialIndex
      );
      if (request !== routeRequest) return;
      if (!indices) {
        clearRoutePath();
        setRouteStatus(`No route connects these systems at jump ${range.toFixed(1)}.`, "error");
        return;
      }
      plottedRoute = indices.map((index) => systems[index]);
      showRoutePath();
      const jumps = Math.max(0, plottedRoute.length - 1);
      const total = routeAlgorithms.routeDistance(systems, indices);
      setRouteStatus(
        `${jumps} jump${jumps === 1 ? "" : "s"} · ${total.toFixed(2)} pc total`,
        "success"
      );
      requestDraw();
    });
  }

  function scheduleRouteCalculation() {
    routeRequest += 1;
    clearTimeout(routeTimer);
    if (!routeOrigin || !routeDestination) return;
    clearRoutePath();
    setRouteStatus("Updating route for the new jump range…");
    routeTimer = setTimeout(calculateRoute, 120);
  }

  function setRouteEndpoint(endpoint, system) {
    if (endpoint === "origin") {
      routeOrigin = system;
      routeOriginInput.value = system.name;
    } else {
      routeDestination = system;
      routeDestinationInput.value = system.name;
    }
  }

  function beginRoutePicking() {
    if (routePickStage) {
      routePickStage = null;
      pickRouteButton.textContent = "Pick on map";
      pickRouteButton.setAttribute("aria-pressed", "false");
      setRouteStatus("Map endpoint selection cancelled.");
      return;
    }
    routeRequest += 1;
    clearTimeout(routeTimer);
    clearRoutePath();
    routeOrigin = null;
    routeDestination = null;
    routeOriginInput.value = "";
    routeDestinationInput.value = "";
    routePickStage = "origin";
    pickRouteButton.textContent = "Cancel picking";
    pickRouteButton.setAttribute("aria-pressed", "true");
    setRouteStatus("Click the route origin on the map.", "success");
  }

  function acceptRoutePick(system) {
    if (!routePickStage) return;
    if (routePickStage === "origin") {
      setRouteEndpoint("origin", system);
      routePickStage = "destination";
      setRouteStatus(`Origin: ${system.name}. Now click the destination.`, "success");
    } else {
      setRouteEndpoint("destination", system);
      routePickStage = null;
      pickRouteButton.textContent = "Pick on map";
      pickRouteButton.setAttribute("aria-pressed", "false");
      calculateRoute();
    }
  }

  searchForm.addEventListener("submit", (event) => {
    event.preventDefault();
    searchInput.setCustomValidity("");
    locateByName(searchInput.value);
  });
  searchInput.addEventListener("input", () => searchInput.setCustomValidity(""));

  routeForm.addEventListener("submit", (event) => {
    event.preventDefault();
    routeOriginInput.setCustomValidity("");
    routeDestinationInput.setCustomValidity("");
    const origin = findSystem(routeOriginInput.value);
    const destination = findSystem(routeDestinationInput.value);
    if (!origin) {
      routeOriginInput.setCustomValidity("No matching origin system in this snapshot");
      routeOriginInput.reportValidity();
      return;
    }
    if (!destination) {
      routeDestinationInput.setCustomValidity("No matching destination system in this snapshot");
      routeDestinationInput.reportValidity();
      return;
    }
    setRouteEndpoint("origin", origin);
    setRouteEndpoint("destination", destination);
    routePickStage = null;
    pickRouteButton.textContent = "Pick on map";
    pickRouteButton.setAttribute("aria-pressed", "false");
    calculateRoute();
  });
  routeOriginInput.addEventListener("input", () => routeOriginInput.setCustomValidity(""));
  routeDestinationInput.addEventListener("input", () => routeDestinationInput.setCustomValidity(""));
  pickRouteButton.addEventListener("click", beginRoutePicking);
  document.getElementById("swap-route").addEventListener("click", () => {
    if (!routeOrigin || !routeDestination) return;
    [routeOrigin, routeDestination] = [routeDestination, routeOrigin];
    routeOriginInput.value = routeOrigin.name;
    routeDestinationInput.value = routeDestination.name;
    calculateRoute();
  });
  document.getElementById("clear-route").addEventListener("click", () => {
    routeRequest += 1;
    clearTimeout(routeTimer);
    routeOrigin = null;
    routeDestination = null;
    routePickStage = null;
    routeOriginInput.value = "";
    routeDestinationInput.value = "";
    pickRouteButton.textContent = "Pick on map";
    pickRouteButton.setAttribute("aria-pressed", "false");
    clearRoutePath();
    setRouteStatus("Enter endpoints or pick two systems on the map.");
  });

  jumpRange.addEventListener("input", () => {
    jumpRangeValue.textContent = `${Number(jumpRange.value).toFixed(1)} pc`;
    routeSpatialIndex = null;
    showDetails(selected);
    scheduleRouteCalculation();
    requestDraw();
  });
  pointSize.addEventListener("input", () => {
    pointSizeValue.textContent = `${Number(pointSize.value).toFixed(1).replace(".0", "")} px`;
    requestDraw();
  });

  frontierButton.addEventListener("click", () => {
    markFrontiers = !markFrontiers;
    updateFrontierButton();
    requestDraw();
  });

  document.getElementById("reset-view").addEventListener("click", () => {
    Object.assign(camera, initialCamera, { panX: 0, panY: 0 });
    frameAll();
    requestDraw();
  });

  canvas.addEventListener("pointerdown", (event) => {
    canvas.setPointerCapture(event.pointerId);
    drag = { x: event.clientX, y: event.clientY, moved: false, pan: event.shiftKey || event.button === 1 };
    canvas.classList.add("dragging");
  });
  canvas.addEventListener("pointermove", (event) => {
    if (!drag) return;
    const dx = event.clientX - drag.x;
    const dy = event.clientY - drag.y;
    drag.x = event.clientX;
    drag.y = event.clientY;
    drag.moved ||= Math.abs(dx) + Math.abs(dy) > 2;
    if (drag.pan) {
      camera.panX += dx;
      camera.panY += dy;
    } else {
      camera.yaw += dx * 0.008;
      camera.pitch = Math.max(-1.48, Math.min(1.48, camera.pitch + dy * 0.008));
    }
    requestDraw();
  });
  canvas.addEventListener("pointerup", (event) => {
    if (drag && !drag.moved) {
      const rect = canvas.getBoundingClientRect();
      const x = event.clientX - rect.left;
      const y = event.clientY - rect.top;
      let closest = null;
      let closestDistance = 12;
      for (let index = 0; index < projected.length; index += 1) {
        const point = projected[index];
        if (!point?.visible) continue;
        const candidateDistance = Math.hypot(point.x - x, point.y - y);
        if (candidateDistance < closestDistance) {
          closestDistance = candidateDistance;
          closest = systems[index];
        }
      }
      if (closest) {
        selectSystem(closest);
        acceptRoutePick(closest);
      }
    }
    drag = null;
    canvas.classList.remove("dragging");
  });
  canvas.addEventListener("pointercancel", () => {
    drag = null;
    canvas.classList.remove("dragging");
  });
  canvas.addEventListener("wheel", (event) => {
    event.preventDefault();
    camera.distance = Math.max(1, Math.min(10000, camera.distance * Math.exp(event.deltaY * 0.001)));
    requestDraw();
  }, { passive: false });
  canvas.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && routePickStage) {
      routePickStage = null;
      pickRouteButton.textContent = "Pick on map";
      pickRouteButton.setAttribute("aria-pressed", "false");
      setRouteStatus("Map endpoint selection cancelled.");
      event.preventDefault();
      return;
    }
    const orbit = 0.07;
    if (event.key === "ArrowLeft") camera.yaw -= orbit;
    else if (event.key === "ArrowRight") camera.yaw += orbit;
    else if (event.key === "ArrowUp") camera.pitch = Math.max(-1.48, camera.pitch - orbit);
    else if (event.key === "ArrowDown") camera.pitch = Math.min(1.48, camera.pitch + orbit);
    else if (event.key === "+" || event.key === "=") camera.distance = Math.max(1, camera.distance * 0.9);
    else if (event.key === "-" || event.key === "_") camera.distance = Math.min(10000, camera.distance * 1.1);
    else return;
    event.preventDefault();
    requestDraw();
  });

  new ResizeObserver(resize).observe(canvas);
  load();
})();
