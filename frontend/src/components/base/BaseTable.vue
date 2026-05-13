<template>
  <div class="base-table">
    <!-- 工具栏 -->
    <div v-if="$slots.toolbar || showToolbar" class="table-toolbar">
      <slot name="toolbar">
        <div class="toolbar-left">
          <slot name="toolbar-left" />
        </div>
        <div class="toolbar-right">
          <slot name="toolbar-right" />
        </div>
      </slot>
    </div>

    <!-- 表格 -->
    <el-table
      ref="tableRef"
      :data="data"
      v-bind="$attrs"
      :stripe="stripe"
      :border="border"
      :size="size"
      :max-height="maxHeight"
      :height="height"
      :row-key="rowKey"
      :default-expand-all="defaultExpandAll"
      :tree-props="treeProps"
      :highlight-current-row="highlightCurrentRow"
      :current-row-key="currentRowKey"
      v-loading="loading"
      @selection-change="handleSelectionChange"
      @sort-change="handleSortChange"
      @filter-change="handleFilterChange"
      @current-change="handleCurrentChange"
      @row-click="handleRowClick"
    >
      <!-- 选择列 -->
      <el-table-column
        v-if="showSelection"
        type="selection"
        width="55"
        fixed="left"
      />

      <!-- 序号列 -->
      <el-table-column
        v-if="showIndex"
        type="index"
        label="#"
        width="60"
        fixed="left"
      />

      <!-- 动态列 -->
      <template v-for="column in columns" :key="column.prop">
        <el-table-column
          :prop="column.prop"
          :label="column.label"
          :width="column.width"
          :min-width="column.minWidth"
          :fixed="column.fixed"
          :sortable="column.sortable"
          :resizable="column.resizable !== false"
          :formatter="column.formatter"
          :show-overflow-tooltip="column.showOverflowTooltip !== false"
        >
          <template #default="{ row, $index }">
            <!-- 自定义插槽 -->
            <slot
              v-if="$slots[`column-${column.prop}`]"
              :name="`column-${column.prop}`"
              :row="row"
              :index="$index"
              :value="row[column.prop]"
            >
            </slot>

            <!-- 状态徽章 -->
            <StatusBadge
              v-else-if="column.type === 'status'"
              :value="row[column.prop]"
              :type-map="column.statusMap"
            />

            <!-- 时间格式化 -->
            <span v-else-if="column.type === 'datetime'">
              {{ formatDateTime(row[column.prop]) }}
            </span>

            <!-- 地址格式化 -->
            <span v-else-if="column.type === 'address'">
              {{ formatAddressUtil(row[column.prop], column.addressLength) }}
            </span>

            <!-- 金额格式化 -->
            <span v-else-if="column.type === 'amount'">
              {{ formatAmount(row[column.prop], column.decimals) }}
            </span>

            <!-- 哈希格式化 -->
            <span v-else-if="column.type === 'hash'">
              {{ formatHash(row[column.prop], column.hashLength) }}
            </span>

            <!-- 默认显示 -->
            <span v-else>{{ row[column.prop] }}</span>
          </template>

          <!-- 表头插槽 -->
          <template #header="{ column: _column }">
            <slot :name="`header-${column.prop}`">
              {{ _column.label }}
            </slot>
          </template>
        </el-table-column>
      </template>

      <!-- 操作列 -->
      <el-table-column
        v-if="$slots.actions || actions.length > 0"
        :label="actionLabel"
        :width="actionWidth"
        :fixed="actionFixed"
      >
        <template #default="{ row, $index }">
          <slot name="actions" :row="row" :index="$index">
            <BaseButton
              v-for="act in actions"
              :key="act.prop"
              v-if="!act.hidden?.(row, $index)"
              :type="act.type"
              :size="act.size || 'small'"
              :disabled="act.disabled?.(row, $index)"
              @click="act.handler?.(row, $index)"
            >
              {{ act.label }}
            </BaseButton>
          </slot>
        </template>
      </el-table-column>
    </el-table>

    <!-- 分页 -->
    <div v-if="pagination" class="table-pagination">
      <slot name="pagination">
        <el-pagination
          v-model:current-page="paginationCurrentPage"
          v-model:page-size="paginationPageSize"
          :page-sizes="paginationPageSizes"
          :total="paginationTotal"
          :layout="paginationLayout"
          :background="paginationBackground"
          @current-change="handleCurrentPageChange"
          @size-change="handlePageSizeChange"
        />
      </slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { ElTable } from 'element-plus'
import BaseButton from './BaseButton.vue'
import StatusBadge from './StatusBadge.vue'
import { formatAddress as formatAddressUtil, formatTime } from '@/utils/format'

export interface TableColumnConfig {
  prop: string
  label: string
  width?: number | string
  minWidth?: number | string
  fixed?: boolean | ('left' | 'right')
  sortable?: boolean | 'custom'
  resizable?: boolean
  formatter?: (row: any, column: any, cellValue: any) => any
  showOverflowTooltip?: boolean
  type?: 'default' | 'status' | 'datetime' | 'address' | 'amount' | 'hash'
  statusMap?: Record<string, { type: 'success' | 'warning' | 'danger' | 'info'; text?: string }>
  addressLength?: number
  decimals?: number
  hashLength?: number
}

export interface TableAction {
  prop: string
  label: string
  type?: 'primary' | 'success' | 'warning' | 'danger' | 'info'
  size?: 'large' | 'default' | 'small'
  hidden?: (row: any, index: number) => boolean
  disabled?: (row: any, index: number) => boolean
  handler: (row: any, index: number) => void
}

export interface PaginationConfig {
  currentPage: number
  pageSize: number
  total: number
  pageSizes?: number[]
  layout?: string
  background?: boolean
}

interface Props {
  data: any[]
  columns: TableColumnConfig[]
  loading?: boolean
  stripe?: boolean
  border?: boolean
  size?: 'large' | 'default' | 'small'
  maxHeight?: number | string
  height?: number | string
  rowKey?: string | ((row: any) => string)
  defaultExpandAll?: boolean
  treeProps?: Record<string, string>
  highlightCurrentRow?: boolean
  currentRowKey?: any
  showSelection?: boolean
  showIndex?: boolean
  showToolbar?: boolean
  actions?: TableAction[]
  actionLabel?: string
  actionFixed?: boolean | 'left' | 'right'
  actionWidth?: number | string
  pagination?: PaginationConfig | boolean
}

const props = withDefaults(defineProps<Props>(), {
  columns: () => [],
  loading: false,
  stripe: false,
  border: true,
  size: 'default',
  defaultExpandAll: false,
  highlightCurrentRow: false,
  showSelection: false,
  showIndex: false,
  showToolbar: true,
  actions: () => [],
  actionLabel: '操作',
  actionFixed: 'right',
  actionWidth: 200,
  pagination: false
})

const emit = defineEmits({
  'selection-change': (_selection: any[]) => true,
  'sort-change': (_sort: { column: any; prop: string; order: string }) => true,
  'filter-change': (_filters: any) => true,
  'current-change': (_row: any, _oldRow: any) => true,
  'row-click': (_row: any, _column: any, _event: MouseEvent) => true,
  'pagination-change': (_page: number, _size: number) => true,
  'refresh': () => true
})

const tableRef = ref<InstanceType<typeof ElTable>>()
const paginationCurrentPage = ref(1)
const paginationPageSize = ref(20)

const paginationTotal = computed(() => {
  if (typeof props.pagination === 'object') {
    return props.pagination.total
  }
  return props.data.length
})

const paginationPageSizes = computed(() => {
  if (typeof props.pagination === 'object' && props.pagination.pageSizes) {
    return props.pagination.pageSizes
  }
  return [10, 20, 50, 100]
})

const paginationLayout = computed(() => {
  return 'total, sizes, prev, pager, next, jumper'
})

const paginationBackground = computed(() => {
  return true
})

watch(() => props.pagination, (newVal) => {
  if (typeof newVal === 'object') {
    paginationCurrentPage.value = newVal.currentPage
    paginationPageSize.value = newVal.pageSize
  }
}, { immediate: true })

const handleSelectionChange = (selection: any[]) => {
  emit('selection-change', selection)
}

const handleSortChange = (sort: any) => {
  emit('sort-change', sort)
}

const handleFilterChange = (filters: any) => {
  emit('filter-change', filters)
}

const handleCurrentChange = (currentRow: any, previousRow: any) => {
  emit('current-change', currentRow, previousRow)
}

const handleRowClick = (row: any, column: any, event: MouseEvent) => {
  emit('row-click', row, column, event)
}

const handleCurrentPageChange = (page: number) => {
  if (typeof props.pagination === 'object') {
    paginationCurrentPage.value = page
    emit('pagination-change', page, paginationPageSize.value)
  }
}

const handlePageSizeChange = (size: number) => {
  if (typeof props.pagination === 'object') {
    paginationPageSize.value = size
    emit('pagination-change', paginationCurrentPage.value, size)
  }
}

const formatDateTime = (value: any): string => {
  return formatTime(value)
}

const formatAmount = (value: number | string, decimals: number = 4): string => {
  const num = Number(value) / Math.pow(10, 18)
  return num.toFixed(decimals)
}

const formatHash = (hash: string, length: number = 8): string => {
  if (!hash || typeof hash !== 'string') return ''
  if (hash.length <= length * 2) return hash
  return `${hash.slice(0, length)}...${hash.slice(-length)}`
}

defineExpose({
  tableRef,
  clearSelection: () => tableRef.value?.clearSelection(),
  toggleRowSelection: (row: any, selected?: boolean) =>
    tableRef.value?.toggleRowSelection(row, selected),
  toggleAllSelection: () => tableRef.value?.toggleAllSelection(),
  sort: (prop: string, order: 'ascending' | 'descending') =>
    tableRef.value?.sort(prop, order),
  clearSort: () => tableRef.value?.clearSort(),
  clearFilter: (columnKey?: string | number) =>
    tableRef.value?.clearFilter(columnKey)
})
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.base-table {
  .table-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: $space-md;
    padding: $space-sm $space-md;
    background: $card;
    border-radius: $radius-md;

    .toolbar-left,
    .toolbar-right {
      display: flex;
      align-items: center;
      gap: $space-sm;
    }
  }

  .table-pagination {
    margin-top: $space-lg;
    display: flex;
    justify-content: flex-end;
    padding: $space-md;
    background: $card;
    border-radius: $radius-md;
  }
}
</style>
