<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title">
        <el-icon><UserFilled /></el-icon>
        {{ t('contacts.title') }}
      </h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="openAddDialog">
          <el-icon><Plus /></el-icon>
          {{ t('contacts.addContact') }}
        </el-button>
        <el-button size="small" @click="exportContacts">
          <el-icon><Download /></el-icon>
          {{ t('contacts.export') }}
        </el-button>
        <el-button size="small" @click="triggerImport">
          <el-icon><Upload /></el-icon>
          {{ t('contacts.import') }}
        </el-button>
        <input
          ref="fileInput"
          type="file"
          accept=".json"
          style="display: none"
          @change="handleImport"
        />
      </div>
    </div>

    <el-card shadow="hover" class="search-card">
      <el-input
        v-model="searchQuery"
        :placeholder="t('contacts.searchPlaceholder')"
        clearable
        style="max-width: 400px"
        prefix-icon="Search"
      />
    </el-card>

    <el-card shadow="hover" class="contacts-card" v-loading="loading">
      <el-table
        :data="filteredContacts"
        stripe
        style="width: 100%"
        :empty-text="t('contacts.noContacts')"
        row-key="account"
      >
        <el-table-column :label="t('common.name')" min-width="180">
          <template #default="{ row }">
            <span class="contact-name cursor-pointer" @click="openEditDialog(row)">{{ row.name || row.accountRS }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('contacts.accountRS')" width="220" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="text-mono text-accent cursor-pointer" @click="showAccountDetail(row)">{{ row.accountRS }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('contacts.email')" width="220">
          <template #default="{ row }">
            <span class="text-sm">{{ row.email || '-' }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('contacts.description')" min-width="180" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="text-sm text-muted">{{ row.description || '-' }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="260" fixed="right">
          <template #default="{ row }">
            <div class="action-btns">
              <el-button size="small" text type="primary" @click="openEditDialog(row)">
                <el-icon><Edit /></el-icon>
                {{ t('common.edit') }}
              </el-button>
              <el-button size="small" text type="success" @click="sendMoney(row)">
                <el-icon><Money /></el-icon>
                {{ t('common.send') }}
              </el-button>
              <el-button size="small" text type="info" @click="sendMessage(row)">
                <el-icon><ChatLineRound /></el-icon>
                {{ t('contacts.message') }}
              </el-button>
              <el-popconfirm
                :title="t('contacts.removeConfirm')"
                @confirm="removeContact(row)"
              >
                <template #reference>
                  <el-button size="small" text type="danger">
                    <el-icon><Delete /></el-icon>
                    {{ t('common.delete') }}
                  </el-button>
                </template>
              </el-popconfirm>
            </div>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- Add/Edit Dialog（对标 nrs.contacts.js addContact/updateContact） -->
    <el-dialog
      v-model="dialogVisible"
      :title="isEditing ? t('contacts.editContact') : t('contacts.addContact')"
      width="500px"
      destroy-on-close
      @close="resetForm"
    >
      <el-form
        ref="formRef"
        :model="form"
        :rules="formRules"
        label-width="100px"
        label-position="top"
      >
        <el-form-item :label="t('common.name')" prop="name">
          <el-input v-model="form.name" :placeholder="t('contacts.namePlaceholder')" clearable />
        </el-form-item>
        <el-form-item :label="t('contacts.accountRS')" prop="accountRS">
          <el-input
            v-model="form.accountRS"
            :placeholder="t('contacts.accountRSPlaceholder')"
            clearable
            :disabled="isEditing"
          >
            <template #append>
              <el-button :loading="validatingAccount" @click="validateAccount">
                {{ t('contacts.validate') }}
              </el-button>
            </template>
          </el-input>
          <span v-if="validatedAccountRS" class="validation-success">
            <el-icon><CircleCheck /></el-icon>
            {{ validatedAccountRS }}
          </span>
        </el-form-item>
        <el-form-item :label="t('contacts.email')" prop="email">
          <el-input v-model="form.email" :placeholder="t('contacts.emailPlaceholder')" clearable type="email" />
        </el-form-item>
        <el-form-item :label="t('contacts.description')" prop="description">
          <el-input
            v-model="form.description"
            :placeholder="t('contacts.descriptionPlaceholder')"
            type="textarea"
            :rows="3"
            clearable
          />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="dialogVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="saving" @click="saveContact">
          {{ t('common.confirm') }}
        </el-button>
      </template>
    </el-dialog>

    <SendMoneyModal
      v-model:visible="showSend"
      :recipient="selectedContact?.accountRS"
      @success="onMoneySent"
    />

    <SendMessageModal
      v-model:visible="showMessage"
      :recipient="selectedContact?.accountRS"
      @success="onMessageSent"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import {
  UserFilled, Plus, Download, Upload, Edit,
  Money, Delete, Search, ChatLineRound, CircleCheck,
} from '@element-plus/icons-vue'
import SendMoneyModal from '@/components/modals/SendMoneyModal.vue'
import SendMessageModal from '@/components/modals/SendMessageModal.vue'
import { storageSelect, storageInsert, storageUpdate, storageDelete } from '@/utils/nrcs-storage'
import { nrcsApi } from '@/api/modules'

const { t } = useI18n()

/**
 * 联系人数据结构（对标 nrs.contacts.js storageInsert("contacts", "name", {...})）。
 *
 * NRCS 联系人基于 IndexedDB 存储（账户级隔离），同时保存 account（数字 ID）
 * 和 accountRS（RS 地址），以及 name/email/description。
 */
interface Contact {
  /** 联系人名称（主键，对标 storageInsert 的 keyPath） */
  name: string
  /** 账户数字 ID（对标 nrs.contacts.js data.account） */
  account: string
  /** 账户 RS 地址（对标 nrs.contacts.js data.accountRS） */
  accountRS: string
  /** 邮箱（可选） */
  email: string
  /** 描述（可选） */
  description: string
}

const contacts = ref<Contact[]>([])
const searchQuery = ref('')
const loading = ref(false)
const dialogVisible = ref(false)
const isEditing = ref(false)
const editingName = ref('')
const saving = ref(false)
const validatingAccount = ref(false)
const validatedAccountRS = ref('')
const validatedAccountId = ref('')
const showSend = ref(false)
const showMessage = ref(false)
const selectedContact = ref<Contact | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)
const formRef = ref<FormInstance>()

const form = ref<Contact>({
  name: '',
  account: '',
  accountRS: '',
  email: '',
  description: '',
})

const formRules = computed<FormRules>(() => ({
  name: [
    { required: true, message: t('contacts.nameRequired'), trigger: 'blur' },
    {
      // 对标 nrs.contacts.js:90 名称不能为纯数字或 NRCS- 开头
      validator: (_rule: any, value: string, callback: any) => {
        if (!value) return callback()
        if (/^\d+$/.test(value) || /^NRCS-/i.test(value)) {
          return callback(new Error(t('contacts.nameAlphaError')))
        }
        callback()
      },
      trigger: 'blur',
    },
  ],
  accountRS: [
    { required: true, message: t('contacts.accountRSRequired'), trigger: 'blur' },
  ],
  email: [
    {
      // 对标 nrs.contacts.js:96 邮箱格式校验
      validator: (_rule: any, value: string, callback: any) => {
        if (!value) return callback()
        if (!/@/.test(value)) {
          return callback(new Error(t('contacts.emailFormatError')))
        }
        callback()
      },
      trigger: 'blur',
    },
  ],
}))

const filteredContacts = computed(() => {
  if (!searchQuery.value.trim()) return contacts.value
  const q = searchQuery.value.toLowerCase()
  return contacts.value.filter((c) =>
    (c.name || '').toLowerCase().includes(q) ||
    (c.accountRS || '').toLowerCase().includes(q) ||
    (c.email || '').toLowerCase().includes(q),
  )
})

onMounted(() => {
  loadContacts()
})

/**
 * 从 IndexedDB 加载所有联系人（对标 nrs.contacts.js:21 NRS.loadContacts）。
 *
 * 使用 storageSelect("contacts", null) 查询全部，按 name 排序。
 */
async function loadContacts(): Promise<void> {
  loading.value = true
  try {
    const result = await storageSelect<Contact>('contacts')
    if (result && result.length) {
      // 对标 nrs.contacts.js:44-52 按 name 排序
      result.sort((a, b) => {
        const an = (a.name || '').toLowerCase()
        const bn = (b.name || '').toLowerCase()
        if (an > bn) return 1
        if (an < bn) return -1
        return 0
      })
      contacts.value = result
    } else {
      contacts.value = []
    }
  } catch (e) {
    console.warn('[contacts] 加载联系人失败:', e)
    contacts.value = []
  } finally {
    loading.value = false
  }
}

function openAddDialog(): void {
  isEditing.value = false
  editingName.value = ''
  resetForm()
  dialogVisible.value = true
}

/**
 * 打开编辑对话框（对标 nrs.contacts.js:198 update_contact_modal show.bs.modal）。
 */
function openEditDialog(row: Contact): void {
  isEditing.value = true
  editingName.value = row.name
  form.value = { ...row }
  validatedAccountRS.value = row.accountRS
  validatedAccountId.value = row.account
  dialogVisible.value = true
}

function resetForm(): void {
  form.value = {
    name: '',
    account: '',
    accountRS: '',
    email: '',
    description: '',
  }
  validatedAccountRS.value = ''
  validatedAccountId.value = ''
  formRef.value?.resetFields()
}

/**
 * 校验账户有效性（对标 nrs.contacts.js:134-144 sendRequest("getAccount")）。
 *
 * 调用 getAccount API 验证账户存在性，同时获取 accountRS 和 account 数字 ID。
 */
async function validateAccount(): Promise<void> {
  const input = form.value.accountRS?.trim()
  if (!input) {
    ElMessage.warning(t('contacts.accountRSRequired'))
    return
  }

  validatingAccount.value = true
  try {
    const response = await nrcsApi.getAccount(input)
    if (response && response.accountRS) {
      // 对标 nrs.contacts.js:138-141 校验返回的 account/accountRS 一致性
      validatedAccountRS.value = response.accountRS
      validatedAccountId.value = response.account || ''
      ElMessage.success(t('contacts.validateSuccess'))
    } else {
      ElMessage.error(t('contacts.accountNotFound'))
    }
  } catch (e: any) {
    ElMessage.error(e?.message || t('contacts.accountNotFound'))
    validatedAccountRS.value = ''
    validatedAccountId.value = ''
  } finally {
    validatingAccount.value = false
  }
}

/**
 * 保存联系人（对标 nrs.contacts.js:77 addContact / :228 updateContact）。
 *
 * 添加流程：
 *   1. 校验 name/accountRS/email 格式
 *   2. 校验账户有效性（validateAccount）
 *   3. 查重（按 account 和 name，对标 nrs.contacts.js:148-152）
 *   4. storageInsert（keyPath = name）
 *
 * 编辑流程：
 *   1. 校验
 *   2. 查重（排除自身，对标 nrs.contacts.js:292-295）
 *   3. storageUpdate（按 name）
 */
async function saveContact(): Promise<void> {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return

    // 未校验账户时提示校验
    if (!validatedAccountRS.value && !isEditing.value) {
      ElMessage.warning(t('contacts.pleaseValidate'))
      return
    }

    saving.value = true
    try {
      const entry: Contact = {
        name: form.value.name.trim(),
        account: validatedAccountRS.value ? validatedAccountId.value : form.value.account,
        accountRS: validatedAccountRS.value || form.value.accountRS,
        email: form.value.email || '',
        description: form.value.description || '',
      }

      if (isEditing.value) {
        // 编辑流程（对标 nrs.contacts.js:228 updateContact）
        // 查重：排除自身（对标 nrs.contacts.js:292-295 contacts[0].id != contactId）
        const existing = await storageSelect<Contact>('contacts', [
          { account: entry.account },
        ])
        if (existing && existing.length && existing[0].name !== editingName.value) {
          ElMessage.error(t('contacts.errorContactExists'))
          saving.value = false
          return
        }

        // storageUpdate（对标 nrs.contacts.js:300-308）
        await storageUpdate('contacts', entry, [{ name: editingName.value }])
        ElMessage.success(t('contacts.editSuccess'))
      } else {
        // 添加流程（对标 nrs.contacts.js:77 addContact）
        // 查重：按 account 和 name（对标 nrs.contacts.js:148-152）
        const existing = await storageSelect<Contact>('contacts', [
          { account: entry.account },
          { name: entry.name },
        ])
        if (existing && existing.length) {
          if (existing[0].name === entry.name) {
            ElMessage.error(t('contacts.errorNameExists'))
          } else {
            ElMessage.error(t('contacts.errorAccountExists'))
          }
          saving.value = false
          return
        }

        // storageInsert（对标 nrs.contacts.js:162-168，keyPath = name）
        await storageInsert('contacts', 'name', entry)
        ElMessage.success(t('contacts.addSuccess'))
      }

      dialogVisible.value = false
      await loadContacts()
    } catch (e: any) {
      ElMessage.error(e?.message || t('common.operationFailed'))
    } finally {
      saving.value = false
    }
  })
}

/**
 * 删除联系人（对标 nrs.contacts.js:355 deleteContact）。
 *
 * 使用 storageDelete("contacts", [{ name }]) 按主键删除。
 */
async function removeContact(row: Contact): Promise<void> {
  try {
    await storageDelete('contacts', [{ name: row.name }])
    ElMessage.success(t('contacts.removeSuccess'))
    await loadContacts()
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.operationFailed'))
  }
}

function sendMoney(row: Contact): void {
  selectedContact.value = row
  showSend.value = true
}

function sendMessage(row: Contact): void {
  selectedContact.value = row
  showMessage.value = true
}

function showAccountDetail(row: Contact): void {
  // 跳转到账户详情或打开 modal（可扩展）
  selectedContact.value = row
}

function onMoneySent(): void {
  ElMessage.success(t('common.operationSuccess'))
}

function onMessageSent(): void {
  ElMessage.success(t('common.operationSuccess'))
}

/**
 * 导出联系人（对标 nrs.contacts.js:376 exportContacts）。
 *
 * NRCS 导出格式为对象：{ account: { name, email, account, accountRS, description } }
 */
function exportContacts(): void {
  if (contacts.value.length === 0) {
    ElMessage.warning(t('contacts.noContactsToExport'))
    return
  }
  // 对标 nrs.contacts.js:379 以 account 为 key 的对象格式
  const exportData: Record<string, Omit<Contact, 'account'>> = {}
  for (const c of contacts.value) {
    exportData[c.account] = {
      name: c.name,
      email: c.email,
      accountRS: c.accountRS,
      description: c.description,
    }
  }
  const json = JSON.stringify(exportData, null, 2)
  const blob = new Blob([json], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `contacts.json`
  a.click()
  URL.revokeObjectURL(url)
  ElMessage.success(t('contacts.exportSuccess'))
}

function triggerImport(): void {
  fileInput.value?.click()
}

/**
 * 导入联系人（对标 nrs.contacts.js:393 importContacts）。
 *
 * 支持 NRCS 格式（对象）和数组格式，逐个查重后插入。
 */
function handleImport(event: Event): void {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (!file) return

  const reader = new FileReader()
  reader.onload = async (e) => {
    try {
      const data = JSON.parse(e.target?.result as string)
      // 兼容 NRCS 对象格式和数组格式
      const items: Contact[] = []
      if (Array.isArray(data)) {
        for (const item of data) {
          items.push(normalizeContact(item))
        }
      } else if (typeof data === 'object') {
        // NRCS 格式：{ account: { name, email, accountRS, description } }
        for (const [account, info] of Object.entries(data)) {
          items.push(normalizeContact({ ...(info as any), account }))
        }
      } else {
        ElMessage.error(t('contacts.invalidFormat'))
        return
      }

      let successCount = 0
      let skipCount = 0
      for (const item of items) {
        if (!item.account || !item.name) {
          skipCount++
          continue
        }
        // 对标 nrs.contacts.js:395-398 查重
        const existing = await storageSelect<Contact>('contacts', [
          { account: item.account },
          { name: item.name },
        ])
        if (existing && existing.length) {
          skipCount++
          continue
        }
        // 对标 nrs.contacts.js:407 storageInsert
        await storageInsert('contacts', 'name', item)
        successCount++
      }

      await loadContacts()
      if (successCount > 0) {
        ElMessage.success(`${successCount} ${t('contacts.importSuccess')}`)
      }
      if (skipCount > 0) {
        ElMessage.warning(`${skipCount} ${t('contacts.importSkipped')}`)
      }
    } catch {
      ElMessage.error(t('contacts.invalidFormat'))
    }
  }
  reader.readAsText(file)

  if (fileInput.value) {
    fileInput.value.value = ''
  }
}

/**
 * 规范化联系人数据（处理缺失字段）。
 */
function normalizeContact(item: any): Contact {
  return {
    name: String(item.name || ''),
    account: String(item.account || ''),
    accountRS: String(item.accountRS || ''),
    email: String(item.email || ''),
    description: String(item.description || ''),
  }
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.page-container {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: $space-lg;
    flex-wrap: wrap;
    gap: $space-sm;
    .page-title {
      font-size: $font-size-lg;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: $space-sm;
      margin: 0;
      color: $text-primary;
    }
    .header-actions {
      display: flex;
      gap: $space-sm;
      flex-wrap: wrap;
    }
  }
}

.search-card {
  margin-bottom: $space-lg;
  border: 1px solid $border-subtle !important;
  border-radius: $radius-lg !important;
  background: rgba($surface-800, 0.6) !important;

  :deep(.el-card__body) {
    padding: $space-md $space-lg !important;
  }
}

.contacts-card {
  border: 1px solid $border-subtle !important;
  border-radius: $radius-lg !important;
  background: linear-gradient(180deg, rgba($surface-800, 0.8), rgba($surface-900, 0.9)) !important;

  :deep(.el-table) {
    background: transparent !important;
    --el-table-bg-color: transparent;
    --el-table-tr-bg-color: transparent;
    --el-table-header-bg-color: rgba(255, 255, 255, 0.02);
    --el-table-row-hover-bg-color: rgba($primary, 0.06);
    --el-table-border-color: $border-subtle;
    --el-table-text-color: $text-primary;
    --el-table-header-text-color: $text-muted;

    th.el-table__cell {
      background: rgba(255, 255, 255, 0.025) !important;
      font-weight: 600;
      font-size: $font-size-xs;
      text-transform: uppercase;
      border-bottom: 1px solid $border-subtle;
    }

    td.el-table__cell {
      border-bottom: 1px solid rgba($border-default, 0.5);
    }
  }
}

.contact-name {
  font-weight: 500;
  color: $text-primary;
}

.cursor-pointer {
  cursor: pointer;
  &:hover {
    color: $primary;
  }
}

.action-btns {
  display: flex;
  flex-wrap: wrap;
  gap: 2px;
}

.text-mono {
  font-family: $font-mono;
  font-size: $font-size-sm;
}

.text-accent {
  color: $primary;
}

.text-sm {
  font-size: $font-size-sm;
}

.text-muted {
  color: $text-muted;
}

// ── 账户校验成功提示 ──
.validation-success {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  margin-top: 4px;
  font-size: $font-size-xs;
  color: $success;
}
</style>
