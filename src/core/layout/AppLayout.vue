<script setup lang="ts">
import { NLayout, NLayoutContent } from "naive-ui";
import Sidebar from "./Sidebar.vue";
</script>

<template>
  <NLayout has-sider style="height: 100vh">
    <Sidebar />
    <NLayoutContent
      content-style="padding: 24px; overflow: auto;"
      :native-scrollbar="false"
    >
      <router-view />
    </NLayoutContent>
  </NLayout>
</template>
