import { useAppState } from '../state';
import { UsageBar } from './UsageBar';


export function CompactIsland() {
  const { state, dispatch } = useAppState();
  const { data, warningLevel } = state;

  const ratio = data?.usage_ratio ?? 0;
  const percentage = Math.min(Math.max(ratio * 100, 0), 100).toFixed(0);

  const pulseClass = {
    none: '',
    yellow: 'animate-pulse-yellow',
    red: 'animate-pulse-red',
  }[warningLevel];

  const handleClick = () => {
    dispatch({ type: 'SET_MODE', payload: 'expanded' });
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
      <div className="flex items-center justify-center w-5 h-5 rounded-md bg-white/10">
        <span className="text-[10px] font-bold text-white leading-none">K</span>
      </div>

      {/* Progress Bar */}
      <div className="flex-1">
        <UsageBar ratio={ratio} warningLevel={warningLevel} height={5} />
      </div>

      {/* Percentage */}
      <span className="text-sm font-medium text-white tabular-nums min-w-[36px] text-right">
        {percentage}%
      </span>
    </div>
  );
}
