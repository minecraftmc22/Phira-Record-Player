<i18n>
en:
  title: Replay
  already-running: Phira Record Player is already running

  open-rec: Open .phirarec
  can-also-drop: You can also drag & drop a .phirarec file here
  drop: DROP REPLAY HERE

  file-info: File Info
  file-size: File size
  file-version: Format version

  chart-info: Chart Info
  chart-name: Chart name
  chart-id: Chart ID

  player-info: Player Info
  player-name: Player name
  player-id: Player ID

  record-time: Record time
  encrypted-warn: This file appears to be encrypted. Decryption requires the Phira client.

  touch-stats: Touch Record
  touch-frames: Touch frames
  touch-points: Touch points
  duration: Duration
  avg-fps: Avg recording FPS

  judge-stats: Judgements
  judge-events: Judge events
  judges: Judgements

  section-touch: Touch Timeline
  section-judge: Judgement Breakdown

  render-replay: Render Replay Video
  render-not-available: Replay rendering requires chart file association (not yet implemented)
  select-chart: Select Chart
  chart-selected: Chart Selected
  render-replay-btn: Render Replay Video
  render-replay-started: Replay render started!
  see-tasks: See Tasks
  no-chart-selected: No chart file selected yet
  change-chart: Change Chart

zh-CN:
  title: 回放
  already-running: Phira Record Player 已经在运行

  open-rec: 打开 .phirarec
  can-also-drop: 也可以直接拖放 .phirarec 文件至此处
  drop: 拖放回放文件至此处

  file-info: 文件信息
  file-size: 文件大小
  file-version: 格式版本

  chart-info: 谱面信息
  chart-name: 谱面名称
  chart-id: 谱面 ID

  player-info: 玩家信息
  player-name: 玩家名称
  player-id: 玩家 ID

  record-time: 录制时间
  encrypted-warn: 此文件已加密，需要 Phira 客户端才能解密。

  touch-stats: 触控记录
  touch-frames: 触控帧数
  touch-points: 触控点数
  duration: 时长
  avg-fps: 平均录制帧率

  judge-stats: 判定统计
  judge-events: 判定事件
  judges: 判定

  section-touch: 触控时间线
  section-judge: 判定分布

  render-replay: 渲染回放视频
  render-not-available: 回放视频渲染需要关联谱面文件（尚未实现）
  select-chart: 选择谱面
  chart-selected: 已选谱面
  change-chart: 更换谱面
  render-replay-btn: 开始渲染回放视频
  render-replay-started: 回放视频渲染已开始！
  see-tasks: 查看任务列表
  no-chart-selected: 尚未选择谱面文件
</i18n>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { invoke, dialog, event } from '@tauri-apps/api';
import { toast, toastError, isString } from './common';

import type { PhiraRecord, ChartInfo } from './model';
import { JUDGEMENT_NAMES } from './model';

const fileHovering = ref(false);
const loading = ref(false);
const record = ref<PhiraRecord | null>(null);
const filePath = ref('');
const fileSize = ref(0);
const isEncrypted = ref(false);

// Choose file via native dialog
const choosingFile = ref(false);
async function chooseRec() {
  if (choosingFile.value) return;
  choosingFile.value = true;
  let file = await dialog.open({
    filters: [
      { name: 'Phira Replay', extensions: ['phirarec'] },
      { name: t('any-filter'), extensions: ['*'] },
    ],
  });
  choosingFile.value = false;
  if (!file) return;
  await loadRec(file as string);
}

// Parse .phirarec via Tauri command
async function loadRec(path: string) {
  try {
    loading.value = true;
    filePath.value = path;
    record.value = null;
    isEncrypted.value = false;

    const result = (await invoke('parse_rec', { path })) as PhiraRecord;
    record.value = result;
  } catch (e: any) {
    const msg = String(e);
    // If the parse failed but magic is valid, it's likely encrypted
    if (msg.includes('ULEB128') || msg.includes('unexpected') || msg.includes('string length')) {
      isEncrypted.value = true;
    }
    // Still try to get file size
    toastError(e);
  } finally {
    loading.value = false;
  }
}

// Chart file for replay rendering
const chartPath = ref('');
const chartInfo = ref<ChartInfo | null>(null);
const choosingChart = ref(false);
const rendering = ref(false);

async function chooseChart() {
  if (choosingChart.value) return;
  choosingChart.value = true;
  let file = await dialog.open({
    filters: [
      { name: 'Phigros Chart', extensions: ['zip', 'pez'] },
      { name: t('any-filter'), extensions: ['*'] },
    ],
  });
  // Also try folder picker if user cancels
  if (!file) {
    file = await dialog.open({ directory: true });
  }
  choosingChart.value = false;
  if (!file) return;
  await loadChart(file as string);
}

async function loadChart(path: string) {
  try {
    chartPath.value = path;
    chartInfo.value = (await invoke('parse_chart', { path })) as ChartInfo;
  } catch (e: any) {
    toastError(e);
  }
}

async function startReplayRender() {
  if (!chartPath.value || !record.value) return;
  try {
    rendering.value = true;
    const taskId = await invoke('render_replay_video', {
      request: {
        chartPath: chartPath.value,
        replayPath: filePath.value,
      },
    });
    toast(t('render-replay-started'), 'success');
    // Navigate to tasks after short delay
    setTimeout(() => {
      goToTasks();
    }, 1000);
  } catch (e: any) {
    toastError(e);
  } finally {
    rendering.value = false;
  }
}

function goToTasks() {
  window.goto('tasks');
}

// Drag & drop handling
event.listen('tauri://file-drop-hover', () => { fileHovering.value = true; });
event.listen('tauri://file-drop-cancelled', () => { fileHovering.value = false; });
event.listen('tauri://file-drop', async (ev) => {
  fileHovering.value = false;
  const files = ev.payload as string[];
  if (files.length > 0) {
    await loadRec(files[0]);
  }
});

// Computed stats
const totalTouchPoints = computed(() => {
  if (!record.value) return 0;
  let n = 0;
  for (const f of record.value.touches) n += f.points.length;
  return n;
});

const duration = computed(() => {
  if (!record.value || record.value.touches.length === 0) return 0;
  return record.value.touches[record.value.touches.length - 1].time;
});

const avgFps = computed(() => {
  const d = duration.value;
  if (!record.value || d <= 0) return 0;
  return Math.round(record.value.touches.length / d);
});

const judgementCounts = computed(() => {
  const counts: Record<number, number> = {};
  if (!record.value) return counts;
  for (const j of record.value.judges) {
    counts[j.judgement] = (counts[j.judgement] || 0) + 1;
  }
  return counts;
});

// Format timestamp
function formatTimestamp(ms: number): string {
  if (ms <= 0) return '-';
  const d = new Date(ms);
  return d.toLocaleString();
}

// Format filesize
function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

const COMPRESSION_NAMES: Record<number, string> = {
  0: 'None',
  1: 'ZSTD',
  2: 'DEFLATE',
};
function compressionName(compression: number): string {
  return COMPRESSION_NAMES[compression] ?? `Unknown(${compression})`;
}

// Touch density visualization data
const touchDensity = computed(() => {
  if (!record.value || record.value.touches.length === 0) return [];
  const d = duration.value || 1;
  const bucketCount = 60;
  const bucketSize = d / bucketCount;
  const buckets = new Array(bucketCount).fill(0);
  for (const f of record.value.touches) {
    const idx = Math.min(Math.floor(f.time / bucketSize), bucketCount - 1);
    buckets[idx] += f.points.length;
  }
  const maxVal = Math.max(1, ...buckets);
  return buckets.map((v, i) => ({
    time: (i * bucketSize).toFixed(1),
    value: v,
    height: (v / maxVal) * 100,
  }));
});

// Judgement chart colors
const judgementColors: Record<number, string> = {
  0: '#FFD700', // Perfect - gold
  1: '#4CAF50', // Good - green
  2: '#2196F3', // Bad - blue
  3: '#9E9E9E', // Miss - grey
  4: '#FF9800', // HoldPerfect - orange
  5: '#FF5722', // HoldGood - deep orange
};
</script>

<template>
  <div class="pa-8 w-100 h-100" style="max-width: 1280px; overflow-y: auto;">
    <h2 class="text-h4 mb-6">{{ t('title') }}</h2>

    <!-- File chooser area -->
    <div v-if="!record && !isEncrypted && !loading" class="mb-6">
      <div
        class="drop-zone d-flex flex-column align-center justify-center py-12"
        :class="{ 'drop-zone--hover': fileHovering }"
        @click="chooseRec"
      >
        <v-icon size="64" color="primary" class="mb-4">mdi-file-video</v-icon>
        <v-btn color="primary" size="large" variant="tonal" @click.stop="chooseRec">
          <v-icon start>mdi-folder-open</v-icon>
          {{ t('open-rec') }}
        </v-btn>
        <p class="mt-4 text-disabled" v-t="'can-also-drop'"></p>
      </div>
      <v-overlay v-model="fileHovering" contained class="align-center justify-center" persistent>
        <h1 v-t="'drop'"></h1>
      </v-overlay>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="d-flex justify-center pa-12">
      <v-progress-circular indeterminate size="48" />
    </div>

    <!-- Encrypted warning -->
    <div v-if="isEncrypted && !record">
      <v-alert type="warning" variant="tonal" class="mb-4">
        {{ t('encrypted-warn') }}
      </v-alert>
      <div class="drop-zone d-flex flex-column align-center justify-center py-8" @click="chooseRec">
        <v-btn variant="tonal">{{ t('open-rec') }}</v-btn>
      </div>
    </div>

    <!-- Record details -->
    <div v-if="record">
      <!-- File Info Card -->
      <v-card class="mb-4">
        <v-card-title v-t="'file-info'"></v-card-title>
        <v-card-text>
          <v-row dense>
            <v-col cols="6" sm="3">
              <div class="text-caption text-disabled">{{ t('file-size') }}</div>
              <div class="text-body-1">{{ formatSize(fileSize) }}</div>
            </v-col>
            <v-col cols="6" sm="3">
              <div class="text-caption text-disabled">{{ t('file-version') }}</div>
              <div class="text-body-1">{{ record.version }}</div>
            </v-col>
            <v-col cols="6" sm="3">
              <div class="text-caption text-disabled">{{ t('record-time') }}</div>
              <div class="text-body-1">{{ formatTimestamp(record.timestamp_ms) }}</div>
            </v-col>
            <v-col cols="6" sm="3">
              <div class="text-caption text-disabled">Compression</div>
              <div class="text-body-1">{{ compressionName(record.compression) }}</div>
            </v-col>
          </v-row>
        </v-card-text>
      </v-card>

      <!-- Chart Info -->
      <v-card class="mb-4">
        <v-card-title v-t="'chart-info'"></v-card-title>
        <v-card-text>
          <v-row dense>
            <v-col cols="12" sm="6">
              <div class="text-caption text-disabled">{{ t('chart-name') }}</div>
              <div class="text-body-1 font-weight-medium">{{ record.chart_name }}</div>
            </v-col>
            <v-col cols="12" sm="6">
              <div class="text-caption text-disabled">{{ t('chart-id') }}</div>
              <div class="text-body-1">{{ record.chart_id }}</div>
            </v-col>
          </v-row>
        </v-card-text>
      </v-card>

      <!-- Player Info -->
      <v-card class="mb-4">
        <v-card-title v-t="'player-info'"></v-card-title>
        <v-card-text>
          <v-row dense>
            <v-col cols="12" sm="6">
              <div class="text-caption text-disabled">{{ t('player-name') }}</div>
              <div class="text-body-1 font-weight-medium">{{ record.user_name }}</div>
            </v-col>
            <v-col cols="12" sm="6">
              <div class="text-caption text-disabled">{{ t('player-id') }}</div>
              <div class="text-body-1">{{ record.user_id }}</div>
            </v-col>
          </v-row>
        </v-card-text>
      </v-card>

      <!-- Touch Stats -->
      <v-card class="mb-4">
        <v-card-title v-t="'touch-stats'"></v-card-title>
        <v-card-text>
          <v-row dense class="mb-4">
            <v-col cols="6" sm="3">
              <div class="text-caption text-disabled">{{ t('touch-frames') }}</div>
              <div class="text-h6">{{ record.touches.length.toLocaleString() }}</div>
            </v-col>
            <v-col cols="6" sm="3">
              <div class="text-caption text-disabled">{{ t('touch-points') }}</div>
              <div class="text-h6">{{ totalTouchPoints.toLocaleString() }}</div>
            </v-col>
            <v-col cols="6" sm="3">
              <div class="text-caption text-disabled">{{ t('duration') }}</div>
              <div class="text-h6">{{ duration.toFixed(1) }}s</div>
            </v-col>
            <v-col cols="6" sm="3">
              <div class="text-caption text-disabled">{{ t('avg-fps') }}</div>
              <div class="text-h6">{{ avgFps }}</div>
            </v-col>
          </v-row>

          <!-- Touch density bar chart -->
          <div v-if="touchDensity.length > 0" class="mt-2">
            <div class="text-subtitle-2 mb-2" v-t="'section-touch'"></div>
            <div class="density-chart">
              <div
                v-for="(bar, i) in touchDensity"
                :key="i"
                class="density-bar"
                :style="{ height: bar.height + '%' }"
                :title="`${bar.time}s: ${bar.value} pts`"
              ></div>
            </div>
            <div class="d-flex justify-space-between mt-1">
              <span class="text-caption">0s</span>
              <span class="text-caption">{{ duration.toFixed(1) }}s</span>
            </div>
          </div>
        </v-card-text>
      </v-card>

      <!-- Judge Stats -->
      <v-card class="mb-4">
        <v-card-title v-t="'judge-stats'"></v-card-title>
        <v-card-text>
          <v-row dense class="mb-4">
            <v-col cols="6" sm="4">
              <div class="text-caption text-disabled">{{ t('judge-events') }}</div>
              <div class="text-h6">{{ record.judges.length.toLocaleString() }}</div>
            </v-col>
          </v-row>

          <!-- Judgement breakdown bar -->
          <div v-if="record.judges.length > 0">
            <div class="text-subtitle-2 mb-2" v-t="'section-judge'"></div>
            <div class="judge-breakdown">
              <div
                v-for="(count, code) in judgementCounts"
                :key="code"
                class="judge-segment"
                :style="{
                  width: (count / record.judges.length * 100) + '%',
                  backgroundColor: judgementColors[Number(code)] || '#888'
                }"
                :title="`${JUDGEMENT_NAMES[Number(code)] || 'Unknown'}: ${count}`"
              ></div>
            </div>
            <div class="d-flex flex-wrap mt-2" style="gap: 8px 16px;">
              <div v-for="(count, code) in judgementCounts" :key="code" class="d-flex align-center">
                <div
                  class="judge-dot mr-1"
                  :style="{ backgroundColor: judgementColors[Number(code)] || '#888' }"
                ></div>
                <span class="text-caption">{{ JUDGEMENT_NAMES[Number(code)] || `(${code})` }}: {{ count }}</span>
              </div>
            </div>
          </div>
        </v-card-text>
      </v-card>

      <!-- Render button -->
      <v-card class="mb-4">
        <v-card-title>{{ t('render-replay') }}</v-card-title>
        <v-card-text>
          <!-- Chart selection -->
          <div v-if="!chartInfo" class="mb-3">
            <p class="text-caption text-disabled mb-2">{{ t('no-chart-selected') }}</p>
            <v-btn
              variant="tonal"
              color="primary"
              prepend-icon="mdi-file-music"
              :loading="choosingChart"
              @click="chooseChart"
            >
              {{ t('select-chart') }}
            </v-btn>
          </div>

          <!-- Selected chart info -->
          <div v-if="chartInfo" class="mb-3">
            <v-chip color="success" size="small" class="mb-2">
              <v-icon start size="small">mdi-check</v-icon>
              {{ t('chart-selected') }}
            </v-chip>
            <div class="text-body-2">
              <strong>{{ chartInfo.name }}</strong>
              <span class="text-disabled ml-2">Lv.{{ chartInfo.level }}</span>
              <span class="text-disabled ml-2">by {{ chartInfo.charter }}</span>
            </div>
            <v-btn variant="text" size="small" class="mt-1" @click="chooseChart">
              {{ t('change-chart') }}
            </v-btn>
          </div>

          <v-divider class="mb-3"></v-divider>

          <!-- Render button -->
          <v-btn
            color="primary"
            size="large"
            :disabled="!chartInfo"
            :loading="rendering"
            @click="startReplayRender"
          >
            <v-icon start>mdi-video</v-icon>
            {{ t('render-replay-btn') }}
          </v-btn>
          <v-btn
            variant="text"
            class="ml-2"
            @click="goToTasks"
          >
            {{ t('see-tasks') }}
          </v-btn>
        </v-card-text>
      </v-card>
    </div>
  </div>
</template>

<style scoped>
.drop-zone {
  border: 2px dashed rgba(var(--v-theme-primary), 0.3);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
}
.drop-zone:hover {
  border-color: rgba(var(--v-theme-primary), 0.6);
  background: rgba(var(--v-theme-primary), 0.03);
}
.drop-zone--hover {
  border-color: rgba(var(--v-theme-primary), 0.8);
  background: rgba(var(--v-theme-primary), 0.06);
}

.density-chart {
  display: flex;
  align-items: flex-end;
  height: 64px;
  gap: 1px;
  background: rgba(var(--v-theme-surface-variant), 0.3);
  border-radius: 4px;
  overflow: hidden;
  padding: 2px;
}
.density-bar {
  flex: 1;
  background: rgba(var(--v-theme-primary), 0.6);
  border-radius: 1px 1px 0 0;
  min-height: 1px;
  transition: background 0.15s;
}
.density-bar:hover {
  background: rgba(var(--v-theme-primary), 0.9);
}

.judge-breakdown {
  display: flex;
  height: 24px;
  border-radius: 4px;
  overflow: hidden;
  gap: 2px;
}
.judge-segment {
  border-radius: 4px;
  transition: opacity 0.15s;
}
.judge-segment:hover {
  opacity: 0.8;
}
.judge-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}
</style>
