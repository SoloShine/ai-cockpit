<script setup lang="ts">
import { ref } from "vue";
import {
  NModal, NCard, NForm, NFormItem, NInput, NButton, NSpace, useMessage,
} from "naive-ui";
import { useI18n } from "vue-i18n";
import type { AgentConfig } from "../types";

defineProps<{ show: boolean }>();
const emit = defineEmits<{
  "update:show": [value: boolean];
  add: [agent: AgentConfig];
}>();

const { t } = useI18n();
const message = useMessage();

const name = ref("");
const agentType = ref("");
const basePath = ref("");

function handleSubmit() {
  if (!name.value.trim()) { message.warning(t("settings.agents.nameRequired")); return; }
  if (!agentType.value.trim()) { message.warning(t("settings.agents.typeRequired")); return; }
  if (!basePath.value.trim()) { message.warning(t("settings.agents.pathRequired")); return; }

  const id = `custom-${Date.now()}`;
  emit("add", {
    id,
    name: name.value.trim(),
    type: agentType.value.trim(),
    basePath: basePath.value.trim(),
    enabled: true,
    isCustom: true,
  });

  name.value = "";
  agentType.value = "";
  basePath.value = "";
  emit("update:show", false);
  message.success(t("settings.agents.addSuccess"));
}
</script>

<template>
  <NModal :show="show" @update:show="emit('update:show', $event)">
    <NCard
      style="width: 480px"
      :title="t('settings.agents.addCustom')"
      :bordered="false"
      size="medium"
      role="dialog"
      closable
      @close="emit('update:show', false)"
    >
      <NForm label-placement="left" label-width="100">
        <NFormItem :label="t('settings.agents.name')">
          <NInput v-model:value="name" />
        </NFormItem>
        <NFormItem :label="t('settings.agents.type')">
          <NInput v-model:value="agentType" placeholder="my-agent" />
        </NFormItem>
        <NFormItem :label="t('settings.agents.basePath')">
          <NInput v-model:value="basePath" />
        </NFormItem>
      </NForm>
      <template #footer>
        <NSpace justify="end">
          <NButton @click="emit('update:show', false)">取消</NButton>
          <NButton type="primary" @click="handleSubmit">确定</NButton>
        </NSpace>
      </template>
    </NCard>
  </NModal>
</template>
