<script setup lang="ts">
interface SosColumn {
  key: string
  label: string
  numeric?: boolean
}

withDefaults(
  defineProps<{
    columns?: SosColumn[]
    rows?: Record<string, unknown>[]
    zebra?: boolean
    rowKey?: string
  }>(),
  {
    columns: () => [],
    rows: () => [],
    zebra: false,
    rowKey: undefined,
  }
)
</script>

<template>
  <div class="sos-table-wrap">
    <table class="sos-table" :class="{ 'sos-table--zebra': zebra }">
      <thead>
        <tr>
          <th v-for="col in columns" :key="col.key" :class="{ 'sos-num': col.numeric }">
            {{ col.label }}
          </th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="(row, index) in rows"
          :key="rowKey && row[rowKey] != null ? String(row[rowKey]) : index"
        >
          <td v-for="col in columns" :key="col.key" :class="{ 'sos-num': col.numeric }">
            <slot :name="`cell-${col.key}`" :row="row" :value="row[col.key]">
              {{ row[col.key] }}
            </slot>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
