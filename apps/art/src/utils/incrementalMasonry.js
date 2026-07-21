export function createMasonryState(columnCount = 1) {
  const columns = Math.max(1, columnCount)
  return {
    columnCount: columns,
    items: null,
    heights: Array(columns).fill(0),
    columns: Array.from({ length: columns }, () => []),
  }
}

function appendItems(state, items, startPosition, createEntry) {
  const columns = state.columns.map(column => column.slice())
  for (let position = startPosition; position < items.length; position += 1) {
    let target = 0
    for (let index = 1; index < state.heights.length; index += 1) {
      if (state.heights[index] < state.heights[target]) target = index
    }
    const entry = createEntry(items[position], position)
    columns[target].push(entry)
    state.heights[target] += (1 / entry.ratio) + 0.06
  }
  state.columns = columns
  state.items = items
  return columns
}

export function syncMasonryLayout(state, items, columnCount, previousItems, createEntry) {
  const columns = Math.max(1, columnCount)
  const previousLength = previousItems?.length || 0
  const isAppend = state.columnCount === columns
    && state.items === previousItems
    && items.length > previousLength
    && (
      previousLength === 0
      || (
        items[0] === previousItems[0]
        && items[previousLength - 1] === previousItems[previousLength - 1]
      )
    )

  if (isAppend) return appendItems(state, items, previousLength, createEntry)

  state.columnCount = columns
  state.heights = Array(columns).fill(0)
  state.columns = Array.from({ length: columns }, () => [])
  return appendItems(state, items, 0, createEntry)
}
