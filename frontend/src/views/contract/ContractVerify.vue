<template>
  <div class="contract-verify-page">
    <el-card>
      <template #header>
        <span>合约验证</span>
      </template>

      <el-form :model="form" label-width="120px">
        <el-form-item label="合约地址">
          <el-input v-model="form.address" placeholder="0x..." />
        </el-form-item>
        <el-form-item label="编译器版本">
          <el-select v-model="form.compilerVersion" placeholder="选择版本">
            <el-option label="v0.8.19+commit.7dd6d404" value="0.8.19" />
            <el-option label="v0.8.17+commit.8df81f82" value="0.8.17" />
            <el-option label="v0.8.7+commit.e28d00a7" value="0.8.7" />
          </el-select>
        </el-form-item>
        <el-form-item label="优化">
          <el-switch v-model="form.optimizer" />
        </el-form-item>
        <el-form-item label="源代码">
          <el-input v-model="form.sourceCode" type="textarea" rows="15" placeholder="粘贴合约源代码..." />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" :loading="verifying" @click="verifyContract">
            验证合约
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
  address: '',
  compilerVersion: '',
  optimizer: true,
  sourceCode: ''
})

const verifying = ref(false)

const verifyContract = async () => {
  verifying.value = true
  try {
    // TODO: API call
    ElMessage.success('合约验证成功')
    form.address = ''
    form.sourceCode = ''
  } catch (error) {
    ElMessage.error('验证失败')
  } finally {
    verifying.value = false
  }
}
</script>
