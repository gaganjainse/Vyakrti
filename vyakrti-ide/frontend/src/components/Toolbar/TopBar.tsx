import { useState, useRef } from 'react';
import {
  Play, Square, FilePlus, Save, Search, Settings, X, CheckCircle,
} from 'lucide-react';
import { useIdeStore } from '../../store/ideStore';

export const TopBar = () => {
  const {
    compileAndRun, buildStatus, transliterationMode, toggleTransliteration,
    saveCurrentFile, setSettingsOpen, searchQuery, setSearchQuery, setModal,
    stopExecution, files, activeFileId,
  } = useIdeStore();

  const [showSaveToast, setShowSaveToast] = useState(false);
  const saveToastTimer = useRef<ReturnType<typeof setTimeout>>();
  const isRunning = buildStatus === 'compiling' || buildStatus === 'running';
  const activeFile = files.find((f) => f.id === activeFileId);

  const handleSave = () => {
    saveCurrentFile();
    setShowSaveToast(true);
    clearTimeout(saveToastTimer.current);
    saveToastTimer.current = setTimeout(() => setShowSaveToast(false), 1500);
  };

  return (
    <header className="h-11 border-b border-ide-border bg-ide-panel flex items-center justify-between px-3 select-none relative z-40">
      {showSaveToast && (
        <div className="absolute top-12 left-1/2 -translate-x-1/2 bg-ide-surface border border-ide-success/40 text-ide-success text-xs font-medium px-4 py-2 rounded shadow-2xl flex items-center gap-2 z-50 animate-fade-in border-l-4 border-l-ide-success">
          <CheckCircle size={14} />
          <span>सञ्चिका रक्षिता</span>
        </div>
      )}

      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2">
          <div className="w-6 h-6 bg-ide-accent rounded flex items-center justify-center text-[#181825] font-bold text-xs font-mono">
            व्
          </div>
          <span className="font-bold text-sm tracking-wide text-ide-text pr-3 border-r border-ide-border">
            Vyākṛti
          </span>
        </div>

        <div className="flex items-center gap-0.5">
          <button onClick={() => setModal('create')} className="toolbar-btn" title="नूतना सञ्चिका (Ctrl+N)">
            <FilePlus size={15} />
          </button>
          <button onClick={handleSave} className="toolbar-btn" title="रक्षतु (Ctrl+S)">
            <Save size={15} />
          </button>

          <div className="w-px h-4 bg-ide-border mx-1" />

          <div className="relative flex items-center">
            <Search size={11} className="absolute left-2 text-ide-muted" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onKeyDown={(e) => e.key === 'Escape' && setSearchQuery('')}
              placeholder="सञ्चिकाः अन्विष्यताम्..."
              className="bg-ide-surface text-xs text-ide-text pl-7 pr-2 py-1 border border-ide-border rounded w-44 focus:outline-none focus:border-ide-accent transition-colors"
            />
            {searchQuery && (
              <button onClick={() => setSearchQuery('')} className="absolute right-1.5 text-ide-muted hover:text-ide-text">
                <X size={10} />
              </button>
            )}
          </div>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <div className="text-[11px] text-ide-muted font-mono truncate max-w-[200px] hidden sm:block">
          {activeFile?.name || ''}
        </div>

        <button
          onClick={toggleTransliteration}
          className="flex items-center gap-1.5 px-2.5 py-1 text-[11px] font-medium bg-ide-surface hover:bg-ide-hover text-ide-text rounded border border-ide-border transition-colors"
          title="लिप्यन्तरम्"
        >
          <span className="text-ide-accent font-semibold text-xs" style={{ fontFamily: "'Noto Sans Devanagari', serif" }}>अ</span>
          <span>{transliterationMode === 'devanagari' ? 'देवनागरी' : 'SLP1'}</span>
        </button>

        <button
          onClick={() => setSettingsOpen(true)}
          className="toolbar-btn"
          title="समायोजनम्"
        >
          <Settings size={15} />
        </button>

        <div className="w-px h-4 bg-ide-border mx-1" />

        {isRunning ? (
          <button
            onClick={stopExecution}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-red-700 hover:bg-red-600 text-white rounded text-[11px] font-medium transition-colors"
          >
            <Square size={11} />
            <span>विरमतु</span>
          </button>
        ) : (
          <button
            onClick={compileAndRun}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-emerald-700 hover:bg-emerald-600 text-white rounded text-[11px] font-medium transition-colors shadow-sm"
          >
            <Play size={11} fill="currentColor" />
            <span>चालयतु</span>
          </button>
        )}
      </div>
    </header>
  );
};
