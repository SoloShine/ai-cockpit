<script setup lang="ts">
import { NResult, NButton } from "naive-ui";
import { useRouter } from "vue-router";

const router = useRouter();
</script>

<template>
  <div data-testid="welcome-page" style="display: flex; align-items: center; justify-content: center; height: 80%">
    <NResult status="info" title="AI Cockpit" description="通用 AI 管理工具箱">
      <template #footer>
        <NButton data-testid="welcome-start-btn" @click="router.push({ name: 'settings' })">开始使用</NButton>
      </template>
    </NResult>
  </div>
</template>
