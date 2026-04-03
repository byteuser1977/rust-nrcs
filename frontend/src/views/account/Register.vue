<template>
  <div class="register-page">
    <el-row justify="center">
      <el-col :xs="24" :sm="20" :md="16" :lg="12" :xl="10">
        <el-card class="register-card">
          <template #header>
            <h2>创建新账户</h2>
            <p class="subtitle">生成您的区块链身份</p>
          </template>

          <el-form
            ref="formRef"
            :model="form"
            :rules="rules"
            label-width="120px"
            label-position="top"
          >
            <!-- 用户信息 -->
            <el-form-item label="用户名" prop="name">
              <el-input
                v-model="form.name"
                placeholder="请输入用户名"
                :prefix-icon="User"
                clearable
              />
            </el-form-item>

            <el-form-item label="邮箱" prop="email">
              <el-input
                v-model="form.email"
                placeholder="请输入邮箱"
                :prefix-icon="Message"
                clearable
              />
            </el-form-item>

            <el-divider content-position="center">钱包信息</el-divider>

            <!-- 钱包地址 -->
            <el-form-item label="钱包地址" prop="wallet_address">
              <el-input
                v-model="form.wallet_address"
                placeholder="请输入您的钱包地址（0x...）"
                :prefix-icon="Wallet"
                clearable
              >
                <template #append>
                  <el-button @click="pasteAddress">粘贴</el-button>
                </template>
              </el-input>
            </el-form-item>

            <!-- 签名验证 -->
            <el-form-item label="签名消息" prop="signed_message">
              <el-input
                v-model="form.signed_message"
                type="textarea"
                :rows="3"
                placeholder="请对注册消息进行签名，并将签名结果粘贴到此处"
                clearable
              />
            </el-form-item>

            <!-- 助记词展示（注册成功后） -->
            <el-alert
              v-if="generatedMnemonic"
              title="重要：请保存您的助记词"
              type="warning"
              :closable="false"
              show-icon
              style="margin-bottom: 16px;"
            >
              <template #default>
                <div class="mnemonic-box">
                  <p class="mnemonic-text">{{ generatedMnemonic }}</p>
                  <p class="mnemonic-hint">
                    这是您账户的唯一恢复方式，请务必安全保存。确认保存后点击下方"完成注册"按钮。
                  </p>
                </div>
              </template>
            </el-alert>

            <!-- 操作按钮 -->
            <el-form-item>
              <el-button
                type="primary"
                :loading="loading"
                native-type="submit"
                style="width: 100%"
              >
                {{ generatedMnemonic ? '完成注册' : '生成密钥对' }}
              </el-button>
            </el-form-item>

            <el-form-item>
              <el-button
                v-if="!generatedMnemonic"
                @click="generateKeypair"
                :loading="generating"
                style="width: 100%"
              >
                生成密钥对
              </el-button>
            </el-form-item>
          </el-form>

          <div class="register-footer">
            <span>已有账户？</span>
            <el-link type="primary" :underline="false" @click="goToLogin">
              立即登录
            </el-link>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { User, Message, Wallet } from '@element-plus/icons-vue'
import { useAccountStore } from '@/stores/modules/account.store'
import { cryptoUtils } from '@/utils/crypto'
import type { FormInstance } from 'element-plus'

const router = useRouter()
const accountStore = useAccountStore()

const formRef = ref<FormInstance>()
const loading = ref(false)
const generating = ref(false)
const generatedMnemonic = ref('')
const generatedKeypair = ref<{ privateKey: string; publicKey: string } | null>(null)

const form = reactive({
  name: '',
  email: '',
  wallet_address: '',
  signed_message: ''
})

const rules = {
  name: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    { min: 2, max: 50, message: '长度在 2 到 50 个字符', trigger: 'blur' }
  ],
  email: [
    { required: true, message: '请输入邮箱', trigger: 'blur' },
    { type: 'email', message: '请输入有效的邮箱地址', trigger: 'blur' }
  ],
  wallet_address: [
    { required: true, message: '请输入钱包地址', trigger: 'blur' },
    {
      pattern: /^0x[a-fA-F0-9]{40}$/,
      message: '请输入有效的以太坊地址格式（0x...）',
      trigger: 'blur'
    }
  ]
}

// 粘贴地址
const pasteAddress = async () => {
  try {
    const text = await navigator.clipboard.readText()
    form.wallet_address = text.trim()
  } catch (error) {
    ElMessage.error('无法访问剪贴板')
  }
}

// 生成密钥对（助记词）
const generateKeypair = async () => {
  try {
    generating.value = true

    // 验证表单基础字段
    if (!form.name || !form.email || !form.wallet_address) {
      ElMessage.warning('请先填写用户名、邮箱和钱包地址')
      return
    }

    // 使用加密工具生成密钥对
    const keypair = cryptoUtils.generateKeypair()
    generatedKeypair.value = keypair
    generatedMnemonic.value = keypair.mnemonic || ''

    ElMessage.success('密钥对生成成功，请保存助记词')
  } catch (error: any) {
    console.error('Failed to generate keypair:', error)
    ElMessage.error('密钥对生成失败：' + error.message)
  } finally {
    generating.value = false
  }
}

// 提交注册
const handleRegister = async () => {
  try {
    await formRef.value?.validate()
    loading.value = true

    // 提交注册
    const registerData = {
      name: form.name,
      email: form.email,
      wallet_address: form.wallet_address,
      signature: form.signed_message,
      mnemonic: generatedMnemonic.value,
      public_key: generatedKeypair.value?.publicKey
    }

    // TODO: 调用注册API
    // await accountApi.register(registerData)
    await accountStore.register(registerData as any)

    ElMessage.success('注册成功！')

    // 提示用户保存助记词
    await ElMessageBox.alert(
      `<div style="font-family: monospace; font-size: 14px; padding: 12px; background: #f5f7fa; border-radius: 4px;">
        ${generatedMnemonic.value}
      </div>
      <p style="margin-top: 12px; color: #e6a23c;">
        请务必保存以上助记词，它是恢复账户的唯一方式！
      </p>`,
      '注册成功',
      {
        confirmButtonText: '我已保存',
        dangerouslyUseHTMLString: true
      }
    )

    // 跳转到登录或仪表盘
    router.push('/dashboard')
  } catch (error: any) {
    console.error('Registration failed:', error)
    ElMessage.error(error.message || '注册失败')
  } finally {
    loading.value = false
  }
}

// 跳转到登录页
const goToLogin = () => {
  router.push('/login')
}
</script>

<style lang="scss" scoped>
.register-page {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  padding: 24px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.register-card {
  width: 100%;
  border-radius: 12px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);

  :deep(.el-card__header) {
    text-align: center;
    padding: 24px;

    h2 {
      margin: 0 0 8px 0;
      color: #303133;
      font-size: 24px;
    }

    .subtitle {
      margin: 0;
      color: #909399;
      font-size: 14px;
    }
  }
}

.mnemonic-box {
  padding: 12px;
  background: #fdf6ec;
  border: 1px solid #faecd8;
  border-radius: 4px;
  text-align: center;

  .mnemonic-text {
    font-family: 'Courier New', monospace;
    font-size: 16px;
    font-weight: bold;
    color: #e6a23c;
    margin: 0 0 8px 0;
    word-break: break-all;
  }

  .mnemonic-hint {
    margin: 0;
    font-size: 12px;
    color: #e6a23c;
  }
}

.register-footer {
  text-align: center;
  margin-top: 16px;
  color: #606266;

  .el-link {
    margin-left: 4px;
  }
}
</style>
