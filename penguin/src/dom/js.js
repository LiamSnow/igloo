export function getDocument() {
    return document;
}

export function getElementById(id) {
    return document.getElementById(id);
}

export function createDiv() {
    return document.createElement('div');
}

export function createButton() {
    return document.createElement('button');
}

export function createInput() {
    return document.createElement('input');
}

export function createTextarea() {
    return document.createElement('textarea');
}

const SVG_NS = 'http://www.w3.org/2000/svg';

export function createSvg() {
    return document.createElementNS(SVG_NS, 'svg');
}

export function createPath() {
    return document.createElementNS(SVG_NS, 'path');
}

export function createCircle() {
    return document.createElementNS(SVG_NS, 'circle');
}

export function createRect() {
    return document.createElementNS(SVG_NS, 'rect');
}

export function createPattern() {
    return document.createElementNS(SVG_NS, 'pattern');
}

export function createDefs() {
    return document.createElementNS(SVG_NS, 'defs');
}

export function createPolygon() {
    return document.createElementNS(SVG_NS, 'polygon');
}

export function setAttr(element, key, value) {
    element.setAttribute(key, value);
}

export function removeAttr(element, key) {
    element.removeAttribute(key);
}

export function setClass(element, name) {
    element.className = name;
}

export function setId(element, id) {
    element.id = id;
}

export function setHtml(element, html) {
    element.innerHTML = html;
}

export function setText(element, text) {
    element.textContent = text;
}

export function show(element) {
    element.style.display = '';
}

export function hide(element) {
    element.style.display = 'none';
}

export function setStyle(element, key, value) {
    element.style.setProperty(key, value);
}

export function setLeft(element, value) {
    element.style.left = `${value}px`;
}

export function setTop(element, value) {
    element.style.top = `${value}px`;
}

export function setRight(element, value) {
    element.style.right = `${value}px`;
}

export function setBottom(element, value) {
    element.style.bottom = `${value}px`;
}

export function setWidth(element, value) {
    element.style.width = `${value}px`;
}

export function setHeight(element, value) {
    element.style.height = `${value}px`;
}

export function setSize(element, width, height) {
    element.style.width = `${width}px`;
    element.style.height = `${height}px`;
}

export function translate(element, x, y) {
    element.style.transform = `translate(${x}px, ${y}px)`;
}

export function translateScale(element, x, y, scale) {
    element.style.transform = `translate(${x}px, ${y}px) scale(${scale})`;
}

export function setViewBox(element, x, y, width, height) {
    element.setAttribute('viewBox', `${x} ${y} ${width} ${height}`);
}

export function setPathD(element, d) {
    element.setAttribute('d', d);
}

export function setStroke(element, color) {
    element.setAttribute('stroke', color);
}

export function setStrokeWidth(element, width) {
    element.setAttribute('stroke-width', width.toString());
}

export function setFill(element, color) {
    element.setAttribute('fill', color);
}

export function setStrokeDasharray(element, pattern) {
    element.setAttribute('stroke-dasharray', pattern);
}

export function setPoints(element, points) {
    element.setAttribute('points', points);
}

export function setCx(element, value) {
    element.setAttribute('cx', value.toString());
}

export function setCy(element, value) {
    element.setAttribute('cy', value.toString());
}

export function setR(element, value) {
    element.setAttribute('r', value.toString());
}

export function setX(element, value) {
    element.setAttribute('x', value.toString());
}

export function setY(element, value) {
    element.setAttribute('y', value.toString());
}

export function setSvgWidth(element, value) {
    element.setAttribute('width', value.toString());
}

export function setSvgHeight(element, value) {
    element.setAttribute('height', value.toString());
}

export function setPatternUnits(element, units) {
    element.setAttribute('patternUnits', units);
}

export function getValue(element) {
    return element.value;
}

export function setValue(element, value) {
    element.value = value;
}

export function getChecked(element) {
    return element.checked;
}

export function setChecked(element, checked) {
    element.checked = checked;
}

export function setPlaceholder(element, text) {
    element.placeholder = text;
}

export function setType(element, type) {
    element.type = type;
}

export function focus(element) {
    element.focus();
}

export function blur(element) {
    element.blur();
}

export function getClientRect(element) {
    const r = element.getBoundingClientRect();
    return [r.left, r.top, r.right, r.bottom];
}

export function getOffsetWidth(element) {
    return element.offsetWidth;
}

export function getOffsetHeight(element) {
    return element.offsetHeight;
}

export function appendChild(parent, child) {
    parent.appendChild(child);
}

export function removeElement(element) {
    element.remove();
}

export function setTabIndex(element, index) {
    element.tabIndex = index;
}

export function redrawWire(element1, element2, from_x, from_y, to_x, to_y) {
    const width = to_x - from_x;
    const height = to_y - from_y;
    const offset = Math.abs(width) * 0.5;
    const cx1 = offset;
    const cx2 = width - offset;
    
    const path = 'M 0 0 C ' + cx1 + ' 0, ' + cx2 + ' ' + height + ', ' + width + ' ' + height;
    
    if (element1.__cachedPath !== path) {
        element1.setAttribute('d', path);
        if (element2) {
            element2.setAttribute('d', path);
        }
        element1.__cachedPath = path;
    }
    
    if (element1.__cachedTransX !== from_x || element1.__cachedTransY !== from_y) {
        const s = 'translate(' + from_x + 'px, ' + from_y + 'px)';
        element1.style.transform = s;
        if (element2) {
            element2.style.transform = s;
        }
        element1.__cachedTransX = from_x;
        element1.__cachedTransY = from_y;
    }
}

