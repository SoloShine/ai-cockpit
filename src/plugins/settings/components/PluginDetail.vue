<script setup lang="ts">
import { NDescriptions, NDescriptionsItem, NTag } from "naive-ui";
import { useI18n } from "vue-i18n";
import type { CockpitPlugin } from "@/core/plugin";

const props = defineProps<{ plugin: CockpitPlugin }>();
const { t } = useI18n();
</script>

<template>
  <div style="padding: 12px 0">
    <NDescriptions label-placement="left" bordered :column="1" size="small">
      <NDescriptionsItem :label="t('settings.plugins.routes')">
        <NTag
          v-for="route in plugin.routes"
          :key="route.path"
          size="small"
          style="margin-right: 4px"
        >
          {{ route.path }}
        </NTag>
      </NDescriptionsItem>
    </NDescriptions>
  </div>
</template>
