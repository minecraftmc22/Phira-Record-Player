<i18n>
en:
  app: Phira Record Player
  authors: Authors
  upstream: Upstream Project

zh-CN:
  app: Phira Record Player
  authors: 作者
  upstream: 上游项目

</i18n>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
useI18n();

import { getVersion } from '@tauri-apps/api/app';
import { open } from '@tauri-apps/api/shell';

const appVersion = await getVersion();
</script>

<template>
  <div class="pa-8 w-100 h-100 d-flex flex-column align-center" style="max-width: 1280px; gap: 1rem">
    <h1 v-t="'app'"></h1>
    <h4 class="mt-n2">v{{ appVersion }}</h4>

    <div class="d-flex flex-column align-center" style="gap: 0.5rem">
      <p class="text-caption text-medium-emphasis" v-t="'upstream'"></p>
      <div class="d-flex flex-wrap justify-center" style="gap: 0.75rem">
        <v-btn prepend-icon="mdi-github" variant="outlined" @click="open('https://github.com/TeamFlos/phira-render')">TeamFlos / phira-render</v-btn>
        <v-btn prepend-icon="mdi-github" @click="open('https://github.com/minecraftmc22/Phira-Record-Player')">minecraftmc22 / Phira-Record-Player</v-btn>
      </div>
    </div>

    <div class="d-flex flex-column align-center" style="gap: 0.5rem">
      <p class="text-caption text-medium-emphasis" v-t="'authors'"></p>
      <div class="d-flex flex-wrap justify-center" style="gap: 0.75rem">
        <v-btn prepend-icon="mdi-github" variant="outlined" @click="open('https://github.com/TeamFlos')">TeamFlos</v-btn>
        <v-btn prepend-icon="mdi-github" variant="outlined" @click="open('https://github.com/minecraftmc22/')">Minecraftmc22</v-btn>
      </div>
    </div>

    <p>Licensed by GPLv3</p>
  </div>
</template>
