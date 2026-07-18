<script setup>
import { computed } from 'vue'
import { resolveUploadUrl } from '@haruhi/api-client'
import { SosAvatar } from '@haruhi/ui'

const props = defineProps({
  units: { type: Array, default: () => [] },
  assignments: { type: Array, default: () => [] },
  displayMode: { type: String, default: 'grouped' },
  parentId: { type: [Number, String], default: null },
  depth: { type: Number, default: 0 },
})

const visibleUnits = computed(() => {
  if (props.displayMode === 'flat') return props.depth === 0 ? props.units : []
  return props.units.filter((unit) => {
    const parent = unit.parentId ?? null
    return String(parent ?? '') === String(props.parentId ?? '')
  })
})

function assignmentsFor(unitId) {
  return props.assignments.filter((assignment) => assignment.unitId === unitId)
}

function kindLabel(kind) {
  return (
    {
      department: '部门',
      committee: '委员会',
      council: '理事会',
      group: '工作组',
      project: '项目组',
      rotation: '轮值团队',
      other: '协作单元',
    }[kind] || '组织单元'
  )
}

function termLabel(assignment) {
  if (!assignment.termStart && !assignment.termEnd) return ''
  return `${assignment.termStart || '任期未定'}—${assignment.termEnd || '至今'}`
}
</script>

<template>
  <div
    class="organization-tree"
    :class="[`organization-tree--${displayMode}`, { 'organization-tree--nested': depth > 0 }]"
  >
    <article v-for="unit in visibleUnits" :key="unit.id" class="organization-unit">
      <header>
        <p class="sos-eyebrow">{{ kindLabel(unit.kind) }}</p>
        <h2>{{ unit.name }}</h2>
        <p v-if="unit.description">{{ unit.description }}</p>
      </header>
      <ul v-if="assignmentsFor(unit.id).length" class="organization-assignment-list">
        <li v-for="assignment in assignmentsFor(unit.id)" :key="assignment.id">
          <SosAvatar
            :src="assignment.avatar ? resolveUploadUrl(assignment.avatar) : undefined"
            :name="assignment.displayName"
            size="sm"
          />
          <span>
            <strong>{{ assignment.displayName }}</strong>
            <small>{{ assignment.title }}</small>
            <small v-if="termLabel(assignment)">{{ termLabel(assignment) }}</small>
          </span>
        </li>
      </ul>
      <OrganizationTree
        v-if="displayMode !== 'flat'"
        :units="units"
        :assignments="assignments"
        :display-mode="displayMode"
        :parent-id="unit.id"
        :depth="depth + 1"
      />
    </article>
  </div>
</template>
