import assert from 'node:assert/strict'
import test from 'node:test'
import { createMasonryState, syncMasonryLayout } from './incrementalMasonry.js'

function entry(item, position) {
  return { item, position, ratio: item.ratio }
}

function placements(columns) {
  return columns
    .flatMap((column, columnIndex) => column.map(value => [value.item.id, columnIndex]))
    .sort((left, right) => left[0] - right[0])
}

test('追加批次只创建新条目，并保持既有作品所在列', () => {
  const first = [
    { id: 1, ratio: 1 },
    { id: 2, ratio: 0.5 },
    { id: 3, ratio: 2 },
  ]
  const state = createMasonryState(2)
  const initial = syncMasonryLayout(state, first, 2, entry)
  const initialPlacements = placements(initial)
  const initialColumns = initial.slice()
  const initialEntries = new Map(initial.flat().map(value => [value.item.id, value]))
  const next = [...first, { id: 4, ratio: 1.5 }, { id: 5, ratio: 0.75 }]
  const appended = syncMasonryLayout(state, next, 2, entry)

  assert.deepEqual(
    placements(appended).filter(([id]) => id <= 3),
    initialPlacements,
  )
  for (let index = 0; index < initialColumns.length; index += 1) {
    assert.equal(appended[index], initialColumns[index])
  }
  for (const value of appended.flat().filter(value => value.item.id <= 3)) {
    assert.equal(value, initialEntries.get(value.item.id))
  }
})

test('重排或列数变化时重建结果与一次性布局一致', () => {
  const items = Array.from({ length: 12 }, (_, index) => ({
    id: index + 1,
    ratio: 0.5 + (index % 5) * 0.35,
  }))
  const state = createMasonryState(2)
  syncMasonryLayout(state, items, 2, entry)
  const reordered = items.slice().reverse()
  const rebuilt = syncMasonryLayout(state, reordered, 3, entry)

  const reference = createMasonryState(3)
  const expected = syncMasonryLayout(reference, reordered, 3, entry)
  assert.deepEqual(placements(rebuilt), placements(expected))
})

test('同一数组就地追加时仍只处理新作品', () => {
  const items = [
    { id: 1, ratio: 1 },
    { id: 2, ratio: 0.8 },
  ]
  const state = createMasonryState(2)
  let created = 0
  const trackedEntry = (item, position) => {
    created += 1
    return entry(item, position)
  }
  syncMasonryLayout(state, items, 2, trackedEntry)
  items.push({ id: 3, ratio: 1.4 }, { id: 4, ratio: 0.6 })
  const columns = syncMasonryLayout(state, items, 2, trackedEntry)

  assert.equal(created, 4)
  assert.deepEqual(placements(columns).map(([id]) => id), [1, 2, 3, 4])
})
