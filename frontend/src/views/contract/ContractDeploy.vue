<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { ElCard, ElForm, ElFormItem, ElInput, ElButton, ElUpload, ElMessage } from 'element-plus'
import { ref } from 'vue'
import { useContractStore } from '@/stores/modules/contract.store'
import { useAccountStore } from '@/stores/modules/account.store'
import { useRouter } from 'vue-router'

const { t } = useI18n()
const contractStore = useContractStore()
const accountStore = useAccountStore()
const router = useRouter()

const formRef = ref()
const contractName = ref('')
const bytecode = ref('')
const abi = ref('')
const fileList = ref([])

async function handleDeploy() {
  if (!accountStore.walletAddress) {
    ElMessage.error('请先连接钱包')
    router.push('/wallet/connect')
    return
  }

  try {
    await formRef.value.validate()
    // 实际部署逻辑（调用 store）
    ElMessage.success(t('contract.deploySuccess'))
    router.push('/contract/list')
  } catch (error) {
    console.error('Deploy failed', error)
  }
}
</script>

<template>
  <div class="contract-deploy">
    <h1 class="page-title">{{ t('contract.deploy') }}</h1>

    <el-card shadow="never">
      <el-form ref="formRef" :model="{ contractName, bytecode, abi }" label-width="120px">
        <el-form-item :label="t('contract.name')" prop="contractName" required>
          <el-input v-model="contractName" placeholder="Contract Name" />
        </el-form-item>

        <el-form-item :label="t('contract.bytecode')" prop="bytecode" required>
          <el-input v-model="bytecode" type="textarea" :rows="6" placeholder="0x..." />
        </el-form-item>

        <el-form-item :label="t('contract.abi')" prop="abi" required>
          <el-input v-model="abi" type="textarea" :rows="10" placeholder='[{"inputs":[],"name":"...","type":"function"}]' />
        </el-form-item>

        <el-form-item>
          <el-button type="primary" @click="handleDeploy">
            {{ t('contract.deploy') }}
          </el-button>
          <el-button @click="router.push('/contract/list')">{{ t('common.cancel') }}</el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<style lang="scss" scoped>
.contract-deploy {
  .page-title {
    margin-bottom: 24px;
    font-size: 24px;
    font-weight: 600;
  }
}
</style>
