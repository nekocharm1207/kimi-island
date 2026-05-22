import { useAppState } from '../state';
import { UsageBar } from './UsageBar';


export function CompactIsland() {
  const { state, dispatch } = useAppState();
  const { data } = state;

  // Compact 显示五小时额度（短周期，更敏感）
  const rpm = data?.rate_limit_details.rpm;
  const ratio = rpm && rpm.limit > 0 ? rpm.current / rpm.limit : (data?.usage_ratio ?? 0);
  const percentage = Math.min(Math.max(ratio * 100, 0), 100).toFixed(0);

  // 基于 compact 显示的数据计算 warning level
  const compactWarning = (() => {
    const red = state.config.red_threshold / 100;
    const yellow = state.config.yellow_threshold / 100;
    const remaining = 1 - ratio;
    if (remaining <= red) return 'red';
    if (remaining <= yellow) return 'yellow';
    return 'none';
  })();

  const pulseClass = {
    none: '',
    yellow: 'animate-pulse-yellow',
    red: 'animate-pulse-red',
  }[compactWarning];

  const handleClick = () => {
    dispatch({ type: 'SET_MODE', payload: 'expanded' });
  };

  const handleLogoClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    dispatch({ type: 'SET_MODE', payload: 'dot' });
  };

  return (
    <div
      onClick={handleClick}
      className={`
        flex items-center gap-3 px-4 py-2 rounded-full
        bg-[rgba(0,0,0,0.82)] backdrop-blur-md
        border border-white/[0.08]
        cursor-pointer select-none
        hover:bg-[rgba(31,31,31,0.9)]
        transition-colors duration-200
        island-transition
        ${pulseClass}
      `}
      style={{ width: state.config.compact_width }}
    >
      {/* Logo */}
      <div
        onClick={handleLogoClick}
        className="flex items-center justify-center w-5 h-5 rounded-md bg-white/10 cursor-pointer hover:bg-white/20 transition-colors"
      >
        <span className="text-[10px] font-bold text-white leading-none">K</span>
      </div>

      {/* Progress Bar */}
      <div className="flex-1">
        <UsageBar ratio={ratio} warningLevel={compactWarning} height={5} />
      </div>

      {/* Percentage */}
      <span className="text-sm font-medium text-white tabular-nums min-w-[36px] text-right">
        {percentage}%
      </span>
    </div>
  );
}
