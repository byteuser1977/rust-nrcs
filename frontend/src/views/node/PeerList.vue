<template>
  <el-card>
    <template #header>
      <span>Peer 节点列表</span>
    </template>
    <el-table :data="peers">
      <el-table-column prop="id" label="节点ID" width="120" />
      <el-table-column prop="enode" label="ENODE URL" show-overflow-tooltip />
      <el-table-column prop="address" label="地址" width="150" />
      <el-table-column prop="connected" label="状态" width="80">
        <template #default="{ row }">
          <el-tag :type="row.connected ? 'success' : 'danger'" size="small">
            {{ row.connected ? '已连接' : '断开' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="latency" label="延迟(ms)" width="100" />
    </el-table>
  </el-card>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const peers = ref<any[]>([])
</script>
