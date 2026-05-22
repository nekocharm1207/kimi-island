import { useCallback, useRef, useState } from 'react';
import { useAppState } from '../state';
import { UsageBar } from './UsageBar';
import { RefreshCw, Globe, Settings, AlertTriangle, AlertCircle, Save } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

export function ExpandedIsland() {
  const { state, dispatch } = useAppState();
  const { data, warningLevel, loading, error, lastUpdated } = state;
  const collapseTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [tokenInput, setTokenInput] = useState('');
  const [saving, setSaving] = useState(false);

  const handleCollapse = useCallback(() => {
    dispatch({ type: 'SET_MODE', payload: 'compact' });
  }, [dispatch]);

  const handleMouseEnter = () => {
    if (collapseTimerRef.current) {
      clearTimeout(collapseTimerRef.current);
      collapseTimerRef.current = null;
    }
  };

  const handleMouseLeave = () => {
    if (state.config.auto_collapse_delay === 0) {
      handleCollapse();
    } else if (state.config.auto_collapse_delay > 0) {
      collapseTimerRef.current = setTimeout(() => {
        handleCollapse();
      }, state.config.auto_collapse_delay);
    }
  };

  const handleRefresh = async () => {
    try {
      await invoke('get_usage_data', { force: true });
    } catch (err) {
      console.error('Refresh failed:', err);
    }
  };

  const handleOpenKimi = () => {
    invoke('open_kimi_website').catch(console.error);
  };

  const handleSaveToken = async () => {
    if (!tokenInput.trim()) return;
    setSaving(true);
    try {
      await invoke('save_kimi_token', { token: tokenInput.trim() });
      setTokenInput('');
    } catch (err) {
      console.error('Save token failed:', err);
    } finally {
      setSaving(false);
    }
  };

  const formatNumber = (n: number) => {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
    return n.toString();
  };

  const WarningBanner = () => {
    if (warningLevel === 'none') return null;
    const isRed = warningLevel === 'red';
    return (
      <div className={`flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium ${
        isRed ? 'bg-red-500/20 text-red-400' : 'bg-yellow-500/20 text-yellow-400'
      }`}>
        {isRed ? <AlertCircle className="w-3.5 h-3.5" /> : <AlertTriangle className="w-3.5 h-3.5" />}
        {isRed ? '额度不足 10%，请注意' : '额度不足 30%'}
      </div>
    );
  };

  return (
    <div
      className="flex flex-col gap-3 p-4 rounded-2xl bg-[rgba(0,0,0,0.88)] backdrop-blur-xl border border-white/[0.08] shadow-2xl island-transition"
      style={{ width: 420 }}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="flex items-center justify-center w-6 h-6 rounded-md bg-white/10">
            <span className="text-xs font-bold text-white leading-none">K</span>
          </div>
          <span className="text-sm font-semibold text-white">
            {data?.current_plan ?? 'Kimi'}
          </span>
          <span className={`w-2 h-2 rounded-full ${loading ? 'bg-yellow-400 animate-pulse' : 'bg-green-400'}`} />
        </div>
        <span className="text-xs text-white/40">
          {data ? `有效期至 ${new Date(data.validity.current_end_time).toLocaleDateString('zh-CN')}` : '--'}
        </span>
      </div>

      {/* Warning Banner */}
      <WarningBanner />

      {/* Usage Card */}
      <div className="flex flex-col gap-2 p-3 rounded-xl bg-white/[0.04]">
        <div className="flex items-center justify-between">
          <span className="text-xs text-white/60">本周额度</span>
          <span className="text-xs text-white/40">
            {lastUpdated ? `更新于 ${lastUpdated.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })}` : ''}
          </span>
        </div>
        <UsageBar 
          ratio={data?.usage_ratio ?? 0} 
          warningLevel={warningLevel} 
          height={8} 
          showLabel 
        />
        <div className="flex items-center justify-between text-xs">
          <span className="text-white/60">
            已用 {data ? formatNumber(data.weekly_usage.used) : '--'} / 
            总额 {data ? formatNumber(data.weekly_usage.total) : '--'}
          </span>
          <span className="text-white/40">{data?.weekly_usage.unit ?? 'tokens'}</span>
        </div>
      </div>

      {/* Rate Limits */}
      {data && (
        <div className="grid grid-cols-3 gap-2">
          {(['rpm', 'tpm', 'rpd'] as const).map((key) => {
            const item = data.rate_limit_details[key];
            const isLimited = item.remaining === 0;
            return (
              <div key={key} className="flex flex-col items-center gap-1 p-2 rounded-lg bg-white/[0.04]">
                <span className="text-[10px] uppercase text-white/40 tracking-wider">{key}</span>
                <span className={`text-sm font-semibold tabular-nums ${isLimited ? 'text-red-400' : 'text-white'}`}>
                  {item.current}/{item.limit}
                </span>
                <span className="text-[10px] text-white/30">剩余 {item.remaining}</span>
              </div>
            );
          })}
        </div>
      )}

      {/* Error / No Token */}
      {error && (
        <div className="flex flex-col gap-2 px-3 py-2 rounded-lg bg-red-500/10 text-xs">
          <span className="text-red-400">{error}</span>
          {(error.includes('token') || error.includes('Token') || error.includes('未找到') || error.includes('未配置')) && (
            <div className="flex flex-col gap-2">
              <p className="text-white/50">请打开浏览器 → 访问 kimi.com/code/console → F12 → Application → Local Storage → 复制 access_token → 粘贴到下方：</p>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={tokenInput}
                  onChange={(e) => setTokenInput(e.target.value)}
                  placeholder="粘贴 access_token..."
                  className="flex-1 px-2 py-1.5 rounded bg-white/5 border border-white/10 text-white text-xs placeholder:text-white/30 focus:outline-none focus:border-white/30"
                />
                <button
                  onClick={handleSaveToken}
                  disabled={saving || !tokenInput.trim()}
                  className="flex items-center gap-1 px-3 py-1.5 rounded-lg bg-white/10 text-white hover:bg-white/20 transition-colors text-xs disabled:opacity-40"
                >
                  <Save className="w-3.5 h-3.5" />
                  {saving ? '保存中...' : '保存'}
                </button>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Actions */}
      <div className="flex items-center gap-2 pt-1">
        <button
          onClick={handleRefresh}
          disabled={loading}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-white/10 text-xs text-white hover:bg-white/20 transition-colors disabled:opacity-50"
        >
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
          刷新
        </button>
        <button
          onClick={handleOpenKimi}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-white/10 text-xs text-white hover:bg-white/20 transition-colors"
        >
          <Globe className="w-3.5 h-3.5" />
          打开 Kimi
        </button>
        <button
          className="flex items-center justify-center w-8 h-8 rounded-lg bg-white/10 text-white hover:bg-white/20 transition-colors ml-auto"
        >
          <Settings className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
}
