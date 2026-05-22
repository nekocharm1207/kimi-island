import { useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { KimeUsageData } from '../types';
import { useAppState } from '../state';

export function useKimeData() {
  const { state, dispatch } = useAppState();
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const fetchData = useCallback(async (force = false, silent = false) => {
    if (!silent) {
      dispatch({ type: 'SET_LOADING', payload: true });
    }
    try {
      const data = await invoke<KimeUsageData>('get_usage_data', { force });
      dispatch({ type: 'SET_DATA', payload: data });
      dispatch({ type: 'SET_LAST_UPDATED', payload: new Date() });
    } catch (err) {
      dispatch({ type: 'SET_ERROR', payload: String(err) });
    } finally {
      if (!silent) {
        dispatch({ type: 'SET_LOADING', payload: false });
      }
    }
  }, [dispatch]);

  // 计算轮询间隔
  const getInterval = useCallback(() => {
    const { warningLevel, config } = state;
    if (warningLevel === 'red') return config.poll_interval_critical * 1000;
    if (warningLevel === 'yellow') return config.poll_interval_warning * 1000;
    return config.poll_interval_normal * 1000;
  }, [state.warningLevel, state.config]);

  // 启动轮询
  useEffect(() => {
    fetchData(); // 初始加载

    const setupInterval = () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
      intervalRef.current = setInterval(() => {
        fetchData(true, true);
      }, getInterval());
    };

    setupInterval();

    // 监听后端推送的数据更新
    const unlistenPromise = listen<KimeUsageData>('usage:updated', (event) => {
      dispatch({ type: 'SET_DATA', payload: event.payload });
      dispatch({ type: 'SET_LAST_UPDATED', payload: new Date() });
    });

    // 监听 token 保存成功，自动刷新数据
    const unlistenTokenPromise = listen('kimi:token_saved', () => {
      fetchData(true);
    });

    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
      unlistenPromise.then((fn) => fn());
      unlistenTokenPromise.then((fn) => fn());
    };
  }, [fetchData, getInterval, dispatch]);

  return { fetchData };
}
