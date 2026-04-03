<template>
  <el-card>
    <template #header>
      <span>区块列表</span>
    </template>
    <el-table :data="blocks">
      <el-table-column prop="number" label="区块高度" width="120" />
      <el-table-column prop="hash" label="区块哈希" show-overflow-tooltip />
      <el-table-column prop="miner" label="矿工" width="150" />
      <el-table-column prop="transactionCount" label="交易数" width="80" />
      <el-table-column prop="timestamp" label="时间">
        <template #default="{ row }">
          {{ new Date(row.timestamp * 1000).toLocaleString() }}
        </template>
      </el-table-column>
    </el-table>
  </el-card>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const blocks = ref<any[]>([])
</script>
