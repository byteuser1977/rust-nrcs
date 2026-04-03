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
      :loading="loading"
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
        label="序号"
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
              {{ formatAddress(row[column.prop], column.addressLength) }}
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
              v-for="action in actions"
              :key="action.prop"
              v-if="!action.hidden?.(row, $index)"
              :type="action.type"
              :size="action.size || 'small'"
              :disabled="action.disabled?.(row, $index)"
              @click="action.handler?.(row, $index)"
            >
              {{ action.label }}
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
import type { TableProps, TableColumn } from 'element-plus'
import BaseButton from './BaseButton.vue'
import StatusBadge from './StatusBadge.vue'
import { formatAddress, formatTime } from '@/utils/format'

// 表格列定义
export interface TableColumnConfig extends TableColumn {
  // 列类型
  type?: 'default' | 'status' | 'datetime' | 'address' | 'amount' | 'hash'
  // 状态映射（type=status时使用）
  statusMap?: Record<string, { type: 'success' | 'warning' | 'danger' | 'info'; text?: string }>
  // 地址截断长度（type=address时使用）
  addressLength?: number
  // 金额小数位数（type=amount时使用）
  decimals?: number
  // 哈希截断长度（type=hash时使用）
  hashLength?: number
  // 单元格格式化函数
  formatter?: (row: any, column: TableColumn, cellValue: any) => any
  // 是否显示溢出提示（默认 true）
  showOverflowTooltip?: boolean
  // 是否可排序
  sortable?: boolean | 'custom'
  // 固定位置
  fixed?: boolean | ('left' | 'right')
}

// 表格操作按钮配置
export interface TableAction {
  prop: string
  label: string
  type?: 'primary' | 'success' | 'warning' | 'danger' | 'info'
  size?: 'large' | 'default' | 'small'
  // 是否隐藏当前行按钮
  hidden?: (row: any, index: number) => boolean
  // 是否禁用
  disabled?: (row: any, index: number) => boolean
  // 点击处理
  handler: (row: any, index: number) => void
}

// 分页配置
export interface PaginationConfig {
  currentPage: number
  pageSize: number
  total: number
  pageSizes?: number[]
  layout?: string
  background?: boolean
}

interface Props {
  // 表格数据
  data: any[]
  // 表格列配置
  columns: TableColumnConfig[]
  // 表格加载状态
  loading?: boolean
  // 斑马纹
  stripe?: boolean
  // 边框
  border?: boolean
  // 尺寸
  size?: TableProps['size']
  // 最大高度
  maxHeight?: number | string
  // 固定高度
  height?: number | string
  // 行key
  rowKey?: string | ((row: any) => string)
  // 默认展开所有行（树形表格）
  defaultExpandAll?: boolean
  // 树形表格的子节点字段
  treeProps?: Record<string, string>
  // 是否高亮当前行
  highlightCurrentRow?: boolean
  // 当前行key
  currentRowKey?: any
  // 是否显示选择框
  showSelection?: boolean
  // 是否显示序号列
  showIndex?: boolean
  // 是否显示工具栏
  showToolbar?: boolean
  // 操作列配置
  actions?: TableAction[]
  // 操作列标题
  actionLabel?: string
  // 操作列固定位置
  actionFixed?: boolean | 'left' | 'right'
  // 操作列宽度
  actionWidth?: number | string
  // 分页配置
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

const emit = defineEmits<{
  // 选择变化
  'selection-change': [selection: any[]]
  // 排序变化
  'sort-change': [sort: { column: any; prop: string; order: string }]
  // 筛选变化
  'filter-change': [filters: any]
  // 当前行变化
  'current-change': [row: any, oldRow: any]
  // 行点击
  'row-click': [row: any, column: any, event: MouseEvent]
  // 分页
  'pagination-change': [page: number; size: number]
  // 刷新
  refresh: []
}>()

const tableRef = ref()
const paginationCurrentPage = ref(1)
const paginationPageSize = ref(20)

// 分页配置计算
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

// 监听分页变化
watch(() => props.pagination, (newVal) => {
  if (typeof newVal === 'object') {
    paginationCurrentPage.value = newVal.currentPage
    paginationPageSize.value = newVal.pageSize
  }
}, { immediate: true })

// 事件处理
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

// 格式化辅助方法
const formatDateTime = (value: any): string => {
  return formatTime(value)
}

const formatAddress = (address: string, length: number = 8): string => {
  return formatAddress(address, length)
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

// 暴露表格实例方法
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
.base-table {
  .table-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
    padding: 8px 12px;
    background: #fff;
    border-radius: 4px;

    .toolbar-left,
    .toolbar-right {
      display: flex;
      align-items: center;
      gap: 8px;
    }
  }

  .table-pagination {
    margin-top: 16px;
    display: flex;
    justify-content: flex-end;
    padding: 12px;
    background: #fff;
    border-radius: 4px;
  }
}
</style>
