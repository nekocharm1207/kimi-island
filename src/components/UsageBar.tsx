import { WarningLevel } from '../types';

interface UsageBarProps {
  ratio: number;
  warningLevel: WarningLevel;
  height?: number;
  showLabel?: boolean;
}

export function UsageBar({ ratio, warningLevel, height = 6, showLabel = false }: UsageBarProps) {
  const percentage = Math.min(Math.max(ratio * 100, 0), 100);
  
  const barColor = {
    none: 'bg-[#10B981]',
    yellow: 'bg-[#F59E0B]',
    red: 'bg-[#EF4444]',
  }[warningLevel];

  return (
    <div className="flex items-center gap-2 w-full">
      <div 
        className="flex-1 rounded-full bg-white/10 overflow-hidden"
        style={{ height }}
      >
        <div
          className={`h-full rounded-full ${barColor} transition-all duration-500 ease-out`}
          style={{ width: `${percentage}%` }}
        />
      </div>
      {showLabel && (
        <span className="text-xs text-white/60 tabular-nums w-10 text-right">
          {percentage.toFixed(0)}%
        </span>
      )}
    </div>
  );
}
