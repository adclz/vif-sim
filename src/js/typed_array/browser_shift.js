export const shiftRight = (collection, steps = 1) => {
    collection.set(collection.subarray(0, -steps), steps)
    collection.fill(0, 0, steps)
    return collection
}

export const shiftLeft = (collection, steps = 1) => {
    collection.set(collection.subarray(steps))
    collection.fill(0, -steps)
    return collection
}