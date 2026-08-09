export function createMasonryState(columnCount = 1) {
  const columns = Math.max(1, columnCount)
  return {
    columnCount: columns,
    itemCount: 0,
    firstItem: undefined,
    lastItem: undefined,
    heights: Array(columns).fill(0),
    columns: Array.from({ length: columns }, () => []),
  }
}

function appendItems(state, items, startPosition, createEntry) {
  const columns = state.columns
  for (let position = startPosition; position < items.length; position += 1) {
    let target = 0
    for (let index = 1; index < state.heights.length; index += 1) {
      if (state.heights[index] < state.heights[target]) target = index
    }
    const entry = createEntry(items[position], position)
    columns[target].push(entry)
    state.heights[target] += (1 / entry.ratio) + 0.06
  }
  // shallowRef 只需要新的外层数组触发视图更新；列本身就地追加，避免每批续载都
  // 复制此前全部作品。这样追加 k 项从 O(n + k·c) 降为 O(k·c)。
  state.columns = columns.slice()
  state.itemCount = items.length
  state.firstItem = items[0]
  state.lastItem = items.at(-1)
  return state.columns
}

export function syncMasonryLayout(state, items, columnCount, createEntry) {
  const columns = Math.max(1, columnCount)
  const previousLength = state.itemCount
  const isAppend = state.columnCount === columns
    && items.length > previousLength
    && (
      previousLength === 0
      || (
        items[0] === state.firstItem
        && items[previousLength - 1] === state.lastItem
      )
    )

  if (isAppend) return appendItems(state, items, previousLength, createEntry)

  state.columnCount = columns
  state.heights = Array(columns).fill(0)
  state.columns = Array.from({ length: columns }, () => [])
  return appendItems(state, items, 0, createEntry)
}
