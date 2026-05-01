<script setup lang="ts">
import { NResult, NButton } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { useSkillsStore } from "../store";

defineProps<{
  type: "noPath" | "noSkills" | "error";
}>();

const { t } = useI18n();
const router = useRouter();
const store = useSkillsStore();
</script>

<template>
  <div style="display: flex; justify-content: center; padding: 48px 0">
    <NResult
      v-if="type === 'noPath'"
      status="info"
      :title="t('skills.empty.noPath')"
      :description="t('skills.empty.noPathHint')"
    >
      <template #footer>
        <NButton @click="router.push({ name: 'settings' })">
          {{ t("skills.empty.goSettings") }}
        </NButton>
      </template>
    </NResult>
    <NResult
      v-else-if="type === 'noSkills'"
      status="info"
      :title="t('skills.empty.noSkills')"
      :description="t('skills.empty.noSkillsHint')"
    />
    <NResult
      v-else
      status="error"
      :title="t('skills.empty.scanError')"
      :description="store.error ?? ''"
    >
      <template #footer>
        <NButton @click="store.scanSkills(store.currentAgentId, store.currentScope)">
          {{ t("skills.empty.retry") }}
        </NButton>
      </template>
    </NResult>
  </div>
</template>
