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

    <el-card shadow="hover" class="contacts-card">
      <el-table
        :data="filteredContacts"
        stripe
        style="width: 100%"
        :empty-text="t('contacts.noContacts')"
        row-key="accountRS"
      >
        <el-table-column :label="t('common.name')" min-width="180">
          <template #default="{ row }">
            <span class="contact-name">{{ row.name || row.accountRS }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('contacts.accountRS')" width="220" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="text-mono text-accent">{{ row.accountRS }}</span>
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
        <el-table-column :label="t('common.actions')" width="200" fixed="right">
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

    <!-- Add/Edit Dialog -->
    <el-dialog
      v-model="dialogVisible"
      :title="isEditing ? t('contacts.editContact') : t('contacts.addContact')"
      width="480px"
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
          <el-input v-model="form.accountRS" :placeholder="t('contacts.accountRSPlaceholder')" clearable />
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
      :recipient="selectedContact"
      @success="onMoneySent"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import {
  UserFilled, Plus, Download, Upload, Edit,
  Money, Delete, Search
} from '@element-plus/icons-vue'
import SendMoneyModal from '@/components/modals/SendMoneyModal.vue'

const { t } = useI18n()

const CONTACTS_STORAGE_KEY = 'nrcs_contacts'

interface Contact {
  name: string
  accountRS: string
  email: string
  description: string
}

const contacts = ref<Contact[]>([])
const searchQuery = ref('')
const dialogVisible = ref(false)
const isEditing = ref(false)
const editingIndex = ref(-1)
const saving = ref(false)
const showSend = ref(false)
const selectedContact = ref<Contact | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)
const formRef = ref<FormInstance>()

const form = ref<Contact>({
  name: '',
  accountRS: '',
  email: '',
  description: ''
})

const formRules = computed<FormRules>(() => ({
  accountRS: [
    { required: true, message: t('contacts.accountRSRequired'), trigger: 'blur' }
  ]
}))

const filteredContacts = computed(() => {
  if (!searchQuery.value.trim()) return contacts.value
  const q = searchQuery.value.toLowerCase()
  return contacts.value.filter((c) =>
    (c.name || '').toLowerCase().includes(q) ||
    (c.accountRS || '').toLowerCase().includes(q) ||
    (c.email || '').toLowerCase().includes(q)
  )
})

onMounted(() => {
  loadContacts()
})

function loadContacts() {
  try {
    const data = localStorage.getItem(CONTACTS_STORAGE_KEY)
    contacts.value = data ? JSON.parse(data) : []
  } catch {
    contacts.value = []
  }
}

function saveToStorage() {
  localStorage.setItem(CONTACTS_STORAGE_KEY, JSON.stringify(contacts.value))
}

function openAddDialog() {
  isEditing.value = false
  editingIndex.value = -1
  resetForm()
  dialogVisible.value = true
}

function openEditDialog(row: Contact) {
  const idx = contacts.value.findIndex((c) => c.accountRS === row.accountRS)
  if (idx === -1) return
  isEditing.value = true
  editingIndex.value = idx
  form.value = { ...contacts.value[idx] }
  dialogVisible.value = true
}

function resetForm() {
  form.value = {
    name: '',
    accountRS: '',
    email: '',
    description: ''
  }
  formRef.value?.resetFields()
}

async function saveContact() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return

    saving.value = true
    try {
      const entry: Contact = { ...form.value }

      if (isEditing.value && editingIndex.value >= 0) {
        contacts.value[editingIndex.value] = entry
        ElMessage.success(t('contacts.editSuccess'))
      } else {
        const exists = contacts.value.find((c) => c.accountRS === entry.accountRS)
        if (exists) {
          ElMessage.warning(t('contacts.alreadyExists'))
          saving.value = false
          return
        }
        contacts.value.push(entry)
        ElMessage.success(t('contacts.addSuccess'))
      }

      saveToStorage()
      dialogVisible.value = false
    } catch (e: any) {
      ElMessage.error(e?.message || t('common.operationFailed'))
    } finally {
      saving.value = false
    }
  })
}

function removeContact(row: Contact) {
  contacts.value = contacts.value.filter((c) => c.accountRS !== row.accountRS)
  saveToStorage()
  ElMessage.success(t('contacts.removeSuccess'))
}

function sendMoney(row: Contact) {
  selectedContact.value = row
  showSend.value = true
}

function onMoneySent() {
  ElMessage.success(t('common.operationSuccess'))
}

function exportContacts() {
  if (contacts.value.length === 0) {
    ElMessage.warning(t('contacts.noContactsToExport'))
    return
  }
  const json = JSON.stringify(contacts.value, null, 2)
  const blob = new Blob([json], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `nrcs-contacts-${new Date().toISOString().split('T')[0]}.json`
  a.click()
  URL.revokeObjectURL(url)
  ElMessage.success(t('contacts.exportSuccess'))
}

function triggerImport() {
  fileInput.value?.click()
}

function handleImport(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (!file) return

  const reader = new FileReader()
  reader.onload = (e) => {
    try {
      const data = JSON.parse(e.target?.result as string)
      if (!Array.isArray(data)) {
        ElMessage.error(t('contacts.invalidFormat'))
        return
      }
      let count = 0
      for (const item of data) {
        if (item.accountRS && !contacts.value.find((c) => c.accountRS === item.accountRS)) {
          contacts.value.push({
            name: item.name || '',
            accountRS: item.accountRS,
            email: item.email || '',
            description: item.description || ''
          })
          count++
        }
      }
      saveToStorage()
      ElMessage.success(`${count} ${t('contacts.importSuccess')}`)
    } catch {
      ElMessage.error(t('contacts.invalidFormat'))
    }
  }
  reader.readAsText(file)

  if (fileInput.value) {
    fileInput.value.value = ''
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
</style>
