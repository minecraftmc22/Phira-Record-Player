export interface ChartInfo {
  name: string;
  level: string;
  charter: string;
  composer: string;
  illustrator: string;

  tip: string | null;

  aspectRatio: number;
  backgroundDim: number;
}

export type TaskStatus =
  | {
      type: 'pending';
    }
  | {
      type: 'loading';
    }
  | {
      type: 'mixing';
    }
  | {
      type: 'rendering';
      progress: number;
      fps: number;
      estimate: number;
      log: string;
    }
  | {
      type: 'done';
      duration: number;
      output: string;
      log: string;
    }
  | {
      type: 'canceled';
    }
  | {
      type: 'failed';
      error: string;
      log: string;
    };

export interface Task {
  id: number;
  name: string;
  output: string;
  path: string;
  cover: string;
  status: TaskStatus;
}

export interface RenderConfig {
  resolution: number[];
  endingLength: number;
  fps: number;
  hardwareAccel: boolean;
  bitrate: string;

  aggressive: boolean;
  challengeColor: string;
  challengeRank: number;
  disableEffect: boolean;
  doubleHint: boolean;
  fxaa: boolean;
  noteScale: number;
  particle: boolean;
  playerAvatar: string | null;
  playerName: string;
  playerRks: number;
  sampleCount: number;
  resPackPath: string | null;
  speed: number;
  volumeMusic: number;
  volumeSfx: number;

  replayPath?: string | null;
}

export interface RPEChart {
  name: string;
  id: string;
  path: string;
  illustration: string;
  charter: string;
}

// -- .phirarec record types (tphira-mp format) --

export interface TouchPoint {
  id: number;
  x: number;
  y: number;
}

export interface TouchFrame {
  time: number;
  points: TouchPoint[];
}

export interface JudgeEvent {
  time: number;
  line_id: number;
  note_id: number;
  judgement: number;
}

export interface PhiraRecord {
  version: number;
  compression: number;
  record_id: number;
  timestamp_ms: number;
  chart_id: number;
  chart_name: string;
  user_id: number;
  user_name: string;
  touches: TouchFrame[];
  judges: JudgeEvent[];
}

export const JUDGEMENT_NAMES: Record<number, string> = {
  0: 'Perfect',
  1: 'Good',
  2: 'Bad',
  3: 'Miss',
  4: 'HoldPerfect',
  5: 'HoldGood',
};
