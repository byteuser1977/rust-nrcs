<template>
  <el-breadcrumb :separator-icon="ArrowRight" class="app-breadcrumb">
    <el-breadcrumb-item
      v-for="(route, index) in routes"
      :key="index"
      :to="index < routes.length - 1 ? { path: route.path } : null"
    >
      {{ route.title }}
    </el-breadcrumb-item>
  </el-breadcrumb>
</template>

<script setup lang="ts">
import { ArrowRight } from '@element-plus/icons-vue'

interface BreadcrumbRoute {
  path: string
  title: string
}

interface Props {
  routes: BreadcrumbRoute[]
}

defineProps<Props>()
</script>

<style scoped lang="scss">
.app-breadcrumb {
  display: flex;
  align-items: center;
  font-size: 14px;

  :deep(.el-breadcrumb__inner) {
    color: #606266;

    &.is-link {
      color: #409eff;

      &:hover {
        color: #66b1ff;
      }
    }
  }

  :deep(.el-breadcrumb__separator) {
    color: #c0c4cc;
    margin: 0 8px;
  }

  :deep(.el-breadcrumb__item:last-child) {
    .el-breadcrumb__inner {
      color: #303133;
      font-weight: 500;
      cursor: default;
    }
  }
}

// 深色模式
html.dark {
  .app-breadcrumb {
    :deep(.el-breadcrumb__inner) {
      color: #9ca3af;

      &.is-link {
        color: #409eff;

        &:hover {
          color: #66b1ff;
        }
      }
    }

    :deep(.el-breadcrumb__separator) {
      color: #6b7280;
    }

    :deep(.el-breadcrumb__item:last-child) {
      .el-breadcrumb__inner {
        color: #e5e7eb;
      }
    }
  }
}
</style>
