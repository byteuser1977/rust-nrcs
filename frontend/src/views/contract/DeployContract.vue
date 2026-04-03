<template>
  <div class="deploy-contract-page">
    <el-card>
      <template #header>
        <span>部署合约</span>
      </template>

      <el-form :model="form" label-width="120px">
        <el-form-item label="合约字节码">
          <el-input v-model="form.bytecode" type="textarea" rows="4" placeholder="0x..." />
        </el-form-item>
        <el-form-item label="构造函数参数">
          <el-input v-model="form.constructorArgs" type="textarea" rows="2" placeholder="JSON 格式" />
        </el-form-item>
        <el-form-item label="Gas Limit">
          <el-input-number v-model="form.gasLimit" :min="21000" />
        </el-form-item>
        <el-form-item label="Gas Price">
          <el-input-number v-model="form.gasPrice" :min="1" />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" :loading="deploying" @click="deployContract">
            部署合约
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import { ElMessage } from 'element-plus'

const form = reactive({
  bytecode: '',
  constructorArgs: '',
  gasLimit: 3000000,
  gasPrice: 20
})

const deploying = ref(false)

const deployContract = async () => {
  deploying.value = true
  try {
    // TODO: API call
    ElMessage.success('合约部署成功')
    form.bytecode = ''
    form.constructorArgs = ''
  } catch (error) {
    ElMessage.error('部署失败')
  } finally {
    deploying.value = false
  }
}
</script>
