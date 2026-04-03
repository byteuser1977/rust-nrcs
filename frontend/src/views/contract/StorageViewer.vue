<template>
  <div class="storage-viewer">
    <div class="toolbar">
      <el-input
        v-model="slotInput"
        placeholder="存储槽位（十进制或十六进制）"
        style="width: 240px; margin-right: 12px;"
        @keyup.enter="readSlot"
      >
        <template #prepend>Slot</template>
      </el-input>
      <el-button type="primary" @click="readSlot" :loading="reading">
        读取
      </el-button>
      <el-button @click="slotInput = ''">清空</el-button>
    </div>

    <el-card shadow="never" style="margin-top: 16px;">
      <template #header>
        <span>存储值</span>
        <el-button
          v-if="storageValue"
          size="small"
          type="primary"
          link
          @click="copyValue"
          style="float: right;"
        >
          <el-icon><CopyDocument /></el-icon>
          复制
        </el-button>
      </template>

      <div v-if="reading" class="reading-status">
        <el-icon class="is-loading"><Loading /></el-icon>
        读取中...
      </div>

      <div v-else-if="storageValue" class="storage-value">
        <div class="value-row">
          <span class="label">原始值:</span>
          <span class="raw-value">{{ storageValue }}</span>
        </div>
        <div class="value-row" v-if="decodedValue">
          <span class="label">解码后:</span>
          <pre class="decoded-value">{{ JSON.stringify(decodedValue, null, 2) }}</pre>
        </div>
      </div>

      <el-empty v-else description="请输入存储槽位并点击读取" :image-size="100" />
    </el-card>

    <!-- 常用存储槽位快速访问 -->
    <el-card shadow="never" style="margin-top: 16px;">
      <template #header>
        <span>常用槽位</span>
      </template>

      <div class="quick-slots">
        <el-tag
          v-for="slot in commonSlots"
          :key="slot.value"
          closable
          @close="removeCommonSlot(slot.value)"
          @click="slotInput = slot.value; readSlot()"
          style="margin: 4px; cursor: pointer;"
        >
          {{ slot.label }}
        </el-tag>

        <el-button
          v-if="commonSlots.length < 5"
          size="small"
          text
          type="primary"
          @click="addCurrentSlot"
        >
          <el-icon><Plus /></el-icon>
          添加当前槽位
        </el-button>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { CopyDocument, Loading, Plus } from '@element-plus/icons-vue'
import { contractApi } from '@/api/modules'

const props = defineProps<{
  contractAddress: string
}>()

const slotInput = ref('')
const reading = ref(false)
const storageValue = ref<string>('')
const decodedValue = ref<any>(null)

const commonSlots = ref<Array<{ label: string; value: string }>>([
  { label: '账户余额(0)', value: '0' },
  { label: '所有者(1)', value: '1' }
])

const readSlot = async () => {
  if (!slotInput.value.trim()) {
    ElMessage.warning('请输入存储槽位')
    return
  }

  try {
    reading.value = true
    storageValue.value = ''
    decodedValue.value = null

    // 解析槽位（支持十进制或十六进制）
    let slot: number
    if (slotInput.value.startsWith('0x')) {
      slot = parseInt(slotInput.value.slice(2), 16)
    } else {
      slot = parseInt(slotInput.value, 10)
    }

    if (isNaN(slot)) {
      ElMessage.error('无效的槽位格式')
      return
    }

    const response = await contractApi.getStorageAt(props.contractAddress, slot)
    storageValue.value = response.data.value

    // 尝试解码（如果是已知的槽位）
    tryDecode(slot, storageValue.value)
  } catch (error: any) {
    console.error('Failed to read storage:', error)
    ElMessage.error('读取存储失败：' + (error.message || '未知错误'))
  } finally {
    reading.value = false
  }
}

const tryDecode = (slot: number, value: string) => {
  // 根据槽位尝试解码
  // 这里只是示例，实际需要根据合约ABI来解码

  // 如果是余额槽位（常见的ERC20余额映射）
  if (slot % 1000 === 0) {
    try {
      const bigIntValue = BigInt(value)
      const wei = bigIntValue.toString()
      decodedValue.value = {
        type: 'uint256',
        value: wei,
        formatted: (BigInt(wei) / BigInt(10**18)).toString()
      }
    } catch {
      decodedValue.value = null
    }
  }
}

const copyValue = async () => {
  try {
    await navigator.clipboard.writeText(storageValue.value)
    ElMessage.success('已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

const addCurrentSlot = () => {
  if (!slotInput.value.trim()) return

  const exists = commonSlots.value.some(s => s.value === slotInput.value)
  if (exists) {
    ElMessage.warning('该槽位已存在')
    return
  }

  commonSlots.value.push({
    label: `槽位 ${slotInput.value}`,
    value: slotInput.value
  })
  ElMessage.success('已添加到常用槽位')
}

const removeCommonSlot = (value: string) => {
  const index = commonSlots.value.findIndex(s => s.value === value)
  if (index !== -1) {
    commonSlots.value.splice(index, 1)
  }
}
</script>

<style scoped lang="scss">
.storage-viewer {
  .toolbar {
    display: flex;
    align-items: center;
  }

  .reading-status {
    text-align: center;
    padding: 40px;
    color: #909399;

    .el-icon {
      font-size: 24px;
      margin-right: 8px;
      vertical-align: middle;
    }
  }

  .storage-value {
    .value-row {
      margin-bottom: 16px;

      &:last-child {
        margin-bottom: 0;
      }

      .label {
        display: block;
        font-size: 13px;
        color: #909399;
        margin-bottom: 4px;
      }

      .raw-value {
        display: block;
        font-family: 'Roboto Mono', monospace;
        font-size: 14px;
        color: #303133;
        word-break: break-all;
        padding: 8px;
        background: #f5f7fa;
        border-radius: 4px;
      }

      .decoded-value {
        margin: 8px 0 0 0;
        padding: 12px;
        background: #f0f9eb;
        border-radius: 4px;
        font-size: 13px;
        overflow-x: auto;
      }
    }
  }

  .quick-slots {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;

    .el-tag {
      cursor: pointer;
      margin: 2px;
    }

    .el-button {
      margin: 2px;
    }
  }
}
</style>
