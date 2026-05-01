<script setup lang="ts">
import { NForm, NFormItem, NSelect, NSlider, NSpace, NText } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "../store";
import ThemeSelector from "../components/ThemeSelector.vue";

const { t } = useI18n();
const store = useSettingsStore();

const languageOptions = [
  { label: "简体中文", value: "zh-CN" },
  { label: "English", value: "en-US" },
];
</script>

<template>
  <NForm label-placement="left" label-width="100">
    <NFormItem :label="t('settings.appearance.theme')">
      <ThemeSelector v-model="store.appearance.theme" />
    </NFormItem>
    <NFormItem :label="t('settings.appearance.language')">
      <NSelect
        :value="store.appearance.language"
        :options="languageOptions"
        style="width: 200px"
        @update:value="store.updateLanguage($event)"
      />
    </NFormItem>
    <NFormItem :label="t('settings.appearance.fontSize')">
      <NSpace align="center">
        <NSlider
          :value="store.appearance.fontSize"
          :min="12"
          :max="20"
          :step="1"
          style="width: 200px"
          @update:value="store.updateFontSize($event)"
        />
        <NText depth="3">{{ store.appearance.fontSize }}px</NText>
      </NSpace>
    </NFormItem>
  </NForm>
</template>
