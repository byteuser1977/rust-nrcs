import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Contract, ContractABI, DeployContractParams, CallContractParams } from '@/types/business'
import { contractApi } from '@/api/modules'
import type { ApiResponse, PageResult } from '@/types'
import { ElMessage } from 'element-plus'
import { useAccountStore } from './account.store'
import { formatAddress } from '@/utils/format'

export const useContractStore = defineStore('contract', () => {
  // 合约列表
  const contracts = ref<Contract[]>([])
  const totalContracts = ref(0)
  const currentPage = ref(1)
  const pageSize = ref(20)
  const isLoading = ref(false)

  // 当前选中的合约
  const selectedContract = ref<Contract | null>(null)

  // 存储的合约 ABI（钱包侧可能需要）
  const savedABIs = ref<Map<string, ContractABI[]>>(new Map())

  // 计算属性
  const contractCount = computed(() => contracts.value.length)
  const contractGroups = computed(() => {
    // 按创建者分组（示例）
    return contracts.value.reduce((acc, contract) => {
      const creator = contract.address.slice(0, 10) // 简化：用地址前10位模拟
      if (!acc[creator]) {
        acc[creator] = []
      }
      acc[creator].push(contract)
      return acc
    }, {} as Record<string, Contract[]>)
  })

  /**
   * 获取合约列表（分页）
   */
  async function fetchContracts(page: number = 1, size: number = 20): Promise<void> {
    try {
      isLoading.value = true
      currentPage.value = page
      pageSize.value = size

      const response: ApiResponse<PageResult<Contract>> = await contractApi.getContracts({
        page,
        size
      })

      const { list, total } = response.data
      contracts.value = list
      totalContracts.value = total
    } catch (error: any) {
      console.error('Failed to fetch contracts', error)
      ElMessage.error('获取合约列表失败')
      throw error
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 获取合约详情
   */
  async function getContract(address: string): Promise<Contract | null> {
    try {
      const response: ApiResponse<Contract> = await contractApi.getContract(address)
      selectedContract.value = response.data
      return response.data
    } catch (error: any) {
      console.error('Failed to fetch contract', error)
      ElMessage.error('获取合约详情失败')
      return null
    }
  }

  /**
   * 部署合约
   */
  async function deployContract(data: DeployContractParams): Promise<{ contract_address: string; tx_hash: string } | null> {
    try {
      const response: ApiResponse<{ contract_address: string; tx_hash: string }> =
        await contractApi.deploy(data)

      const { contract_address, tx_hash } = response.data

      // 添加到列表顶部
      const newContract: Contract = {
        address: contract_address,
        name: data.name,
        abi: data.abi,
        bytecode: data.bytecode,
        created_at: new Date().toISOString()
      }
      contracts.value.unshift(newContract)
      totalContracts.value += 1
      savedABIs.value.set(contract_address, data.abi)

      ElMessage.success('合约部署成功')
      return response.data
    } catch (error: any) {
      console.error('Failed to deploy contract', error)
      ElMessage.error(error.message || '合约部署失败')
      return null
    }
  }

  /**
   * 只读调用合约
   */
  async function callContract(data: CallContractParams): Promise<any> {
    try {
      const response: ApiResponse<any> = await contractApi.call(data)
      return response.data
    } catch (error: any) {
      console.error('Failed to call contract', error)
      ElMessage.error('合约调用失败')
      return null
    }
  }

  /**
   * 发送合约交易（写入操作）
   */
  async function sendContractTx(data: CallContractParams): Promise<{ tx_hash: string } | null> {
    try {
      const response: ApiResponse<{ tx_hash: string }> = await contractApi.send(data)
      ElMessage.success('交易已发送')
      return response.data
    } catch (error: any) {
      console.error('Failed to send contract transaction', error)
      ElMessage.error(error.message || '交易发送失败')
      return null
    }
  }

  /**
   * 获取合约事件日志
   */
  async function getContractEvents(
    address: string,
    params?: { from_block?: number; to_block?: number; event_name?: string }
  ): Promise<any[]> {
    try {
      const response: ApiResponse<any[]> = await contractApi.getEvents(address, params)
      return response.data
    } catch (error: any) {
      console.error('Failed to fetch contract events', error)
      ElMessage.error('获取事件日志失败')
      return []
    }
  }

  /**
   * 选择合约（用于详情页）
   */
  function selectContract(contract: Contract) {
    selectedContract.value = contract
    // 保存 ABI
    if (!savedABIs.value.has(contract.address)) {
      savedABIs.value.set(contract.address, contract.abi)
    }
  }

  /**
   * 清除选中的合约
   */
  function clearSelectedContract() {
    selectedContract.value = null
  }

  /**
   * 获取保存的 ABI
   */
  function getSavedABI(address: string): ContractABI[] | undefined {
    return savedABIs.value.get(address)
  }

  /**
   * 删除已保存的 ABI
   */
  function removeSavedABI(address: string) {
    savedABIs.value.delete(address)
  }

  /**
   * 清除所有缓存
   */
  function clearCache() {
    contracts.value = []
    totalContracts.value = 0
    currentPage.value = 1
    pageSize.value = 20
    selectedContract.value = null
    savedABIs.value.clear()
  }

  return {
    // state
    contracts,
    totalContracts,
    currentPage,
    pageSize,
    isLoading,
    selectedContract,
    savedABIs,
    // getters
    contractCount,
    contractGroups,
    // actions
    fetchContracts,
    getContract,
    deployContract,
    callContract,
    sendContractTx,
    getContractEvents,
    selectContract,
    clearSelectedContract,
    getSavedABI,
    removeSavedABI,
    clearCache
  }
})
