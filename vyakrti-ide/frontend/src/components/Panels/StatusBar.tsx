import { useIdeStore } from '../../store/ideStore';
import { Wifi, WifiOff, Circle } from 'lucide-react';

export const StatusBar = () => {
  const { buildStatus, transliterationMode, wsConnected, cursorLine, cursorColumn } = useIdeStore();

  const statusColors: Record<string, string> = {
    idle: 'text-ide-muted',
    compiling: 'text-ide-accent',
    running: 'text-ide-success',
    error: 'text-ide-error',
    success: 'text-ide-success',
  };

  const statusLabels: Record<string, string> = {
    idle: 'सज्जः',
    compiling: 'संगृह्णन्...',
    running: 'चालनम्',
    error: 'दोषः',
    success: 'सफलम्',
  };

  const cursorPos = `Ln ${cursorLine}, Col ${cursorColumn}`;

  return (
    <footer className="h-6 bg-ide-surface border-t border-ide-border flex items-center px-3 text-[11px] text-ide-muted select-none flex-shrink-0">
      <div className="flex items-center gap-3">
        <span className={statusColors[buildStatus]}>
          <Circle size={8} className="inline mr-1" fill="currentColor" />
          {statusLabels[buildStatus]}
        </span>
      </div>

      <div className="flex-1" />

      <div className="flex items-center gap-3">
        {wsConnected ? (
          <span className="text-ide-success flex items-center gap-1">
            <Wifi size={10} /> यन्त्रम्
          </span>
        ) : (
          <span className="text-ide-muted flex items-center gap-1">
            <WifiOff size={10} /> असन्नद्धः
          </span>
        )}

        <span className="text-ide-muted">|</span>

        <span className="flex items-center gap-1">
          <span className="uppercase text-[10px]">{transliterationMode === 'devanagari' ? 'देव' : 'SLP1'}</span>
        </span>

        <span className="text-ide-muted">|</span>

        <span className="text-ide-muted">{cursorPos}</span>
      </div>
    </footer>
  );
};
