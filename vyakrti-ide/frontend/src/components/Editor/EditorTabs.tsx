import { X, FileText } from 'lucide-react';
import { useIdeStore } from '../../store/ideStore';

export const EditorTabs = () => {
  const { files, activeFileId, setActiveFile, closeFile } = useIdeStore();

  return (
    <div className="flex items-center bg-ide-surface border-b border-ide-border overflow-x-auto">
      {files.map((file) => {
        const isActive = file.id === activeFileId;
        return (
          <div
            key={file.id}
            onClick={() => setActiveFile(file.id)}
            className={`group flex items-center gap-1.5 px-3 py-1.5 text-xs cursor-pointer border-r border-ide-border transition-colors select-none ${
              isActive
                ? 'bg-ide-panel text-ide-text border-t-2 border-t-ide-accent mt-0'
                : 'bg-ide-surface text-ide-muted hover:text-ide-text hover:bg-ide-hover'
            }`}
          >
            <FileText size={12} className={isActive ? 'text-ide-accent' : 'text-ide-muted'} />
            <span className="truncate max-w-[120px]">{file.name}</span>
            {file.isDirty && (
              <span className="w-1.5 h-1.5 rounded-full bg-ide-warning flex-shrink-0" />
            )}
            <button
              onClick={(e) => {
                e.stopPropagation();
                closeFile(file.id);
              }}
              className="p-0.5 rounded hover:bg-ide-hover opacity-0 group-hover:opacity-100 transition-opacity ml-0.5"
              title="सञ्चिकां पिदधातु"
            >
              <X size={10} />
            </button>
          </div>
        );
      })}
      {files.length === 0 && (
        <div className="px-4 py-1.5 text-xs text-ide-muted">न कोऽपि सञ्चिका</div>
      )}
    </div>
  );
};
