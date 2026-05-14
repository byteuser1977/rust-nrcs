<template>
  <el-dialog v-model="visible" :title="t('voting.castVote')" width="520px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <div v-if="poll" class="poll-summary mb-4">
      <div class="summary-row"><span class="label">{{ t('voting.name') }}</span><span class="value">{{ poll.name }}</span></div>
      <div class="summary-row"><span class="label">{{ t('voting.description') }}</span><span class="value">{{ poll.description }}</span></div>
      <div class="summary-row"><span class="label">{{ t('voting.range') }}</span><span class="value">{{ poll.minRangeValue }} - {{ poll.maxRangeValue }}</span></div>
    </div>
    <el-form ref="formRef" :model="form" label-position="top">
      <el-form-item v-for="(opt, idx) in poll?.options || []" :key="idx" :label="opt">
        <el-input-number v-model="form.votes[idx]" :min="0" :max="poll?.maxRangeValue || 1" style="width:100%" />
      </el-form-item>
      <el-row :gutter="16">
        <el-col :span="12"><el-form-item :label="t('common.fee')"><el-input v-model="form.feeNQT" placeholder="1" type="number" clearable><template #append>NRC</template></el-input></el-form-item></el-col>
        <el-col :span="12"><el-form-item :label="t('common.deadline')"><el-input v-model="form.deadline" placeholder="1440" type="number" clearable><template #append>{{ t('common.minutes') }}</template></el-input></el-form-item></el-col>
      </el-row>
      <el-form-item :label="t('common.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" show-password clearable />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('voting.castVote') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const props = defineProps<{ poll: any }>()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()
const formRef = ref<FormInstance>()
const loading = ref(false)
const form = reactive<{ votes: Record<number, number>; feeNQT: string; deadline: string; secretPhrase: string }>({ votes: {}, feeNQT: '1', deadline: '1440', secretPhrase: '' })

watch(() => props.poll, (p) => {
  if (p?.options) {
    const v: Record<number, number> = {}
    p.options.forEach((_: any, i: number) => { v[i] = 0 })
    form.votes = v
  }
}, { immediate: true })

const rules = computed<FormRules>(() => ({
  secretPhrase: [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
}))

async function handleSubmit() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      const voteBytes: number[] = []
      for (let i = 0; i < (props.poll?.options?.length || 0); i++) {
        voteBytes.push(form.votes[i] || 0)
      }
      await nrcsApi.castVote({
        secretPhrase: form.secretPhrase, poll: props.poll.poll,
        vote: voteBytes,
        feeNQT: String(Math.round(Number(form.feeNQT) * 1e8)), deadline: Number(form.deadline)
      })
      ElMessage.success(t('voting.voteSuccess'))
      emit('success'); handleClose()
    } catch (err: any) { ElMessage.error(err?.message || t('voting.voteError')) }
    finally { loading.value = false }
  })
}
function handleClose() { formRef.value?.resetFields(); visible.value = false }
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.poll-summary { background: rgba(255,255,255,0.02); border: 1px solid $border-subtle; border-radius: $radius-md; padding: $space-lg; }
.summary-row { display: flex; justify-content: space-between; padding: $space-xs 0; .label { color: $text-muted; font-size: $font-size-sm; } .value { color: $text-primary; font-weight: 500; } }
.mb-4 { margin-bottom: $space-lg; }
</style>
