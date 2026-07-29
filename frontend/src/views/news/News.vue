<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Notebook /></el-icon> {{ t('news.title') || 'News' }}</h2>
      <div class="header-actions">
        <el-button size="small" @click="refreshAll"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>

    <el-row :gutter="16">
      <el-col :span="12" v-for="feed in feeds" :key="feed.name">
        <el-card shadow="hover" class="feed-card">
          <template #header>
            <div class="feed-header">
              <span class="feed-title">{{ feed.label }}</span>
              <el-tag size="small" v-if="feed.loading">{{ t('common.loading') }}</el-tag>
              <el-tag size="small" type="danger" v-else-if="feed.error">{{ t('common.loadError') }}</el-tag>
              <el-tag size="small" type="success" v-else>{{ feed.items.length }} {{ t('news.items') || 'items' }}</el-tag>
            </div>
          </template>
          <div class="feed-items" v-loading="feed.loading">
            <div v-for="(item, idx) in feed.items" :key="idx" class="feed-item">
              <a :href="item.link" target="_blank" class="item-title">{{ item.title }}</a>
              <div class="item-meta">
                <span class="item-date">{{ item.date }}</span>
                <span class="item-author" v-if="item.author">— {{ item.author }}</span>
              </div>
            </div>
            <el-empty v-if="!feed.loading && feed.items.length === 0" :description="t('common.noData')" :image-size="40" />
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { Notebook, Refresh } from '@element-plus/icons-vue'

const { t } = useI18n()

interface FeedItem {
  title: string
  link: string
  date: string
  author?: string
}

interface Feed {
  name: string
  label: string
  url: string
  items: FeedItem[]
  loading: boolean
  error: boolean
}

const feeds = reactive<Feed[]>([
  {
    name: 'nrcs-announcements',
    label: 'NRCS Announcements',
    url: '',
    items: [],
    loading: false,
    error: false,
  },
  {
    name: 'blockchain-news',
    label: 'Blockchain Industry News',
    url: '',
    items: [],
    loading: false,
    error: false,
  },
])

async function fetchFeed(feed: Feed): Promise<void> {
  feed.loading = true
  feed.error = false
  try {
    // Attempt to fetch RSS/Atom feed via a CORS proxy
    if (!feed.url) {
      // No URL configured - show empty state
      feed.items = []
      return
    }
    const response = await fetch(feed.url, { signal: AbortSignal.timeout(10000) })
    const text = await response.text()
    feed.items = parseFeedItems(text)
  } catch {
    feed.error = true
  } finally {
    feed.loading = false
  }
}

function parseFeedItems(xml: string): FeedItem[] {
  const items: FeedItem[] = []
  // Parse RSS items using regex
  const itemRegex = /<item>([\s\S]*?)<\/item>/gi
  let match
  while ((match = itemRegex.exec(xml)) !== null) {
    const content = match[1]
    const title = extractTag(content, 'title')
    const link = extractTag(content, 'link')
    const pubDate = extractTag(content, 'pubDate')
    const author = extractTag(content, 'author') || extractTag(content, 'dc:creator')

    if (title) {
      items.push({
        title: decodeHtmlEntities(title),
        link: link || '#',
        date: pubDate ? formatRssDate(pubDate) : '',
        author: author ? decodeHtmlEntities(author) : undefined,
      })
    }
  }
  return items
}

function extractTag(content: string, tag: string): string {
  const regex = new RegExp(`<${tag}[^>]*>([\\s\\S]*?)</${tag}>`, 'i')
  const match = content.match(regex)
  return match ? match[1].trim() : ''
}

function decodeHtmlEntities(text: string): string {
  return text
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&apos;/g, "'")
}

function formatRssDate(dateStr: string): string {
  try {
    const d = new Date(dateStr)
    if (isNaN(d.getTime())) return dateStr
    return d.toLocaleDateString()
  } catch {
    return dateStr
  }
}

function refreshAll() {
  feeds.forEach((f) => fetchFeed(f))
}

onMounted(() => {
  refreshAll()
})
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.page-container {
  .page-header {
    display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;
    .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; }
    .header-actions { display: flex; gap: 8px; }
  }
  .feed-card {
    margin-bottom: 16px;
    .feed-header { display: flex; justify-content: space-between; align-items: center; .feed-title { font-weight: 600; } }
    .feed-items {
      max-height: 400px; overflow-y: auto;
      .feed-item {
        padding: 8px 0; border-bottom: 1px solid var(--el-border-color-lighter);
        &:last-child { border-bottom: none; }
        .item-title { font-size: 14px; color: var(--el-color-primary); text-decoration: none;
          &:hover { text-decoration: underline; }
        }
        .item-meta { font-size: 12px; color: $text-muted; margin-top: 2px; }
      }
    }
  }
}
</style>
