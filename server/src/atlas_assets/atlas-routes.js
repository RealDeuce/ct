"use strict";

((root, factory) => {
  const routes = factory();
  if (typeof module === "object" && module.exports) module.exports = routes;
  root.CepheusAtlasRoutes = routes;
})(globalThis, () => {
  function distance(a, b) {
    return Math.hypot(
      a.position[0] - b.position[0],
      a.position[1] - b.position[1],
      a.position[2] - b.position[2]
    );
  }

  function cellKey(x, y, z) {
    return `${x},${y},${z}`;
  }

  function buildSpatialIndex(systems, jumpRange) {
    if (!Number.isFinite(jumpRange) || jumpRange <= 0) {
      throw new Error("jump range must be positive");
    }
    const buckets = new Map();
    for (let index = 0; index < systems.length; index += 1) {
      const position = systems[index].position;
      const x = Math.floor(position[0] / jumpRange);
      const y = Math.floor(position[1] / jumpRange);
      const z = Math.floor(position[2] / jumpRange);
      const key = cellKey(x, y, z);
      const bucket = buckets.get(key);
      if (bucket) bucket.push(index);
      else buckets.set(key, [index]);
    }
    return { jumpRange, buckets };
  }

  function neighborIndices(systems, systemIndex, spatialIndex) {
    const range = spatialIndex.jumpRange;
    const position = systems[systemIndex].position;
    const cell = position.map((coordinate) => Math.floor(coordinate / range));
    const neighbors = [];
    for (let x = cell[0] - 1; x <= cell[0] + 1; x += 1) {
      for (let y = cell[1] - 1; y <= cell[1] + 1; y += 1) {
        for (let z = cell[2] - 1; z <= cell[2] + 1; z += 1) {
          const bucket = spatialIndex.buckets.get(cellKey(x, y, z));
          if (!bucket) continue;
          for (const candidate of bucket) {
            if (candidate !== systemIndex && distance(systems[systemIndex], systems[candidate]) <= range + 1e-9) {
              neighbors.push(candidate);
            }
          }
        }
      }
    }
    neighbors.sort((left, right) => systems[left].id - systems[right].id);
    return neighbors;
  }

  // Breadth-first search produces the fewest-jump route. Stable system-ID
  // ordering makes equally short choices reproducible for a given snapshot.
  function shortestRoute(systems, originIndex, destinationIndex, spatialIndex) {
    if (originIndex === destinationIndex) return [originIndex];
    if (originIndex < 0 || destinationIndex < 0 ||
        originIndex >= systems.length || destinationIndex >= systems.length) return null;
    const unseen = -2;
    const parent = new Int32Array(systems.length);
    parent.fill(unseen);
    parent[originIndex] = -1;
    const queue = new Int32Array(systems.length);
    let head = 0;
    let tail = 0;
    queue[tail++] = originIndex;

    while (head < tail && parent[destinationIndex] === unseen) {
      const current = queue[head++];
      for (const neighbor of neighborIndices(systems, current, spatialIndex)) {
        if (parent[neighbor] !== unseen) continue;
        parent[neighbor] = current;
        queue[tail++] = neighbor;
        if (neighbor === destinationIndex) break;
      }
    }
    if (parent[destinationIndex] === unseen) return null;

    const route = [];
    for (let current = destinationIndex; current !== -1; current = parent[current]) {
      route.push(current);
    }
    route.reverse();
    return route;
  }

  function routeDistance(systems, route) {
    let total = 0;
    for (let index = 1; index < route.length; index += 1) {
      total += distance(systems[route[index - 1]], systems[route[index]]);
    }
    return total;
  }

  return { buildSpatialIndex, distance, routeDistance, shortestRoute };
});
