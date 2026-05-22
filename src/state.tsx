import React, { createContext, useContext, useReducer, ReactNode } from 'react';
import { KimeUsageData, IslandMode, WarningLevel, AppConfig } from './types';

interface AppState {
  mode: IslandMode;
  data: KimeUsageData | null;
  loading: boolean;
  error: string | null;
  lastUpdated: Date | null;
  warningLevel: WarningLevel;
  config: AppConfig;
}

type AppAction =
  | { type: 'SET_MODE'; payload: IslandMode }
  | { type: 'SET_DATA'; payload: KimeUsageData }
  | { type: 'SET_LOADING'; payload: boolean }
  | { type: 'SET_ERROR'; payload: string | null }
  | { type: 'SET_LAST_UPDATED'; payload: Date }
  | { type: 'SET_WARNING_LEVEL'; payload: WarningLevel }
  | { type: 'SET_CONFIG'; payload: AppConfig };

const defaultConfig: AppConfig = {
  preferred_display: 'primary',
  compact_width: 320,
  yellow_threshold: 30,
  red_threshold: 10,
  poll_interval_normal: 60,
  poll_interval_warning: 30,
  poll_interval_critical: 15,
  auto_collapse_delay: 800,
  auto_expand_on_warning: false,
  theme: 'system',
  sound_on_warning: false,
  autostart: false,
};

const initialState: AppState = {
  mode: 'compact',
  data: null,
  loading: true,
  error: null,
  lastUpdated: null,
  warningLevel: 'none',
  config: defaultConfig,
};

function calculateWarningLevel(ratio: number, config: AppConfig): WarningLevel {
  const red = config.red_threshold / 100;
  const yellow = config.yellow_threshold / 100;
  const remaining = 1 - ratio;
  if (remaining <= red) return 'red';
  if (remaining <= yellow) return 'yellow';
  return 'none';
}

function appReducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case 'SET_MODE':
      return { ...state, mode: action.payload };
    case 'SET_DATA':
      const warningLevel = calculateWarningLevel(action.payload.usage_ratio, state.config);
      return { ...state, data: action.payload, warningLevel, error: null };
    case 'SET_LOADING':
      return { ...state, loading: action.payload };
    case 'SET_ERROR':
      return { ...state, error: action.payload, loading: false };
    case 'SET_LAST_UPDATED':
      return { ...state, lastUpdated: action.payload };
    case 'SET_WARNING_LEVEL':
      return { ...state, warningLevel: action.payload };
    case 'SET_CONFIG':
      return { ...state, config: action.payload };
    default:
      return state;
  }
}

const AppContext = createContext<{
  state: AppState;
  dispatch: React.Dispatch<AppAction>;
}>({ state: initialState, dispatch: () => null });

export function AppProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(appReducer, initialState);
  return (
    <AppContext.Provider value={{ state, dispatch }}>
      {children}
    </AppContext.Provider>
  );
}

export function useAppState() {
  return useContext(AppContext);
}
