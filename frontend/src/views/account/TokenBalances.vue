<template>
  <div class="token-balances">
    <el-table :data="tokenBalances" style="width: 100%" v-loading="loading">
      <el-table-column prop="symbol" label="代币" width="120" />
      <el-table-column prop="contract" label="合约地址" min-width="200">
        <template #default="{ row }">
          <el-tooltip :content="row.contract" placement="top">
            <span class="address-text">{{ formatAddress(row.contract) }}</span>
          </el-tooltip>
        </template>
      </el-table-column>
      <el-table-column prop="balance" label="余额" width="180">
        <template #default="{ row }">
          <span class="balance-text">{{ formatBalance(row.balance, row.decimals) }}</span>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="120" fixed="right">
        <template #default="{ row }">
          <el-button
            size="small"
            type="primary"
            link
            @click="viewContract(row.contract)"
          >
            查看合约
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-empty
      v-if="tokenBalances.length === 0 && !loading"
      description="暂无代币余额"
      :image-size="100"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAccountStore } from '@/stores/modules/account.store'
import { accountApi } from '@/api/modules'
import { formatAddress } from '@/utils/format'

const props = defineProps<{
  address: string
}>()

const router = useRouter()
const accountStore = useAccountStore()

const loading = ref(false)
const tokenBalances = ref<
  Array<{
    contract: string
    symbol: string
    balance: string
    decimals: number
  }>
>([])

const fetchTokenBalances = async () => {
  try {
    loading.value = true

    // 获取原生代币余额
    const balanceResponse = await accountApi.getBalance(props.address)
    const nativeBalance = {
      contract: '0x0000000000000000000000000000000000000000',
      symbol: 'NRCS',
      balance: balanceResponse.data.balance,
      decimals: 18
    }

    tokenBalances.value = [nativeBalance]
  } catch (error: any) {
    console.error('Failed to fetch token balances:', error)
  } finally {
    loading.value = false
  }
}

const viewContract = (contract: string) => {
  router.push(`/contract/detail/${contract}`)
}

const formatAddress = (addr: string): string => {
  return formatAddress(addr, 8)
}

const formatBalance = (balance: string, decimals: number): string => {
  const wei = BigInt(balance)
  const divisor = BigInt(10) ** BigInt(decimals)
  const whole = wei / divisor
  const fraction = wei % divisor

  // 格式化为易读形式
  const fractionStr = fraction.toString().padStart(decimals, '0').replace(/0+$/, '')
  return `${whole.toLocaleString()}${fractionStr ? '.' + fractionStr : ''}`
}

onMounted(() => {
  fetchTokenBalances()
})
</script>

<style lang="scss" scoped>
.token-balances {
  .address-text {
    font-family: 'Roboto Mono', monospace;
    color: #666;
    cursor: pointer;

    &:hover {
      color: #409eff;
    }
  }

  .balance-text {
    font-family: 'Roboto Mono', monospace;
    font-weight: 500;
    color: #303133;
  }
}
</style>
