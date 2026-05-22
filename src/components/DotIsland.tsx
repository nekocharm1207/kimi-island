import { useRef } from 'react';
import { useAppState } from '../state';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

const DRAG_DELAY_MS = 150;

export function DotIsland() {
  const { state, dispatch } = useAppState();
  const { warningLevel } = state;
  const dragTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleMouseDown = () => {
    // 延迟启动拖拽：快速点击（<150ms）视为切换模式，长按才拖拽
    dragTimerRef.current = setTimeout(() => {
      const win = getCurrentWebviewWindow();
      win.startDragging().catch(console.error);
      dragTimerRef.current = null;
    }, DRAG_DELAY_MS);
  };

  const handleMouseUp = () => {
    if (dragTimerRef.current !== null) {
      // 定时器还在说明是快速点击 → 切换回 compact
      clearTimeout(dragTimerRef.current);
      dragTimerRef.current = null;
      dispatch({ type: 'SET_MODE', payload: 'compact' });
    }
  };

  const dotColor = {
    none: 'bg-green-400',
    yellow: 'bg-yellow-400',
    red: 'bg-red-400',
  }[warningLevel];

  return (
    <div
      onMouseDown={handleMouseDown}
      onMouseUp={handleMouseUp}
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
