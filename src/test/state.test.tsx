import { describe, it, expect } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { AppProvider, useAppState } from '../state';
import { KimeUsageData } from '../types';

function wrapper({ children }: { children: React.ReactNode }) {
  return <AppProvider>{children}</AppProvider>;
}

const mockData: KimeUsageData = {
  current_plan: 'Pro',
  validity: { current_end_time: '2026-06-06T00:00:00Z', days_remaining: 14 },
  weekly_usage: { used: 50, total: 100, unit: 'tokens' },
  usage_ratio: 0.5,
  rate_limit_details: {
    rpm: { current: 1, limit: 100, remaining: 99 },
    tpm: { current: 0, limit: 0, remaining: 0 },
    rpd: { current: 63, limit: 100, remaining: 37 },
  },
  model_permissions: ['FEATURE_CODING'],
};

describe('App State', () => {
  it('should have initial state', () => {
    const { result } = renderHook(() => useAppState(), { wrapper });
    expect(result.current.state.mode).toBe('compact');
    expect(result.current.state.data).toBeNull();
    expect(result.current.state.loading).toBe(true);
    expect(result.current.state.error).toBeNull();
    expect(result.current.state.warningLevel).toBe('none');
  });

  it('should set data and calculate yellow warning level', () => {
    const { result } = renderHook(() => useAppState(), { wrapper });
    // usage_ratio 0.5 => remaining 0.5, between red(0.1) and yellow(0.3) => none
    const data = { ...mockData, usage_ratio: 0.5 };

    act(() => {
      result.current.dispatch({ type: 'SET_DATA', payload: data });
    });

    expect(result.current.state.data).toEqual(data);
    expect(result.current.state.warningLevel).toBe('none');
    expect(result.current.state.loading).toBe(true);
    expect(result.current.state.error).toBeNull();
  });

  it('should calculate red warning level when almost exhausted', () => {
    const { result } = renderHook(() => useAppState(), { wrapper });
    // usage_ratio 0.95 => remaining 0.05 <= red(0.1) => red
    const redData = { ...mockData, usage_ratio: 0.95 };

    act(() => {
      result.current.dispatch({ type: 'SET_DATA', payload: redData });
    });

    expect(result.current.state.warningLevel).toBe('red');
  });

  it('should calculate yellow warning level when low', () => {
    const { result } = renderHook(() => useAppState(), { wrapper });
    // usage_ratio 0.8 => remaining 0.2 <= yellow(0.3) => yellow
    const yellowData = { ...mockData, usage_ratio: 0.8 };

    act(() => {
      result.current.dispatch({ type: 'SET_DATA', payload: yellowData });
    });

    expect(result.current.state.warningLevel).toBe('yellow');
  });

  it('should calculate none warning level when safe', () => {
    const { result } = renderHook(() => useAppState(), { wrapper });
    // usage_ratio 0.5 => remaining 0.5 > yellow(0.3) => none
    const safeData = { ...mockData, usage_ratio: 0.5 };

    act(() => {
      result.current.dispatch({ type: 'SET_DATA', payload: safeData });
    });

    expect(result.current.state.warningLevel).toBe('none');
  });

  it('should set mode', () => {
    const { result } = renderHook(() => useAppState(), { wrapper });

    act(() => {
      result.current.dispatch({ type: 'SET_MODE', payload: 'expanded' });
    });

    expect(result.current.state.mode).toBe('expanded');

    act(() => {
      result.current.dispatch({ type: 'SET_MODE', payload: 'dot' });
    });

    expect(result.current.state.mode).toBe('dot');
  });

  it('should set loading', () => {
    const { result } = renderHook(() => useAppState(), { wrapper });

    act(() => {
      result.current.dispatch({ type: 'SET_LOADING', payload: false });
    });

    expect(result.current.state.loading).toBe(false);
  });

  it('should set error', () => {
    const { result } = renderHook(() => useAppState(), { wrapper });

    act(() => {
      result.current.dispatch({ type: 'SET_ERROR', payload: 'Network error' });
    });

    expect(result.current.state.error).toBe('Network error');
    expect(result.current.state.loading).toBe(false);
  });

  it('should set config', () => {
    const { result } = renderHook(() => useAppState(), { wrapper });

    act(() => {
      result.current.dispatch({
        type: 'SET_CONFIG',
        payload: { ...result.current.state.config, auto_collapse_delay: 500 },
      });
    });

    expect(result.current.state.config.auto_collapse_delay).toBe(500);
  });
});
