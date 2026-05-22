import { useEffect } from 'react';
import { useAppState } from './state';
import { useKimeData } from './hooks/useKimeData';
import { CompactIsland } from './components/CompactIsland';
import { ExpandedIsland } from './components/ExpandedIsland';
import { invoke } from '@tauri-apps/api/core';

function App() {
  const { state, dispatch } = useAppState();
  useKimeData();

  // 窗口模式变更时通知后端调整窗口
  useEffect(() => {
    invoke('set_island_mode', { mode: state.mode }).catch(console.error);
  }, [state.mode]);

  // 预警自动展开
  useEffect(() => {
    if (state.warningLevel !== 'none' && state.config.auto_expand_on_warning && state.mode === 'compact') {
      dispatch({ type: 'SET_MODE', payload: 'expanded' });
    }
  }, [state.warningLevel, state.config.auto_expand_on_warning, state.mode, dispatch]);

  if (state.mode === 'hidden') {
    return null;
  }

  return (
    <div className="flex items-start justify-center min-h-screen bg-transparent p-0">
      {state.mode === 'compact' ? <CompactIsland /> : <ExpandedIsland />}
    </div>
  );
}

export default App;
