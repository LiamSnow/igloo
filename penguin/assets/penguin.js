let viewport = {
  pan: { x: 0, y: 0 },
  zoom: 1.0
};

let interaction = {
  mode: 'idle',
  startPos: null,
  startPan: null,
  draggedNodes: [],
  boxSelectStart: null,
  wiringData: null,
};

let gridSettings = {
  enabled: false,
  snap: false,
  size: 20
};

export function init() {
  const penguin = document.getElementById('penguin');
  penguin.focus();

  document.addEventListener('mousemove', onMouseMove);
  document.addEventListener('mouseup', onMouseUp);
  penguin.addEventListener('keydown', onKeyDown);
  penguin.addEventListener('mousedown', onMouseDown);
  penguin.addEventListener('contextmenu', onContextMenu);
  penguin.addEventListener('wheel', onWheel, { passive: false });

  rerender();
  setTimeout(rerender, 100);
}

export function rerender() {
  updateViewportTransform();
  updateViewBox();
  updateAllWires();
}

export function delayedRerender() {
  setTimeout(rerender, 20);
}

export function getAllNodePositions() {
  const nodes = document.querySelectorAll('.penguin-node[data-node-id]');
  return Array.from(nodes).map(node => {
    const pos = getNodeWorldPosition(node);
    return [
      parseInt(node.dataset.nodeId),
      pos.x,
      pos.y
    ];
  });
}

export function getSelectedNodeIds() {
  const nodes = document.querySelectorAll('.penguin-node.selected');
  return Array.from(nodes).map(node => parseInt(node.dataset.nodeId));
}

export function getSelectedWireIds() {
  const wires = document.querySelectorAll('.penguin-wire.selected');
  return Array.from(wires).map(wire => parseInt(wire.dataset.wireId));
}

export function startWiring(startNodeId, startPinId, isOutput) {
  interaction.mode = 'wiring';
  interaction.wiringData = { startNodeId, startPinId, isOutput };
}

export function stopWiring() {
  interaction.mode = 'idle';
  interaction.wiringData = null;
}

function onKeyDown(e) {
  if (e.key === 'Escape') {
    clearSelection();
  }
}

function onContextMenu(e) {
  e.preventDefault();
  const penguin = document.getElementById('penguin');
  penguin.focus();

  const node = e.target.closest('.penguin-node');
  const wire = e.target.closest('.penguin-wire');

  if (!node && !wire) {
    const penguin = document.getElementById('penguin');
    penguin.focus();

    interaction.mode = 'box-selecting';
    interaction.boxSelectStart = { x: e.clientX, y: e.clientY };

    if (!e.shiftKey && !e.ctrlKey) {
      clearSelection();
    }
  }
}

function onMouseDown(e) {
  const penguin = document.getElementById('penguin');
  penguin.focus();

  if (e.button === 0) {
    e.preventDefault();

    const node = e.target.closest('.penguin-node');
    const wire = e.target.closest('.penguin-wire');

    if (node) {
      if (e.shiftKey || e.ctrlKey) {
        node.classList.add('selected');
      } else {
        clearSelection();
        node.classList.add('selected');
      }

      changeInteractionMode('dragging');
      interaction.draggedNodes = Array.from(
        document.querySelectorAll('.penguin-node.selected')
      );
    }
    else if (wire) {
      if (e.shiftKey || e.ctrlKey) {
        wire.classList.add('selected');
      } else {
        clearSelection();
        wire.classList.add('selected');
      }
    }
    else {
      clearSelection();
      changeInteractionMode('panning');
      interaction.startPos = { x: e.clientX, y: e.clientY };
      interaction.startPan = { x: viewport.pan.x, y: viewport.pan.y };
    }
  }
}

function onMouseMove(e) {
  if (interaction.mode === 'panning') {
    handlePanning(e);
  }
  else if (interaction.mode === 'dragging') {
    handleDragging(e);
  }
  else if (interaction.mode === 'box-selecting') {
    updateSelectionBox(e.clientX, e.clientY);
  }
  else if (interaction.mode === 'wiring') {
    updateTempWire(e);
  }
}

function onMouseUp(e) {
  changeInteractionMode('idle');
  interaction.startPos = null;
  interaction.startPan = null;
  interaction.draggedNodes = [];
  interaction.boxSelectStart = null;
}

function onWheel(e) {
  e.preventDefault();

  const penguin = document.getElementById('penguin');
  penguin.focus();

  const rect = e.currentTarget.getBoundingClientRect();
  const mouseX = e.clientX - rect.left;
  const mouseY = e.clientY - rect.top;

  const zoomDelta = e.deltaY > 0 ? 0.9 : 1.1;
  const newZoom = Math.max(0.1, Math.min(3.0, viewport.zoom * zoomDelta));

  const zoomRatio = newZoom / viewport.zoom;
  viewport.pan.x = mouseX - (mouseX - viewport.pan.x) * zoomRatio;
  viewport.pan.y = mouseY - (mouseY - viewport.pan.y) * zoomRatio;
  viewport.zoom = newZoom;

  rerender();
}

function changeInteractionMode(mode) {
  if (interaction.mode === 'box-selecting') {
    completeBoxSelection();
  }

  if (interaction.mode === 'dragging' && gridSettings.snap) {
    snapNodesToGrid();
  }
  
  interaction.mode = mode;
}

function handlePanning(e) {
  const dx = e.clientX - interaction.startPos.x;
  const dy = e.clientY - interaction.startPos.y;

  viewport.pan.x = interaction.startPan.x + dx;
  viewport.pan.y = interaction.startPan.y + dy;

  rerender();
}

function handleDragging(e) {
  const dx = e.movementX / viewport.zoom;
  const dy = e.movementY / viewport.zoom;

  interaction.draggedNodes.forEach(node => {
    const pos = getNodeWorldPosition(node);
    node.style.transform = `translate(${pos.x + dx}px, ${pos.y + dy}px)`;
  });

  updateAllWires();
}

export function setGridSettings(enabled, snap, size) {
  gridSettings.enabled = enabled;
  gridSettings.snap = snap;
  gridSettings.size = size;
}

function snapNodesToGrid() {
  interaction.draggedNodes.forEach(node => {
    const pos = getNodeWorldPosition(node);
    const snapped = snapToGrid(pos.x, pos.y, gridSettings.size);
    node.style.transform = `translate(${snapped.x}px, ${snapped.y}px)`;
  });

  updateAllWires();
}

function snapToGrid(x, y, gridSize) {
  return {
    x: Math.round(x / gridSize) * gridSize,
    y: Math.round(y / gridSize) * gridSize
  };
}

function updateSelectionBox(currentX, currentY) {
  const box = document.getElementById('penguin-selection-box');

  const startX = interaction.boxSelectStart.x;
  const startY = interaction.boxSelectStart.y;

  const left = Math.min(startX, currentX);
  const top = Math.min(startY, currentY);
  const width = Math.abs(currentX - startX);
  const height = Math.abs(currentY - startY);

  box.style.left = `${left}px`;
  box.style.top = `${top}px`;
  box.style.width = `${width}px`;
  box.style.height = `${height}px`;
  box.style.display = 'block';
}

function updateTempWire(e) {
  const tempWire = document.getElementById('penguin-temp-wire');
  if (!tempWire) return;

  const { startNodeId, startPinId, isOutput } = interaction.wiringData;

  const startPos = getPinWorldPosition(startNodeId, startPinId, isOutput);
  if (!startPos) return;

  const penguin = document.getElementById('penguin');
  const penguinRect = penguin.getBoundingClientRect();
  const mouseWorld = {
    x: (e.clientX - penguinRect.left - viewport.pan.x) / viewport.zoom,
    y: (e.clientY - penguinRect.top - viewport.pan.y) / viewport.zoom
  };

  const d = isOutput
    ? createBezierPath(startPos, mouseWorld)
    : createBezierPath(mouseWorld, startPos);

  tempWire.setAttribute('d', d);
}

function clearSelection() {
  document.querySelectorAll('.penguin-node.selected').forEach(node => {
    node.classList.remove('selected');
  });
  document.querySelectorAll('.penguin-wire.selected').forEach(wire => {
    wire.classList.remove('selected');
  });
}

function completeBoxSelection() {
  const box = document.getElementById('penguin-selection-box');
  const boxRect = box.getBoundingClientRect();
  box.style.display = 'none';

  const nodes = document.querySelectorAll('.penguin-node');
  nodes.forEach(node => {
    const nodeRect = node.getBoundingClientRect();
    if (rectsIntersect(boxRect, nodeRect)) {
      node.classList.add('selected');
    }
  });

  const penguin = document.getElementById('penguin');
  const penguinRect = penguin.getBoundingClientRect();

  const worldBox = {
    left: (boxRect.left - penguinRect.left - viewport.pan.x) / viewport.zoom,
    top: (boxRect.top - penguinRect.top - viewport.pan.y) / viewport.zoom,
    right: (boxRect.right - penguinRect.left - viewport.pan.x) / viewport.zoom,
    bottom: (boxRect.bottom - penguinRect.top - viewport.pan.y) / viewport.zoom
  };

  const wires = document.querySelectorAll('.penguin-wire[data-wire-id]');
  wires.forEach(wire => {
    if (wireIntersectsBox(wire, worldBox)) {
      wire.classList.add('selected');
    }
  });
}

function wireIntersectsBox(pathElement, worldBox) {
  const totalLength = pathElement.getTotalLength();
  const sampleInterval = 5.0;
  const numSamples = Math.ceil(totalLength / sampleInterval);

  for (let i = 0; i <= numSamples; i++) {
    const distance = i * sampleInterval;
    const point = pathElement.getPointAtLength(distance);

    if (point.x >= worldBox.left && point.x <= worldBox.right &&
      point.y >= worldBox.top && point.y <= worldBox.bottom) {
      return true;
    }
  }

  return false;
}

function updateViewportTransform() {
  const viewportEl = document.getElementById('penguin-viewport');

  if (!viewportEl) {
    console.error('Missing viewport element');
    return;
  }

  viewportEl.style.transform = `translate(${viewport.pan.x}px, ${viewport.pan.y}px) scale(${viewport.zoom})`;
}

function updateViewBox() {
  const penguin = document.getElementById('penguin');
  const svg = document.getElementById('penguin-wires');

  if (!penguin || !svg) {
    console.error('Missing penguin or svg element');
    return;
  }

  const rect = penguin.getBoundingClientRect();

  const x = -viewport.pan.x / viewport.zoom;
  const y = -viewport.pan.y / viewport.zoom;
  const width = rect.width / viewport.zoom;
  const height = rect.height / viewport.zoom;

  svg.setAttribute('viewBox', `${x} ${y} ${width} ${height}`);
}

function updateAllWires() {
  const wires = document.querySelectorAll('.penguin-wire[data-wire-id]');

  wires.forEach(wirePath => {
    const fromNodeId = wirePath.dataset.fromNode;
    const fromPinId = wirePath.dataset.fromPin;
    const toNodeId = wirePath.dataset.toNode;
    const toPinId = wirePath.dataset.toPin;

    const fromPos = getPinWorldPosition(fromNodeId, fromPinId, true);
    const toPos = getPinWorldPosition(toNodeId, toPinId, false);

    if (fromPos && toPos) {
      const d = createBezierPath(fromPos, toPos);
      wirePath.setAttribute('d', d);
    } else {
      console.warn('Missing pin positions', { fromPos, toPos });
    }
  });
}

function getNodeWorldPosition(nodeEl) {
  const transform = nodeEl.style.transform;
  
  // translate(Xpx, Ypx)
  let match = transform.match(/translate\(([-\d.]+)px,\s*([-\d.]+)px\)/);
  if (match) {
    return { x: parseFloat(match[1]), y: parseFloat(match[2]) };
  }
  
  // translate(Xpx) (browser optimizes this when at 0,0)
  match = transform.match(/translate\(([-\d.]+)px\)/);
  if (match) {
    return { x: parseFloat(match[1]), y: 0 };
  }
  
  console.error('Could not parse transform, defaulting to (0, 0):', transform);
  return { x: 0, y: 0 };
}

function getPinWorldPosition(nodeId, pinId, isOutput) {
  const node = document.querySelector(`.penguin-node[data-node-id="${nodeId}"]`);
  const pin = document.querySelector(
    `.penguin-pin-hitbox[data-node-id="${nodeId}"][data-pin-id="${pinId}"][data-is-output="${isOutput}"]`
  );

  if (!node || !pin) {
    console.warn('Missing node or pin', { nodeId, pinId, isOutput, node, pin });
    return null;
  }

  const nodeRect = node.getBoundingClientRect();
  const pinRect = pin.getBoundingClientRect();

  const pinCenterX = pinRect.left + pinRect.width / 2;
  const pinCenterY = pinRect.top + pinRect.height / 2;

  const offsetX = (pinCenterX - nodeRect.left) / viewport.zoom;
  const offsetY = (pinCenterY - nodeRect.top) / viewport.zoom;

  const nodePos = getNodeWorldPosition(node);

  return {
    x: nodePos.x + offsetX,
    y: nodePos.y + offsetY
  };
}

function rectsIntersect(rect1, rect2) {
  return !(rect1.right < rect2.left ||
    rect1.left > rect2.right ||
    rect1.bottom < rect2.top ||
    rect1.top > rect2.bottom);
}

function createBezierPath(from, to) {
  const dist = Math.abs(to.x - from.x);
  const coff = Math.min(dist / 2, 100);
  return `M ${from.x} ${from.y} C ${from.x + coff} ${from.y}, ${to.x - coff} ${to.y}, ${to.x} ${to.y}`;
}
