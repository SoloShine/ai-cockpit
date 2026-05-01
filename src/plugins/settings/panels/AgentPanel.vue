<script setup lang="ts">
import { ref } from "vue";
import { NButton, NSpace, NText, useMessage } from "naive-ui";
import { AddOutline } from "@vicons/ionicons5";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "../store";
import AgentCard from "../components/AgentCard.vue";
import AddAgentDialog from "../components/AddAgentDialog.vue";

const { t } = useI18n();
const store = useSettingsStore();
const message = useMessage();
const showAddDialog = ref(false);

function handleUpdateAgent(id: string, updates: Record<string, any>) {
  store.updateAgent(id, updates);
}

function handleDeleteAgent(id: string) {
  store.removeAgent(id);
  message.success(t("settings.agents.deleteSuccess"));
}
</script>

<template>
  <div>
    <NSpace justify="space-between" align="center" style="margin-bottom: 16px">
      <NText strong style="font-size: 16px">{{ t("settings.agents.title") }}</NText>
      <NButton type="primary" @click="showAddDialog = true">
        <template #icon><AddOutline /></template>
        {{ t("settings.agents.addCustom") }}
      </NButton>
    </NSpace>

    <AgentCard
      v-for="agent in store.agents"
      :key="agent.id"
      :agent="agent"
      @update:agent="handleUpdateAgent(agent.id, $event)"
      @delete="handleDeleteAgent(agent.id)"
    />

    <AddAgentDialog
      :show="showAddDialog"
      @update:show="showAddDialog = $event"
      @add="store.addAgent($event)"
    />
  </div>
</template>
