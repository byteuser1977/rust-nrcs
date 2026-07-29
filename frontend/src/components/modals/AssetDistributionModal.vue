<template>
  <el-dialog v-model="visible" :title="t('asset.distribution')" width="650px" destroy-on-close>
    <el-table :data="holders" stripe size="small" v-loading="loading" :empty-text="t('common.noData')">
      <el-table-column prop="accountRS" :label="t('common.account')" min-width="200" show-overflow-tooltip />
      <el-table-column :label="t('asset.quantity')" width="150" align="right">
        <template #default="{ row }">{{ formatQty(row.quantityQNT) }}</template>
      </el-table-column>
      <el-table-column :label="t('asset.percentage')" width="120" align="right">
        <template #default="{ row }">
          <el-progress :percentage="Number(row.percentage || 0)" :stroke-width="6" />
        </template>
      </el-table-column>
    </el-table>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { qntToQntf } from '@/utils/format'

const { t } = useI18n()

const props = defineProps<{ asset: any }>()
const visible = defineModel<boolean>('visible', { default: false })

const holders = ref<any[]>([])
const loading = ref(false)

function formatQty(qnt: string) {
  try { return qntToQntf(qnt, props.asset?.decimals || 0) } catch { return qnt }
}

watch(visible, async (val) => {
  if (val && props.asset?.asset) {
    loading.value = true
    try {
      const result = await nrcsApi.getAssetAccounts(props.asset.asset)
      const accounts = (result as any).accountAssets || []
      const totalQty = BigInt(props.asset.quantityQNT || '1')
      holders.value = accounts.map((a: any) => ({
        ...a,
        percentage: totalQty > 0n
          ? ((BigInt(a.quantityQNT || '0') * 10000n) / totalQty).toString()
          : '0',
      }))
    } catch (e) {
      console.error('Failed to load asset distribution:', e)
    } finally {
      loading.value = false
    }
  }
})
</script>
