import { useState, useEffect, useRef } from 'react';
import {
  Play, FilePlus, FolderOpen, Search, Settings, Save, Languages, Sun, Terminal, X,
} from 'lucide-react';
import { useIdeStore } from '../../store/ideStore';

interface Command {
  id: string;
  label: string;
  icon: typeof Play;
  keys?: string;
  action: () => void;
}

export const CommandPalette = () => {
  const {
    modalOpen, setModal, compileAndRun, toggleTransliteration, toggleTheme,
    setSettingsOpen, setSearchQuery, clearLogs, loadWorkspaceFiles,
  } = useIdeStore();

  const [query, setQuery] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);
  const [selectedIdx, setSelectedIdx] = useState(0);

  const isOpen = modalOpen === 'command-palette';

  useEffect(() => {
    if (isOpen) {
      setQuery('');
      setSelectedIdx(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isOpen]);

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (!isOpen) return;
      if (e.key === 'Escape') setModal('none');
    };
    window.addEventListener('keydown', handleKey);
    return () => window.removeEventListener('keydown', handleKey);
  }, [isOpen, setModal]);

  const commands: Command[] = [
    { id: 'run', label: 'संग्रहणं च चालनम्', icon: Play, keys: 'Ctrl+Enter', action: () => { setModal('none'); compileAndRun(); } },
    { id: 'new', label: 'नूतना सञ्चिका', icon: FilePlus, keys: 'Ctrl+N', action: () => setModal('create') },
    { id: 'load-workspace', label: 'कार्यक्षेत्रम् आनयतु', icon: FolderOpen, action: () => { setModal('none'); loadWorkspaceFiles(); } },
    { id: 'save', label: 'सञ्चिकां रक्षतु', icon: Save, keys: 'Ctrl+S', action: () => { useIdeStore.getState().saveCurrentFile(); setModal('none'); } },
    { id: 'search', label: 'सञ्चिकाः अन्विष्यताम्', icon: Search, action: () => { setSearchQuery(''); setModal('none'); } },
    { id: 'transliteration', label: 'लिप्यन्तरम्', icon: Languages, action: () => { toggleTransliteration(); setModal('none'); } },
    { id: 'theme', label: 'रूपं परिवर्त्यताम्', icon: Sun, action: () => { toggleTheme(); setModal('none'); } },
    { id: 'settings', label: 'समायोजनम्', icon: Settings, action: () => { setSettingsOpen(true); setModal('none'); } },
    { id: 'clear', label: 'फलकं शोध्यताम्', icon: Terminal, action: () => { clearLogs(); setModal('none'); } },
  ];

  const filtered = commands.filter((c) =>
    c.label.toLowerCase().includes(query.toLowerCase())
  );

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (filtered.length === 0) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIdx((i) => Math.min(i + 1, filtered.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIdx((i) => Math.max(i - 1, 0));
    } else if (e.key === 'Enter' && filtered[selectedIdx]) {
      filtered[selectedIdx].action();
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh] bg-[var(--ide-overlay)] backdrop-blur-sm">
      <div className="absolute inset-0" onClick={() => setModal('none')} />

      <div className="bg-ide-panel border border-ide-border rounded-lg shadow-2xl w-full max-w-lg overflow-hidden relative z-10 animate-fade-in">
        <div className="flex items-center border-b border-ide-border px-3">
          <Search size={14} className="text-ide-muted mr-2" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => { setQuery(e.target.value); setSelectedIdx(0); }}
            onKeyDown={handleKeyDown}
            placeholder="आज्ञां लिख्यताम्..."
            className="flex-1 bg-transparent text-sm text-ide-text py-3 outline-none placeholder-[var(--ide-muted)]"
          />
          <button onClick={() => setModal('none')} className="toolbar-btn">
            <X size={14} />
          </button>
        </div>

        <div className="max-h-64 overflow-y-auto py-1">
          {filtered.length === 0 && (
            <div className="px-4 py-6 text-xs text-ide-muted text-center">आज्ञा न प्राप्ता</div>
          )}

          {filtered.map((cmd, idx) => (
            <button
              key={cmd.id}
              onClick={cmd.action}
              className={`w-full flex items-center gap-3 px-4 py-2 text-sm text-left transition-colors ${
                idx === selectedIdx
                  ? 'bg-ide-active text-ide-text'
                  : 'text-ide-text hover:bg-ide-hover'
              }`}
            >
              <cmd.icon size={14} className="text-ide-muted shrink-0" />
              <span className="flex-1">{cmd.label}</span>
              {cmd.keys && (
                <span className="text-[10px] text-ide-muted font-mono">{cmd.keys}</span>
              )}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
};
