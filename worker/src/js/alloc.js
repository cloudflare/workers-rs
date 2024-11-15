/**
 * @typedef {Object} AllocInfo
 * @property {number} size
 * @property {number} count
 */

/** @type {Map<string, AllocInfo>} */
const allocs = new Map();
/** @type {Map<number, string>} */
const pointersToNames = new Map();

/**
 * @param {number} pointer
 * @param {number} size 
 */
export function onAlloc(pointer, size) {
    const name = callerName();
    const info = allocs.get(name);

    if (info) {
        info.size += size;
        info.count++;
    } else {
        allocs.set(name, { size, count: 1 });
    }

    pointersToNames.set(pointer, name);
}

/**
 * @param {number} pointer 
 */
export function onDealloc(pointer) {
    const name = pointersToNames.get(pointer);
    const info = allocs.get(name);

    if (info) {
        info.count--;
    } else {
        console.warn(`Unknown pointer ${pointer} deallocated`);
    }

    pointersToNames.delete(pointer);
}

export function printAllocs() {
    console.log("Allocations:");
    for (const [name, info] of allocs) {
        console.log(`${name}: ${info.size} bytes, ${info.count} allocations`);
    }
}

/**
 * @returns {string}
 */
function callerName() {
    const [_cause, ...stackFrames] = new Error().stack.split('\n');
    return stackFrames.find(frame => !frame.includes(".js")) ?? "<unknown>";
}