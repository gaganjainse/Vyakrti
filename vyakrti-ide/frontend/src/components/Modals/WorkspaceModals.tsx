import { useState, useEffect } from 'react';
import { X, FilePlus, Edit2, AlertTriangle } from 'lucide-react';
import { useIdeStore } from '../../store/ideStore';

export const WorkspaceModals = () => {
  const {
    modalOpen, modalTargetFileId, files,
    createNewFile, renameFile, deleteFile, setModal, activeFileId,
  } = useIdeStore();

  const [inputValue, setInputValue] = useState('');

  const targetFile = files.find((f) => f.id === (modalTargetFileId || activeFileId));

  useEffect(() => {
    if (modalOpen === 'rename' && targetFile) {
      setInputValue(targetFile.name.replace(/\.vya$/, ''));
    } else if (modalOpen === 'create') {
      setInputValue('');
    }
  }, [modalOpen, modalTargetFileId, targetFile]);

  if (modalOpen === 'none') return null;

  const handleClose = () => setModal('none');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!inputValue.trim()) return;
    if (modalOpen === 'create') {
      createNewFile(inputValue.trim());
    } else if (modalOpen === 'rename' && modalTargetFileId) {
      renameFile(modalTargetFileId, inputValue.trim());
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--ide-overlay)] backdrop-blur-sm select-none p-4">
      <div className="absolute inset-0" onClick={handleClose} />

      <div className="bg-ide-panel border border-ide-border rounded-lg shadow-2xl w-full max-w-md overflow-hidden relative z-10 animate-fade-in">
        <div className="flex items-center justify-between border-b border-ide-border px-4 py-3 bg-ide-surface">
          <div className="flex items-center gap-2 text-sm font-semibold text-ide-text">
            {modalOpen === 'create' && <FilePlus size={15} className="text-ide-accent" />}
            {modalOpen === 'rename' && <Edit2 size={15} className="text-ide-success" />}
            {modalOpen === 'delete' && <AlertTriangle size={15} className="text-ide-error" />}
            <span>
              {modalOpen === 'create' && 'नूतना सञ्चिका'}
              {modalOpen === 'rename' && 'नामान्तरम्'}
              {modalOpen === 'delete' && 'सञ्चिकां नाशयतु'}
            </span>
          </div>
          <button onClick={handleClose} className="toolbar-btn">
            <X size={14} />
          </button>
        </div>

        {modalOpen === 'delete' ? (
          <div className="p-4 space-y-4">
            <p className="text-sm text-ide-text leading-relaxed">
              <span className="font-mono text-ide-error font-semibold bg-ide-error/10 px-1 py-0.5 rounded border border-ide-error/20">{targetFile?.name}</span> इदं नाशयितुम् इच्छति? एतत् परावर्तयितुं न शक्यते।
            </p>
            <div className="flex items-center justify-end gap-2 pt-2 border-t border-ide-border/50">
              <button onClick={handleClose} className="px-3 py-1.5 text-xs font-medium text-ide-muted hover:text-ide-text hover:bg-ide-hover rounded transition-colors">
                मा
              </button>
              <button
                onClick={() => { if (modalTargetFileId) deleteFile(modalTargetFileId); }}
                className="px-3 py-1.5 text-xs font-medium bg-red-600 hover:bg-red-500 text-white rounded transition-colors shadow-sm"
              >
                नाशयतु
              </button>
            </div>
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="p-4 space-y-4">
            <div>
              <label className="block text-xs font-semibold text-ide-muted uppercase tracking-wider mb-1.5">
                {modalOpen === 'create' ? 'सञ्चिका-नाम' : 'नूतनं नाम'}
              </label>
              <input
                type="text"
                autoFocus
                value={inputValue}
                onChange={(e) => setInputValue(e.target.value)}
                placeholder={modalOpen === 'create' ? 'main.vya' : 'नूतनं नाम लिख्यताम्...'}
                className="ide-input w-full"
              />
              <p className="text-[11px] text-ide-muted mt-1.5">
                विस्तारः <span className="font-mono text-ide-muted">.vya</span> स्वचलितरूपेण योक्ष्यते।
              </p>
            </div>

            <div className="flex items-center justify-end gap-2 pt-2 border-t border-ide-border/50">
              <button type="button" onClick={handleClose} className="px-3 py-1.5 text-xs font-medium text-ide-muted hover:text-ide-text hover:bg-ide-hover rounded transition-colors">
                मा
              </button>
              <button
                type="submit"
                disabled={!inputValue.trim()}
                className="px-3 py-1.5 text-xs font-medium bg-ide-accent hover:bg-blue-400 disabled:opacity-40 text-[#181825] rounded transition-colors shadow-sm font-semibold"
              >
                {modalOpen === 'create' ? 'सृज्यताम्' : 'नामान्तरम्'}
              </button>
            </div>
          </form>
        )}
      </div>
    </div>
  );
};
