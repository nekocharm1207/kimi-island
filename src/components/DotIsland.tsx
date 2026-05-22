import { useAppState } from '../state';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

export function DotIsland() {
  const { state, dispatch } = useAppState();
  const { warningLevel } = state;

  const handleClick = () => {
    dispatch({ type: 'SET_MODE', payload: 'compact' });
  };

  const handleMouseDown = () => {
    const win = getCurrentWebviewWindow();
    win.startDragging().catch(console.error);
  };

  const dotColor = {
    none: 'bg-green-400',
    yellow: 'bg-yellow-400',
    red: 'bg-red-400',
  }[warningLevel];

  return (
    <div
      onClick={handleClick}
      onMouseDown={handleMouseDown}
      className="flex items-center justify-center w-12 h-12 rounded-full bg-[rgba(0,0,0,0.82)] backdrop-blur-md border border-white/[0.08] cursor-pointer select-none hover:bg-[rgba(31,31,31,0.9)] transition-colors island-transition"
      title="Kimi Island"
    >
      <div className="flex items-center justify-center w-6 h-6 rounded-full bg-white/10">
        <span className="text-xs font-bold text-white leading-none">K</span>
      </div>
      <span className={`absolute top-1 right-1 w-2 h-2 rounded-full ${dotColor}`} />
    </div>
  );
}
