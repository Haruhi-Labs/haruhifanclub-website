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
  const initial = syncMasonryLayout(state, first, 2, undefined, entry)
  const initialEntries = new Map(initial.flat().map(value => [value.item.id, value]))
  const next = [...first, { id: 4, ratio: 1.5 }, { id: 5, ratio: 0.75 }]
  const appended = syncMasonryLayout(state, next, 2, first, entry)

  assert.deepEqual(placements(appended).slice(0, 3), placements(initial))
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
  syncMasonryLayout(state, items, 2, undefined, entry)
  const reordered = items.slice().reverse()
  const rebuilt = syncMasonryLayout(state, reordered, 3, items, entry)

  const reference = createMasonryState(3)
  const expected = syncMasonryLayout(reference, reordered, 3, undefined, entry)
  assert.deepEqual(placements(rebuilt), placements(expected))
})
